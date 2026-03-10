use crate::error::TradeError;
use crate::models::types::{NonEmptyString, PositiveDecimal};
use crate::trade::{TradeRequest, TradeSide};
use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use serde::Serialize;
use std::fmt;

// Domain Model
#[derive(Debug)]
pub struct Trade {
    pub trade_id: NonEmptyString,
    pub user_id: NonEmptyString,
    pub asset: NonEmptyString,
    pub price: PositiveDecimal,
    pub quantity: PositiveDecimal,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

// Enum for trade side
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

// Helper for outputting Side
impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell"),
        }
    }
}

// Helper to convert from incoming model to domain model
impl TryFrom<&TradeRequest> for Trade {
    type Error = TradeError;

    fn try_from(request: &TradeRequest) -> Result<Self, Self::Error> {
        let trade_id = Self::parse_trade_id(&request.trade_id)?;
        let user_id = Self::parse_user_id(&request.user_id)?;
        let asset = Self::parse_asset(&request.asset)?;
        let price = Self::parse_price(&request.price)?;
        let quantity = Self::parse_quantity(&request.quantity)?;
        let side = Self::parse_side(&request.side())?;
        let timestamp = Self::parse_timestamp(&request.timestamp)?;

        let trade = Trade {
            trade_id,
            user_id,
            asset,
            price,
            quantity,
            side,
            timestamp,
        };
        Ok(trade)
    }
}

// Support functions to parse data
impl Trade {
    fn parse_trade_id(id: &str) -> Result<NonEmptyString, TradeError> {
        let checked_id = NonEmptyString::try_from(id).map_err(|_| TradeError::MissingTradeId)?;
        Ok(checked_id)
    }

    fn parse_user_id(id: &str) -> Result<NonEmptyString, TradeError> {
        let checked_id = NonEmptyString::try_from(id).map_err(|_| TradeError::MissingUserId)?;
        Ok(checked_id)
    }

    fn parse_price(price: &str) -> Result<PositiveDecimal, TradeError> {
        let price = PositiveDecimal::try_from(price).map_err(|_| TradeError::InvalidPrice)?;
        Ok(price)
    }

    fn parse_quantity(quantity: &str) -> Result<PositiveDecimal, TradeError> {
        let quantity =
            PositiveDecimal::try_from(quantity).map_err(|_| TradeError::InvalidQuantity)?;
        Ok(quantity)
    }

    fn parse_timestamp(timestamp: &Option<Timestamp>) -> Result<DateTime<Utc>, TradeError> {
        let prost_ts = timestamp.ok_or(TradeError::MissingTimestamp)?;

        DateTime::<Utc>::from_timestamp(prost_ts.seconds, prost_ts.nanos as u32)
            .ok_or(TradeError::FailedTimestampConversion)
    }

    fn parse_side(trade_side: &TradeSide) -> Result<Side, TradeError> {
        match trade_side {
            TradeSide::Buy => Ok(Side::Buy),
            TradeSide::Sell => Ok(Side::Sell),
            _ => Err(TradeError::InvalidSide),
        }
    }

    fn parse_asset(asset: &str) -> Result<NonEmptyString, TradeError> {
        let checked_asset =
            NonEmptyString::try_from(asset).map_err(|_| TradeError::MissingAsset)?;
        Ok(checked_asset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trade::TradeRequest;
    use prost_types::Timestamp; // or wherever Timestamp resolves

    fn valid_trade_request() -> TradeRequest {
        TradeRequest {
            trade_id: "t1".into(),
            user_id: "u1".into(),
            asset: "BTC".into(),
            price: "100.0".into(),
            quantity: "1.0".into(),
            side: 1, // or Side::Buy as i32 depending on your proto
            timestamp: Some(Timestamp {
                seconds: 1,
                nanos: 1,
            }),
        }
    }

    mod valid_trade {
        use super::*;

        #[test]
        fn create_trade_from_valid_request() {
            let request = valid_trade_request();
            let trade = Trade::try_from(&request).unwrap();
            assert_eq!(trade.trade_id.to_string(), "t1");
            assert_eq!(trade.user_id.to_string(), "u1");
            assert_eq!(trade.asset.to_string(), "BTC");
            assert_eq!(trade.price.to_string(), "100.0");
            assert_eq!(trade.quantity.to_string(), "1.0");
            assert_eq!(trade.side, Side::Buy);
        }

        #[test]
        fn valid_sell_side() {
            let mut request = valid_trade_request();
            request.side = 2; // whatever maps to Sell
            assert_eq!(Trade::try_from(&request).unwrap().side, Side::Sell);
        }
    }

    mod invalid_trade {
        use super::*;

        #[test]
        fn empty_trade_id() {
            let mut request = valid_trade_request();
            request.trade_id = "".to_string();
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::MissingTradeId
            );
        }

        #[test]
        fn empty_user_id() {
            let mut request = valid_trade_request();
            request.user_id = "".to_string();
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::MissingUserId
            );
        }

        #[test]
        fn invalid_price() {
            let mut request = valid_trade_request();
            request.price = "NotANumber".to_string();
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::InvalidPrice
            );
        }

        #[test]
        fn negative_price() {
            let mut request = valid_trade_request();
            request.price = "-1".to_string();
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::InvalidPrice
            );
        }

        #[test]
        fn invalid_quantity() {
            let mut request = valid_trade_request();
            request.quantity = "NotANumber".to_string();
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::InvalidQuantity
            );
        }

        #[test]
        fn negative_quantity() {
            let mut request = valid_trade_request();
            request.quantity = "-1".to_string();
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::InvalidQuantity
            );
        }

        #[test]
        fn missing_timestamp() {
            let mut request = valid_trade_request();
            request.timestamp = None;
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::MissingTimestamp
            );
        }

        #[test]
        fn invalid_side() {
            let mut request = valid_trade_request();
            request.side = 0;
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::InvalidSide
            );
        }

        #[test]
        fn empty_asset() {
            let mut request = valid_trade_request();
            request.asset = "".to_string();
            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::MissingAsset
            );
        }

        #[test]
        fn invalid_timestamp_conversion() {
            let mut request = valid_trade_request();
            request.timestamp = Some(Timestamp {
                seconds: i64::MAX,
                nanos: 0,
            });

            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::FailedTimestampConversion
            );
        }

        #[test]
        fn unknown_side_value() {
            let mut request = valid_trade_request();
            request.side = 99;

            assert_eq!(
                Trade::try_from(&request).unwrap_err(),
                TradeError::InvalidSide
            );
        }
    }
}
