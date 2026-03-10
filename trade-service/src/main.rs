use trade_service::{config, grpc, health, producer, trade};

use config::AppConfig;
use grpc::trade_service::GrpcTradeService;
use health::state::{LivenessState, ReadinessState};
use producer::{TradeEventProducer, factory::build_producer};
use trade::trade_service_server::TradeServiceServer;

use dotenv::dotenv;
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load AppConfig
    dotenv().ok();
    let trade_service_config: AppConfig = AppConfig::from_env();
    let loaded_config = serde_json::to_string_pretty(&trade_service_config)?.replace("\\", "");

    // Initialize Tracing - Log format is dynamic via LOG_FORMAT env var
    init_tracing(&trade_service_config.log_format);
    tracing::debug!(config=%loaded_config);

    // Livez and Readyz state setup
    let liveness = Arc::new(RwLock::new(LivenessState::default()));
    let readiness = Arc::new(RwLock::new(ReadinessState::default()));

    // Construct the TradeEventProducer via the factory
    // Chooses Kafka or Stub based on environment configuration
    // Errors here are logged and propagated, causing service to fail fast
    let producer: Arc<dyn TradeEventProducer> =
        build_producer(&trade_service_config.publish_backend)
            .await
            .inspect_err(|e| {
                tracing::error!(error = e.to_string(), "Failed to build producer");
            })?;

    // Initial producer readiness
    match producer.readiness_check().await {
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
        trade_service_config.health_endpoint_port,
        liveness.clone(),
        readiness.clone(),
    );

    // Start Readiness process
    spawn_readiness_monitor(
        readiness.clone(),
        producer.clone(),
        trade_service_config.monitor_interval_secs,
    );

    // Build Server
    let addr = format!("0.0.0.0:{}", &trade_service_config.port).parse()?;
    let service = GrpcTradeService::new(producer);
    tracing::debug!(address = %addr, "TradeService Live!");

    // Mark service as live
    liveness.write().await.mark_live();

    // Start Server
    Server::builder()
        .add_service(TradeServiceServer::new(service))
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
    producer: Arc<dyn TradeEventProducer>,
    interval: u64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let interval = Duration::from_secs(interval);

        loop {
            // Wait between checks
            sleep(interval).await;

            match producer.readiness_check().await {
                Ok(_) => {
                    let mut state = readiness.write().await;
                    if !state.is_ready() {
                        tracing::info!("Producer recovered, now ready")
                    }
                    state.mark_ready();
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Producer readiness check failed");
                    let mut state = readiness.write().await;
                    state.mark_unready();
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::task::JoinHandle;
    use trade_service::producer::StubProducer;

    #[serial]
    #[test]
    fn init_tracing_compact() {
        init_tracing("compact");
    }

    #[serial]
    #[test]
    fn init_tracing_json() {
        init_tracing("json");
    }

    #[serial]
    #[test]
    fn init_tracing_fallback() {
        init_tracing("anything");
    }

    #[tokio::test]
    async fn spawn_health_server_returns_joinhandle() {
        let liveness = Arc::new(RwLock::new(LivenessState::default()));
        let readiness = Arc::new(RwLock::new(ReadinessState::default()));

        let handle: JoinHandle<()> = spawn_health_server(0, liveness, readiness);
        // Just assert it returns a JoinHandle
        assert!(handle.is_finished() == false);
    }

    #[tokio::test]
    async fn spawn_readiness_monitor_returns_joinhandle() {
        let readiness = Arc::new(RwLock::new(ReadinessState::default()));
        let producer: Arc<dyn TradeEventProducer> = Arc::new(StubProducer);

        let handle: JoinHandle<()> = spawn_readiness_monitor(readiness, producer, 1);
        assert!(handle.is_finished() == false);
    }
}
