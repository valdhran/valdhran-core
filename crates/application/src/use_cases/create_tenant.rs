use uuid::Uuid;
use valdhran_domain::{
    entities::tenant::Tenant,
    errors::{DomainError, DomainResult},
    repositories::tenant_repository::TenantRepository,
    value_objects::tenant_slug::TenantSlug,
};

pub struct CreateTenantInput {
    pub slug: String,
    pub name: String,
}

pub struct CreateTenantOutput {
    pub tenant_id: Uuid,
    pub schema_name: String,
}

pub struct CreateTenantUseCase<R: TenantRepository> {
    tenant_repo: R,
}

impl<R: TenantRepository> CreateTenantUseCase<R> {
    pub fn new(tenant_repo: R) -> Self {
        Self { tenant_repo }
    }

    pub async fn execute(&self, input: CreateTenantInput) -> DomainResult<CreateTenantOutput> {
        let slug = TenantSlug::new(&input.slug)?;

        if self.tenant_repo.exists_by_slug(&slug).await? {
            return Err(DomainError::Conflict(format!(
                "Tenant with slug '{}' already exists",
                input.slug
            )));
        }

        let tenant = Tenant::new(slug, input.name);
        let schema_name = tenant.schema_name.clone();
        let tenant_id = tenant.id;

        self.tenant_repo.save(&tenant).await?;

        Ok(CreateTenantOutput { tenant_id, schema_name })
    }
}
