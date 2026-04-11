use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Position quantity is below zero")]
    QuantityBelowZero,

    #[error("Position average price is below zero")]
    AvgPriceBelowZero,

    #[error("Empty field")]
    EmptyField,

    #[error("Non Positive field")]
    NonPositiveDecimalField,
}
