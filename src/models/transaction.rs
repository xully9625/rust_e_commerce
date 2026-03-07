use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub from_user: Option<Uuid>,
    pub to_user: Option<Uuid>,
    pub amount: i32,
    pub order_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
}
