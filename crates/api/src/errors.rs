use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use valdhran_domain::errors::DomainError;

pub type ApiResult<T> = Result<T, ApiError>;

pub struct ApiError(DomainError);

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        ApiError(e)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError(DomainError::Validation(e.to_string()))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            DomainError::Validation(msg)  => (StatusCode::BAD_REQUEST, msg),
            DomainError::NotFound(msg)    => (StatusCode::NOT_FOUND, msg),
            DomainError::Conflict(msg)    => (StatusCode::CONFLICT, msg),
            DomainError::Unauthorized     => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            DomainError::Forbidden(msg)   => (StatusCode::FORBIDDEN, msg),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
