use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    service: &'static str,
}
pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        service: "complexe-sms-api",
    })
}

pub async fn readiness(State(s): State<AppState>) -> Result<Json<Health>, (StatusCode, String)> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&s.db)
        .await
        .map_err(|e| {
            tracing::error!(error=%e, "database readiness check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database is not ready".into(),
            )
        })?;
    Ok(Json(Health {
        status: "ready",
        service: "complexe-sms-api",
    }))
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProductRow {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub selling_price: Decimal,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}
#[derive(Deserialize)]
pub struct CreateProduct {
    pub sku: String,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub selling_price: Decimal,
}

pub async fn list_products(
    State(s): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ProductRow>>, (StatusCode, String)> {
    require_permission(&s, user.id, "catalog.read").await?;
    let rows = sqlx::query_as::<_, ProductRow>("SELECT id, sku, name, category, unit, selling_price, active, created_at FROM products ORDER BY name")
        .fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(rows))
}
pub async fn create_product(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<CreateProduct>,
) -> Result<(StatusCode, Json<ProductRow>), (StatusCode, String)> {
    require_permission(&s, user.id, "catalog.manage").await?;
    let sku = input.sku.trim().to_ascii_uppercase();
    let name = input.name.trim();
    let category = input.category.trim();
    let unit = input.unit.trim();
    if sku.is_empty()
        || sku.len() > 64
        || name.is_empty()
        || name.chars().count() > 160
        || category.is_empty()
        || category.chars().count() > 80
        || unit.is_empty()
        || unit.chars().count() > 24
        || input.selling_price < Decimal::ZERO
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Provide a valid SKU, name, category, unit and non-negative price".into(),
        ));
    }
    let row = sqlx::query_as::<_, ProductRow>(
        "INSERT INTO products (sku,name,category,unit,selling_price)
         VALUES ($1,$2,$3,$4,$5)
         RETURNING id,sku,name,category,unit,selling_price,active,created_at",
    )
    .bind(sku)
    .bind(name)
    .bind(category)
    .bind(unit)
    .bind(input.selling_price)
    .fetch_one(&s.db)
    .await
    .map_err(product_insert_error)?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Serialize, FromRow)]
pub struct BalanceRow {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub unit: String,
    pub quantity: Decimal,
}
pub async fn inventory_balances(
    State(s): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<BalanceRow>>, (StatusCode, String)> {
    require_permission(&s, user.id, "inventory.read").await?;
    let rows = sqlx::query_as::<_, BalanceRow>("SELECT p.id product_id,p.sku,p.name,p.unit,COALESCE(SUM(m.quantity_delta),0)::numeric quantity FROM products p LEFT JOIN inventory_movements m ON m.product_id=p.id GROUP BY p.id ORDER BY p.name")
        .fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct CreateMovement {
    pub product_id: Uuid,
    pub kind: String,
    pub quantity_delta: Decimal,
    pub reason: String,
    pub idempotency_key: Uuid,
}

#[derive(Serialize, FromRow)]
pub struct MovementRow {
    pub id: Uuid,
    pub product_id: Uuid,
    pub kind: String,
    pub quantity_delta: Decimal,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct IdempotencyRecord {
    id: Uuid,
    product_id: Uuid,
    kind: String,
    quantity_delta: Decimal,
    reason: String,
    actor_user_id: Uuid,
    created_at: DateTime<Utc>,
}

pub async fn create_movement(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<CreateMovement>,
) -> Result<(StatusCode, Json<MovementRow>), (StatusCode, String)> {
    let reason = input.reason.trim();

    if input.quantity_delta <= Decimal::ZERO || reason.is_empty() || input.idempotency_key.is_nil()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "A positive receipt quantity, reason, and valid idempotency key are required".into(),
        ));
    }

    if input.kind == "adjustment" {
        return Err((
            StatusCode::CONFLICT,
            "Stock adjustments require the approval workflow, which is not yet enabled".into(),
        ));
    }

    if input.kind != "receipt" {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "This movement type is reserved for its dedicated business workflow".into(),
        ));
    }

    require_permission(&s, user.id, "inventory.receive").await?;

    let mut tx = s.db.begin().await.map_err(internal)?;

    // Serialize requests using the same key, including requests for
    // different products. PostgreSQL releases this lock at transaction end.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(input.idempotency_key.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    // If this key was already used, return the original movement only
    // when every significant request field matches.
    let existing = sqlx::query_as::<_, IdempotencyRecord>(
        "SELECT id, product_id, kind, quantity_delta, reason, actor_user_id, created_at
         FROM inventory_movements
         WHERE idempotency_key = $1",
    )
    .bind(input.idempotency_key)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;

    if let Some(existing) = existing {
        let same_request = existing.product_id == input.product_id
            && existing.kind == input.kind
            && existing.quantity_delta == input.quantity_delta
            && existing.reason == reason
            && existing.actor_user_id == user.id;

        if !same_request {
            return Err((
                StatusCode::CONFLICT,
                "This idempotency key has already been used for a different request".into(),
            ));
        }

        let row = MovementRow {
            id: existing.id,
            product_id: existing.product_id,
            kind: existing.kind,
            quantity_delta: existing.quantity_delta,
            reason: existing.reason,
            created_at: existing.created_at,
        };

        tx.commit().await.map_err(internal)?;
        return Ok((StatusCode::OK, Json(row)));
    }

    // Lock the product before calculating its current balance.
    sqlx::query("SELECT id FROM products WHERE id = $1 FOR UPDATE")
        .bind(input.product_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, "Product not found".into()))?;

    let current: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(quantity_delta), 0)::numeric
         FROM inventory_movements
         WHERE product_id = $1",
    )
    .bind(input.product_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;

    if current + input.quantity_delta < Decimal::ZERO {
        return Err((
            StatusCode::CONFLICT,
            "Movement would make stock negative".into(),
        ));
    }

    let row = sqlx::query_as::<_, MovementRow>(
        "INSERT INTO inventory_movements
         (product_id, kind, quantity_delta, reason, actor_user_id, idempotency_key)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, product_id, kind, quantity_delta, reason, created_at",
    )
    .bind(input.product_id)
    .bind(input.kind)
    .bind(input.quantity_delta)
    .bind(reason)
    .bind(user.id)
    .bind(input.idempotency_key)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;

    tx.commit().await.map_err(internal)?;

    Ok((StatusCode::CREATED, Json(row)))
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    tracing::error!(error=%e, "request failed");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".into(),
    )
}
fn product_insert_error(e: sqlx::Error) -> (StatusCode, String) {
    if let sqlx::Error::Database(db_error) = &e {
        if db_error.code().as_deref() == Some("23505") {
            return (
                StatusCode::CONFLICT,
                "A product with this SKU already exists".into(),
            );
        }
    }

    internal(e)
}

// ---------------------------- Identity & access ----------------------------
use crate::auth::{
    hash_password, hash_token, new_token, require_permission, verify_password, AuthenticatedUser,
};
use axum::http::header::AUTHORIZATION;
use subtle::ConstantTimeEq;

#[derive(Deserialize)]
pub struct BootstrapAdmin {
    pub bootstrap_secret: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
}
#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in_seconds: i64,
    pub user: UserSummary,
}
#[derive(Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

// TEMPORARY LOCAL-DEVELOPMENT RECOVERY. Keep disabled except during a supervised
// local recovery, then disable the environment flag and restart the API.
#[derive(Deserialize)]
pub struct LocalAdminRecovery {
    pub bootstrap_secret: String,
    pub username: String,
    pub new_password: String,
}

pub async fn local_admin_recovery(
    State(s): State<AppState>,
    Json(input): Json<LocalAdminRecovery>,
) -> Result<StatusCode, (StatusCode, String)> {
    let enabled = std::env::var("ALLOW_LOCAL_PASSWORD_RECOVERY").unwrap_or_default() == "true";
    let bind: std::net::SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                "Local recovery is unavailable".into(),
            )
        })?;
    if !enabled || !bind.ip().is_loopback() {
        return Err((
            StatusCode::FORBIDDEN,
            "Local password recovery is disabled".into(),
        ));
    }
    let configured = std::env::var("BOOTSTRAP_SECRET").unwrap_or_default();
    if configured.len() < 24
        || input
            .bootstrap_secret
            .as_bytes()
            .ct_eq(configured.as_bytes())
            .unwrap_u8()
            != 1
    {
        return Err((
            StatusCode::FORBIDDEN,
            "Recovery authorization failed".into(),
        ));
    }
    let username = input.username.trim().to_ascii_lowercase();
    if username.is_empty() || username.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, "Invalid username".into()));
    }
    let password_hash =
        hash_password(&input.new_password).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let mut tx = s.db.begin().await.map_err(internal)?;
    let user_id: Uuid = sqlx::query_scalar(
        "UPDATE app_users SET password_hash=$1, failed_login_count=0, locked_until=NULL, must_change_password=false, updated_at=now() WHERE username=$2 AND is_active=true RETURNING id"
    )
    .bind(password_hash)
    .bind(&username)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?
    .ok_or((StatusCode::NOT_FOUND, "Active administrator account not found".into()))?;

    sqlx::query(
        "UPDATE auth_sessions SET revoked_at=now() WHERE user_id=$1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    sqlx::query("INSERT INTO audit_events(actor_user_id,action,target_type,target_id,reason) VALUES(NULL,'identity.local_password_recovery','user',$1,'Local development recovery authorized by bootstrap secret')")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn bootstrap_admin(
    State(s): State<AppState>,
    Json(input): Json<BootstrapAdmin>,
) -> Result<(StatusCode, Json<UserSummary>), (StatusCode, String)> {
    let configured = std::env::var("BOOTSTRAP_SECRET").unwrap_or_default();
    if configured.len() < 24
        || input
            .bootstrap_secret
            .as_bytes()
            .ct_eq(configured.as_bytes())
            .unwrap_u8()
            != 1
    {
        return Err((
            StatusCode::FORBIDDEN,
            "Initial administrator setup is disabled or the setup secret is invalid".into(),
        ));
    }
    let username = input.username.trim().to_ascii_lowercase();
    let display_name = input.display_name.trim();
    if username.len() < 3
        || username.len() > 64
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        || display_name.is_empty()
        || display_name.chars().count() > 120
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Enter a valid username and display name".into(),
        ));
    }
    let password_hash = hash_password(&input.password).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let mut tx = s.db.begin().await.map_err(internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock(7219042026)")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_users")
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
    if count > 0 {
        return Err((
            StatusCode::CONFLICT,
            "Initial administrator has already been created".into(),
        ));
    }
    let row=sqlx::query("INSERT INTO app_users(username,display_name,password_hash,must_change_password) VALUES($1,$2,$3,false) RETURNING id,username,display_name")
        .bind(username).bind(display_name).bind(password_hash).fetch_one(&mut *tx).await.map_err(internal)?;
    let user_id: Uuid = row.get("id");
    sqlx::query(
        "INSERT INTO user_roles(user_id,role_id) SELECT $1,id FROM roles WHERE code='owner'",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    sqlx::query("INSERT INTO audit_events(actor_user_id,action,target_type,target_id,reason) VALUES($1,'identity.bootstrap','user',$1,'Initial administrator bootstrap')").bind(user_id).execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok((
        StatusCode::CREATED,
        Json(UserSummary {
            id: user_id,
            username: row.get("username"),
            display_name: row.get("display_name"),
            roles: vec!["owner".into()],
        }),
    ))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
pub async fn login(
    State(s): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let username = input.username.trim().to_ascii_lowercase();
    if username.len() > 64 || input.password.len() > 256 {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".into(),
        ));
    }
    let row=sqlx::query("SELECT id,username,display_name,password_hash,is_active,failed_login_count,locked_until FROM app_users WHERE username=$1")
        .bind(&username).fetch_optional(&s.db).await.map_err(internal)?;
    let Some(row) = row else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".into(),
        ));
    };
    let user_id: Uuid = row.get("id");
    let active: bool = row.get("is_active");
    let locked_until: Option<DateTime<Utc>> = row.get("locked_until");
    let stored_hash: String = row.get("password_hash");
    if !active
        || locked_until.map(|t| t > Utc::now()).unwrap_or(false)
        || !verify_password(&input.password, &stored_hash)
    {
        if active && locked_until.map(|t| t <= Utc::now()).unwrap_or(true) {
            let _=sqlx::query("UPDATE app_users SET failed_login_count=failed_login_count+1, locked_until=CASE WHEN failed_login_count+1 >= 8 THEN now()+interval '15 minutes' ELSE locked_until END WHERE id=$1").bind(user_id).execute(&s.db).await;
        }
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".into(),
        ));
    }
    sqlx::query("UPDATE app_users SET failed_login_count=0,locked_until=NULL,last_login_at=now() WHERE id=$1").bind(user_id).execute(&s.db).await.map_err(internal)?;
    let token = new_token();
    let token_hash = hash_token(&token);
    let expires_in = 12 * 60 * 60i64;
    sqlx::query("INSERT INTO auth_sessions(user_id,token_hash,expires_at) VALUES($1,$2,now()+($3::double precision * interval '1 second'))").bind(user_id).bind(token_hash).bind(expires_in).execute(&s.db).await.map_err(internal)?;
    let roles=sqlx::query_scalar::<_,String>("SELECT r.code FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=$1 ORDER BY r.code").bind(user_id).fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer",
        expires_in_seconds: expires_in,
        user: UserSummary {
            id: user_id,
            username: row.get("username"),
            display_name: row.get("display_name"),
            roles,
        },
    }))
}

pub async fn logout(
    State(s): State<AppState>,
    headers: axum::http::HeaderMap,
    user: AuthenticatedUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    sqlx::query("UPDATE auth_sessions SET revoked_at=now() WHERE user_id=$1 AND token_hash=$2 AND revoked_at IS NULL").bind(user.id).bind(hash_token(token)).execute(&s.db).await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(s): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<UserSummary>, (StatusCode, String)> {
    let roles=sqlx::query_scalar::<_,String>("SELECT r.code FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=$1 ORDER BY r.code").bind(user.id).fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(UserSummary {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        roles,
    }))
}

// ========================= PROCUREMENT ROUTES (Phase 4) =========================

use crate::procurement;

#[derive(Serialize)]
pub struct SupplierResponse {
    pub id: Uuid,
    pub name: String,
    pub contact_person: String,
    pub phone: String,
    pub email: String,
    pub physical_address: Option<String>,
    pub payment_terms: Option<String>,
    pub tax_id: Option<String>,
    pub active: bool,
}

/// POST /api/v1/procurement/suppliers - Create a new supplier
pub async fn create_supplier(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<procurement::CreateSupplierRequest>,
) -> Result<(StatusCode, Json<SupplierResponse>), (StatusCode, String)> {
    // Permission check: manager or admin
    require_permission(&s, user.id, "procurement.supplier.create").await?;

    let supplier =
        procurement::create_supplier(&s.db, input, user.id)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let response = SupplierResponse {
        id: supplier.id,
        name: supplier.name,
        contact_person: supplier.contact_person,
        phone: supplier.phone,
        email: supplier.email,
        physical_address: supplier.physical_address,
        payment_terms: supplier.payment_terms,
        tax_id: supplier.tax_id,
        active: supplier.active,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/procurement/suppliers - List suppliers
pub async fn list_suppliers(
    State(s): State<AppState>,
    _user: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let search_query = params.get("q").map(|s| s.clone());
    let active_only = params
        .get("active")
        .map(|s| s == "true")
        .unwrap_or(true);
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let suppliers = procurement::list_suppliers(&s.db, search_query, active_only, limit, offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = suppliers
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "contact_person": s.contact_person,
                "phone": s.phone,
                "email": s.email,
                "payment_terms": s.payment_terms,
                "active": s.active,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({
        "suppliers": response,
        "total": response.len(),
        "limit": limit,
        "offset": offset,
    })))
}

/// GET /api/v1/procurement/suppliers/:id - Get supplier by ID
pub async fn get_supplier_detail(
    State(s): State<AppState>,
    _user: AuthenticatedUser,
    axum::extract::Path(supplier_id): axum::extract::Path<Uuid>,
) -> Result<Json<SupplierResponse>, (StatusCode, String)> {
    let supplier = procurement::get_supplier(&s.db, supplier_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok(Json(SupplierResponse {
        id: supplier.id,
        name: supplier.name,
        contact_person: supplier.contact_person,
        phone: supplier.phone,
        email: supplier.email,
        physical_address: supplier.physical_address,
        payment_terms: supplier.payment_terms,
        tax_id: supplier.tax_id,
        active: supplier.active,
    }))
}

/// POST /api/v1/procurement/suppliers/:id/archive - Archive a supplier
pub async fn archive_supplier_route(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(supplier_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Permission check
    require_permission(&s, user.id, "procurement.supplier.archive").await?;

    procurement::archive_supplier(&s.db, supplier_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(serde_json::json!({
        "id": supplier_id,
        "active": false,
        "archived_at": Utc::now(),
    })))
}

/// POST /api/v1/procurement/purchase-orders - Create a purchase order
pub async fn create_po_route(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<procurement::CreatePurchaseOrderRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    // Permission check
    require_permission(&s, user.id, "procurement.po.create").await?;

    if input.line_items.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Purchase order must have at least one line item".into(),
        ));
    }

    let po = procurement::create_purchase_order(&s.db, input, user.id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": po.id,
            "po_number": po.po_number,
            "supplier_id": po.supplier_id,
            "status": po.status,
            "subtotal_amount": po.subtotal_amount,
            "created_at": po.created_at,
        })),
    ))
}

/// GET /api/v1/procurement/purchase-orders - List purchase orders
pub async fn list_pos_route(
    State(s): State<AppState>,
    _user: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let pos = procurement::list_purchase_orders(&s.db, limit, offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = pos
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "po_number": p.po_number,
                "supplier_id": p.supplier_id,
                "status": p.status,
                "subtotal_amount": p.subtotal_amount,
                "created_at": p.created_at,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({
        "purchase_orders": response,
        "total": response.len(),
    })))
}

/// GET /api/v1/procurement/purchase-orders/:id - Get purchase order
pub async fn get_po_route(
    State(s): State<AppState>,
    _user: AuthenticatedUser,
    axum::extract::Path(po_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let po = procurement::get_purchase_order(&s.db, po_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    let items = procurement::get_po_items(&s.db, po_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({
        "id": po.id,
        "po_number": po.po_number,
        "supplier_id": po.supplier_id,
        "status": po.status,
        "subtotal_amount": po.subtotal_amount,
        "expected_delivery_date": po.expected_delivery_date,
        "line_items": items.iter().map(|i| serde_json::json!({
            "id": i.id,
            "product_id": i.product_id,
            "quantity_ordered": i.quantity_ordered,
            "unit_price": i.unit_price,
            "line_total": i.line_total,
            "notes": i.notes,
        })).collect::<Vec<_>>(),
        "created_at": po.created_at,
    })))
}

/// POST /api/v1/procurement/purchase-orders/:id/submit - Submit a purchase order
pub async fn submit_po_route(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(po_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Permission check
    require_permission(&s, user.id, "procurement.po.submit").await?;

    let po = procurement::submit_purchase_order(&s.db, po_id, user.id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(serde_json::json!({
        "id": po.id,
        "po_number": po.po_number,
        "status": po.status,
        "submitted_at": po.submitted_at,
    })))
}

/// POST /api/v1/procurement/purchase-orders/:id/cancel - Cancel a purchase order
pub async fn cancel_po_route(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(po_id): axum::extract::Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Permission check
    require_permission(&s, user.id, "procurement.po.cancel").await?;

    let reason = body
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("No reason provided")
        .to_string();

    let po = procurement::cancel_purchase_order(&s.db, po_id, reason, user.id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(serde_json::json!({
        "id": po.id,
        "po_number": po.po_number,
        "status": po.status,
        "cancelled_at": po.cancelled_at,
    })))
}

/// POST /api/v1/procurement/receipts - Create a goods receipt
pub async fn create_receipt_route(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<procurement::CreateReceiptRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    // Permission check: storekeeper, manager, admin
    require_permission(&s, user.id, "procurement.receipt.create").await?;

    if input.line_items.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Receipt must have at least one line item".into(),
        ));
    }

    let receipt = procurement::create_receipt(&s.db, input, user.id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": receipt.id,
            "receipt_number": receipt.receipt_number,
            "purchase_order_id": receipt.purchase_order_id,
            "receipt_date": receipt.receipt_date,
            "created_at": receipt.created_at,
        })),
    ))
}

/// GET /api/v1/procurement/receipts - List receipts
pub async fn list_receipts_route(
    State(_s): State<AppState>,
    _user: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _limit = params
        .get("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(50);
    let _offset = params
        .get("offset")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    // For now, return empty list (can be extended with filters later)
    Ok(Json(serde_json::json!({
        "receipts": [],
        "total": 0,
    })))
}

/// GET /api/v1/procurement/receipts/:id - Get receipt by ID
pub async fn get_receipt_route(
    State(s): State<AppState>,
    _user: AuthenticatedUser,
    axum::extract::Path(receipt_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let receipt = procurement::get_receipt(&s.db, receipt_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok(Json(serde_json::json!({
        "id": receipt.id,
        "receipt_number": receipt.receipt_number,
        "purchase_order_id": receipt.purchase_order_id,
        "receipt_date": receipt.receipt_date,
        "supplier_invoice_number": receipt.supplier_invoice_number,
        "created_at": receipt.created_at,
    })))
}
