mod ami;
mod auth;
mod config;
mod config_gen;
mod db;
mod models;
mod recordings_indexer;
mod recordings_retention;
mod routes;
mod web;

use anyhow::Result;
use auth::rate_limit::RateLimiter;
use axum::{middleware, routing::get, Router};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cfg: Arc<config::Config>,
    pub events: broadcast::Sender<serde_json::Value>,
    pub rate_limiter: RateLimiter,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = Arc::new(config::Config::from_env()?);
    cfg.validate_secrets()?;
    tracing::info!("Starting Asterisk Manager (env: {})", cfg.app_env);

    let pool = db::pool::init(&cfg.database_url).await?;

    if cfg.seed_dev_users {
        tracing::warn!("SEED_DEV_USERS is enabled — seeding dev users");
        db::seed::dev_users(&pool).await?;
    }

    let (tx, _) = broadcast::channel::<serde_json::Value>(1024);
    let rate_limiter = RateLimiter::new(10, Duration::from_secs(60));

    let state = AppState {
        pool: pool.clone(),
        cfg: cfg.clone(),
        events: tx.clone(),
        rate_limiter: rate_limiter.clone(),
    };

    let cleanup_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            cleanup_limiter.cleanup().await;
        }
    });

    let ami_tx = tx.clone();
    let ami_cfg = (*cfg).clone();
    tokio::spawn(async move {
        if let Err(e) = ami::client::run(ami_cfg, ami_tx).await {
            tracing::error!("AMI client exited: {:#}", e);
        }
    });

    let indexer_state = state.clone();
    tokio::spawn(async move {
        recordings_indexer::run(indexer_state).await;
    });

    let retention_state = state.clone();
    tokio::spawn(async move {
        recordings_retention::run(retention_state).await;
    });

    let auth_public = routes::auth::public_router().with_state(state.clone());
    let auth_protected = routes::auth::protected_router()
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ))
        .with_state(state.clone());

    let admin_protected = routes::users::router()
        .merge(routes::extensions::router())
        .merge(routes::queues::router())
        .merge(routes::cdr::router())
        .merge(routes::recordings::router())
        .merge(routes::contacts::router())
        .merge(routes::settings::router())
        .merge(routes::agents::router())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    let admin = admin_protected
        .merge(routes::recordings::audio_router())
        .with_state(state.clone());

    let agent = routes::agent::router()
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ))
        .with_state(state.clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws/events", get(web::ws::handler))
        .nest("/api/auth", auth_public.merge(auth_protected))
        .nest("/api/admin", admin)
        .nest("/api/agent", agent)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.ws_bind).await?;
    tracing::info!("HTTP/WebSocket listening on {}", cfg.ws_bind);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
