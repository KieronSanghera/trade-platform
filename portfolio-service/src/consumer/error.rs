use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConsumerError {
    #[error("Kafka transport error: {0}")]
    KafkaTransportError(#[from] rdkafka::error::KafkaError),

    #[error("Kafka topic is missing")]
    TopicMissing,

    #[error("Kafka payload is empty")]
    EmptyPayload,

    #[error("Payload is invalid bytes")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error("Connection to broker failed: {0}")]
    ConnectionFailed(rdkafka::error::KafkaError),

    #[error("Serialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("Failed getting config for consumer: {0}")]
    ConsumerConfigError(#[from] crate::error::ConfigError),

    // Error to be used to demonstrate, Real system would work differently
    #[error("Bad message - cannot be processed: {0}")]
    BadMessage(String),

    #[error("Infrastructure Error - cannot be processed: {0}")]
    InfraError(String),
}
