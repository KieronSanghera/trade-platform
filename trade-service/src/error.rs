use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TradeError {
    #[error("trade_id is required")]
    MissingTradeId,

    #[error("user_id is required")]
    MissingUserId,

    #[error("asset is missing")]
    MissingAsset,

    #[error("invalid price")]
    InvalidPrice,

    #[error("invalid quantity")]
    InvalidQuantity,

    #[error("timestamp is required")]
    MissingTimestamp,

    #[error("timestamp conversion failed")]
    FailedTimestampConversion,

    #[error("invalid side")]
    InvalidSide,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid Kafka config: {0}")]
    InvalidKafkaConfig(#[from] envy::Error),
}
