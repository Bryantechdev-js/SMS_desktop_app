//! Procurement Module - Supplier and Purchase Order Management
//! Phase 4 Implementation

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Supplier record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Supplier {
    pub id: Uuid,
    pub name: String,
    pub contact_person: String,
    pub phone: String,
    pub email: String,
    pub physical_address: Option<String>,
    pub payment_terms: Option<String>,
    pub tax_id: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<Uuid>,
}

/// Create supplier request
#[derive(Debug, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub contact_person: String,
    pub phone: String,
    pub email: String,
    pub physical_address: Option<String>,
    pub payment_terms: Option<String>,
    pub tax_id: Option<String>,
}

/// Purchase order record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PurchaseOrder {
    pub id: Uuid,
    pub po_number: String,
    pub supplier_id: Uuid,
    pub expected_delivery_date: Option<NaiveDate>,
    pub status: String,
    pub cancellation_reason: Option<String>,
    pub subtotal_amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub submitted_at: Option<DateTime<Utc>>,
    pub submitted_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<Uuid>,
}

/// Purchase order item
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PurchaseOrderItem {
    pub id: Uuid,
    pub purchase_order_id: Uuid,
    pub product_id: Uuid,
    pub quantity_ordered: Decimal,
    pub unit_price: Decimal,
    pub line_total: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Line item for creating a purchase order
#[derive(Debug, Deserialize)]
pub struct CreatePOLineItem {
    pub product_id: Uuid,
    pub quantity_ordered: Decimal,
    pub unit_price: Decimal,
    pub notes: Option<String>,
}

/// Create purchase order request
#[derive(Debug, Deserialize)]
pub struct CreatePurchaseOrderRequest {
    pub supplier_id: Uuid,
    pub expected_delivery_date: Option<NaiveDate>,
    pub line_items: Vec<CreatePOLineItem>,
}

/// Purchase receipt record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PurchaseReceipt {
    pub id: Uuid,
    pub receipt_number: String,
    pub purchase_order_id: Uuid,
    pub receipt_date: NaiveDate,
    pub supplier_invoice_number: Option<String>,
    pub supplier_invoice_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Create receipt request
#[derive(Debug, Deserialize)]
pub struct CreateReceiptRequest {
    pub purchase_order_id: Uuid,
    pub receipt_date: NaiveDate,
    pub supplier_invoice_number: Option<String>,
    pub supplier_invoice_date: Option<NaiveDate>,
    pub line_items: Vec<CreateReceiptLineItem>,
}

/// Line item for creating a receipt
#[derive(Debug, Deserialize)]
pub struct CreateReceiptLineItem {
    pub purchase_order_item_id: Uuid,
    pub quantity_received: Decimal,
    pub quality_status: String,
    pub discrepancy_notes: Option<String>,
}

// ============================================================================
// SUPPLIER OPERATIONS
// ============================================================================

/// Create a new supplier
pub async fn create_supplier(
    db: &PgPool,
    req: CreateSupplierRequest,
    user_id: Uuid,
) -> Result<Supplier, String> {
    let supplier_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query_as::<_, Supplier>(
        r#"
        INSERT INTO suppliers 
        (id, name, contact_person, phone, email, physical_address, payment_terms, tax_id, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, name, contact_person, phone, email, physical_address, payment_terms, tax_id,
                  active, created_at, created_by, updated_at, updated_by
        "#
    )
    .bind(supplier_id)
    .bind(req.name)
    .bind(req.contact_person)
    .bind(req.phone)
    .bind(req.email)
    .bind(req.physical_address)
    .bind(req.payment_terms)
    .bind(req.tax_id)
    .bind(user_id)
    .bind(now)
    .fetch_one(db)
    .await
    .map_err(|e| format!("Failed to create supplier: {}", e))
}

/// Get a supplier by ID
pub async fn get_supplier(db: &PgPool, supplier_id: Uuid) -> Result<Supplier, String> {
    sqlx::query_as::<_, Supplier>(
        r#"
        SELECT id, name, contact_person, phone, email, physical_address, payment_terms, tax_id,
               active, created_at, created_by, updated_at, updated_by
        FROM suppliers
        WHERE id = $1
        "#
    )
    .bind(supplier_id)
    .fetch_optional(db)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Supplier not found".to_string())
}

/// List suppliers with search and filter
pub async fn list_suppliers(
    db: &PgPool,
    search_query: Option<String>,
    active_only: bool,
    limit: i32,
    offset: i32,
) -> Result<Vec<Supplier>, String> {
    if let Some(q) = search_query {
        let search = format!("%{}%", q.to_lowercase());
        sqlx::query_as::<_, Supplier>(
            r#"
            SELECT id, name, contact_person, phone, email, physical_address, payment_terms, tax_id,
                   active, created_at, created_by, updated_at, updated_by
            FROM suppliers
            WHERE (LOWER(name) LIKE $1 OR LOWER(contact_person) LIKE $1)
            AND (active = TRUE OR $3 = FALSE)
            ORDER BY name
            LIMIT $2 OFFSET $4
            "#
        )
        .bind(search)
        .bind(limit)
        .bind(active_only)
        .bind(offset)
        .fetch_all(db)
        .await
        .map_err(|e| format!("Database error: {}", e))
    } else {
        sqlx::query_as::<_, Supplier>(
            r#"
            SELECT id, name, contact_person, phone, email, physical_address, payment_terms, tax_id,
                   active, created_at, created_by, updated_at, updated_by
            FROM suppliers
            WHERE active = TRUE OR $1 = FALSE
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(active_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await
        .map_err(|e| format!("Database error: {}", e))
    }
}

/// Archive a supplier
pub async fn archive_supplier(
    db: &PgPool,
    supplier_id: Uuid,
) -> Result<Supplier, String> {
    let now = Utc::now();
    
    sqlx::query_as::<_, Supplier>(
        r#"
        UPDATE suppliers
        SET active = FALSE, updated_at = $1
        WHERE id = $2
        RETURNING id, name, contact_person, phone, email, physical_address, payment_terms, tax_id,
                  active, created_at, created_by, updated_at, updated_by
        "#
    )
    .bind(now)
    .bind(supplier_id)
    .fetch_one(db)
    .await
    .map_err(|e| format!("Failed to archive supplier: {}", e))
}

// ============================================================================
// PURCHASE ORDER OPERATIONS
// ============================================================================

/// Generate the next PO number
pub async fn generate_po_number(db: &PgPool) -> Result<String, String> {
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    
    let result: (Option<i32>,) = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(CAST(SUBSTRING(po_number FROM 10) AS INTEGER)), 0)
        FROM purchase_orders
        WHERE po_number LIKE $1 || '%'
        "#
    )
    .bind(format!("PO-{}-", year_month))
    .fetch_one(db)
    .await
    .map_err(|e| format!("Failed to generate PO number: {}", e))?;

    let next_seq = result.0.unwrap_or(0) + 1;
    Ok(format!("PO-{}-{:03}", year_month, next_seq))
}

/// Create a new purchase order
pub async fn create_purchase_order(
    db: &PgPool,
    req: CreatePurchaseOrderRequest,
    user_id: Uuid,
) -> Result<PurchaseOrder, String> {
    let po_id = Uuid::new_v4();
    let po_number = generate_po_number(db).await?;
    let now = Utc::now();

    // Calculate subtotal
    let mut subtotal = Decimal::from(0);
    for item in &req.line_items {
        let line_total = item.quantity_ordered * item.unit_price;
        subtotal += line_total;
    }

    let mut tx = db.begin().await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Insert PO header
    sqlx::query(
        r#"
        INSERT INTO purchase_orders
        (id, po_number, supplier_id, expected_delivery_date, status, subtotal_amount, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(po_id)
    .bind(&po_number)
    .bind(req.supplier_id)
    .bind(req.expected_delivery_date)
    .bind("draft")
    .bind(subtotal)
    .bind(user_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to create PO: {}", e))?;

    // Insert line items
    for item in req.line_items {
        let item_id = Uuid::new_v4();
        let line_total = item.quantity_ordered * item.unit_price;
        
        sqlx::query(
            r#"
            INSERT INTO purchase_order_items
            (id, purchase_order_id, product_id, quantity_ordered, unit_price, line_total, notes, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(item_id)
        .bind(po_id)
        .bind(item.product_id)
        .bind(item.quantity_ordered)
        .bind(item.unit_price)
        .bind(line_total)
        .bind(item.notes)
        .bind(user_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert PO item: {}", e))?;
    }

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    get_purchase_order(db, po_id).await
}

/// Get a purchase order
pub async fn get_purchase_order(db: &PgPool, po_id: Uuid) -> Result<PurchaseOrder, String> {
    sqlx::query_as::<_, PurchaseOrder>(
        r#"
        SELECT id, po_number, supplier_id, expected_delivery_date, status, cancellation_reason,
               subtotal_amount, created_at, created_by, submitted_at, submitted_by, cancelled_at,
               cancelled_by, updated_at, updated_by
        FROM purchase_orders
        WHERE id = $1
        "#
    )
    .bind(po_id)
    .fetch_optional(db)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Purchase order not found".to_string())
}

/// List purchase orders
pub async fn list_purchase_orders(
    db: &PgPool,
    limit: i32,
    offset: i32,
) -> Result<Vec<PurchaseOrder>, String> {
    sqlx::query_as::<_, PurchaseOrder>(
        r#"
        SELECT id, po_number, supplier_id, expected_delivery_date, status, cancellation_reason,
               subtotal_amount, created_at, created_by, submitted_at, submitted_by, cancelled_at,
               cancelled_by, updated_at, updated_by
        FROM purchase_orders
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| format!("Database error: {}", e))
}

/// Submit a purchase order
pub async fn submit_purchase_order(
    db: &PgPool,
    po_id: Uuid,
    user_id: Uuid,
) -> Result<PurchaseOrder, String> {
    let now = Utc::now();
    
    sqlx::query(
        r#"
        UPDATE purchase_orders
        SET status = 'submitted', submitted_at = $1, submitted_by = $2
        WHERE id = $3 AND status = 'draft'
        "#
    )
    .bind(now)
    .bind(user_id)
    .bind(po_id)
    .execute(db)
    .await
    .map_err(|e| format!("Failed to submit PO: {}", e))?;

    get_purchase_order(db, po_id).await
}

/// Cancel a purchase order
pub async fn cancel_purchase_order(
    db: &PgPool,
    po_id: Uuid,
    reason: String,
    user_id: Uuid,
) -> Result<PurchaseOrder, String> {
    let now = Utc::now();
    
    sqlx::query(
        r#"
        UPDATE purchase_orders
        SET status = 'cancelled', cancellation_reason = $1, cancelled_at = $2, cancelled_by = $3
        WHERE id = $4
        "#
    )
    .bind(reason)
    .bind(now)
    .bind(user_id)
    .bind(po_id)
    .execute(db)
    .await
    .map_err(|e| format!("Failed to cancel PO: {}", e))?;

    get_purchase_order(db, po_id).await
}

// ============================================================================
// RECEIPT OPERATIONS
// ============================================================================

/// Generate receipt number
pub async fn generate_receipt_number(db: &PgPool) -> Result<String, String> {
    let now = chrono::Local::now();
    let year_month = now.format("%Y-%m").to_string();
    
    let result: (Option<i32>,) = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(CAST(SUBSTRING(receipt_number FROM 10) AS INTEGER)), 0)
        FROM purchase_receipts
        WHERE receipt_number LIKE $1 || '%'
        "#
    )
    .bind(format!("RCP-{}-", year_month))
    .fetch_one(db)
    .await
    .map_err(|e| format!("Failed to generate receipt number: {}", e))?;

    let next_seq = result.0.unwrap_or(0) + 1;
    Ok(format!("RCP-{}-{:03}", year_month, next_seq))
}

/// Create a goods receipt
pub async fn create_receipt(
    db: &PgPool,
    req: CreateReceiptRequest,
    user_id: Uuid,
) -> Result<PurchaseReceipt, String> {
    let receipt_id = Uuid::new_v4();
    let receipt_number = generate_receipt_number(db).await?;
    let now = Utc::now();

    let mut tx = db.begin().await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Insert receipt
    sqlx::query(
        r#"
        INSERT INTO purchase_receipts
        (id, receipt_number, purchase_order_id, receipt_date, supplier_invoice_number, 
         supplier_invoice_date, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(receipt_id)
    .bind(&receipt_number)
    .bind(req.purchase_order_id)
    .bind(req.receipt_date)
    .bind(req.supplier_invoice_number)
    .bind(req.supplier_invoice_date)
    .bind(user_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to create receipt: {}", e))?;

    // Insert receipt items and create stock movements
    for item in req.line_items {
        let receipt_item_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO purchase_receipt_items
            (id, purchase_receipt_id, purchase_order_item_id, quantity_received, quality_status, discrepancy_notes, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(receipt_item_id)
        .bind(receipt_id)
        .bind(item.purchase_order_item_id)
        .bind(item.quantity_received)
        .bind(&item.quality_status)
        .bind(item.discrepancy_notes)
        .bind(user_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert receipt item: {}", e))?;

        // Get product_id
        let po_item: (Uuid,) = sqlx::query_as(
            "SELECT product_id FROM purchase_order_items WHERE id = $1"
        )
        .bind(item.purchase_order_item_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to get PO item: {}", e))?;

        // Create stock movement
        let movement_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO inventory_movements
            (id, product_id, kind, quantity, movement_reason, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(movement_id)
        .bind(po_item.0)
        .bind("receipt")
        .bind(item.quantity_received)
        .bind(format!("Purchase receipt {}", receipt_number))
        .bind(user_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create stock movement: {}", e))?;
    }

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    get_receipt(db, receipt_id).await
}

/// Get a receipt
pub async fn get_receipt(db: &PgPool, receipt_id: Uuid) -> Result<PurchaseReceipt, String> {
    sqlx::query_as::<_, PurchaseReceipt>(
        r#"
        SELECT id, receipt_number, purchase_order_id, receipt_date, supplier_invoice_number,
               supplier_invoice_date, created_at, created_by
        FROM purchase_receipts
        WHERE id = $1
        "#
    )
    .bind(receipt_id)
    .fetch_optional(db)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Receipt not found".to_string())
}

/// Get PO items
pub async fn get_po_items(db: &PgPool, po_id: Uuid) -> Result<Vec<PurchaseOrderItem>, String> {
    sqlx::query_as::<_, PurchaseOrderItem>(
        r#"
        SELECT id, purchase_order_id, product_id, quantity_ordered, unit_price, line_total,
               notes, created_at, created_by
        FROM purchase_order_items
        WHERE purchase_order_id = $1
        "#
    )
    .bind(po_id)
    .fetch_all(db)
    .await
    .map_err(|e| format!("Database error: {}", e))
}
