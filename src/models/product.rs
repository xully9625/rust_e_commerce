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

#[derive(Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: Option<String>,
    pub price: i32,
    pub stock: i32,
}

#[derive(serde::Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<u32>,
    pub stock: Option<u32>,
}
