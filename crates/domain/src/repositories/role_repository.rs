use uuid::Uuid;
use crate::entities::role::Role;
use crate::errors::DomainResult;

#[async_trait::async_trait]
pub trait RoleRepository: Send + Sync {
    async fn find_by_id(&self, tenant_id: Uuid, role_id: Uuid) -> DomainResult<Option<Role>>;
    async fn find_all_by_tenant(&self, tenant_id: Uuid) -> DomainResult<Vec<Role>>;
    async fn save(&self, role: &Role) -> DomainResult<()>;
    async fn delete(&self, tenant_id: Uuid, role_id: Uuid) -> DomainResult<()>;
}
