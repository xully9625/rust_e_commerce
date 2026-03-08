use crate::error::AppError;
use crate::handler::user::AuthUser;
use crate::models::product::{CreateProduct, Product, UpdateProduct};

use axum::Router;
use axum::routing::{delete, get, post, put}; // Added missing imports
use axum::{Json, extract::State, http::StatusCode};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct BuyRequest {
    pub product_id: Uuid,
    pub quantity: i32,
}

pub fn product_routes() -> Router<PgPool> {
    Router::new()
        .route("/buy", post(buy_product))
        .route("/", get(list_all_products)) // GET /products
        .route("/", post(create_product)) // POST /products
        .route("/me", get(get_my_products)) // GET /products/me
        .route("/{id}", put(update_product)) // PUT /products/:id
        .route("/{id}", delete(delete_product)) // DELETE /products/:id
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
    .bind(payload.description)
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

pub async fn list_all_products(State(pool): State<PgPool>) -> Result<Json<Vec<Product>>, AppError> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(products))
}

pub async fn buy_product(
    State(pool): State<PgPool>,
    AuthUser { user_id: buyer_id }: AuthUser,
    Json(payload): Json<BuyRequest>,
) -> Result<Json<String>, AppError> {
    // 1. Start a transaction
    let mut tx = pool.begin().await?;

    // 2. Get product details and check stock (runtime query + row extraction)
    let product_row =
        sqlx::query("SELECT seller_id, price, stock, name FROM products WHERE id = $1 FOR UPDATE")
            .bind(payload.product_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound)?;

    let seller_id: Uuid = product_row.try_get("seller_id")?;
    let price: i32 = product_row.try_get("price")?;
    let stock: i32 = product_row.try_get("stock")?;
    let name: String = product_row.try_get("name")?;

    if stock < payload.quantity {
        return Err(AppError::BadRequest("Out of stock".into()));
    }

    let total_cost = price * payload.quantity;

    // 3. Check buyer's wallet balance (runtime query + row extraction)
    let buyer_wallet_row = sqlx::query("SELECT balance FROM wallets WHERE user_id = $1 FOR UPDATE")
        .bind(buyer_id)
        .fetch_one(&mut *tx)
        .await?;

    let buyer_balance: i32 = buyer_wallet_row.try_get("balance")?;

    if buyer_balance < total_cost {
        return Err(AppError::BadRequest(
            "Insufficient in-house currency".into(),
        ));
    }

    // 4. Subtract from Buyer
    sqlx::query("UPDATE wallets SET balance = balance - $1 WHERE user_id = $2")
        .bind(total_cost)
        .bind(buyer_id)
        .execute(&mut *tx)
        .await?;

    // 5. Add to Seller
    sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE user_id = $2")
        .bind(total_cost)
        .bind(seller_id)
        .execute(&mut *tx)
        .await?;

    // 6. Reduce Product Stock
    sqlx::query("UPDATE products SET stock = stock - $1 WHERE id = $2")
        .bind(payload.quantity)
        .bind(payload.product_id)
        .execute(&mut *tx)
        .await?;

    // 7. Create the Activity Log (The combined Buy/Sell record)
    sqlx::query(
        "INSERT INTO activity_logs (buyer_id, seller_id, product_id, amount_paid, quantity)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(buyer_id)
    .bind(seller_id)
    .bind(payload.product_id)
    .bind(total_cost)
    .bind(payload.quantity)
    .execute(&mut *tx)
    .await?;

    // 8. Commit everything
    tx.commit().await?;

    Ok(Json(format!(
        "Successfully bought {} x {}",
        payload.quantity, name
    )))
}
