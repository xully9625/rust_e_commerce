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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    //println!("server is sucefully start at port 3000");
    tracing::info!("Server is starting on http://localhost:3000");

    let app = routes::endpoints::app(pool);
    axum::serve(listener, app).await.unwrap();
}
