use crate::routes::product::product_routes;
use crate::routes::user::user_routes;
use crate::routes::wallet::wallet_routes;

use axum::{Json, Router, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::TraceLayer;
pub fn app(pool: PgPool) -> Router {
    // 1. Define your main routes and provide state
    let api_routes = Router::new()
        .nest("/auth", user_routes())
        .nest("/wallet", wallet_routes())
        .nest("/products", product_routes())
        .fallback(handler_404)
        .with_state(pool);

    // 2. Use fallback_service to wrap everything in NormalizePath
    // This effectively makes NormalizePath the "outer shell" of your app
    Router::new()
        .fallback_service(
            ServiceBuilder::new()
                .layer(NormalizePathLayer::trim_trailing_slash())
                .service(api_routes),
        )
        .layer(TraceLayer::new_for_http())
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "Not Found",
            "message": "Endpoint not found. Check your URL and method."
        })),
    )
}
