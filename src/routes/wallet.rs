use crate::error::AppError;
use crate::handler::user::AuthUser;
use crate::models::wallet::Wallet;
use axum::routing::get;
use axum::{Json, Router, extract::State};
use sqlx::{PgPool, Row};

pub fn wallet_routes() -> Router<PgPool> {
    Router::new().route("/", get(get_wallet))
}

pub async fn get_wallet(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Wallet>, AppError> {
    let wallet = sqlx::query("SELECT id, user_id, balance FROM wallets WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| AppError::NotFound)?;

    let wallet = Wallet {
        id: wallet
            .try_get("id")
            .map_err(|_| AppError::InternalServerError("wallet id not found".to_string()))?,
        user_id: wallet
            .try_get("user_id")
            .map_err(|_| AppError::InternalServerError("user id nor found".to_string()))?,
        balance: wallet
            .try_get("balance")
            .map_err(|_| AppError::InternalServerError("balance not found".to_string()))?,
    };

    Ok(Json(wallet))
}
