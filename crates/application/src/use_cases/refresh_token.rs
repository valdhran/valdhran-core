use uuid::Uuid;
use valdhran_domain::errors::DomainResult;
use crate::use_cases::authenticate_user::TokenService;

pub struct RefreshTokenInput {
    pub refresh_token: String,
}

pub struct RefreshTokenOutput {
    pub access_token: String,
    pub refresh_token: String, // rotación de refresh token en cada uso
}

/// Port para validación de refresh tokens
#[async_trait::async_trait]
pub trait RefreshTokenValidator: Send + Sync {
    /// Valida el refresh token y retorna (user_id, tenant_id) si es válido
    async fn validate_refresh_token(&self, token: &str) -> DomainResult<(Uuid, Uuid)>;
    /// Revoca el refresh token usado (rotación obligatoria)
    async fn revoke_refresh_token(&self, token: &str) -> DomainResult<()>;
}

pub struct RefreshTokenUseCase<T: TokenService, V: RefreshTokenValidator> {
    token_service: T,
    validator: V,
}

impl<T: TokenService, V: RefreshTokenValidator> RefreshTokenUseCase<T, V> {
    pub fn new(token_service: T, validator: V) -> Self {
        Self { token_service, validator }
    }

    pub async fn execute(&self, input: RefreshTokenInput) -> DomainResult<RefreshTokenOutput> {
        // Validar token entrante
        let (user_id, tenant_id) = self.validator.validate_refresh_token(&input.refresh_token).await?;

        // Revocar el token usado ANTES de emitir el nuevo (rotación)
        self.validator.revoke_refresh_token(&input.refresh_token).await?;

        // Emitir nuevos tokens
        let access_token = self.token_service.generate_access_token(user_id, tenant_id).await?;
        let refresh_token = self.token_service.generate_refresh_token(user_id, tenant_id).await?;

        Ok(RefreshTokenOutput { access_token, refresh_token })
    }
}
