use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum CustomTypeError {
    #[error("Field is empty")]
    EmptyString,

    #[error("Decimal is invalid")]
    InvalidDecimal,

    #[error("Decimal is negative")]
    NonPositiveDecimal,
}
