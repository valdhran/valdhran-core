use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use valdhran_domain::errors::DomainError;

pub struct ApiError(DomainError);

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            DomainError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            DomainError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            DomainError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            DomainError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
