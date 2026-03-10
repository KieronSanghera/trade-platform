pub mod error;
pub mod factory;
pub mod kafka;
pub mod producer_trait;
pub mod stub;

pub use error::ProducerError;
pub use kafka::KafkaProducer;
pub use producer_trait::TradeEventProducer;
pub use stub::StubProducer;
