use std::{sync::Arc, time::Duration};

use dotenv::dotenv;
use portfolio_service::{
    config::{AppConfig, KafkaConfig, PostgresConfig},
    consumer::{ConsumerError, consumer_trait::TradeEventConsumer, kafka::KafkaConsumer},
    db::postgres::PostgresDB,
    grpc::portfolio_service::GrpcPortfolioService,
    health::{
        self,
        state::{LivenessState, ReadinessState},
    },
    portfolio::portfolio_service_server::PortfolioServiceServer,
};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load AppConfig
    dotenv().ok();
    let portfolio_service_config = AppConfig::from_env();
    let postgres_config = PostgresConfig::from_env()?;
    let kafka_config = KafkaConfig::from_env()?;
    let loaded_config = serde_json::to_string_pretty(&portfolio_service_config)?.replace("\\", "");

    init_tracing(&portfolio_service_config.log_format);
    tracing::debug!(config = &loaded_config);

    // Livez and Readyz set up needs to be added
    let liveness = Arc::new(RwLock::new(LivenessState::default()));
    let readiness = Arc::new(RwLock::new(ReadinessState::default()));

    let postgres = Arc::new(PostgresDB::new(&postgres_config).await.inspect_err(|err| {
        tracing::error!(error = err.to_string(), "Failed to connect to database")
    })?);

    postgres
        .readiness_check()
        .await
        .expect("Database is not ready");

    // Construct Consumer
    let consumer = Arc::new(KafkaConsumer::new(&kafka_config).inspect_err(|e| {
        tracing::error!(error = e.to_string(), "Failed to build consumer");
    })?);

    // Initial consumer readiness
    match consumer.readiness_check().await {
        Ok(_) => {
            readiness.write().await.mark_ready();
        }
        Err(e) => {
            tracing::warn!(error = %e, "Producer readiness check failed");
            readiness.write().await.mark_unready();
        }
    }

    // Start health/ready endpoints
    spawn_health_server(
        portfolio_service_config.health_endpoint_port,
        liveness.clone(),
        readiness.clone(),
    );

    // Start Readiness process
    spawn_readiness_monitor(
        readiness.clone(),
        consumer.clone(),
        postgres.clone(),
        portfolio_service_config.monitor_interval_secs,
    );

    let consumer_db = postgres.clone();
    let consumer_handle = consumer.clone();
    spawn_kafka_consumer(consumer_db, consumer_handle).await;

    // Build Server
    let addr = format!("0.0.0.0:{}", &portfolio_service_config.port).parse()?;
    let service = GrpcPortfolioService::new(postgres);
    tracing::debug!(address = %addr, "PortfolioService Live!");

    // Mark service as live
    liveness.write().await.mark_live();

    // gRPC server on main task
    Server::builder()
        .add_service(PortfolioServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}

// Tracing config options
fn init_tracing(log_format: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match log_format {
        // Pretty Logs
        "compact" => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .compact()
                .try_init()
                .ok();
        }
        // Functional Logs (JSON format)
        _ => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .json()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_target(true)
                .try_init()
                .ok();
        }
    }
}

fn spawn_health_server(
    port: u16,
    liveness: Arc<RwLock<LivenessState>>,
    readiness: Arc<RwLock<ReadinessState>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(listener) => {
                if let Err(e) =
                    axum::serve(listener, health::create_health_router(liveness, readiness)).await
                {
                    tracing::error!(error = ?e, "Health server failed");
                }
            }
            Err(e) => {
                tracing::error!(error = ?e, "Failed to bind health server");
            }
        }
    })
}

fn spawn_readiness_monitor(
    readiness: Arc<RwLock<ReadinessState>>,
    consumer: Arc<KafkaConsumer>,
    db: Arc<PostgresDB>,
    interval: u64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let interval = Duration::from_secs(interval);
        loop {
            sleep(interval).await;

            let kafka_ok = consumer.readiness_check().await;
            let db_ok = db.readiness_check().await;

            match (kafka_ok, db_ok) {
                (Ok(_), Ok(_)) => readiness.write().await.mark_ready(),
                (kafka_result, db_result) => {
                    if let Err(e) = kafka_result {
                        tracing::warn!(error = %e, "Kafka readiness check failed");
                    }
                    if let Err(e) = db_result {
                        tracing::warn!(error = %e, "Postgres readiness check failed");
                    }
                    readiness.write().await.mark_unready();
                }
            }
        }
    })
}

async fn spawn_kafka_consumer(db: Arc<PostgresDB>, consumer: Arc<KafkaConsumer>) -> JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!("Starting Kafka consumer");
        let mut retry_status = false;
        loop {
            if retry_status {
                tracing::info!("Recovered consumer");
                retry_status = false;
            }
            let db = db.clone();
            if let Err(e) = consumer
                .start(Box::new(move |trade| {
                    let db = db.clone();
                    Box::pin(async move {
                        db.handle_position(&trade)
                            .await
                            .map_err(|e| ConsumerError::ProcessingFailed(e.to_string()))
                    })
                }))
                .await
            {
                tracing::warn!(error = %e, "Consumer stopped - retrying in 5 seconds");
                retry_status = true;
            }

            sleep(Duration::from_secs(5)).await;
        }
    })
}
