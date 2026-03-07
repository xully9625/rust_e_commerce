use crate::utils::jwt::verify_jwt;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use uuid::Uuid;

pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let auth_header = parts.headers.get("Authorization").cloned();

        async move {
            let auth_header = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;
            let auth_header = auth_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(StatusCode::BAD_REQUEST)?;

            let claims = verify_jwt(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

            Ok(AuthUser {
                user_id: claims.sub,
            })
        }
    }
}
