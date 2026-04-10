use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Connection to database error")]
    DatabaseConnectionError,

    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("P quantity is below zero")]
    QuantityBelowZero,

    #[error("Trade average price is below zero")]
    AvgPriceBelowZero,

    #[error("Empty field")]
    EmptyField,

    #[error("Non Positive field")]
    NonPositiveDecimalField,
}
