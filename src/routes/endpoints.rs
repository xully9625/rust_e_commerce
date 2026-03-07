use crate::routes::user::{login, me, register};
use crate::routes::wallet::get_wallet;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/wallet", get(get_wallet))
        .with_state(pool)
}
