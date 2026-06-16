use uuid::Uuid;
use valdhran_domain::{
    errors::{DomainError, DomainResult},
    repositories::user_repository::UserRepository,
    value_objects::email::Email,
};
use crate::use_cases::register_user::PasswordHasher;

pub struct AuthenticateUserInput {
    pub tenant_id: Uuid,
    pub email: String,
    pub password_plaintext: String,
}

pub struct AuthenticateUserOutput {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
}

/// Port para generación y validación de tokens JWT
#[async_trait::async_trait]
pub trait TokenService: Send + Sync {
    async fn generate_access_token(&self, user_id: Uuid, tenant_id: Uuid) -> DomainResult<String>;
    async fn generate_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) -> DomainResult<String>;
}

pub struct AuthenticateUserUseCase<R: UserRepository, H: PasswordHasher, T: TokenService> {
    user_repo: R,
    hasher: H,
    token_service: T,
}

impl<R: UserRepository, H: PasswordHasher, T: TokenService> AuthenticateUserUseCase<R, H, T> {
    pub fn new(user_repo: R, hasher: H, token_service: T) -> Self {
        Self { user_repo, hasher, token_service }
    }

    pub async fn execute(&self, input: AuthenticateUserInput) -> DomainResult<AuthenticateUserOutput> {
        let email = Email::new(&input.email)?;

        let user = self
            .user_repo
            .find_by_email(input.tenant_id, &email)
            .await?
            .ok_or_else(|| DomainError::Unauthorized)?;

        if !user.is_active() {
            return Err(DomainError::Forbidden("User account is not active".into()));
        }

        let valid = self.hasher.verify(&input.password_plaintext, &user.password_hash).await?;
        if !valid {
            return Err(DomainError::Unauthorized);
        }

        let access_token = self.token_service.generate_access_token(user.id, user.tenant_id).await?;
        let refresh_token = self.token_service.generate_refresh_token(user.id, user.tenant_id).await?;

        Ok(AuthenticateUserOutput {
            user_id: user.id,
            access_token,
            refresh_token,
        })
    }
}
