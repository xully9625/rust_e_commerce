use crate::handler::user::AuthUser;
use crate::models::wallet::Wallet;
use axum::routing::get;
use axum::{Json, Router, extract::State, http::StatusCode};
use sqlx::{PgPool, Row};

pub fn wallet_routes() -> Router<PgPool> {
    Router::new().route("/", get(get_wallet))
}

pub async fn get_wallet(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Wallet>, StatusCode> {
    let wallet = sqlx::query("SELECT id, user_id, balance FROM wallets WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let wallet = Wallet {
        id: wallet
            .try_get("id")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        user_id: wallet
            .try_get("user_id")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        balance: wallet
            .try_get("balance")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    };

    Ok(Json(wallet))
}
