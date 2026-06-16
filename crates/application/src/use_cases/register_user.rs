use uuid::Uuid;
use valdhran_domain::{
    entities::user::User,
    errors::{DomainError, DomainResult},
    repositories::user_repository::UserRepository,
    value_objects::email::Email,
};

pub struct RegisterUserInput {
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    /// Password en claro — el use case lo hashea con Argon2 via el port
    pub password_plaintext: String,
}

pub struct RegisterUserOutput {
    pub user_id: Uuid,
}

/// Port para hashing de passwords — implementado en infrastructure con Argon2
#[async_trait::async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &str) -> DomainResult<String>;
    async fn verify(&self, password: &str, hash: &str) -> DomainResult<bool>;
}

pub struct RegisterUserUseCase<R: UserRepository, H: PasswordHasher> {
    user_repo: R,
    hasher: H,
}

impl<R: UserRepository, H: PasswordHasher> RegisterUserUseCase<R, H> {
    pub fn new(user_repo: R, hasher: H) -> Self {
        Self { user_repo, hasher }
    }

    pub async fn execute(&self, input: RegisterUserInput) -> DomainResult<RegisterUserOutput> {
        let email = Email::new(&input.email)?;

        if self.user_repo.find_by_email(input.tenant_id, &email).await?.is_some() {
            return Err(DomainError::Conflict(format!(
                "Email '{}' already registered in this tenant",
                input.email
            )));
        }

        if input.password_plaintext.len() < 8 {
            return Err(DomainError::Validation("Password must be at least 8 characters".into()));
        }

        let password_hash = self.hasher.hash(&input.password_plaintext).await?;
        let user = User::new(input.tenant_id, email, input.display_name, password_hash);
        let user_id = user.id;

        self.user_repo.save(&user).await?;

        Ok(RegisterUserOutput { user_id })
    }
}
