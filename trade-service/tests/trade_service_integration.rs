use std::sync::Arc;

use prost_types::Timestamp;
use tokio::net::TcpListener;
use tonic::transport::Server;

use shared::TradeExecuted;
use trade_service::grpc::trade_service::GrpcTradeService;
use trade_service::producer::{ProducerError, StubProducer, TradeEventProducer};
use trade_service::trade::TradeRequest;
use trade_service::trade::trade_service_client::TradeServiceClient;
use trade_service::trade::trade_service_server::TradeServiceServer;

// Spins up a tonic server on a random port and returns the address, with everything
async fn start_valid_test_server() -> String {
    // Port 0 tells the OS to pick a free port for us
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Basic Producer
    let producer = Arc::new(StubProducer);
    let service = GrpcTradeService::new(producer);

    // Start server
    tokio::spawn(async move {
        Server::builder()
            .add_service(TradeServiceServer::new(service))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    format!("http://{}", addr)
}

// Spins up a tonic server on a random port and returns the address with an invalid_producer
async fn start_test_server_invalid_producer() -> String {
    struct FailProducer;

    #[async_trait::async_trait]
    impl TradeEventProducer for FailProducer {
        async fn publish_trade_executed(&self, _: &TradeExecuted) -> Result<(), ProducerError> {
            Err(ProducerError::KafkaTransportError(
                rdkafka::error::KafkaError::NoMessageReceived,
            ))
        }

        async fn readiness_check(&self) -> Result<(), ProducerError> {
            Ok(())
        }
    }

    // Port 0 tells the OS to pick a free port for us
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Basic Producer
    let producer = Arc::new(FailProducer);
    let service = GrpcTradeService::new(producer);

    // Start server
    tokio::spawn(async move {
        Server::builder()
            .add_service(TradeServiceServer::new(service))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    format!("http://{}", addr)
}

// Helper to build a valid trade request
fn valid_trade_request() -> TradeRequest {
    TradeRequest {
        trade_id: "trade-1".to_string(),
        user_id: "user-1".to_string(),
        asset: "BTC".to_string(),
        price: "100.0".to_string(),
        quantity: "1.0".to_string(),
        side: 1, // Buy
        timestamp: Some(Timestamp {
            seconds: 1,
            nanos: 0,
        }),
    }
}

#[tokio::test]
async fn valid_trade_is_accepted() {
    let addr = start_valid_test_server().await;
    let mut client = TradeServiceClient::connect(addr).await.unwrap();

    let response = client.submit_trade(valid_trade_request()).await.unwrap();

    assert!(response.get_ref().accepted);
    assert_eq!(response.get_ref().message, "Trade accepted");
}

#[tokio::test]
async fn missing_trade_id_returns_invalid_argument() {
    let addr = start_valid_test_server().await;
    let mut client = TradeServiceClient::connect(addr).await.unwrap();

    let mut request = valid_trade_request();
    request.trade_id = "".to_string();

    let status = client.submit_trade(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn missing_timestamp_returns_invalid_argument() {
    let addr = start_valid_test_server().await;
    let mut client = TradeServiceClient::connect(addr).await.unwrap();

    let mut request = valid_trade_request();
    request.timestamp = None;

    let status = client.submit_trade(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn invalid_price_returns_invalid_argument() {
    let addr = start_valid_test_server().await;
    let mut client = TradeServiceClient::connect(addr).await.unwrap();

    let mut request = valid_trade_request();
    request.price = "not-a-number".to_string();

    let status = client.submit_trade(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn invalid_side_returns_invalid_argument() {
    let addr = start_valid_test_server().await;
    let mut client = TradeServiceClient::connect(addr).await.unwrap();

    let mut request = valid_trade_request();
    request.side = 0;

    let status = client.submit_trade(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn failed_producer_returns_internal_error() {
    let addr = start_test_server_invalid_producer().await;
    let mut client = TradeServiceClient::connect(addr).await.unwrap();

    let request = valid_trade_request();

    let status = client.submit_trade(request).await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Internal);
}
