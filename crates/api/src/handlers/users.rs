use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use valdhran_application::use_cases::register_user::{
    RegisterUserInput, RegisterUserUseCase,
};
use valdhran_domain::{
    errors::DomainError,
    repositories::tenant_repository::TenantRepository,
    value_objects::tenant_slug::TenantSlug,
};
use valdhran_infrastructure::{
    auth::argon2_hasher::Argon2PasswordHasher,
    db::pg_user_repository::PgUserRepository,
};
use crate::{app::AppState, errors::ApiError};

// ── POST /tenants/:slug/users ─────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterUserResponse {
    pub user_id: Uuid,
}

pub async fn register_user(
    State(state): State<AppState>,
    Path(tenant_slug): Path<String>,
    Json(body): Json<RegisterUserRequest>,
) -> Result<(StatusCode, Json<RegisterUserResponse>), ApiError> {
    // 1. Resolver tenant por slug
    let slug = TenantSlug::new(&tenant_slug)?;
    let tenant = state
        .tenant_repo
        .find_by_slug(&slug)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Tenant '{}' not found", tenant_slug)))?;

    // 2. Crear repo de usuarios para el schema del tenant
    let user_repo = PgUserRepository::new(state.pool.clone(), tenant.schema_name.clone());

    // 3. Crear use case con el repo específico del tenant
    let register_uc = RegisterUserUseCase::new(user_repo, Argon2PasswordHasher);

    // 4. Ejecutar
    let input = RegisterUserInput {
        tenant_id: tenant.id,
        email: body.email,
        display_name: body.display_name,
        password_plaintext: body.password,
    };

    let output = register_uc.execute(input).await?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterUserResponse {
            user_id: output.user_id,
        }),
    ))
}
