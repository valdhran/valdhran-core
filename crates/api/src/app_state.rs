use std::sync::Arc;
use sqlx::PgPool;
use valdhran_infrastructure::{
    auth::{
        argon2_hasher::Argon2PasswordHasher,
        jwt_token_service::JwtTokenService,
    },
    config::app_config::AppConfig,
};

/// Estado compartido entre todos los handlers de Axum.
/// Se pasa como Extension o State (Arc<AppState>).
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub hasher: Argon2PasswordHasher,
    pub token_service: JwtTokenService,
}

impl AppState {
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        let token_service = JwtTokenService::new(
            config.jwt.secret.clone(),
            config.jwt.access_token_expiry_seconds,
            config.jwt.refresh_token_expiry_seconds,
        );
        Self {
            pool,
            config,
            hasher: Argon2PasswordHasher,
            token_service,
        }
    }
}

pub type SharedState = Arc<AppState>;
