use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use valdhran_application::use_cases::{
    authenticate_user::{AuthenticateUserInput, AuthenticateUserUseCase},
    refresh_token::{RefreshTokenInput, RefreshTokenUseCase},
    register_user::{RegisterUserInput, RegisterUserUseCase},
};
use valdhran_infrastructure::{
    auth::jwt_token_service::JwtTokenService,
    db::pg_user_repository::PgUserRepository,
};

use crate::{app_state::SharedState, errors::ApiResult};

// ── Register ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub tenant_id: Uuid,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
}

pub async fn register(
    State(state): State<SharedState>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<RegisterResponse>)> {
    // Obtener schema_name del tenant para construir PgUserRepository
    let tenant = sqlx::query!(
        "SELECT schema_name FROM public.tenants WHERE id = $1 AND status = 'active'",
        body.tenant_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?
    .ok_or_else(|| valdhran_domain::errors::DomainError::NotFound("Tenant not found".into()))?;

    let user_repo = PgUserRepository::new(state.pool.clone(), tenant.schema_name);
    let use_case = RegisterUserUseCase::new(user_repo, state.hasher);

    let output = use_case
        .execute(RegisterUserInput {
            tenant_id: body.tenant_id,
            email: body.email,
            password_plaintext: body.password,
            display_name: body.display_name,
        })
        .await?;

    tracing::info!(user_id = %output.user_id, "User registered");

    Ok((StatusCode::CREATED, Json(RegisterResponse { user_id: output.user_id })))
}

// ── Login ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub tenant_id: Uuid,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn login(
    State(state): State<SharedState>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let tenant = sqlx::query!(
        "SELECT schema_name FROM public.tenants WHERE id = $1 AND status = 'active'",
        body.tenant_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?
    .ok_or_else(|| valdhran_domain::errors::DomainError::NotFound("Tenant not found".into()))?;

    let user_repo = PgUserRepository::new(state.pool.clone(), tenant.schema_name);
    let use_case = AuthenticateUserUseCase::new(
        user_repo,
        state.hasher,
        state.token_service.clone(),
    );

    let output = use_case
        .execute(AuthenticateUserInput {
            tenant_id: body.tenant_id,
            email: body.email,
            password_plaintext: body.password,
        })
        .await?;

    // Persistir hash del refresh token en BD (DEC-006)
    let token_hash = JwtTokenService::hash_token(&output.refresh_token);
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(state.config.jwt.refresh_token_expiry_seconds as i64);

    sqlx::query!(
        "INSERT INTO public.refresh_tokens (user_id, tenant_id, token_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
        output.user_id,
        body.tenant_id,
        token_hash,
        expires_at,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?;

    tracing::info!(user_id = %output.user_id, "User authenticated");

    Ok(Json(LoginResponse {
        user_id: output.user_id,
        access_token: output.access_token,
        refresh_token: output.refresh_token,
    }))
}

// ── Refresh token ─────────────────────────────────────────────────────────────

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
    State(state): State<SharedState>,
    Json(body): Json<RefreshRequest>,
) -> ApiResult<Json<RefreshResponse>> {
    // Verificar que el token existe en BD y no está revocado (DEC-006)
    let token_hash = JwtTokenService::hash_token(&body.refresh_token);

    let stored = sqlx::query!(
        "SELECT id, user_id, tenant_id FROM public.refresh_tokens
         WHERE token_hash = $1
           AND revoked_at IS NULL
           AND expires_at > NOW()",
        token_hash,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?
    .ok_or(valdhran_domain::errors::DomainError::Unauthorized)?;

    let use_case = RefreshTokenUseCase::new(
        state.token_service.clone(),
        state.token_service.clone(),
    );

    let output = use_case
        .execute(RefreshTokenInput {
            refresh_token: body.refresh_token.clone(),
        })
        .await?;

    // Revocar token viejo (rotación — DEC-006)
    sqlx::query!(
        "UPDATE public.refresh_tokens SET revoked_at = NOW() WHERE id = $1",
        stored.id,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?;

    // Persistir nuevo token hash
    let new_hash = JwtTokenService::hash_token(&output.refresh_token);
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(state.config.jwt.refresh_token_expiry_seconds as i64);

    sqlx::query!(
        "INSERT INTO public.refresh_tokens (user_id, tenant_id, token_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
        stored.user_id,
        stored.tenant_id,
        new_hash,
        expires_at,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?;

    tracing::info!(user_id = %stored.user_id, "Refresh token rotated");

    Ok(Json(RefreshResponse {
        access_token: output.access_token,
        refresh_token: output.refresh_token,
    }))
}

// ── Logout ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

pub async fn logout(
    State(state): State<SharedState>,
    Json(body): Json<LogoutRequest>,
) -> ApiResult<StatusCode> {
    let token_hash = JwtTokenService::hash_token(&body.refresh_token);

    sqlx::query!(
        "UPDATE public.refresh_tokens SET revoked_at = NOW()
         WHERE token_hash = $1 AND revoked_at IS NULL",
        token_hash,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
