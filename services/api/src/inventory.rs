/// Inventory management endpoints
/// Handles product CRUD with extended inventory fields, stock movements, adjustments, and physical counts
use crate::{auth::AuthenticatedUser, AppState};
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct ProductDetailRow {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub product_type: String,
    pub unit: String,
    pub barcode: Option<String>,
    pub selling_price: Decimal,
    pub purchase_price: Option<Decimal>,
    pub tax_rate: Decimal,
    pub supplier_id: Option<Uuid>,
    pub minimum_stock_threshold: Option<Decimal>,
    pub reorder_quantity: Option<Decimal>,
    pub maximum_stock_threshold: Option<Decimal>,
    pub storage_location: Option<String>,
    pub is_perishable: bool,
    pub image_path: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: String,
    pub description: String,
    pub barcode: Option<String>,
    pub category: String,
    pub product_type: String,
    pub unit: String,
    pub selling_price: Decimal,
    pub purchase_price: Option<Decimal>,
    pub tax_rate: Decimal,
    pub supplier_id: Option<Uuid>,
    pub minimum_stock_threshold: Option<Decimal>,
    pub reorder_quantity: Option<Decimal>,
    pub maximum_stock_threshold: Option<Decimal>,
    pub storage_location: Option<String>,
    pub is_perishable: bool,
}

/// GET /api/v1/inventory/products/:id
pub async fn get_product(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<ProductDetailRow>, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "catalog.read").await?;
    
    let row = sqlx::query_as::<_, ProductDetailRow>(
        "SELECT id, sku, name, description, category, product_type, unit, barcode, selling_price, 
                purchase_price, tax_rate, supplier_id, minimum_stock_threshold, reorder_quantity, 
                maximum_stock_threshold, storage_location, is_perishable, image_path, active, created_at, updated_at
         FROM products WHERE id = $1"
    )
    .bind(product_id)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| {
        tracing::error!(error=%e, "product lookup failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".into(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Product not found".into()))?;

    Ok(Json(row))
}

/// PUT /api/v1/inventory/products/:id
pub async fn update_product(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
    Json(input): Json<UpdateProductRequest>,
) -> Result<Json<ProductDetailRow>, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "catalog.manage").await?;

    let name = input.name.trim();
    let description = input.description.trim();
    let category = input.category.trim();
    let unit = input.unit.trim();
    let product_type = input.product_type.trim();

    // Validate
    if name.is_empty() || name.chars().count() > 160 {
        return Err((StatusCode::BAD_REQUEST, "Invalid product name".into()));
    }
    if category.is_empty() || category.chars().count() > 80 {
        return Err((StatusCode::BAD_REQUEST, "Invalid category".into()));
    }
    if unit.is_empty() || unit.chars().count() > 24 {
        return Err((StatusCode::BAD_REQUEST, "Invalid unit".into()));
    }
    if !["ingredient", "finished_product", "packaging", "other"].contains(&product_type) {
        return Err((StatusCode::BAD_REQUEST, "Invalid product type".into()));
    }
    if input.selling_price < Decimal::ZERO {
        return Err((StatusCode::BAD_REQUEST, "Selling price cannot be negative".into()));
    }
    if let Some(pp) = input.purchase_price {
        if pp < Decimal::ZERO {
            return Err((StatusCode::BAD_REQUEST, "Purchase price cannot be negative".into()));
        }
    }
    if input.tax_rate < Decimal::ZERO || input.tax_rate > Decimal::from(100) {
        return Err((StatusCode::BAD_REQUEST, "Tax rate must be between 0 and 100".into()));
    }

    let row = sqlx::query_as::<_, ProductDetailRow>(
        "UPDATE products 
         SET name = $1, description = $2, category = $3, unit = $4, product_type = $5,
             barcode = $6, selling_price = $7, purchase_price = $8, tax_rate = $9,
             supplier_id = $10, minimum_stock_threshold = $11, reorder_quantity = $12,
             maximum_stock_threshold = $13, storage_location = $14, is_perishable = $15,
             updated_at = now()
         WHERE id = $16
         RETURNING id, sku, name, description, category, product_type, unit, barcode, selling_price, 
                   purchase_price, tax_rate, supplier_id, minimum_stock_threshold, reorder_quantity, 
                   maximum_stock_threshold, storage_location, is_perishable, image_path, active, created_at, updated_at"
    )
    .bind(name)
    .bind(description)
    .bind(category)
    .bind(unit)
    .bind(product_type)
    .bind(input.barcode.map(|b| b.trim().to_string()))
    .bind(input.selling_price)
    .bind(input.purchase_price)
    .bind(input.tax_rate)
    .bind(input.supplier_id)
    .bind(input.minimum_stock_threshold)
    .bind(input.reorder_quantity)
    .bind(input.maximum_stock_threshold)
    .bind(input.storage_location.map(|s| s.trim().to_string()))
    .bind(input.is_perishable)
    .bind(product_id)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| {
        tracing::error!(error=%e, "product update failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".into(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Product not found".into()))?;

    Ok(Json(row))
}

/// POST /api/v1/inventory/products/:id/archive
pub async fn archive_product(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "catalog.manage").await?;

    let reason = body
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Archived by user")
        .trim()
        .to_string();

    let mut tx = s.db.begin().await.map_err(internal)?;

    // Verify product exists
    let _: Uuid = sqlx::query_scalar("SELECT id FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, "Product not found".into()))?;

    // Archive the product
    sqlx::query("UPDATE products SET active = false WHERE id = $1")
        .bind(product_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    // Record the archival
    sqlx::query(
        "INSERT INTO archived_products(product_id, archived_by, reason) VALUES($1, $2, $3)"
    )
    .bind(product_id)
    .bind(user.id)
    .bind(&reason)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    // Log audit event
    sqlx::query(
        "INSERT INTO audit_events(actor_user_id, action, target_type, target_id, reason) 
         VALUES($1, 'inventory.archive_product', 'product', $2, $3)"
    )
    .bind(user.id)
    .bind(product_id)
    .bind(reason)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    tx.commit().await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/inventory/products/:id/reactivate
pub async fn reactivate_product(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "catalog.manage").await?;

    let mut tx = s.db.begin().await.map_err(internal)?;

    // Reactivate the product
    let count = sqlx::query("UPDATE products SET active = true WHERE id = $1 AND active = false")
        .bind(product_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?
        .rows_affected();

    if count == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Product not found or is already active".into(),
        ));
    }

    // Record the reactivation
    sqlx::query(
        "UPDATE archived_products SET restored_at = now(), restored_by = $1 WHERE product_id = $2 AND restored_at IS NULL"
    )
    .bind(user.id)
    .bind(product_id)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    // Log audit event
    sqlx::query(
        "INSERT INTO audit_events(actor_user_id, action, target_type, target_id, reason) 
         VALUES($1, 'inventory.reactivate_product', 'product', $2, 'Product reactivated')"
    )
    .bind(user.id)
    .bind(product_id)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    tx.commit().await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize, FromRow)]
pub struct MovementHistoryRow {
    pub id: Uuid,
    pub product_id: Uuid,
    pub kind: String,
    pub quantity_delta: Decimal,
    pub reason: String,
    pub lot_id: Option<Uuid>,
    pub batch_code: Option<String>,
    pub actor_user_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// GET /api/v1/inventory/movements?product_id=xxx&kind=receipt&limit=100
pub async fn get_movements(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<MovementQuery>,
) -> Result<Json<Vec<MovementHistoryRow>>, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "inventory.read").await?;

    let limit = (params.limit.unwrap_or(100)).min(1000);
    let offset = params.offset.unwrap_or(0);

    let mut query = String::from(
        "SELECT id, product_id, kind, quantity_delta, reason, lot_id, batch_code, 
                actor_user_id, reference_type, reference_id, created_at
         FROM inventory_movements WHERE 1=1"
    );

    if let Some(product_id) = params.product_id {
        query.push_str(" AND product_id = '");
        query.push_str(&product_id.to_string());
        query.push('\'');
    }

    if let Some(kind) = &params.kind {
        query.push_str(" AND kind = '");
        query.push_str(kind);
        query.push('\'');
    }

    query.push_str(" ORDER BY created_at DESC LIMIT ");
    query.push_str(&limit.to_string());
    query.push_str(" OFFSET ");
    query.push_str(&offset.to_string());

    let rows = sqlx::query_as::<_, MovementHistoryRow>(&query)
        .fetch_all(&s.db)
        .await
        .map_err(internal)?;

    Ok(Json(rows))
}

#[derive(Debug, serde::Deserialize)]
pub struct MovementQuery {
    pub product_id: Option<Uuid>,
    pub kind: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockMovementSummary {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub unit: String,
    pub total_quantity: Decimal,
    pub low_stock: bool,
    pub minimum_threshold: Option<Decimal>,
}

/// GET /api/v1/inventory/stock-status?category=ingredient
pub async fn get_stock_status(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<StockStatusQuery>,
) -> Result<Json<Vec<StockMovementSummary>>, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "inventory.read").await?;

    let mut query_str = String::from(
        "SELECT p.id product_id, p.sku, p.name, p.unit, COALESCE(SUM(m.quantity_delta), 0)::numeric total_quantity,
                (COALESCE(SUM(m.quantity_delta), 0) <= COALESCE(p.minimum_stock_threshold, 0))::boolean low_stock,
                p.minimum_stock_threshold
         FROM products p
         LEFT JOIN inventory_movements m ON m.product_id = p.id
         WHERE p.active = true"
    );

    if let Some(category) = &params.category {
        query_str.push_str(" AND p.category = '");
        query_str.push_str(category);
        query_str.push('\'');
    }

    if params.low_stock_only.unwrap_or(false) {
        query_str.push_str(
            " AND COALESCE(SUM(m.quantity_delta), 0) <= COALESCE(p.minimum_stock_threshold, 0)",
        );
    }

    query_str.push_str(" GROUP BY p.id, p.sku, p.name, p.unit, p.minimum_stock_threshold");
    query_str.push_str(" ORDER BY p.name");

    let rows = sqlx::query_as::<_, StockMovementSummary>(&query_str)
        .fetch_all(&s.db)
        .await
        .map_err(internal)?;

    Ok(Json(rows))
}

#[derive(Debug, serde::Deserialize)]
pub struct StockStatusQuery {
    pub category: Option<String>,
    pub low_stock_only: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PhysicalCountRow {
    pub id: Uuid,
    pub count_number: String,
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub counted_quantity: Decimal,
    pub system_quantity: Decimal,
    pub variance: Decimal,
    pub counted_at: DateTime<Utc>,
    pub counted_by_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreatePhysicalCountRequest {
    pub product_id: Uuid,
    pub lot_id: Option<Uuid>,
    pub counted_quantity: Decimal,
    pub notes: Option<String>,
}

/// POST /api/v1/inventory/physical-counts
pub async fn create_physical_count(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<CreatePhysicalCountRequest>,
) -> Result<(StatusCode, Json<PhysicalCountRow>), (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "inventory.read").await?;

    if input.counted_quantity < Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            "Counted quantity cannot be negative".into(),
        ));
    }

    let mut tx = s.db.begin().await.map_err(internal)?;

    // Get product
    let product: (Uuid, String, String, String) = sqlx::query_as(
        "SELECT id, sku, name, unit FROM products WHERE id = $1 AND active = true"
    )
    .bind(input.product_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?
    .ok_or((StatusCode::NOT_FOUND, "Product not found".into()))?;

    // Get system balance
    let system_qty: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(quantity_delta), 0)::numeric FROM inventory_movements WHERE product_id = $1"
    )
    .bind(input.product_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;

    let count_number = format!(
        "COUNT-{}-{}",
        Uuid::new_v4().to_string()[..8].to_uppercase(),
        chrono::Local::now().format("%Y%m%d%H%M%S")
    );

    let row = sqlx::query_as::<_, (Uuid, String, String, Decimal, Decimal, DateTime<Utc>)>(
        "INSERT INTO physical_stock_counts(count_number, product_id, lot_id, counted_quantity, system_quantity, counted_at, counted_by, notes)
         VALUES($1, $2, $3, $4, $5, now(), $6, $7)
         RETURNING id, count_number, (SELECT sku FROM products WHERE id = $2), (SELECT name FROM products WHERE id = $2),
                   counted_quantity, system_quantity, now()"
    )
    .bind(&count_number)
    .bind(input.product_id)
    .bind(input.lot_id)
    .bind(input.counted_quantity)
    .bind(system_qty)
    .bind(user.id)
    .bind(input.notes.unwrap_or_default())
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;

    // Log audit event
    sqlx::query(
        "INSERT INTO audit_events(actor_user_id, action, target_type, target_id, reason) 
         VALUES($1, 'inventory.physical_count', 'product', $2, $3)"
    )
    .bind(user.id)
    .bind(input.product_id)
    .bind(format!("Physical count: {} vs system {}", input.counted_quantity, system_qty))
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    tx.commit().await.map_err(internal)?;

    Ok((
        StatusCode::CREATED,
        Json(PhysicalCountRow {
            id: row.0,
            count_number: row.1,
            product_id: input.product_id,
            sku: product.1,
            name: product.2,
            counted_quantity: row.3,
            system_quantity: row.4,
            variance: row.3 - row.4,
            counted_at: row.5,
            counted_by_id: user.id,
        }),
    ))
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    tracing::error!(error=%e, "request failed");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".into(),
    )
}
