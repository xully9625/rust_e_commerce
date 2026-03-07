use crate::utils::jwt::verify_jwt;
use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use serde_json::{Value, json};
use uuid::Uuid;

pub struct AuthUser {
    pub user_id: Uuid,
}

// Define a custom rejection type to make the code cleaner
pub type AuthRejection = (StatusCode, Json<Value>);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        // Extract the header before moving into the async block
        let auth_header = parts.headers.get("Authorization").cloned();

        async move {
            // 1. Check if header exists
            let auth_header = auth_header.ok_or((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing Authorization header" })),
            ))?;

            // 2. Check if header is valid string
            let auth_str = auth_header.to_str().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Invalid characters in Authorization header" })),
                )
            })?;

            // 3. Check for Bearer prefix
            let token = auth_str.strip_prefix("Bearer ").ok_or((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Authorization header must start with 'Bearer '" })),
            ))?;

            // 4. Verify JWT and extract claims
            let claims = verify_jwt(token).map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "Invalid or expired token",
                        "details": format!("{}", e) // Optional: show the specific JWT error
                    })),
                )
            })?;

            Ok(AuthUser {
                user_id: claims.sub,
            })
        }
    }
}
