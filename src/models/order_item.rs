use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: i32,
    pub price: i32,
}
