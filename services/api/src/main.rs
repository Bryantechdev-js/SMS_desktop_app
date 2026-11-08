use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr};
use socket2::{Socket, Domain, Type, Protocol};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

mod auth;
mod routes;
mod inventory;
mod pos;
mod procurement;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_filename(".env").ok();
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
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
        .route(
            "/api/v1/auth/local-recovery",
            post(routes::local_admin_recovery),
        )
        .route("/api/v1/auth/login", post(routes::login))
        .route("/api/v1/auth/logout", post(routes::logout))
        .route("/api/v1/auth/me", get(routes::me))
        .route(
            "/api/v1/products",
            get(routes::list_products).post(routes::create_product),
        )
        .route(
            "/api/v1/products/{id}",
            get(inventory::get_product).put(inventory::update_product),
        )
        .route("/api/v1/products/{id}/archive", post(inventory::archive_product))
        .route("/api/v1/products/{id}/reactivate", post(inventory::reactivate_product))
        .route(
            "/api/v1/inventory/balances",
            get(routes::inventory_balances),
        )
        .route("/api/v1/inventory/movements", 
            get(inventory::get_movements).post(routes::create_movement))
        .route("/api/v1/inventory/stock-status", get(inventory::get_stock_status))
        .route("/api/v1/inventory/physical-counts", post(inventory::create_physical_count))
        .route("/api/v1/sales/checkout", post(pos::checkout))
        .route("/api/v1/sales", get(pos::list_sales))
        .route("/api/v1/sales/{id}", get(pos::get_sale))
        // Procurement routes (Phase 4)
        .route(
            "/api/v1/procurement/suppliers",
            get(routes::list_suppliers).post(routes::create_supplier),
        )
        .route(
            "/api/v1/procurement/suppliers/{id}",
            get(routes::get_supplier_detail),
        )
        .route("/api/v1/procurement/suppliers/{id}/archive", post(routes::archive_supplier_route))
        .route(
            "/api/v1/procurement/purchase-orders",
            get(routes::list_pos_route).post(routes::create_po_route),
        )
        .route(
            "/api/v1/procurement/purchase-orders/{id}",
            get(routes::get_po_route),
        )
        .route("/api/v1/procurement/purchase-orders/{id}/submit", post(routes::submit_po_route))
        .route("/api/v1/procurement/purchase-orders/{id}/cancel", post(routes::cancel_po_route))
        .route(
            "/api/v1/procurement/receipts",
            get(routes::list_receipts_route).post(routes::create_receipt_route),
        )
        .route(
            "/api/v1/procurement/receipts/{id}",
            get(routes::get_receipt_route),
        )
        .with_state(state)
        // This API remains loopback-only and development-only until authentication,
        // authorization, TLS and production deployment hardening are implemented.
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    let bind: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()?;
    if !bind.ip().is_loopback() {
        anyhow::bail!("This development API is intentionally restricted to loopback. Do not expose it to LAN until authentication, authorization and TLS are implemented.");
    }
    
    // Create socket with SO_REUSEADDR to allow quick restart during development
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&bind.into())?;
    socket.listen(128)?;
    
    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(std_listener)?;
    tracing::info!(%bind, "Complexe SMS API listening");
    axum::serve(listener, app).await?;
    Ok(())
}
