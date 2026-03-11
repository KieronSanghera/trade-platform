use super::producer_trait::TradeEventProducer;
use crate::config::KafkaConfig;
use crate::producer::ProducerError;
use rdkafka::producer::Producer;
use rdkafka::{
    ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use shared::models::TradeExecuted;
use std::time::Duration;

pub struct KafkaProducer {
    // Server connection
    producer: FutureProducer,
    // Configured Topic
    topic: String,
}

// Extension of basic TradeEventProducer
#[async_trait::async_trait]
impl TradeEventProducer for KafkaProducer {
    // Handles publishing to kafka on a specific server and topic
    async fn publish_trade_executed(&self, trade: &TradeExecuted) -> Result<(), ProducerError> {
        tracing::debug!(trade = ?trade, "Attempting to publish");
        // Build payload by serializing TradeExecuted object
        let payload = self.build_payload(trade)?;
        tracing::debug!(payload = &payload);
        // Build item to publish packing payload and key for kafka to handle
        let record = self.build_record(&trade.user_id, &payload);
        // Publish item to kafka
        self.publish(record).await?;
        tracing::info!("Published to Trade");
        Ok(())
    }

    async fn readiness_check(&self) -> Result<(), ProducerError> {
        let metadata = self
            .producer
            .client()
            .fetch_metadata(None, Duration::from_secs(1))
            .map_err(ProducerError::ConnectionFailed)?;

        let topic_exists = metadata.topics().iter().any(|t| t.name() == self.topic);

        if !topic_exists {
            return Err(ProducerError::TopicMissing);
        }
        Ok(())
    }
}

impl KafkaProducer {
    // Create KafkaProducer
    pub fn new(config: &KafkaConfig) -> Result<Self, ProducerError> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &config.broker) // Server to connect to
            .set("message.timeout.ms", "5000") // Timeout when publishing message
            .create()?;

        Ok(Self {
            producer,
            topic: config.topic.clone(),
        })
    }

    fn build_payload(&self, trade: &TradeExecuted) -> Result<String, ProducerError> {
        // Build payload by serializing TradeExecuted object
        serde_json::to_string(&trade).map_err(ProducerError::SerializationError)
    }

    fn build_record<'a>(&'a self, id: &'a str, payload: &'a str) -> FutureRecord<'a, str, str> {
        // Build item to publish packing payload and key for kafka to handle
        FutureRecord::to(&self.topic).payload(payload).key(id)
    }

    async fn publish(&self, record: FutureRecord<'_, str, str>) -> Result<(), ProducerError> {
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| ProducerError::KafkaTransportError(err))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared::{NonEmptyString, PositiveDecimal, Side};

    // --- Helpers ---
    fn create_kafka_producer() -> KafkaProducer {
        KafkaProducer::new(&KafkaConfig {
            broker: "broker".to_string(),
            topic: "topic".to_string(),
        })
        .unwrap()
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

    // --- Unit tests for valid scenarios ---
    mod valid {
        use super::*;

        #[test]
        fn producer_creation_works() {
            let producer = KafkaProducer::new(&KafkaConfig {
                broker: "broker".to_string(),
                topic: "topic".to_string(),
            });
            assert!(producer.is_ok());
        }

        #[test]
        fn build_payload_creates_valid_json() {
            let producer = create_kafka_producer();
            let trade = create_trade_executed();

            let payload = producer.build_payload(&trade).unwrap();

            // Simple sanity checks
            assert!(payload.contains("trade-1"));
            assert!(payload.contains("BTC"));
            assert!(payload.contains("100.0"));
        }

        #[test]
        fn build_record_creates_future_record() {
            let producer = create_kafka_producer();
            let trade_id = "trade-1";
            let payload = "payload";

            let record = producer.build_record(trade_id, payload);

            // Fields are private, so just confirm it compiles as a FutureRecord
            let _: FutureRecord<'_, str, str> = record;
        }
    }
}
