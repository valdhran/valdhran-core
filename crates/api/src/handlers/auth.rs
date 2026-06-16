use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use valdhran_application::use_cases::{
    authenticate_user::{AuthenticateUserInput, AuthenticateUserUseCase},
    refresh_token::{RefreshTokenInput, RefreshTokenUseCase},
};
use valdhran_domain::{
    repositories::tenant_repository::TenantRepository,
    value_objects::tenant_slug::TenantSlug,
};
use crate::{app::AppState, errors::ApiError};

// ── POST /auth/login ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub tenant_slug: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Resolver tenant_slug → tenant_id
    let slug = TenantSlug::new(&body.tenant_slug)?;
    let tenant = state.tenant_repo
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| valdhran_domain::errors::DomainError::NotFound(format!("Tenant '{}' not found", body.tenant_slug)))?;

    let input = AuthenticateUserInput {
        tenant_id: tenant.id,
        email: body.email,
        password_plaintext: body.password,
    };
    let output = state.authenticate_user.execute(input).await?;
    Ok(Json(LoginResponse {
        access_token: output.access_token,
        refresh_token: output.refresh_token,
        user_id: output.user_id,
    }))
}

// ── POST /auth/refresh ────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, ApiError> {
    let input = RefreshTokenInput {
        refresh_token: body.refresh_token,
    };
    let output = state.refresh_token.execute(input).await?;
    Ok(Json(RefreshResponse {
        access_token: output.access_token,
        refresh_token: output.refresh_token,
    }))
}
