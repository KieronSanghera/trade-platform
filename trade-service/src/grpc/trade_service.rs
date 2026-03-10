use crate::error::TradeError;
use crate::models::trade::Trade;
use crate::models::trade_executed::TradeExecuted;
use crate::producer::TradeEventProducer;
use crate::trade::trade_service_server::TradeService;
use crate::trade::{TradeRequest, TradeResponse};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument, warn};

pub struct GrpcTradeService {
    producer: Arc<dyn TradeEventProducer>,
}

#[tonic::async_trait]
impl TradeService for GrpcTradeService {
    #[instrument(skip(self, request))]
    async fn submit_trade(
        &self,
        request: Request<TradeRequest>,
    ) -> Result<Response<TradeResponse>, Status> {
        // currently just print the request
        let request = request.into_inner();
        info!(
            trade_id = %request.trade_id,
            user_id = %request.user_id,
            asset = %request.asset,
            "Received trade request"
        );

        let trade = Trade::try_from(&request)
            .inspect_err(|err| {
                warn!(
                    trade_id = %request.trade_id,
                    error = ?err,
                    "Trade validation failed"
                );
            })
            .map_err(Self::map_trade_error)?;

        info!(
        trade_id = %trade.trade_id,
        user_id = %trade.user_id,
        asset = %trade.asset,
        side = %trade.side,
        price = %*trade.price,
        quantity = %*trade.quantity,
        timestamp = %trade.timestamp,
        "Trade validated"
        );

        // Move to function that creates the TradeExecuted
        let executed_trade = TradeExecuted::from(&trade);

        self.producer
            .publish_trade_executed(&executed_trade)
            .await
            .inspect_err(|err| {
                error!(
                    trade_id = %executed_trade.trade_id,
                    error = ?err,
                    "Error while publishing event"
                );
            })
            .map_err(|_| Status::internal("Failed to publish event"))?;

        // return a stub response
        Ok(Response::new(TradeResponse {
            accepted: true,
            message: "Trade accepted".into(),
        }))
    }
}

impl GrpcTradeService {
    pub fn new(producer: Arc<dyn TradeEventProducer>) -> Self {
        tracing::info!("TradeService has started!");
        Self { producer }
    }

    // Helper function to handle error to response
    fn map_trade_error(err: TradeError) -> Status {
        match err {
            TradeError::MissingTradeId => Status::invalid_argument("trade_id is required"),
            TradeError::MissingUserId => Status::invalid_argument("user_id is required"),
            TradeError::InvalidPrice => Status::invalid_argument("price must be positive"),
            TradeError::InvalidQuantity => Status::invalid_argument("quantity must be positive"),
            TradeError::InvalidSide => Status::invalid_argument("side must be BUY or SELL"),
            TradeError::MissingTimestamp => Status::invalid_argument("timestamp is required"),
            TradeError::MissingAsset => Status::invalid_argument("asset is required"),
            TradeError::FailedTimestampConversion => {
                Status::invalid_argument("timestamp format is incorrect")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::producer::{ProducerError, StubProducer};
    use prost_types::Timestamp;

    fn create_failing_producer_trade_service() -> GrpcTradeService {
        struct FailProducer;

        #[async_trait::async_trait]
        impl TradeEventProducer for FailProducer {
            async fn publish_trade_executed(&self, _: &TradeExecuted) -> Result<(), ProducerError> {
                Err(ProducerError::TopicMissing)
            }

            async fn readiness_check(&self) -> Result<(), ProducerError> {
                Ok(())
            }
        }
        let producer = Arc::new(FailProducer);
        GrpcTradeService::new(producer)
    }

    fn create_grpc_trade_service() -> GrpcTradeService {
        let producer = Arc::new(StubProducer);
        GrpcTradeService::new(producer)
    }

    #[test]
    fn create_valid_trade_service() {
        let _ = create_grpc_trade_service();
    }

    fn create_trade_request() -> TradeRequest {
        TradeRequest {
            trade_id: "tid".to_string(),
            user_id: "uid".to_string(),
            asset: "BTC".to_string(),
            quantity: "1".to_string(),
            price: "1".to_string(),
            side: 1,
            timestamp: Some(Timestamp::default()),
        }
    }

    #[test]
    fn valid_trade() {
        let service = create_grpc_trade_service();
        let request = tonic::Request::new(create_trade_request());
        let response = tokio_test::block_on(service.submit_trade(request)).unwrap();
        assert!(response.get_ref().accepted);
    }

    #[test]
    fn invalid_trade_request() {
        let service = create_grpc_trade_service();
        let mut trade_request = create_trade_request();
        trade_request.timestamp = None;

        let request = Request::new(trade_request);
        let response = tokio_test::block_on(service.submit_trade(request));
        let status = response.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert_eq!(status.message(), "timestamp is required");
    }

    #[test]
    fn producer_failure() {
        let failing_producer_service = create_failing_producer_trade_service();
        let trade_request = create_trade_request();
        let request = Request::new(trade_request);
        let response = tokio_test::block_on(failing_producer_service.submit_trade(request));
        let status = response.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
