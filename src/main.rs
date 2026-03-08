mod error;
mod handler;
mod models;
mod routes;
mod utils;
use dotenvy;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_app=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is required");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    // 1. Get the PORT from .env
    let port = std::env::var("PORT").expect("PORT environment variable is required");

    // 2. Format the address string correctly
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to port");

    // 3. Log the actual address being used
    tracing::info!("Server is starting on http://{}", addr);

    let app = routes::endpoints::app(pool);
    axum::serve(listener, app).await.unwrap();
}
