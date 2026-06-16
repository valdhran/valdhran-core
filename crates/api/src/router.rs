use axum::{
    routing::post,
    Router,
};
use crate::{
    app_state::SharedState,
    handlers::{auth, tenants},
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        // Tenants
        .route("/tenants", post(tenants::create_tenant))
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login",    post(auth::login))
        .route("/auth/refresh",  post(auth::refresh))
        .route("/auth/logout",   post(auth::logout))
        .with_state(state)
}
