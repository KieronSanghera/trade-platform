use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProducerError {
    #[error("Kafka transport error: {0}")]
    KafkaTransportError(#[from] rdkafka::error::KafkaError),

    #[error("Kafka topic is missing")]
    TopicMissing,

    #[error("Connection to broker failed: {0}")]
    ConnectionFailed(rdkafka::error::KafkaError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed getting config for producer: {0}")]
    ProducerConfigError(#[from] crate::error::ConfigError),
}
