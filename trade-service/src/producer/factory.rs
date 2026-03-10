use crate::config::KafkaConfig;
use crate::config::TradeServiceBackend;
use crate::producer::KafkaProducer;
use crate::producer::StubProducer;
use crate::producer::error::ProducerError;
use crate::producer::producer_trait::TradeEventProducer;
use std::sync::Arc;

// Factory for backend publisher
pub async fn build_producer(
    backend: &TradeServiceBackend,
) -> Result<Arc<dyn TradeEventProducer>, ProducerError> {
    // Read backend selection from environment
    let producer: Arc<dyn TradeEventProducer> = match backend {
        TradeServiceBackend::Stub => Arc::new(StubProducer),
        TradeServiceBackend::Kafka => {
            let kafka_config = KafkaConfig::from_env()?;
            let producer = KafkaProducer::new(&kafka_config)?;
            Arc::new(producer)
        }
    };
    Ok(producer)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod stub {
        use super::*;

        #[tokio::test]
        async fn stub_backend_returns_stub_producer() {
            let backend = TradeServiceBackend::Stub;
            let producer = build_producer(&backend).await.unwrap();
            assert!(producer.readiness_check().await.is_ok());
        }
    }

    mod kafka {
        use super::*;

        #[serial_test::serial]
        #[test]
        fn kafka_backend_with_env_returns_producer() {
            temp_env::with_vars(
                [
                    ("KAFKA_BROKER", Some("localhost:9092")),
                    ("KAFKA_TOPIC", Some("trades")),
                ],
                || {
                    let backend = TradeServiceBackend::Kafka;
                    let producer = tokio_test::block_on(build_producer(&backend));
                    assert!(producer.is_ok());
                },
            );
        }

        #[serial_test::serial]
        #[test]
        fn kafka_backend_without_env_returns_error() {
            temp_env::with_vars(
                [
                    ("KAFKA_BROKER", None::<&str>),
                    ("KAFKA_TOPIC", None::<&str>),
                ],
                || {
                    let backend = TradeServiceBackend::Kafka;
                    let producer = tokio_test::block_on(build_producer(&backend));
                    assert!(producer.is_err());
                },
            );
        }
    }
}
