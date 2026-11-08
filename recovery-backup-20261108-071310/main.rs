use axum::{extract::DefaultBodyLimit, http::{header::{AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method}, routing::{get, post}, Router};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr};
use tower_http::{cors::{AllowOrigin, CorsLayer}, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

mod routes;
mod auth;

#[derive(Clone)]
pub struct AppState { pub db: sqlx::PgPool }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_filename(".env").ok();
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))).init();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");
    let db = PgPoolOptions::new().max_connections(10).connect(&database_url).await?;
    sqlx::migrate!("../../database/migrations").run(&db).await?;
    let state = AppState { db };
    let allowed_origins = [
        HeaderValue::from_static("http://127.0.0.1:1420"),
        HeaderValue::from_static("http://localhost:1420"),
        HeaderValue::from_static("tauri://localhost"),
        HeaderValue::from_static("http://tauri.localhost"),
    ];
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/ready", get(routes::readiness))
        .route("/api/v1/auth/bootstrap", post(routes::bootstrap_admin))
        .route("/api/v1/auth/login", post(routes::login))
        .route("/api/v1/auth/logout", post(routes::logout))
        .route("/api/v1/auth/me", get(routes::me))
        .route("/api/v1/products", get(routes::list_products).post(routes::create_product))
        .route("/api/v1/inventory/balances", get(routes::inventory_balances))
        .route("/api/v1/inventory/movements", post(routes::create_movement))
        .with_state(state)
        // This API remains loopback-only and development-only until authentication,
        // authorization, TLS and production deployment hardening are implemented.
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    let bind: SocketAddr = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()).parse()?;
    if !bind.ip().is_loopback() {
        anyhow::bail!("This development API is intentionally restricted to loopback. Do not expose it to LAN until authentication, authorization and TLS are implemented.");
    }
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "Complexe SMS API listening");
    axum::serve(listener, app).await?;
    Ok(())
}
