use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub buyer_id: Option<Uuid>,
    pub total_price: i32,
    pub created_at: Option<NaiveDateTime>,
}
