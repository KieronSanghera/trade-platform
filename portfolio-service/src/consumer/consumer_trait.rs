use crate::consumer::{ConsumerError, Handler};

// Base trait, defines contract for others that extend
#[async_trait::async_trait]
pub trait TradeEventConsumer: Send + Sync {
    async fn start(&self, handler: Handler) -> Result<(), ConsumerError>;

    async fn readiness_check(&self) -> Result<(), ConsumerError>;
}
