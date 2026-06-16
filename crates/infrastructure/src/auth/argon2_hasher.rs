use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use valdhran_application::use_cases::register_user::PasswordHasher as PasswordHasherPort;
use valdhran_domain::errors::{DomainError, DomainResult};

pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasherPort for Argon2PasswordHasher {
    async fn hash(&self, password: &str) -> DomainResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| DomainError::Validation(format!("Password hashing failed: {}", e)))
    }

    async fn verify(&self, password: &str, hash: &str) -> DomainResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| DomainError::Validation(format!("Invalid hash format: {}", e)))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
