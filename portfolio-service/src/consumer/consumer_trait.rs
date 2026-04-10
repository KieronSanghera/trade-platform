use crate::consumer::{ConsumerError, Handler};

#[async_trait::async_trait]
pub trait TradeEventConsumer: Send + Sync {
    async fn start(&self, handler: Handler) -> Result<(), ConsumerError>;

    async fn readiness_check(&self) -> Result<(), ConsumerError>;
}
