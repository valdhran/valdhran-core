use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use valdhran_application::use_cases::create_tenant::{CreateTenantInput, CreateTenantUseCase};
use valdhran_infrastructure::db::pg_tenant_repository::PgTenantRepository;

use crate::{app_state::SharedState, errors::ApiResult};

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
    State(state): State<SharedState>,
    Json(body): Json<CreateTenantRequest>,
) -> ApiResult<(axum::http::StatusCode, Json<CreateTenantResponse>)> {
    let repo = PgTenantRepository::new(state.pool.clone());
    let use_case = CreateTenantUseCase::new(repo);

    let output = use_case
        .execute(CreateTenantInput {
            slug: body.slug,
            name: body.name,
        })
        .await?;

    // Provisionar schema del tenant en PostgreSQL (DEC-004)
    // El use case solo guarda en public.tenants — el schema real lo crea esta llamada
    sqlx::query("SELECT public.provision_tenant_schema($1)")
        .bind(&output.schema_name)
        .execute(&state.pool)
        .await
        .map_err(|e| valdhran_domain::errors::DomainError::Validation(e.to_string()))?;

    tracing::info!(
        tenant_id = %output.tenant_id,
        schema = %output.schema_name,
        "Tenant created and schema provisioned"
    );

    Ok((
        axum::http::StatusCode::CREATED,
        Json(CreateTenantResponse {
            tenant_id: output.tenant_id,
            schema_name: output.schema_name,
        }),
    ))
}
