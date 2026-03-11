use crate::types::{NonEmptyString, PositiveDecimal};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// Transport Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecuted {
    pub trade_id: NonEmptyString,
    pub user_id: NonEmptyString,
    pub asset: NonEmptyString,
    pub price: PositiveDecimal,
    pub quantity: PositiveDecimal,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

// Trade Side
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
