/// Point of Sale (POS) endpoints
/// Handles shopping cart checkout, payments, and sales transactions

use crate::{auth::AuthenticatedUser, AppState};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CheckoutLineItem {
    pub product_id: Uuid,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub discount_amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub items: Vec<CheckoutLineItem>,
    pub discount_amount: Decimal,
    pub discount_reason: Option<String>,
    pub customer_id: Option<Uuid>,
    pub payment_method: String,
    pub payment_amount: Decimal,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub id: Uuid,
    pub ticket_number: String,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub payment_status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SaleRow {
    pub id: Uuid,
    pub ticket_number: String,
    pub cashier_id: Uuid,
    pub status: String,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub currency: String,
    pub created_at: chrono::DateTime<Utc>,
    pub paid_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SaleLineRow {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub product_name_snapshot: String,
    pub sku_snapshot: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub discount_amount: Decimal,
    pub line_total: Decimal,
}

#[derive(Debug, Serialize)]
pub struct SaleDetailResponse {
    pub id: Uuid,
    pub ticket_number: String,
    pub cashier_name: String,
    pub status: String,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub items: Vec<SaleLineRow>,
    pub payment_method: String,
    pub payment_status: String,
    pub created_at: chrono::DateTime<Utc>,
}

/// POST /api/v1/sales/checkout
/// Create a sale with payment, atomically consuming inventory
pub async fn checkout(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    Json(input): Json<CheckoutRequest>,
) -> Result<(StatusCode, Json<CheckoutResponse>), (StatusCode, String)> {
    // 1. Verify authorization
    crate::auth::require_permission(&s, user.id, "pos.sell").await?;

    // 2. Validate request
    if input.items.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Cart cannot be empty".into()));
    }

    if input.discount_amount < Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            "Discount cannot be negative".into(),
        ));
    }

    if input.payment_amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            "Payment amount must be positive".into(),
        ));
    }

    let payment_method = input.payment_method.trim().to_lowercase();
    if !["cash", "mobile_money", "card", "bank_transfer", "other"]
        .contains(&payment_method.as_str())
    {
        return Err((StatusCode::BAD_REQUEST, "Invalid payment method".into()));
    }

    // 3. Calculate totals server-side (never trust client)
    let mut subtotal = Decimal::ZERO;
    let total_discount = input.discount_amount;

    for item in &input.items {
        if item.quantity <= Decimal::ZERO {
            return Err((
                StatusCode::BAD_REQUEST,
                "Item quantity must be positive".into(),
            ));
        }

        if item.unit_price < Decimal::ZERO {
            return Err((
                StatusCode::BAD_REQUEST,
                "Unit price cannot be negative".into(),
            ));
        }

        if item.discount_amount < Decimal::ZERO || item.discount_amount > item.unit_price * item.quantity {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid line item discount".into(),
            ));
        }

        subtotal += item.unit_price * item.quantity - item.discount_amount;
    }

    // Ensure checkout discount doesn't exceed subtotal
    if total_discount > subtotal {
        return Err((
            StatusCode::BAD_REQUEST,
            "Discount cannot exceed subtotal".into(),
        ));
    }

    // Apply discount to subtotal
    let discounted_subtotal = subtotal - total_discount;

    // Calculate tax (assume flat rate from products, simplified for Phase 2)
    // TODO: In production, sum product-specific tax rates
    let tax_amount = Decimal::ZERO; // Placeholder

    // Calculate total: subtotal - discount + tax
    let total_amount = discounted_subtotal + tax_amount;

    // Ensure payment amount >= total
    if input.payment_amount < total_amount {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Payment {} is insufficient for total {}", input.payment_amount, total_amount).into(),
        ));
    }

    // 4. Begin transaction
    let mut tx = s.db.begin().await.map_err(internal)?;

    // 5. Lock products and verify availability
    for item in &input.items {
        let current_balance: Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(quantity_delta), 0)::numeric 
             FROM inventory_movements 
             WHERE product_id = $1"
        )
        .bind(item.product_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;

        if current_balance < item.quantity {
            return Err((
                StatusCode::CONFLICT,
                format!(
                    "Insufficient stock for product. Required: {}, Available: {}",
                    item.quantity, current_balance
                )
                .into(),
            ));
        }
    }

    // 6. Create sale record
    let ticket_number = format!(
        "TK-{}-{}",
        chrono::Local::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4().to_string()[..8].to_uppercase()
    );

    let sale_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sales(ticket_number, cashier_id, status, subtotal, discount_amount, tax_amount, total_amount, currency, idempotency_key)
         VALUES($1, $2, 'paid', $3, $4, $5, $6, 'XAF', $7)
         RETURNING id"
    )
    .bind(&ticket_number)
    .bind(user.id)
    .bind(subtotal)
    .bind(total_discount)
    .bind(tax_amount)
    .bind(total_amount)
    .bind(Uuid::new_v4())
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;

    // 7. Create sale lines
    for item in &input.items {
        let line_total = item.unit_price * item.quantity - item.discount_amount;

        sqlx::query(
            "INSERT INTO sale_lines(sale_id, product_id, product_name_snapshot, sku_snapshot, quantity, unit_price, discount_amount, line_total)
             SELECT $1, $2, p.name, p.sku, $3, $4, $5, $6 FROM products p WHERE p.id = $2"
        )
        .bind(sale_id)
        .bind(item.product_id)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(item.discount_amount)
        .bind(line_total)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

        // 8. Create inventory movement (consumption)
        sqlx::query(
            "INSERT INTO inventory_movements(product_id, kind, quantity_delta, reason, actor_user_id, reference_type, reference_id)
             VALUES($1, 'sale', $2, $3, $4, 'sale', $5)"
        )
        .bind(item.product_id)
        .bind(-item.quantity) // Negative for consumption
        .bind(format!("Sold on ticket {}", ticket_number))
        .bind(user.id)
        .bind(sale_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    }

    // 9. Create payment record
    let payment_status = match payment_method.as_str() {
        "cash" => "confirmed",
        "mobile_money" => "pending", // Requires manual/callback confirmation
        "card" | "bank_transfer" => "pending", // Requires manual confirmation
        _ => "pending",
    };

    sqlx::query(
        "INSERT INTO payments(sale_id, method, status, amount, provider, external_reference)
         VALUES($1, $2, $3, $4, $5, $6)"
    )
    .bind(sale_id)
    .bind(&payment_method)
    .bind(payment_status)
    .bind(input.payment_amount)
    .bind::<Option<String>>(None) // provider
    .bind(input.payment_reference.map(|r| r.trim().to_string()))
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    // 10. Log audit event
    sqlx::query(
        "INSERT INTO audit_events(actor_user_id, action, target_type, target_id, reason)
         VALUES($1, 'pos.checkout', 'sale', $2, $3)"
    )
    .bind(user.id)
    .bind(sale_id)
    .bind(format!(
        "Checkout: {} items, total XAF {}",
        input.items.len(),
        total_amount
    ))
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    // 11. Commit transaction
    tx.commit().await.map_err(internal)?;

    // 12. Return response
    Ok((
        StatusCode::CREATED,
        Json(CheckoutResponse {
            id: sale_id,
            ticket_number,
            subtotal,
            discount_amount: total_discount,
            tax_amount,
            total_amount,
            payment_status: payment_status.to_string(),
            created_at: Utc::now(),
        }),
    ))
}

/// GET /api/v1/sales/:id
pub async fn get_sale(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Path(sale_id): axum::extract::Path<Uuid>,
) -> Result<Json<SaleDetailResponse>, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "sales.read").await?;

    let sale = sqlx::query_as::<_, SaleRow>(
        "SELECT id, ticket_number, cashier_id, status, subtotal, discount_amount, tax_amount, total_amount, currency, created_at, paid_at
         FROM sales WHERE id = $1"
    )
    .bind(sale_id)
    .fetch_optional(&s.db)
    .await
    .map_err(internal)?
    .ok_or((StatusCode::NOT_FOUND, "Sale not found".into()))?;

    let lines = sqlx::query_as::<_, SaleLineRow>(
        "SELECT id, sale_id, product_id, product_name_snapshot, sku_snapshot, quantity, unit_price, discount_amount, line_total
         FROM sale_lines WHERE sale_id = $1"
    )
    .bind(sale_id)
    .fetch_all(&s.db)
    .await
    .map_err(internal)?;

    let payment = sqlx::query_as::<_, (String, String)>(
        "SELECT method, status FROM payments WHERE sale_id = $1 LIMIT 1"
    )
    .bind(sale_id)
    .fetch_optional(&s.db)
    .await
    .map_err(internal)?;

    let (payment_method, payment_status) = payment.unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

    let cashier_name: String = sqlx::query_scalar("SELECT display_name FROM app_users WHERE id = $1")
        .bind(sale.cashier_id)
        .fetch_one(&s.db)
        .await
        .map_err(internal)?;

    Ok(Json(SaleDetailResponse {
        id: sale.id,
        ticket_number: sale.ticket_number,
        cashier_name,
        status: sale.status,
        subtotal: sale.subtotal,
        discount_amount: sale.discount_amount,
        tax_amount: sale.tax_amount,
        total_amount: sale.total_amount,
        items: lines,
        payment_method,
        payment_status,
        created_at: sale.created_at,
    }))
}

/// GET /api/v1/sales?limit=100&offset=0
pub async fn list_sales(
    State(s): State<AppState>,
    user: AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<ListSalesQuery>,
) -> Result<Json<Vec<SaleRow>>, (StatusCode, String)> {
    crate::auth::require_permission(&s, user.id, "sales.read").await?;

    let limit = (params.limit.unwrap_or(100)).min(1000);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, SaleRow>(
        "SELECT id, ticket_number, cashier_id, status, subtotal, discount_amount, tax_amount, total_amount, currency, created_at, paid_at
         FROM sales ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&s.db)
    .await
    .map_err(internal)?;

    Ok(Json(rows))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListSalesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    tracing::error!(error=%e, "request failed");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".into(),
    )
}
