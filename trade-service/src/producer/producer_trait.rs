use crate::models::trade_executed::TradeExecuted;
use crate::producer::ProducerError;

// Base Trait for other to extend
#[async_trait::async_trait]
pub trait TradeEventProducer: Send + Sync {
    async fn publish_trade_executed(&self, trade: &TradeExecuted) -> Result<(), ProducerError>;

    async fn readiness_check(&self) -> Result<(), ProducerError>;
}
