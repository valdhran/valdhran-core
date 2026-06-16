use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use valdhran_application::use_cases::create_tenant::{
    CreateTenantInput, CreateTenantUseCase,
};
use crate::{app::AppState, errors::ApiError};

// ── POST /tenants ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateTenantRequest {
    pub slug: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateTenantResponse {
    pub tenant_id: Uuid,
    pub schema_name: String,
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(body): Json<CreateTenantRequest>,
) -> Result<Json<CreateTenantResponse>, ApiError> {
    let input = CreateTenantInput {
        slug: body.slug,
        name: body.name,
    };
    let output = state.create_tenant.execute(input).await?;
    Ok(Json(CreateTenantResponse {
        tenant_id: output.tenant_id,
        schema_name: output.schema_name,
    }))
}
