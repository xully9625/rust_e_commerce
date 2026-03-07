use crate::models::user::{LoginUser, RegisterUser};
use crate::utils::jwt::create_jwt;
use crate::utils::password::{hash_password, verify_password};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use chrono::Utc;
use sqlx::{PgPool, Row};

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(pool)
}

async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<String>, StatusCode> {
    let password_hash = hash_password(&payload.password);

    sqlx::query(
        "INSERT INTO users (username, email, password_hash)
         VALUES ($1, $2, $3)",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(format!("User registered: {}", payload.username)))
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    token: String,
}

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

    let user_id: uuid::Uuid = user
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
