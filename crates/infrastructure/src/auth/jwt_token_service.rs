use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use valdhran_application::use_cases::{
    authenticate_user::TokenService,
    refresh_token::RefreshTokenValidator,
};
use valdhran_domain::errors::{DomainError, DomainResult};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,        // user_id
    tid: String,        // tenant_id
    exp: usize,
    iat: usize,
    token_type: String, // "access" | "refresh"
}

#[derive(Clone)]
pub struct JwtTokenService {
    secret: String,
    access_expiry_secs: u64,
    refresh_expiry_secs: u64,
}

impl JwtTokenService {
    pub fn new(secret: String, access_expiry_secs: u64, refresh_expiry_secs: u64) -> Self {
        Self { secret, access_expiry_secs, refresh_expiry_secs }
    }

    fn create_token(&self, user_id: Uuid, tenant_id: Uuid, expiry_secs: u64, token_type: &str) -> DomainResult<String> {
        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            tid: tenant_id.to_string(),
            exp: now + expiry_secs as usize,
            iat: now,
            token_type: token_type.to_string(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| DomainError::Validation(format!("Token generation failed: {}", e)))
    }

    fn decode_token(&self, token: &str, expected_type: &str) -> DomainResult<(Uuid, Uuid)> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| DomainError::Unauthorized)?;

        if token_data.claims.token_type != expected_type {
            return Err(DomainError::Unauthorized);
        }

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| DomainError::Unauthorized)?;
        let tenant_id = Uuid::parse_str(&token_data.claims.tid)
            .map_err(|_| DomainError::Unauthorized)?;

        Ok((user_id, tenant_id))
    }

    /// Decode access token — expuesto para middleware de autenticación
    pub fn decode_access_token(&self, token: &str) -> DomainResult<(Uuid, Uuid)> {
        self.decode_token(token, "access")
    }

    /// Hash SHA-256 del token para almacenar en BD (nunca el token en claro — DEC-006)
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[async_trait]
impl TokenService for JwtTokenService {
    async fn generate_access_token(&self, user_id: Uuid, tenant_id: Uuid) -> DomainResult<String> {
        self.create_token(user_id, tenant_id, self.access_expiry_secs, "access")
    }

    async fn generate_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) -> DomainResult<String> {
        self.create_token(user_id, tenant_id, self.refresh_expiry_secs, "refresh")
    }
}

#[async_trait]
impl RefreshTokenValidator for JwtTokenService {
    async fn validate_refresh_token(&self, token: &str) -> DomainResult<(Uuid, Uuid)> {
        self.decode_token(token, "refresh")
    }

    async fn revoke_refresh_token(&self, _token: &str) -> DomainResult<()> {
        // La revocación real se hace en BD desde el use case vía el repositorio
        // Este método existe para satisfacer el trait; la lógica de BD va en el handler de Axum
        Ok(())
    }
}
