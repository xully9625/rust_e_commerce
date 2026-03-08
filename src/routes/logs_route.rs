use axum::{Json, Router, extract::State, routing::get};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, handler::user::AuthUser};

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub product_id: Uuid,
    pub product_name: String, // Joined from products table
    pub amount_paid: i32,
    pub quantity: i32,
    pub created_at: chrono::NaiveDateTime,
}

pub fn log_routes() -> Router<PgPool> {
    Router::new()
        .route("/bought", get(get_buy_logs))
        .route("/sold", get(get_sell_logs))
}

// Get everything the user has BOUGHT
pub async fn get_buy_logs(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<ActivityLog>>, AppError> {
    let logs = sqlx::query_as::<_, ActivityLog>(
        "SELECT l.*, p.name as product_name
         FROM activity_logs l
         JOIN products p ON l.product_id = p.id
         WHERE l.buyer_id = $1
         ORDER BY l.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(logs))
}

// Get everything the user has SOLD
pub async fn get_sell_logs(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<ActivityLog>>, AppError> {
    let logs = sqlx::query_as::<_, ActivityLog>(
        "SELECT l.*, p.name as product_name
         FROM activity_logs l
         JOIN products p ON l.product_id = p.id
         WHERE l.seller_id = $1
         ORDER BY l.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(logs))
}
