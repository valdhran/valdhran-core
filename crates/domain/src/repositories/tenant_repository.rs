use uuid::Uuid;
use crate::entities::tenant::Tenant;
use crate::errors::DomainResult;
use crate::value_objects::tenant_slug::TenantSlug;

/// Trait de repositorio para Tenant.
/// El domain define la interfaz — la infrastructure la implementa.
#[async_trait::async_trait]
pub trait TenantRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Tenant>>;
    async fn find_by_slug(&self, slug: &TenantSlug) -> DomainResult<Option<Tenant>>;
    async fn save(&self, tenant: &Tenant) -> DomainResult<()>;
    async fn exists_by_slug(&self, slug: &TenantSlug) -> DomainResult<bool>;
}
