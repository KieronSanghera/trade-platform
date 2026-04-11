use shared::{CustomTypeError, NonEmptyString, PositiveDecimal};

use crate::{db::model::RawPosition, portfolio::Position};

pub struct PortfolioPosition {
    pub user_id: NonEmptyString,
    pub asset: NonEmptyString,
    pub quantity: PositiveDecimal,
    pub avg_price: PositiveDecimal,
}

impl TryFrom<RawPosition> for PortfolioPosition {
    type Error = CustomTypeError;

    fn try_from(raw_position: RawPosition) -> Result<Self, Self::Error> {
        Ok(PortfolioPosition {
            user_id: NonEmptyString::try_from(raw_position.user_id.as_str())?,
            asset: NonEmptyString::try_from(raw_position.asset.as_str())?,
            quantity: PositiveDecimal::try_from(raw_position.quantity)?,
            avg_price: PositiveDecimal::try_from(raw_position.avg_price)?,
        })
    }
}

impl From<PortfolioPosition> for Position {
    fn from(portfolio_position: PortfolioPosition) -> Self {
        Position {
            asset: portfolio_position.asset.to_string(),
            net_quantity: portfolio_position.quantity.to_string(),
            average_price: portfolio_position.avg_price.to_string(),
        }
    }
}
