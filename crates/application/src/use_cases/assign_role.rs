use uuid::Uuid;
use valdraegorn_domain::{
    errors::{DomainError, DomainResult},
    repositories::{role_repository::RoleRepository, user_repository::UserRepository},
};

pub struct AssignRoleInput {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

pub struct AssignRoleUseCase<U: UserRepository, R: RoleRepository> {
    user_repo: U,
    role_repo: R,
}

impl<U: UserRepository, R: RoleRepository> AssignRoleUseCase<U, R> {
    pub fn new(user_repo: U, role_repo: R) -> Self {
        Self { user_repo, role_repo }
    }

    pub async fn execute(&self, input: AssignRoleInput) -> DomainResult<()> {
        let mut user = self
            .user_repo
            .find_by_id(input.tenant_id, input.user_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("User {}", input.user_id)))?;

        // Verificar que el rol pertenece al mismo tenant
        let role = self
            .role_repo
            .find_by_id(input.tenant_id, input.role_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Role {}", input.role_id)))?;

        if role.tenant_id != input.tenant_id {
            return Err(DomainError::Forbidden("Role does not belong to this tenant".into()));
        }

        user.assign_role(input.role_id);
        self.user_repo.save(&user).await?;

        Ok(())
    }
}
