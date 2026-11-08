-- Complexe SMS operational core v1.0.0
-- Money uses NUMERIC; operational records are append-only after posting.

CREATE TABLE IF NOT EXISTS app_users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  username TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  password_hash TEXT NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  must_change_password BOOLEAN NOT NULL DEFAULT TRUE,
  failed_login_count INTEGER NOT NULL DEFAULT 0 CHECK (failed_login_count >= 0),
  locked_until TIMESTAMPTZ,
  last_login_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS roles (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  code TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  is_system BOOLEAN NOT NULL DEFAULT TRUE
);
CREATE TABLE IF NOT EXISTS permissions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  code TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS user_roles (
  user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
  granted_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY(user_id, role_id)
);
CREATE TABLE IF NOT EXISTS role_permissions (
  role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
  permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,
  PRIMARY KEY(role_id, permission_id)
);
CREATE TABLE IF NOT EXISTS auth_sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  refresh_token_hash TEXT NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  revoked_at TIMESTAMPTZ,
  ip_address INET,
  user_agent TEXT,
  CHECK (expires_at > created_at)
);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_live ON auth_sessions(user_id, expires_at) WHERE revoked_at IS NULL;
CREATE TABLE IF NOT EXISTS user_mfa_factors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  factor_type TEXT NOT NULL CHECK (factor_type IN ('totp')),
  secret_ciphertext TEXT NOT NULL,
  is_primary BOOLEAN NOT NULL DEFAULT FALSE,
  confirmed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  disabled_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_mfa_primary_active ON user_mfa_factors(user_id) WHERE is_primary = TRUE AND disabled_at IS NULL;

CREATE TABLE IF NOT EXISTS suppliers (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  code TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  phone TEXT,
  email TEXT,
  address TEXT,
  tax_identifier TEXT,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS purchase_receipts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  receipt_number TEXT NOT NULL UNIQUE,
  supplier_id UUID NOT NULL REFERENCES suppliers(id) ON DELETE RESTRICT,
  supplier_reference TEXT,
  status TEXT NOT NULL CHECK (status IN ('draft','partially_received','received','checked','cancelled')),
  received_at TIMESTAMPTZ,
  received_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  checked_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  notes TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS purchase_receipt_lines (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  receipt_id UUID NOT NULL REFERENCES purchase_receipts(id) ON DELETE RESTRICT,
  product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  quantity_ordered NUMERIC(14,3) NOT NULL CHECK (quantity_ordered > 0),
  quantity_received NUMERIC(14,3) NOT NULL DEFAULT 0 CHECK (quantity_received >= 0),
  unit_cost NUMERIC(14,2) NOT NULL CHECK (unit_cost >= 0),
  discrepancy_reason TEXT NOT NULL DEFAULT '',
  CHECK (quantity_received <= quantity_ordered OR length(trim(discrepancy_reason)) > 0)
);
CREATE TABLE IF NOT EXISTS inventory_lots (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  lot_code TEXT NOT NULL,
  received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ,
  unit_cost NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (unit_cost >= 0),
  source_receipt_line_id UUID REFERENCES purchase_receipt_lines(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(product_id, lot_code),
  CHECK (expires_at IS NULL OR expires_at >= received_at)
);
ALTER TABLE inventory_movements ADD COLUMN IF NOT EXISTS lot_id UUID REFERENCES inventory_lots(id) ON DELETE RESTRICT;
ALTER TABLE inventory_movements ADD COLUMN IF NOT EXISTS actor_user_id UUID REFERENCES app_users(id) ON DELETE RESTRICT;
ALTER TABLE inventory_movements ADD COLUMN IF NOT EXISTS reference_type TEXT;
ALTER TABLE inventory_movements ADD COLUMN IF NOT EXISTS reference_id UUID;
ALTER TABLE inventory_movements ADD COLUMN IF NOT EXISTS idempotency_key UUID UNIQUE;
CREATE INDEX IF NOT EXISTS idx_inventory_movements_lot_created ON inventory_movements(lot_id, created_at DESC);

CREATE TABLE IF NOT EXISTS recipes (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  version INTEGER NOT NULL CHECK (version > 0),
  yield_quantity NUMERIC(14,3) NOT NULL CHECK (yield_quantity > 0),
  yield_unit TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('draft','active','retired')),
  created_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(product_id, version)
);
CREATE TABLE IF NOT EXISTS recipe_items (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE RESTRICT,
  ingredient_product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  quantity NUMERIC(14,3) NOT NULL CHECK (quantity > 0),
  unit TEXT NOT NULL,
  UNIQUE(recipe_id, ingredient_product_id)
);
CREATE TABLE IF NOT EXISTS production_batches (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  batch_number TEXT NOT NULL UNIQUE,
  recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE RESTRICT,
  planned_quantity NUMERIC(14,3) NOT NULL CHECK (planned_quantity > 0),
  actual_quantity NUMERIC(14,3) CHECK (actual_quantity IS NULL OR actual_quantity >= 0),
  status TEXT NOT NULL CHECK (status IN ('draft','planned','approved','in_progress','completed','cancelled','rejected')),
  planned_for DATE,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  created_by UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  approved_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  notes TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS production_batch_consumptions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  batch_id UUID NOT NULL REFERENCES production_batches(id) ON DELETE RESTRICT,
  ingredient_product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  lot_id UUID REFERENCES inventory_lots(id) ON DELETE RESTRICT,
  planned_quantity NUMERIC(14,3) NOT NULL CHECK (planned_quantity > 0),
  actual_quantity NUMERIC(14,3) NOT NULL CHECK (actual_quantity >= 0),
  unit TEXT NOT NULL,
  UNIQUE(batch_id, ingredient_product_id, lot_id)
);

CREATE TABLE IF NOT EXISTS cash_sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_number TEXT NOT NULL UNIQUE,
  cashier_id UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  status TEXT NOT NULL CHECK (status IN ('open','submitted','verified','closed','reopened')),
  opening_amount NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (opening_amount >= 0),
  opened_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  closed_at TIMESTAMPTZ,
  submitted_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  verified_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  closed_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  notes TEXT NOT NULL DEFAULT ''
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_cash_one_open_per_cashier ON cash_sessions(cashier_id) WHERE status IN ('open','submitted','verified','reopened');
CREATE TABLE IF NOT EXISTS sales (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  ticket_number TEXT NOT NULL UNIQUE,
  cash_session_id UUID REFERENCES cash_sessions(id) ON DELETE RESTRICT,
  cashier_id UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  status TEXT NOT NULL CHECK (status IN ('draft','pending_payment','paid','closed','cancelled','partially_refunded','refunded')),
  subtotal NUMERIC(14,2) NOT NULL CHECK (subtotal >= 0),
  discount_amount NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (discount_amount >= 0),
  tax_amount NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (tax_amount >= 0),
  total_amount NUMERIC(14,2) NOT NULL CHECK (total_amount >= 0),
  currency CHAR(3) NOT NULL DEFAULT 'XAF',
  idempotency_key UUID NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  paid_at TIMESTAMPTZ,
  CHECK (total_amount = subtotal - discount_amount + tax_amount)
);
CREATE TABLE IF NOT EXISTS sale_lines (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
  product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  product_name_snapshot TEXT NOT NULL,
  sku_snapshot TEXT NOT NULL,
  quantity NUMERIC(14,3) NOT NULL CHECK (quantity > 0),
  unit_price NUMERIC(14,2) NOT NULL CHECK (unit_price >= 0),
  discount_amount NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (discount_amount >= 0),
  line_total NUMERIC(14,2) NOT NULL CHECK (line_total >= 0)
);
CREATE TABLE IF NOT EXISTS payments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
  method TEXT NOT NULL CHECK (method IN ('cash','mobile_money','card','bank_transfer','other')),
  status TEXT NOT NULL CHECK (status IN ('declared','pending','confirmed','failed','expired','cancelled','refunded')),
  amount NUMERIC(14,2) NOT NULL CHECK (amount > 0),
  provider TEXT,
  external_reference TEXT,
  confirmed_at TIMESTAMPTZ,
  confirmed_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (status <> 'confirmed' OR confirmed_at IS NOT NULL)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_payments_external_reference ON payments(provider, external_reference) WHERE external_reference IS NOT NULL;
CREATE TABLE IF NOT EXISTS sale_refunds (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  refund_number TEXT NOT NULL UNIQUE,
  sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
  requested_by UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  approved_by UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  amount NUMERIC(14,2) NOT NULL CHECK (amount > 0),
  reason TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('requested','approved','rejected','processed')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  processed_at TIMESTAMPTZ,
  CHECK (approved_by IS NULL OR approved_by <> requested_by)
);
CREATE TABLE IF NOT EXISTS cash_movements (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  cash_session_id UUID NOT NULL REFERENCES cash_sessions(id) ON DELETE RESTRICT,
  kind TEXT NOT NULL CHECK (kind IN ('opening_float','cash_sale','cash_refund','expense','cash_in','cash_out','closing_count')),
  amount NUMERIC(14,2) NOT NULL CHECK (amount >= 0),
  direction SMALLINT NOT NULL CHECK (direction IN (-1,1)),
  reason TEXT NOT NULL DEFAULT '',
  actor_user_id UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cash_counts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  cash_session_id UUID NOT NULL REFERENCES cash_sessions(id) ON DELETE RESTRICT,
  denomination NUMERIC(14,2) NOT NULL CHECK (denomination > 0),
  quantity INTEGER NOT NULL CHECK (quantity >= 0),
  counted_by UUID NOT NULL REFERENCES app_users(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS audit_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  actor_user_id UUID REFERENCES app_users(id) ON DELETE RESTRICT,
  action TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_id UUID,
  before_data JSONB,
  after_data JSONB,
  reason TEXT NOT NULL DEFAULT '',
  request_id UUID,
  ip_address INET,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_audit_events_target ON audit_events(target_type, target_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_events_actor ON audit_events(actor_user_id, created_at DESC);

-- Prevent destructive edits to posted stock and audit history. Corrections are compensating records.
CREATE OR REPLACE FUNCTION sms_reject_ledger_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'Posted ledger records are immutable; create a compensating record instead.';
END; $$;
DROP TRIGGER IF EXISTS trg_inventory_movements_immutable ON inventory_movements;
CREATE TRIGGER trg_inventory_movements_immutable BEFORE UPDATE OR DELETE ON inventory_movements
FOR EACH ROW EXECUTE FUNCTION sms_reject_ledger_mutation();
DROP TRIGGER IF EXISTS trg_audit_events_immutable ON audit_events;
CREATE TRIGGER trg_audit_events_immutable BEFORE UPDATE OR DELETE ON audit_events
FOR EACH ROW EXECUTE FUNCTION sms_reject_ledger_mutation();

-- Least-privilege role catalogue. Permissions are assigned explicitly by an administrator.
INSERT INTO roles(code,name,description) VALUES
 ('owner','Owner','Business owner with explicitly delegated oversight permissions'),
 ('manager','Manager','Operational manager'),
 ('cashier','Cashier','Point of sale and own cash session'),
 ('storekeeper','Storekeeper','Inventory and receiving operations'),
 ('production','Production Officer','Recipe and production operations'),
 ('procurement','Procurement Officer','Supplier and purchase receiving operations'),
 ('accountant','Accountant','Financial read and reconciliation'),
 ('auditor','Auditor','Read-only audit and reporting'),
 ('technical_admin','Technical Administrator','User and system administration; no implicit financial approval')
ON CONFLICT(code) DO NOTHING;
INSERT INTO permissions(code,description) VALUES
 ('dashboard.read','Read operational dashboard'),('catalog.read','Read product catalog'),('catalog.manage','Manage catalog and prices'),
 ('inventory.read','Read inventory'),('inventory.receive','Receive inventory'),('inventory.adjust.request','Request stock adjustment'),('inventory.adjust.approve','Approve stock adjustment'),
 ('supplier.read','Read suppliers'),('supplier.manage','Manage suppliers'),('procurement.create','Create purchase receipt'),('procurement.check','Verify received goods'),
 ('production.read','Read production'),('production.create','Create production batch'),('production.approve','Approve production batch'),('production.complete','Complete production batch'),
 ('pos.read','Read point of sale'),('pos.sell','Create sales'),('pos.discount.approve','Approve exceptional discounts'),('sale.refund.request','Request refund'),('sale.refund.approve','Approve refund'),
 ('cash.open','Open cash session'),('cash.close.submit','Submit cash closing'),('cash.close.verify','Verify cash closing'),('cash.expense.create','Record cash expense'),
 ('sales.read','Read sales and ticket history'),('cash.read','Read cash sessions and reconciliation'),
 ('reports.read','Read reports'),('reports.export','Export reports'),('audit.read','Read audit trail'),
 ('users.read','Read user accounts'),('users.manage','Create and deactivate user accounts'),('roles.assign','Assign roles within delegated authority'),('settings.manage','Manage application settings')
ON CONFLICT(code) DO NOTHING;


-- Default role grants are explicit and intentionally conservative. API checks remain mandatory.
WITH role_grants(role_code, permission_code) AS (VALUES
 ('owner','dashboard.read'),('owner','catalog.read'),('owner','catalog.manage'),('owner','inventory.read'),('owner','inventory.receive'),('owner','inventory.adjust.request'),('owner','inventory.adjust.approve'),('owner','supplier.read'),('owner','supplier.manage'),('owner','procurement.create'),('owner','procurement.check'),('owner','production.read'),('owner','production.create'),('owner','production.approve'),('owner','production.complete'),('owner','pos.read'),('owner','pos.sell'),('owner','pos.discount.approve'),('owner','sale.refund.request'),('owner','sale.refund.approve'),('owner','sales.read'),('owner','cash.read'),('owner','cash.open'),('owner','cash.close.submit'),('owner','cash.close.verify'),('owner','cash.expense.create'),('owner','reports.read'),('owner','reports.export'),('owner','audit.read'),('owner','users.read'),('owner','users.manage'),('owner','roles.assign'),('owner','settings.manage'),
 ('manager','dashboard.read'),('manager','catalog.read'),('manager','catalog.manage'),('manager','inventory.read'),('manager','inventory.receive'),('manager','inventory.adjust.request'),('manager','inventory.adjust.approve'),('manager','supplier.read'),('manager','supplier.manage'),('manager','procurement.create'),('manager','procurement.check'),('manager','production.read'),('manager','production.create'),('manager','production.approve'),('manager','production.complete'),('manager','pos.read'),('manager','pos.sell'),('manager','pos.discount.approve'),('manager','sale.refund.request'),('manager','sale.refund.approve'),('manager','sales.read'),('manager','cash.read'),('manager','cash.open'),('manager','cash.close.submit'),('manager','cash.close.verify'),('manager','cash.expense.create'),('manager','reports.read'),('manager','reports.export'),('manager','audit.read'),('manager','users.read'),('manager','roles.assign'),
 ('cashier','dashboard.read'),('cashier','catalog.read'),('cashier','inventory.read'),('cashier','pos.read'),('cashier','pos.sell'),('cashier','sale.refund.request'),('cashier','sales.read'),('cashier','cash.open'),('cashier','cash.close.submit'),
 ('storekeeper','dashboard.read'),('storekeeper','catalog.read'),('storekeeper','inventory.read'),('storekeeper','inventory.receive'),('storekeeper','inventory.adjust.request'),('storekeeper','supplier.read'),('storekeeper','procurement.check'),
 ('production','dashboard.read'),('production','catalog.read'),('production','inventory.read'),('production','production.read'),('production','production.create'),('production','production.complete'),
 ('procurement','dashboard.read'),('procurement','catalog.read'),('procurement','inventory.read'),('procurement','supplier.read'),('procurement','supplier.manage'),('procurement','procurement.create'),
 ('accountant','dashboard.read'),('accountant','catalog.read'),('accountant','inventory.read'),('accountant','sales.read'),('accountant','cash.read'),('accountant','cash.close.verify'),('accountant','reports.read'),('accountant','reports.export'),('accountant','audit.read'),
 ('auditor','dashboard.read'),('auditor','catalog.read'),('auditor','inventory.read'),('auditor','supplier.read'),('auditor','production.read'),('auditor','pos.read'),('auditor','sales.read'),('auditor','cash.read'),('auditor','reports.read'),('auditor','reports.export'),('auditor','audit.read'),
 ('technical_admin','users.read'),('technical_admin','users.manage'),('technical_admin','settings.manage')
)
INSERT INTO role_permissions(role_id, permission_id)
SELECT r.id, p.id FROM role_grants g
JOIN roles r ON r.code = g.role_code
JOIN permissions p ON p.code = g.permission_code
ON CONFLICT DO NOTHING;
