mod models;
mod routes;
mod utils;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let app = routes::user::app();
    axum::serve(listener, app).await.unwrap();
}
