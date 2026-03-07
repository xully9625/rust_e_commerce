use crate::error::AppError;
use crate::handler::user::AuthUser;
use crate::models::product::{CreateProduct, Product, UpdateProduct};

use axum::Router;
use axum::routing::{delete, get, post, put}; // Added missing imports
use axum::{Json, extract::State, http::StatusCode};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub fn product_routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(create_product))
        .route("/", get(get_my_products))
        .route("/{id}", put(update_product))
        .route("/{id}", delete(delete_product))
}

pub async fn create_product(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<CreateProduct>,
) -> Result<Json<String>, AppError> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        "INSERT INTO products (seller_id, name, description, price, stock)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id",
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.price as i32)
    .bind(payload.stock as i32)
    .fetch_one(&mut *tx)
    .await?; // sqlx::Error converts to AppError automatically

    let product_id: Uuid = row.get("id");
    tx.commit().await?;

    Ok(Json(format!(
        "Product '{}' created (ID: {})",
        payload.name, product_id
    )))
}

pub async fn get_my_products(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<Product>>, AppError> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE seller_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(products))
}

pub async fn update_product(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateProduct>,
) -> Result<Json<String>, AppError> {
    let result = sqlx::query(
        "
        UPDATE products
        SET
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            price = COALESCE($3, price),
            stock = COALESCE($4, stock)
        WHERE id = $5 AND seller_id = $6
        ",
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.price.map(|p| p as i32))
    .bind(payload.stock.map(|s| s as i32))
    .bind(product_id)
    .bind(user_id)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json("Product updated successfully".to_string()))
}

pub async fn delete_product(
    State(pool): State<PgPool>,
    AuthUser { user_id }: AuthUser,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM products WHERE id = $1 AND seller_id = $2")
        .bind(product_id)
        .bind(user_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
