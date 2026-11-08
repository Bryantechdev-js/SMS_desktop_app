use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;
use crate::AppState;

#[derive(Serialize)]
pub struct Health { status: &'static str, service: &'static str }
pub async fn health() -> Json<Health> { Json(Health { status: "ok", service: "complexe-sms-api" }) }

pub async fn readiness(State(s): State<AppState>) -> Result<Json<Health>, (StatusCode, String)> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&s.db)
        .await
        .map_err(|e| {
            tracing::error!(error=%e, "database readiness check failed");
            (StatusCode::SERVICE_UNAVAILABLE, "Database is not ready".into())
        })?;
    Ok(Json(Health { status: "ready", service: "complexe-sms-api" }))
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProductRow { pub id: Uuid, pub sku: String, pub name: String, pub category: String, pub unit: String, pub selling_price: Decimal, pub active: bool, pub created_at: DateTime<Utc> }
#[derive(Deserialize)]
pub struct CreateProduct { pub sku: String, pub name: String, pub category: String, pub unit: String, pub selling_price: Decimal }

pub async fn list_products(State(s): State<AppState>, user: AuthenticatedUser) -> Result<Json<Vec<ProductRow>>, (StatusCode, String)> {
    require_permission(&s,user.id,"catalog.read").await?;
    let rows = sqlx::query_as::<_, ProductRow>("SELECT id, sku, name, category, unit, selling_price, active, created_at FROM products ORDER BY name")
        .fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(rows))
}
pub async fn create_product(State(s): State<AppState>, user: AuthenticatedUser, Json(input): Json<CreateProduct>) -> Result<(StatusCode, Json<ProductRow>), (StatusCode, String)> {
    require_permission(&s,user.id,"catalog.manage").await?;
    let sku = input.sku.trim().to_ascii_uppercase();
    let name = input.name.trim();
    let category = input.category.trim();
    let unit = input.unit.trim();
    if sku.is_empty() || sku.len() > 64 || name.is_empty() || name.chars().count() > 160
        || category.is_empty() || category.chars().count() > 80
        || unit.is_empty() || unit.chars().count() > 24
        || input.selling_price < Decimal::ZERO
    {
        return Err((StatusCode::BAD_REQUEST, "Provide a valid SKU, name, category, unit and non-negative price".into()));
    }
    let row = sqlx::query_as::<_, ProductRow>("INSERT INTO products (sku,name,category,unit,selling_price) VALUES ($1,$2,$3,$4,$5) RETURNING id,sku,name,category,unit,selling_price,active,created_at")
        .bind(sku).bind(name).bind(category).bind(unit).bind(input.selling_price)
        .fetch_one(&s.db).await.map_err(internal)?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Serialize, FromRow)]
pub struct BalanceRow { pub product_id: Uuid, pub sku: String, pub name: String, pub unit: String, pub quantity: Decimal }
pub async fn inventory_balances(State(s): State<AppState>, user: AuthenticatedUser) -> Result<Json<Vec<BalanceRow>>, (StatusCode, String)> {
    require_permission(&s,user.id,"inventory.read").await?;
    let rows = sqlx::query_as::<_, BalanceRow>("SELECT p.id product_id,p.sku,p.name,p.unit,COALESCE(SUM(m.quantity_delta),0)::numeric quantity FROM products p LEFT JOIN inventory_movements m ON m.product_id=p.id GROUP BY p.id ORDER BY p.name")
        .fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(rows))
}
#[derive(Deserialize)]
pub struct CreateMovement { pub product_id: Uuid, pub kind: String, pub quantity_delta: Decimal, pub reason: String }
#[derive(Serialize, FromRow)]
pub struct MovementRow { pub id: Uuid, pub product_id: Uuid, pub kind: String, pub quantity_delta: Decimal, pub reason: String, pub created_at: DateTime<Utc> }
pub async fn create_movement(State(s): State<AppState>, user: AuthenticatedUser, Json(input): Json<CreateMovement>) -> Result<(StatusCode, Json<MovementRow>), (StatusCode, String)> {
    if input.quantity_delta == Decimal::ZERO || input.reason.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "A non-zero quantity and a reason are required".into()));
    }
    // Only supplier/opening receipts are exposed through this early API slice.
    // Sales, production, waste, returns and adjustments must be posted by their
    // own atomic workflows; allowing clients to submit them here would bypass controls.
    if input.kind == "adjustment" {
        return Err((StatusCode::CONFLICT, "Stock adjustments require the approval workflow, which is not yet enabled".into()));
    }
    if input.kind != "receipt" {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "This movement type is reserved for its dedicated business workflow".into()));
    }
    require_permission(&s,user.id,"inventory.receive").await?;
    let mut tx = s.db.begin().await.map_err(internal)?;
    sqlx::query("SELECT id FROM products WHERE id=$1 FOR UPDATE").bind(input.product_id).fetch_optional(&mut *tx).await.map_err(internal)?.ok_or((StatusCode::NOT_FOUND, "Product not found".into()))?;
    let current: Decimal = sqlx::query_scalar("SELECT COALESCE(SUM(quantity_delta),0)::numeric FROM inventory_movements WHERE product_id=$1").bind(input.product_id).fetch_one(&mut *tx).await.map_err(internal)?;
    if current + input.quantity_delta < Decimal::ZERO { return Err((StatusCode::CONFLICT, "Movement would make stock negative".into())); }
    let row = sqlx::query_as::<_, MovementRow>("INSERT INTO inventory_movements (product_id,kind,quantity_delta,reason,actor_user_id) VALUES ($1,$2,$3,$4,$5) RETURNING id,product_id,kind,quantity_delta,reason,created_at")
        .bind(input.product_id).bind(input.kind).bind(input.quantity_delta).bind(input.reason.trim()).bind(user.id).fetch_one(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok((StatusCode::CREATED, Json(row)))
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) { tracing::error!(error=%e, "request failed"); (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()) }

// ---------------------------- Identity & access ----------------------------
use axum::http::header::AUTHORIZATION;
use crate::auth::{AuthenticatedUser, hash_password, hash_token, new_token, require_permission, verify_password};
use subtle::ConstantTimeEq;

#[derive(Deserialize)]
pub struct BootstrapAdmin { pub bootstrap_secret: String, pub username: String, pub display_name: String, pub password: String }
#[derive(Serialize)]
pub struct AuthResponse { pub access_token: String, pub token_type: &'static str, pub expires_in_seconds: i64, pub user: UserSummary }
#[derive(Serialize)]
pub struct UserSummary { pub id: Uuid, pub username: String, pub display_name: String, pub roles: Vec<String> }

pub async fn bootstrap_admin(State(s): State<AppState>, Json(input): Json<BootstrapAdmin>) -> Result<(StatusCode, Json<UserSummary>), (StatusCode,String)> {
    let configured = std::env::var("BOOTSTRAP_SECRET").unwrap_or_default();
    if configured.len() < 24 || input.bootstrap_secret.as_bytes().ct_eq(configured.as_bytes()).unwrap_u8() != 1 {
        return Err((StatusCode::FORBIDDEN,"Initial administrator setup is disabled or the setup secret is invalid".into()));
    }
    let username = input.username.trim().to_ascii_lowercase();
    let display_name = input.display_name.trim();
    if username.len() < 3 || username.len() > 64 || !username.chars().all(|c| c.is_ascii_alphanumeric() || c=='.' || c=='_' || c=='-') || display_name.is_empty() || display_name.chars().count()>120 {
        return Err((StatusCode::BAD_REQUEST,"Enter a valid username and display name".into()));
    }
    let password_hash = hash_password(&input.password).map_err(|e|(StatusCode::BAD_REQUEST,e))?;
    let mut tx=s.db.begin().await.map_err(internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock(7219042026)").execute(&mut *tx).await.map_err(internal)?;
    let count:i64=sqlx::query_scalar("SELECT COUNT(*) FROM app_users").fetch_one(&mut *tx).await.map_err(internal)?;
    if count>0 { return Err((StatusCode::CONFLICT,"Initial administrator has already been created".into())); }
    let row=sqlx::query("INSERT INTO app_users(username,display_name,password_hash,must_change_password) VALUES($1,$2,$3,false) RETURNING id,username,display_name")
        .bind(username).bind(display_name).bind(password_hash).fetch_one(&mut *tx).await.map_err(internal)?;
    let user_id:Uuid=row.get("id");
    sqlx::query("INSERT INTO user_roles(user_id,role_id) SELECT $1,id FROM roles WHERE code='owner'").bind(user_id).execute(&mut *tx).await.map_err(internal)?;
    sqlx::query("INSERT INTO audit_events(actor_user_id,action,target_type,target_id,reason) VALUES($1,'identity.bootstrap','user',$1,'Initial administrator bootstrap')").bind(user_id).execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok((StatusCode::CREATED,Json(UserSummary{id:user_id,username:row.get("username"),display_name:row.get("display_name"),roles:vec!["owner".into()]})))
}

#[derive(Deserialize)] pub struct LoginRequest { pub username:String, pub password:String }
pub async fn login(State(s):State<AppState>,Json(input):Json<LoginRequest>)->Result<Json<AuthResponse>,(StatusCode,String)> {
    let username=input.username.trim().to_ascii_lowercase();
    if username.len()>64 || input.password.len()>256 { return Err((StatusCode::UNAUTHORIZED,"Invalid username or password".into())); }
    let row=sqlx::query("SELECT id,username,display_name,password_hash,is_active,failed_login_count,locked_until FROM app_users WHERE username=$1")
        .bind(&username).fetch_optional(&s.db).await.map_err(internal)?;
    let Some(row)=row else { return Err((StatusCode::UNAUTHORIZED,"Invalid username or password".into())); };
    let user_id:Uuid=row.get("id"); let active:bool=row.get("is_active");
    let locked_until:Option<DateTime<Utc>>=row.get("locked_until");
    let stored_hash:String=row.get("password_hash");
    if !active || locked_until.map(|t|t>Utc::now()).unwrap_or(false) || !verify_password(&input.password,&stored_hash) {
        if active && locked_until.map(|t|t<=Utc::now()).unwrap_or(true) {
            let _=sqlx::query("UPDATE app_users SET failed_login_count=failed_login_count+1, locked_until=CASE WHEN failed_login_count+1 >= 8 THEN now()+interval '15 minutes' ELSE locked_until END WHERE id=$1").bind(user_id).execute(&s.db).await;
        }
        return Err((StatusCode::UNAUTHORIZED,"Invalid username or password".into()));
    }
    sqlx::query("UPDATE app_users SET failed_login_count=0,locked_until=NULL,last_login_at=now() WHERE id=$1").bind(user_id).execute(&s.db).await.map_err(internal)?;
    let token=new_token(); let token_hash=hash_token(&token); let expires_in=12*60*60i64;
    sqlx::query("INSERT INTO auth_sessions(user_id,token_hash,expires_at) VALUES($1,$2,now()+($3::double precision * interval '1 second'))").bind(user_id).bind(token_hash).bind(expires_in).execute(&s.db).await.map_err(internal)?;
    let roles=sqlx::query_scalar::<_,String>("SELECT r.code FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=$1 ORDER BY r.code").bind(user_id).fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(AuthResponse{access_token:token,token_type:"Bearer",expires_in_seconds:expires_in,user:UserSummary{id:user_id,username:row.get("username"),display_name:row.get("display_name"),roles}}))
}

pub async fn logout(State(s):State<AppState>,headers:axum::http::HeaderMap,user:AuthenticatedUser)->Result<StatusCode,(StatusCode,String)> {
    let token=headers.get(AUTHORIZATION).and_then(|v|v.to_str().ok()).and_then(|v|v.strip_prefix("Bearer ")).unwrap_or("");
    sqlx::query("UPDATE auth_sessions SET revoked_at=now() WHERE user_id=$1 AND token_hash=$2 AND revoked_at IS NULL").bind(user.id).bind(hash_token(token)).execute(&s.db).await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(State(s):State<AppState>,user:AuthenticatedUser)->Result<Json<UserSummary>,(StatusCode,String)> {
    let roles=sqlx::query_scalar::<_,String>("SELECT r.code FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=$1 ORDER BY r.code").bind(user.id).fetch_all(&s.db).await.map_err(internal)?;
    Ok(Json(UserSummary{id:user.id,username:user.username,display_name:user.display_name,roles}))
}
