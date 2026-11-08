use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub selling_price: Decimal,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductFull {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockMovementKind {
    Opening,
    Receipt,
    ProductionOutput,
    ProductionConsumption,
    Sale,
    Waste,
    Adjustment,
    Return,
}

impl StockMovementKind {
    pub fn permits_negative(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBalance {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub unit: String,
    pub quantity: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLot {
    pub id: Uuid,
    pub product_id: Uuid,
    pub lot_code: String,
    pub received_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub unit_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAdjustment {
    pub id: Uuid,
    pub adjustment_number: String,
    pub product_id: Uuid,
    pub quantity_delta: Decimal,
    pub reason: String,
    pub status: String,
    pub requested_by_id: Uuid,
    pub approved_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// POS & Sales Types (NEW for Phase 2)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub id: Uuid,
    pub ticket_number: String,
    pub cashier_id: Uuid,
    pub status: String,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleLine {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub method: String,
    pub status: String,
    pub amount: Decimal,
    pub provider: Option<String>,
    pub external_reference: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleRefund {
    pub id: Uuid,
    pub refund_number: String,
    pub sale_id: Uuid,
    pub requested_by_id: Uuid,
    pub approved_by_id: Option<Uuid>,
    pub amount: Decimal,
    pub reason: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub items: Vec<CheckoutLineItem>,
    pub discount_amount: Decimal,
    pub discount_reason: Option<String>,
    pub tax_amount: Decimal,
    pub customer_id: Option<Uuid>,
    pub payment_method: String,
    pub payment_amount: Decimal,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutLineItem {
    pub product_id: Uuid,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub discount_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub id: Uuid,
    pub ticket_number: String,
    pub total_amount: Decimal,
    pub payment_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub ticket_number: String,
    pub sale_date: DateTime<Utc>,
    pub cashier_name: String,
    pub items: Vec<ReceiptLine>,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub payment_method: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptLine {
    pub product_name: String,
    pub sku: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub line_total: Decimal,
}
