use crate::errors::CustomTypeError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonEmptyString(String);

impl TryFrom<&str> for NonEmptyString {
    type Error = CustomTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(CustomTypeError::EmptyString);
        }

        Ok(Self(value.to_string()))
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = CustomTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(CustomTypeError::EmptyString);
        }

        Ok(Self(value))
    }
}

impl Deref for NonEmptyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositiveDecimal(Decimal);

impl TryFrom<&str> for PositiveDecimal {
    type Error = CustomTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parsed = value
            .parse::<Decimal>()
            .map_err(|_| CustomTypeError::InvalidDecimal)?;

        if parsed <= Decimal::ZERO {
            return Err(CustomTypeError::NonPositiveDecimal);
        }

        Ok(Self(parsed))
    }
}

impl TryFrom<Decimal> for PositiveDecimal {
    type Error = CustomTypeError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        if value <= Decimal::ZERO {
            return Err(CustomTypeError::NonPositiveDecimal);
        };

        Ok(Self(value))
    }
}

impl Deref for PositiveDecimal {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod positive_decimal {
        use super::*;

        #[test]
        fn valid_positive_decimal() {
            let decimal = PositiveDecimal::try_from("1.0").unwrap();
            assert_eq!(*decimal, Decimal::ONE);
        }

        #[test]
        fn invalid_decimal() {
            assert_eq!(
                PositiveDecimal::try_from("not a number").unwrap_err(),
                CustomTypeError::InvalidDecimal
            )
        }

        #[test]
        fn non_positive_decimal() {
            assert_eq!(
                PositiveDecimal::try_from("-1.0").unwrap_err(),
                CustomTypeError::NonPositiveDecimal
            );
            assert_eq!(
                PositiveDecimal::try_from("0").unwrap_err(),
                CustomTypeError::NonPositiveDecimal
            )
        }
    }

    mod non_empty_string {
        use super::*;

        #[test]
        fn valid_non_empty_string() {
            assert!(NonEmptyString::try_from("im a string").is_ok())
        }

        #[test]
        fn empty_string() {
            assert_eq!(
                NonEmptyString::try_from("").unwrap_err(),
                CustomTypeError::EmptyString
            );
            assert_eq!(
                NonEmptyString::try_from(" ").unwrap_err(),
                CustomTypeError::EmptyString
            );
        }
    }
}
