use crate::handler::user::AuthUser;
use crate::models::user::{LoginUser, RegisterUser};
use crate::utils::jwt::create_jwt;
use crate::utils::password::{hash_password, verify_password};

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct LoginResponse {
    token: String,
}

// ----------------- Register ------------------------
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<String>, StatusCode> {
    let password_hash = hash_password(&payload.password);

    let user = sqlx::query(
        "INSERT INTO users (username, email, password_hash)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id: Uuid = user
        .try_get("id")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("INSERT INTO wallets (user_id, balance) VALUES ($1, 0)")
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();

    Ok(Json(format!("User registered: {}", payload.username)))
}

// ----------------- Login ---------------------------
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    let user_id: Uuid = user
        .try_get("id")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let password_hash: String = user
        .try_get("password_hash")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !verify_password(&password_hash, &payload.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let exp = (Utc::now().timestamp() + 3600) as usize;
    let token = create_jwt(user_id, exp);

    Ok(Json(LoginResponse { token }))
}

// ----------------- /me -----------------------------
pub async fn me(AuthUser { user_id }: AuthUser) -> Json<String> {
    Json(format!("Your user ID is: {}", user_id))
}
