use axum::{Router, routing::get};

async fn root() -> &'static str {
    "Hello, World!"
}

pub fn app() -> Router {
    Router::new().route("/", get(root))
}
