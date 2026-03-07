use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub seller_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price: i32,
    pub stock: i32,
    pub created_at: Option<NaiveDateTime>,
}
