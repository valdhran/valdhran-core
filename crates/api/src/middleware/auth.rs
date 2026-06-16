use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;

/// Claims extraídos de un JWT válido — disponibles en handlers via extractor
#[derive(Clone, Debug)]
pub struct AuthClaims {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
}

impl FromRequestParts<AppState> for AuthClaims {
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // Extract bearer token from Authorization header
            let auth_header = parts
                .headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .ok_or_else(|| {
                    (StatusCode::UNAUTHORIZED, Json(json!({"error": "Missing or invalid Authorization header"})))
                        .into_response()
                })?;

            let (user_id, tenant_id) = state
                .jwt_service
                .decode_access_token(auth_header)
                .map_err(|_| {
                    (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid or expired token"})))
                        .into_response()
                })?;

            Ok(AuthClaims { user_id, tenant_id })
        }
    }
}
