use super::producer_trait::TradeEventProducer;
use crate::models::trade_executed::TradeExecuted;
use crate::producer::ProducerError;

pub struct StubProducer;

// Extension of basic TradeEventProducer
#[async_trait::async_trait]
impl TradeEventProducer for StubProducer {
    // Handle dummy publish by just logging an attempt to publish
    async fn publish_trade_executed(&self, trade: &TradeExecuted) -> Result<(), ProducerError> {
        tracing::info!(?trade, "Stub publishing TradeExecuted event");
        Ok(())
    }
    async fn readiness_check(&self) -> Result<(), ProducerError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{
        trade::Side,
        types::{NonEmptyString, PositiveDecimal},
    };
    use chrono::Utc;

    use super::*;

    fn create_stub() -> StubProducer {
        StubProducer
    }

    fn create_trade_executed() -> TradeExecuted {
        TradeExecuted {
            trade_id: NonEmptyString::try_from("trade-1").unwrap(),
            user_id: NonEmptyString::try_from("user-1").unwrap(),
            asset: NonEmptyString::try_from("BTC").unwrap(),
            price: PositiveDecimal::try_from("100.0").unwrap(),
            quantity: PositiveDecimal::try_from("0.5").unwrap(),
            side: Side::Buy,
            timestamp: Utc::now(),
        }
    }

    mod publish {
        use super::*;

        #[test]
        fn successful_publish() {
            let producer = create_stub();
            let trade = create_trade_executed();
            let result = tokio_test::block_on(producer.publish_trade_executed(&trade));
            assert!(result.is_ok())
        }
    }
    mod readiness {
        use super::*;

        #[test]
        fn successful_check() {
            let producer = create_stub();
            assert!(tokio_test::block_on(producer.readiness_check()).is_ok())
        }
    }
}
