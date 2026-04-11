use crate::{
    config::PostgresConfig,
    db::{error::DbError, model::RawPosition},
    models::position::PortfolioPosition,
};
use rust_decimal::Decimal;
use shared::{CustomTypeError, PositiveDecimal, Side, TradeExecuted};
use sqlx::PgPool;
use tracing::warn;

pub struct PostgresDB {
    pool: PgPool,
}

impl PostgresDB {
    pub async fn new(config: &PostgresConfig) -> Result<Self, DbError> {
        Ok(Self {
            pool: PgPool::connect(&config.url).await?,
        })
    }

    pub async fn readiness_check(&self) -> Result<(), DbError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn handle_position(&self, trade: &TradeExecuted) -> Result<(), DbError> {
        let existing = sqlx::query!(
            "SELECT quantity, avg_price FROM positions WHERE user_id = $1 AND asset = $2",
            &*trade.user_id,
            &*trade.asset
        )
        .fetch_optional(&self.pool)
        .await?;

        match existing {
            None => {
                tracing::info!(user_id = &*trade.user_id, "New position");
                match trade.side {
                    Side::Sell => {
                        warn!(
                            asset = &*trade.asset,
                            user = &*trade.user_id,
                            "Attempting sell with no current position"
                        );
                        return Err(DbError::QuantityBelowZero);
                    }
                    Side::Buy => {
                        let position = PortfolioPosition {
                            user_id: trade.user_id.clone(),
                            asset: trade.asset.clone(),
                            quantity: trade.quantity.clone(),
                            avg_price: trade.price.clone(),
                        };
                        self.upsert_position(&position).await?;
                    }
                }
            }
            Some(row) => {
                tracing::info!(user_id = &*trade.user_id, "Existing position found");
                let (new_quantity, new_avg_price) = apply_trade_to_position(
                    row.quantity,
                    row.avg_price,
                    *trade.quantity,
                    *trade.price,
                    &trade.side,
                );

                match new_quantity {
                    quantity if quantity < Decimal::ZERO => return Err(DbError::QuantityBelowZero),
                    quantity if quantity == Decimal::ZERO => {
                        self.delete_position(&trade.user_id, &trade.asset).await?;
                    }
                    _ => {
                        let position = PortfolioPosition {
                            user_id: trade.user_id.clone(),
                            asset: trade.asset.clone(),
                            quantity: PositiveDecimal::try_from(new_quantity)
                                .map_err(|_| DbError::QuantityBelowZero)?,
                            avg_price: PositiveDecimal::try_from(new_avg_price)
                                .map_err(|_| DbError::AvgPriceBelowZero)?,
                        };
                        self.upsert_position(&position).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn upsert_position(&self, position: &PortfolioPosition) -> Result<(), DbError> {
        sqlx::query!(
            "INSERT INTO positions (user_id, asset, quantity, avg_price)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, asset)
            DO UPDATE SET quantity = $3, avg_price = $4",
            &*position.user_id,
            &*position.asset,
            *position.quantity,
            *position.avg_price
        )
        .execute(&self.pool)
        .await?;
        tracing::info!("Successfully updated position");
        Ok(())
    }

    async fn delete_position(&self, user_id: &str, asset: &str) -> Result<(), DbError> {
        sqlx::query!(
            "DELETE FROM positions WHERE user_id = $1 AND asset = $2",
            user_id,
            asset
        )
        .execute(&self.pool)
        .await?;
        tracing::info!(
            user_id = user_id,
            asset = asset,
            "Position deleted - quantity reached zero"
        );
        Ok(())
    }

    pub async fn get_portfolio(&self, user_id: &str) -> Result<Vec<PortfolioPosition>, DbError> {
        let rows = sqlx::query_as!(
            RawPosition,
            "SELECT user_id, asset, quantity, avg_price FROM positions WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let positions: Vec<PortfolioPosition> = rows
            .into_iter()
            .map(|raw| {
                PortfolioPosition::try_from(raw).map_err(|e| match e {
                    CustomTypeError::EmptyString => DbError::EmptyField,
                    _ => DbError::NonPositiveDecimalField,
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(positions)
    }
}

fn apply_trade_to_position(
    // No zero decimals as the possible 0 division values are passed down as positive decimals
    current_quantity: Decimal,
    current_avg_price: Decimal,
    trade_quantity: Decimal,
    trade_price: Decimal,
    side: &Side,
) -> (Decimal, Decimal) {
    match side {
        Side::Buy => {
            let new_quantity = current_quantity + trade_quantity;
            let new_avg_price = ((current_quantity * current_avg_price)
                + (trade_quantity * trade_price))
                / new_quantity;
            (new_quantity, new_avg_price)
        }
        Side::Sell => (current_quantity - trade_quantity, current_avg_price),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod apply_trade_to_position {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn buy_empty_position() {
            let (updated_quantity, updated_avg_price) = apply_trade_to_position(
                Decimal::new(0, 0),
                Decimal::new(0, 0),
                Decimal::new(1, 0),
                Decimal::new(10, 0),
                &Side::Buy,
            );
            assert_eq!(updated_quantity, Decimal::ONE);
            assert_eq!(updated_avg_price, Decimal::TEN);
        }

        #[test]
        fn buy_existing_position() {
            let (updated_quantity, updated_avg_price) = apply_trade_to_position(
                Decimal::new(1, 0),
                Decimal::new(5, 0),
                Decimal::new(1, 0),
                Decimal::new(10, 0),
                &Side::Buy,
            );
            assert_eq!(updated_quantity, Decimal::TWO);
            assert_eq!(updated_avg_price, Decimal::from_str("7.5").unwrap());
        }

        #[test]
        fn sell_existing_position() {
            let (updated_quantity, updated_avg_price) = apply_trade_to_position(
                Decimal::new(10, 0),
                Decimal::new(5, 0),
                Decimal::new(8, 0),
                Decimal::new(10, 0),
                &Side::Sell,
            );
            assert_eq!(updated_quantity, Decimal::TWO);
            assert_eq!(updated_avg_price, Decimal::new(5, 0));
        }

        #[test]
        fn sell_whole_position() {
            let (updated_quantity, updated_avg_price) = apply_trade_to_position(
                Decimal::new(10, 0),
                Decimal::new(5, 0),
                Decimal::new(10, 0),
                Decimal::new(10, 0),
                &Side::Sell,
            );
            assert_eq!(updated_quantity, Decimal::ZERO);
            assert_eq!(updated_avg_price, Decimal::new(5, 0)); // unchanged as it is thrown away
        }
    }
}
