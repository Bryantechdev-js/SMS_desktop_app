-- Complexe SMS inventory management extensions v1.0.0
-- Add comprehensive inventory fields to products table
-- Support for batch/expiry traceability, supplier linkage, and stock thresholds

ALTER TABLE products ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT '';
ALTER TABLE products ADD COLUMN IF NOT EXISTS barcode TEXT UNIQUE;
ALTER TABLE products ADD COLUMN IF NOT EXISTS product_type TEXT NOT NULL DEFAULT 'other' CHECK (product_type IN ('ingredient', 'finished_product', 'packaging', 'other'));
ALTER TABLE products ADD COLUMN IF NOT EXISTS purchase_price NUMERIC(14,2) CHECK (purchase_price IS NULL OR purchase_price >= 0);
ALTER TABLE products ADD COLUMN IF NOT EXISTS tax_rate NUMERIC(5,2) NOT NULL DEFAULT 0 CHECK (tax_rate >= 0 AND tax_rate <= 100);
ALTER TABLE products ADD COLUMN IF NOT EXISTS supplier_id UUID REFERENCES suppliers(id) ON DELETE SET NULL;
ALTER TABLE products ADD COLUMN IF NOT EXISTS minimum_stock_threshold NUMERIC(14,3) CHECK (minimum_stock_threshold IS NULL OR minimum_stock_threshold > 0);
ALTER TABLE products ADD COLUMN IF NOT EXISTS reorder_quantity NUMERIC(14,3) CHECK (reorder_quantity IS NULL OR reorder_quantity > 0);
ALTER TABLE products ADD COLUMN IF NOT EXISTS maximum_stock_threshold NUMERIC(14,3) CHECK (maximum_stock_threshold IS NULL OR maximum_stock_threshold > 0);
ALTER TABLE products ADD COLUMN IF NOT EXISTS storage_location TEXT;
ALTER TABLE products ADD COLUMN IF NOT EXISTS is_perishable BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE products ADD COLUMN IF NOT EXISTS image_path TEXT;

-- Constraint: if max threshold is set, it must be >= min threshold
ALTER TABLE products ADD CONSTRAINT products_stock_thresholds_valid 
CHECK (maximum_stock_threshold IS NULL OR minimum_stock_threshold IS NULL OR maximum_stock_threshold >= minimum_stock_threshold);

-- Extended inventory_movements table for richer context
ALTER TABLE inventory_movements ADD COLUMN IF NOT EXISTS batch_code TEXT;

-- New table for archived products (soft deletes with history)
CREATE TABLE IF NOT EXISTS archived_products (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  archived_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_by UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  reason TEXT NOT NULL DEFAULT '',
  restored_at TIMESTAMPTZ,
  restored_by UUID REFERENCES app_users(id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_archived_products_product_id ON archived_products(product_id);

-- Stock adjustment request workflow (for multi-step approval)
CREATE TABLE IF NOT EXISTS stock_adjustments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  adjustment_number TEXT NOT NULL UNIQUE,
  product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  lot_id UUID REFERENCES inventory_lots(id) ON DELETE RESTRICT,
  quantity_delta NUMERIC(14,3) NOT NULL CHECK (quantity_delta <> 0),
  reason TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('draft', 'requested', 'approved', 'rejected', 'cancelled', 'processed')),
  requested_by UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  approved_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  rejected_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  rejection_reason TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  approved_at TIMESTAMPTZ,
  processed_at TIMESTAMPTZ,
  CHECK (approved_by IS NULL OR approved_by <> requested_by),
  CHECK (rejected_by IS NULL OR (status = 'rejected' AND rejection_reason IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS idx_stock_adjustments_product_created ON stock_adjustments(product_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_stock_adjustments_status ON stock_adjustments(status, created_at DESC);

-- Physical stock count records
CREATE TABLE IF NOT EXISTS physical_stock_counts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  count_number TEXT NOT NULL UNIQUE,
  product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  lot_id UUID REFERENCES inventory_lots(id) ON DELETE RESTRICT,
  counted_quantity NUMERIC(14,3) NOT NULL CHECK (counted_quantity >= 0),
  system_quantity NUMERIC(14,3) NOT NULL CHECK (system_quantity >= 0),
  counted_at TIMESTAMPTZ NOT NULL,
  counted_by UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  counted_at_time TIMESTAMP NOT NULL DEFAULT now(),
  notes TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_physical_stock_counts_product_created ON physical_stock_counts(product_id, counted_at DESC);
CREATE INDEX IF NOT EXISTS idx_physical_stock_counts_lot_created ON physical_stock_counts(lot_id, counted_at DESC) WHERE lot_id IS NOT NULL;

-- Immutability constraints for audit-critical ledger tables
DROP TRIGGER IF EXISTS trg_physical_stock_counts_immutable ON physical_stock_counts;
CREATE TRIGGER trg_physical_stock_counts_immutable BEFORE UPDATE OR DELETE ON physical_stock_counts
FOR EACH ROW EXECUTE FUNCTION sms_reject_ledger_mutation();

DROP TRIGGER IF EXISTS trg_stock_adjustments_immutable_posted ON stock_adjustments;
CREATE TRIGGER trg_stock_adjustments_immutable_posted BEFORE UPDATE OR DELETE ON stock_adjustments
FOR EACH ROW WHEN (OLD.status = 'processed')
EXECUTE FUNCTION sms_reject_ledger_mutation();
