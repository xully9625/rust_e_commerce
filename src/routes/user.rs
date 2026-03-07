use crate::error::AppError;
use crate::handler::user::AuthUser;
use crate::models::user::{LoginUser, RegisterUser};
use crate::utils::jwt::create_jwt;
use crate::utils::password::{hash_password, verify_password};

use axum::Router;
use axum::routing::{get, post};
use axum::{Json, extract::State};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct LoginResponse {
    token: String,
}

pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
}

// ----------------- Register ------------------------
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<String>, AppError> {
    let password_hash = hash_password(&payload.password);

    // Use a transaction so both User and Wallet are created together
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        "INSERT INTO users (username, email, password_hash)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&mut *tx) // Use the transaction
    .await?;

    let user_id: Uuid = row.get("id");

    sqlx::query("INSERT INTO wallets (user_id, balance) VALUES ($1, 0)")
        .bind(&user_id)
        .execute(&mut *tx) // Use the transaction
        .await?;

    tx.commit().await?;

    Ok(Json(format!("User registered: {}", payload.username)))
}

// ----------------- Login ---------------------------
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ))?;

    let user_id: Uuid = user.get("id");
    let password_hash: String = user.get("password_hash");

    if !verify_password(&password_hash, &payload.password) {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    let exp = (Utc::now().timestamp() + 3600) as usize;
    let token = create_jwt(user_id, exp);

    Ok(Json(LoginResponse { token }))
}

// ----------------- /me -----------------------------
pub async fn me(AuthUser { user_id }: AuthUser) -> Json<String> {
    Json(format!("Your user ID is: {}", user_id))
}
