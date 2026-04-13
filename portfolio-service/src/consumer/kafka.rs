use crate::{
    config::KafkaConfig,
    consumer::{ConsumerError, Handler, consumer_trait::TradeEventConsumer},
};
use rdkafka::{
    ClientConfig, Message,
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::BorrowedMessage,
    producer::{FutureProducer, FutureRecord},
};
use shared::TradeExecuted;
use std::{str::from_utf8, time::Duration};

// Struct for kafka consumer that also ensures dead letter publishing
pub struct KafkaConsumer {
    consumer: StreamConsumer,
    topic: String,
    dead_letter_producer: FutureProducer,
    dead_letter_topic: String,
}

impl KafkaConsumer {
    // Build consumer with dead letter producer baked in
    pub fn new(config: &KafkaConfig) -> Result<Self, ConsumerError> {
        let consumer = ClientConfig::new()
            .set("bootstrap.servers", &config.broker)
            .set("group.id", &config.group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()?;

        let dead_letter_producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.broker)
            .set("message.timeout.ms", "5000")
            .create()?;

        Ok(Self {
            consumer,
            topic: config.topic.clone(),
            dead_letter_producer,
            dead_letter_topic: config.dead_letter_topic.clone(),
        })
    }

    // Get bytes from the message
    pub fn get_payload_bytes<'a>(
        message: &'a BorrowedMessage<'a>,
    ) -> Result<&'a [u8], ConsumerError> {
        let payload_bytes = message.payload().ok_or(ConsumerError::EmptyPayload)?;
        Ok(payload_bytes)
    }

    // Convert Bytes to TradeExecuted transport model
    pub fn get_trade_from_message_bytes(bytes: &[u8]) -> Result<TradeExecuted, ConsumerError> {
        let payload = from_utf8(bytes)?;
        let trade: TradeExecuted = serde_json::from_str(payload)?;
        Ok(trade)
    }

    // Publishing to dead letter
    async fn send_to_dead_letter(&self, message_as_bytes: &[u8]) -> Result<(), ConsumerError> {
        let record: FutureRecord<str, [u8]> =
            FutureRecord::to(&self.dead_letter_topic).payload(message_as_bytes);

        self.dead_letter_producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| ConsumerError::KafkaTransportError(err))?;
        Ok(())
    }

    // Handler for bad messages that need to be dead lettered
    async fn handle_bad_message(
        &self,
        message: &BorrowedMessage<'_>,
        bytes: &[u8],
    ) -> Result<(), ConsumerError> {
        tracing::info!("Pushing to dead letter queue");
        self.send_to_dead_letter(bytes).await?;
        if let Err(e) = self.consumer.commit_message(message, CommitMode::Sync) {
            tracing::error!(error = ?e, "Failed to commit offset for bad message");
        }
        Ok(())
    }
}

// Implement contract from trait
#[async_trait::async_trait]
impl TradeEventConsumer for KafkaConsumer {
    // Start Consumer loop
    async fn start(&self, handler: Handler) -> Result<(), ConsumerError> {
        self.consumer.subscribe(&[&self.topic])?;
        loop {
            let message = self.consumer.recv().await?;
            tracing::info!("consumed message");

            let message_bytes = match KafkaConsumer::get_payload_bytes(&message) {
                Err(ConsumerError::EmptyPayload) => {
                    tracing::error!(
                        topic = %self.topic,
                        partition = ?message.partition(),
                        offset = ?message.offset(),
                        "Empty payload received - possible tampering, skipping"
                    );
                    self.consumer.commit_message(&message, CommitMode::Sync)?;
                    continue;
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Unexpected error processing message");
                    continue;
                }
                Ok(bytes) => bytes,
            };

            let trade = match KafkaConsumer::get_trade_from_message_bytes(message_bytes) {
                Ok(t) => t,
                Err(ConsumerError::InvalidUtf8(e)) => {
                    tracing::error!(error = ?e, "Invalid UTF-8 in payload - possible bad actor");
                    self.handle_bad_message(&message, message_bytes).await?;
                    continue;
                }
                Err(ConsumerError::DeserializationError(e)) => {
                    tracing::error!(error = ?e, "Failed to deserialize payload - possible bad actor or schema mismatch");
                    self.handle_bad_message(&message, message_bytes).await?;
                    continue;
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Unexpected error processing message");
                    self.handle_bad_message(&message, message_bytes).await?;
                    continue;
                }
            };

            tracing::info!(trade_id=%trade.trade_id, "Successfully Consumed Trade - Handling Trade");
            match handler(trade).await {
                Ok(_) => {
                    self.consumer.commit_message(&message, CommitMode::Sync)?;
                }
                Err(ConsumerError::BadMessage(_)) => {
                    self.handle_bad_message(&message, message_bytes).await?;
                }
                Err(e) => {
                    return Err(e); // infrastructure - halt and retry
                }
            }
        }
    }

    // Readiness handler
    async fn readiness_check(&self) -> Result<(), ConsumerError> {
        let metadata = self
            .consumer
            .client()
            .fetch_metadata(None, Duration::from_secs(1))
            .map_err(ConsumerError::ConnectionFailed)?;

        // Check both topics exist
        let topic_exists = metadata.topics().iter().any(|t| t.name() == self.topic);
        let dead_letter_topic_exists = metadata
            .topics()
            .iter()
            .any(|t| t.name() == self.dead_letter_topic);

        if !topic_exists || !dead_letter_topic_exists {
            return Err(ConsumerError::TopicMissing);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use shared::{NonEmptyString, PositiveDecimal, Side};

    use super::*;

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

    #[tokio::test]
    async fn consumer_creation_works() {
        let consumer = KafkaConsumer::new(&KafkaConfig {
            broker: "broker".to_string(),
            topic: "topic".to_string(),
            group_id: "group_id".to_string(),
            dead_letter_topic: "deadletter".to_string(),
        });
        assert!(consumer.is_ok())
    }

    #[test]
    fn valid_get_trade_from_message_bytes() {
        let initial_trade = create_trade_executed();
        let bytes: Vec<u8> = serde_json::to_vec(&initial_trade).unwrap();
        let rebuilt_trade_result = KafkaConsumer::get_trade_from_message_bytes(&bytes);

        assert!(rebuilt_trade_result.is_ok());

        let rebuilt_trade = rebuilt_trade_result.unwrap();
        assert_eq!(initial_trade.user_id, rebuilt_trade.user_id);
        assert_eq!(initial_trade.trade_id, rebuilt_trade.trade_id);
        assert_eq!(initial_trade.asset, rebuilt_trade.asset);
        assert_eq!(initial_trade.price, rebuilt_trade.price);
        assert_eq!(initial_trade.quantity, rebuilt_trade.quantity);
        assert_eq!(initial_trade.timestamp, rebuilt_trade.timestamp);
    }

    #[test]
    fn invalid_bytes_get_trade_from_message_bytes() {
        let invalid_bytes = vec![1];
        let invalid_trade_result = KafkaConsumer::get_trade_from_message_bytes(&invalid_bytes);

        assert!(invalid_trade_result.is_err())
    }

    #[test]
    fn invalid_trade_get_trade_from_message_bytes() {
        let invalid_bytes = b"hello world";
        let invalid_trade_result = KafkaConsumer::get_trade_from_message_bytes(invalid_bytes);
        assert!(invalid_trade_result.is_err())
    }
}
