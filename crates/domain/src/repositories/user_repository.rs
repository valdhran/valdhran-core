use uuid::Uuid;
use crate::entities::user::User;
use crate::errors::DomainResult;
use crate::value_objects::email::Email;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, tenant_id: Uuid, user_id: Uuid) -> DomainResult<Option<User>>;
    async fn find_by_email(&self, tenant_id: Uuid, email: &Email) -> DomainResult<Option<User>>;
    async fn save(&self, user: &User) -> DomainResult<()>;
    async fn delete(&self, tenant_id: Uuid, user_id: Uuid) -> DomainResult<()>;
}
