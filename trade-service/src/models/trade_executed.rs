use crate::models::trade::{Side, Trade};
use crate::models::types::{NonEmptyString, PositiveDecimal};
use chrono::{DateTime, Utc};
use serde::Serialize;

// Transport Model
#[derive(Debug, Clone, Serialize)]
pub struct TradeExecuted {
    pub trade_id: NonEmptyString,
    pub user_id: NonEmptyString,
    pub asset: NonEmptyString,
    pub price: PositiveDecimal,
    pub quantity: PositiveDecimal,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

impl From<&Trade> for TradeExecuted {
    fn from(trade: &Trade) -> Self {
        TradeExecuted {
            trade_id: trade.trade_id.clone(),
            user_id: trade.user_id.clone(),
            asset: trade.asset.clone(),
            side: trade.side,
            price: trade.price.clone(),
            quantity: trade.quantity.clone(),
            timestamp: trade.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_trade() -> Trade {
        Trade {
            trade_id: NonEmptyString::try_from("trade-1").unwrap(),
            user_id: NonEmptyString::try_from("user-1").unwrap(),
            asset: NonEmptyString::try_from("BTC").unwrap(),
            price: PositiveDecimal::try_from("100.0").unwrap(),
            quantity: PositiveDecimal::try_from("0.5").unwrap(),
            side: Side::Buy,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn trade_to_trade_executed() {
        let trade = create_trade();
        let trade_executed = TradeExecuted::from(&trade);
        assert_eq!(trade.trade_id, trade_executed.trade_id);
        assert_eq!(trade.user_id, trade_executed.user_id);
        assert_eq!(trade.asset, trade_executed.asset);
        assert_eq!(trade.price, trade_executed.price);
        assert_eq!(trade.quantity, trade_executed.quantity);
        assert_eq!(trade.side, trade_executed.side);
        assert_eq!(trade.timestamp, trade_executed.timestamp);
    }
}
