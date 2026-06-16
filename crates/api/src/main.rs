use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use valdhran_infrastructure::config::app_config::AppConfig;

mod app_state;
mod errors;
mod handlers;
mod router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Cargar .env si existe (solo en desarrollo)
    let _ = dotenvy::dotenv();

    // Tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,valdhran=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configuración desde variables de entorno
    let config = AppConfig::load()?;

    // Pool de conexiones PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    tracing::info!("Connected to PostgreSQL");

    // Ejecutar migraciones al arrancar
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await?;

    tracing::info!("Migrations applied");

    // Estado compartido
    let state = Arc::new(app_state::AppState::new(pool, config.clone()));

    // Router
    let app = router::build_router(state);

    // Servidor
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Valdhran API listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
