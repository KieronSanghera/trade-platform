use rust_decimal::Decimal;

pub struct RawPosition {
    pub user_id: String,
    pub asset: String,
    pub quantity: Decimal,
    pub avg_price: Decimal,
}
