use std::sync::Arc;
use axum::{routing::{get, post}, Router};
use sqlx::PgPool;

use valdhran_application::use_cases::{
    authenticate_user::AuthenticateUserUseCase,
    refresh_token::RefreshTokenUseCase,
    create_tenant::CreateTenantUseCase,
};
use valdhran_infrastructure::{
    auth::jwt_token_service::JwtTokenService,
    auth::argon2_hasher::Argon2PasswordHasher,
    db::{
        pg_tenant_repository::PgTenantRepository,
        pg_user_repository::PgUserRepository,
    },
    config::app_config::AppConfig,
};
use crate::handlers::{auth, tenants, users};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_service: Arc<JwtTokenService>,
    pub hasher: Arc<Argon2PasswordHasher>,
    pub tenant_repo: Arc<PgTenantRepository>,
    pub authenticate_user: Arc<AuthenticateUserUseCase<PgUserRepository, Argon2PasswordHasher, JwtTokenService>>,
    pub refresh_token: Arc<RefreshTokenUseCase<JwtTokenService, JwtTokenService>>,
    pub create_tenant: Arc<CreateTenantUseCase<PgTenantRepository>>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/auth/login",   post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/tenants",      post(tenants::create_tenant))
        .route("/tenants/:slug/users", post(users::register_user))
        .route("/graphify",     get(crate::handlers::graphify::graphify))
        .with_state(state)
}

pub async fn build_state(pool: PgPool, config: &AppConfig) -> AppState {
    let jwt = JwtTokenService::new(
        config.jwt.secret.clone(),
        config.jwt.access_token_expiry_seconds,
        config.jwt.refresh_token_expiry_seconds,
    );
    let hasher = Argon2PasswordHasher;
    let tenant_repo = PgTenantRepository::new(pool.clone());

    // PgUserRepository se crea dinámicamente en el handler con el schema correcto
    // Aquí usamos schema vacío como placeholder — el handler resolverá el tenant y creará el repo apropiado
    let user_repo = PgUserRepository::new(pool.clone(), String::new());

    let jwt_arc = Arc::new(jwt.clone());
    let hasher_arc = Arc::new(hasher);
    let tenant_repo_arc = Arc::new(tenant_repo.clone());

    AppState {
        pool: pool.clone(),
        jwt_service: jwt_arc.clone(),
        hasher: hasher_arc.clone(),
        tenant_repo: tenant_repo_arc.clone(),
        authenticate_user: Arc::new(AuthenticateUserUseCase::new(
            user_repo,
            Argon2PasswordHasher,
            jwt.clone(),
        )),
        refresh_token: Arc::new(RefreshTokenUseCase::new(
            jwt.clone(),
            jwt,
        )),
        create_tenant: Arc::new(CreateTenantUseCase::new(
            PgTenantRepository::new(pool),
        )),
    }
}
