# Phase 1 Verification Checklist

**Objective**: Verify Phase 1 (Complete Inventory Management) implementation.  
**Environment**: Windows 11 with Rust, Node.js, PostgreSQL 16  
**Estimated Time**: 2-3 hours

---

## Pre-Verification Setup

### 1. Verify Development Tools

```powershell
# Rust version
rustc --version
cargo --version
# Expected: rustc 1.70+, cargo 1.70+

# Node version
node --version
pnpm --version
# Expected: node 22.16+, pnpm 8+

# PostgreSQL version
psql --version
# Expected: PostgreSQL 16+
```

**If any fail**: Install from official websites before proceeding.

### 2. Environment Variables

Create or update `.env` in project root:

```env
DATABASE_URL=postgres://postgres:yourpassword@localhost:5432/complexe_sms_dev
BOOTSTRAP_SECRET=SuperSecureBootstrapSecret12345
BIND_ADDR=127.0.0.1:8080
```

### 3. PostgreSQL Setup

```powershell
# Start PostgreSQL service
Get-Service postgresql-x64-16 | Start-Service

# Create database
psql -U postgres -c "CREATE DATABASE complexe_sms_dev IF NOT EXISTS;"

# Verify connection
psql -U postgres -d complexe_sms_dev -c "SELECT 1;"
# Should return: 1
```

---

## Phase 1 Verification Tests

### Test 1: Database Migrations

**Objective**: Verify database schema is correctly created.

```powershell
cd services/api

# Run migrations (via API startup, or direct psql)
# Option A: Via API (recommended)
cargo run
# Wait for "Complexe SMS API listening" message
# Ctrl+C to stop

# Option B: Direct SQL
psql -U postgres -d complexe_sms_dev -f ../../database/migrations/0001_initial.sql
psql -U postgres -d complexe_sms_dev -f ../../database/migrations/0002_operations_security.sql
psql -U postgres -d complexe_sms_dev -f ../../database/migrations/0003_identity_sessions.sql
psql -U postgres -d complexe_sms_dev -f ../../database/migrations/0004_inventory_receipt_constraints.sql
psql -U postgres -d complexe_sms_dev -f ../../database/migrations/0005_inventory_extended_fields.sql
```

**Verify Schema** (in psql):

```sql
-- Connect to database
\c complexe_sms_dev

-- Check products table has new columns
\d products
-- Should show: description, barcode, product_type, purchase_price, tax_rate, 
-- supplier_id, minimum_stock_threshold, reorder_quantity, maximum_stock_threshold,
-- storage_location, is_perishable, image_path

-- Check new tables exist
\d archived_products
\d stock_adjustments
\d physical_stock_counts

-- Check triggers exist
SELECT routine_name FROM information_schema.routines 
WHERE routine_schema = 'public' AND routine_name LIKE '%inventory%';
-- Should include: sms_reject_ledger_mutation
```

**Status**: ✅ PASS if all migrations execute without errors and tables exist.

---

### Test 2: Backend Unit Tests

**Objective**: Verify application validators and auth functions work.

```powershell
cd services/api

# Run all tests
cargo test --all-targets

# Expected output:
# running X tests
# test_positive_quantity_accepts_positive_decimal ... ok
# test_sku_validation ... ok
# test_tax_rate_validation ... ok
# ... (10+ new tests)
# test result: ok. X passed; 0 failed

# Run specific test crate
cargo test -p sms-application
```

**Status**: ✅ PASS if all tests pass (no failures).

---

### Test 3: TypeScript Type Checking

**Objective**: Verify frontend types are valid.

```powershell
cd apps/desktop

# Install dependencies
pnpm install

# Type check
pnpm typecheck

# Expected output:
# ✓ compiles successfully, no errors found

# Build
pnpm build
# Expected output ends with: "built successfully"
```

**Status**: ✅ PASS if no TypeScript errors.

---

### Test 4: Rust Code Quality

**Objective**: Verify code formatting and best practices.

```powershell
cd services/api

# Format check
cargo fmt --all -- --check

# Clippy (linter)
cargo clippy --all-targets -- -D warnings

# Expected: No warnings or errors
```

**Status**: ✅ PASS if no warnings.

---

### Test 5: API Integration Tests

**Objective**: Test API endpoints manually via curl.

```powershell
# Terminal 1: Start API
cd services/api
cargo run
# Wait for: "Complexe SMS API listening on 127.0.0.1:8080"

# Terminal 2: Run curl tests

# 1. Health check
curl http://127.0.0.1:8080/health
# Expected: {"status":"ok","service":"complexe-sms-api"}

# 2. Database readiness
curl http://127.0.0.1:8080/ready
# Expected: {"status":"ready","service":"complexe-sms-api"}

# 3. Bootstrap admin (first time only)
curl -X POST http://127.0.0.1:8080/api/v1/auth/bootstrap `
  -H "Content-Type: application/json" `
  -d '{
    "bootstrap_secret": "SuperSecureBootstrapSecret12345",
    "username": "admin",
    "display_name": "Administrator",
    "password": "SecurePassword123456"
  }'
# Expected: {"id":"<uuid>","username":"admin","display_name":"Administrator","roles":["owner"]}

# 4. Login
$loginResp = curl -X POST http://127.0.0.1:8080/api/v1/auth/login `
  -H "Content-Type: application/json" `
  -d '{"username":"admin","password":"SecurePassword123456"}' | ConvertFrom-Json
$token = $loginResp.access_token

# 5. Get current user
curl -H "Authorization: Bearer $token" http://127.0.0.1:8080/api/v1/auth/me
# Expected: {"id":"<uuid>","username":"admin","display_name":"Administrator","roles":["owner"]}

# 6. Create product
curl -X POST http://127.0.0.1:8080/api/v1/products `
  -H "Authorization: Bearer $token" `
  -H "Content-Type: application/json" `
  -d '{
    "sku": "BAK-001",
    "name": "Butter Croissant",
    "category": "Bakery",
    "unit": "piece",
    "selling_price": 2500
  }'
# Expected: {"id":"<uuid>","sku":"BAK-001",...}

# Save product ID from response
$productId = "<uuid-from-response>"

# 7. Get product details
curl -H "Authorization: Bearer $token" `
  http://127.0.0.1:8080/api/v1/products/$productId
# Expected: Full product details with new fields

# 8. Update product
curl -X PUT http://127.0.0.1:8080/api/v1/products/$productId `
  -H "Authorization: Bearer $token" `
  -H "Content-Type: application/json" `
  -d '{
    "name": "Premium Butter Croissant",
    "description": "Delicious French butter croissant",
    "category": "Bakery",
    "product_type": "finished_product",
    "unit": "piece",
    "selling_price": 2500,
    "purchase_price": 1000,
    "tax_rate": 5.5,
    "minimum_stock_threshold": 10,
    "reorder_quantity": 50,
    "barcode": "123456789",
    "storage_location": "Shelf A1",
    "is_perishable": true
  }'
# Expected: 200 OK with updated product

# 9. List products
curl -H "Authorization: Bearer $token" `
  http://127.0.0.1:8080/api/v1/products
# Expected: Array with 1 product

# 10. Get balances (initially empty)
curl -H "Authorization: Bearer $token" `
  http://127.0.0.1:8080/api/v1/inventory/balances
# Expected: Empty array [] (no movements yet)

# 11. Create stock receipt (add to inventory)
$idempKey = (New-Guid).Guid
curl -X POST http://127.0.0.1:8080/api/v1/inventory/movements `
  -H "Authorization: Bearer $token" `
  -H "Content-Type: application/json" `
  -d "{
    \"product_id\": \"$productId\",
    \"kind\": \"receipt\",
    \"quantity_delta\": 100,
    \"reason\": \"Initial stock receipt from supplier\",
    \"idempotency_key\": \"$idempKey\"
  }"
# Expected: 201 CREATED with movement record

# 12. Get balances (now should show 100)
curl -H "Authorization: Bearer $token" `
  http://127.0.0.1:8080/api/v1/inventory/balances
# Expected: [{"product_id":"<uuid>","sku":"BAK-001","name":"Butter Croissant",...,"quantity":"100"}]

# 13. Get stock status
curl -H "Authorization: Bearer $token" `
  "http://127.0.0.1:8080/api/v1/inventory/stock-status"
# Expected: Array with low_stock flag (should be false since 100 > 10)

# 14. Archive product
curl -X POST http://127.0.0.1:8080/api/v1/products/$productId/archive `
  -H "Authorization: Bearer $token" `
  -H "Content-Type: application/json" `
  -d '{"reason":"Test archival"}'
# Expected: 204 NO CONTENT

# 15. Reactivate product
curl -X POST http://127.0.0.1:8080/api/v1/products/$productId/reactivate `
  -H "Authorization: Bearer $token"
# Expected: 204 NO CONTENT

# 16. Create physical count
curl -X POST http://127.0.0.1:8080/api/v1/inventory/physical-counts `
  -H "Authorization: Bearer $token" `
  -H "Content-Type: application/json" `
  -d "{
    \"product_id\": \"$productId\",
    \"counted_quantity\": 98,
    \"notes\": \"Physical count by Alice\"
  }"
# Expected: 201 CREATED with count record showing variance: -2
```

**Status**: ✅ PASS if all API calls return expected responses.

---

### Test 6: Frontend Integration

**Objective**: Test desktop app UI functionality.

```powershell
# Terminal 1: Keep API running
cd services/api
cargo run

# Terminal 2: Start desktop app
cd apps/desktop
pnpm dev

# Wait for Vite dev server to start
# Opens at http://127.0.0.1:5173
```

**Manual UI Tests** (in desktop app):

1. **Setup**
   - [ ] App loads and shows login screen
   - [ ] Enter username: admin, password: SecurePassword123456
   - [ ] Click "Sign in securely"
   - [ ] Dashboard loads with welcome message

2. **Inventory Page Access**
   - [ ] Click "Inventory" in sidebar
   - [ ] Page loads with tabs: Stock Balances, Products, Low Stock, Movements
   - [ ] "Add product" button visible

3. **Create Product Workflow**
   - [ ] Click "Add product" button
   - [ ] Modal opens with form
   - [ ] Fill in:
     - Product name: "Croissant"
     - SKU: "BAK-002"
     - Category: "Bakery"
     - Unit: "piece"
     - Selling price: "2500"
   - [ ] Click "Create product"
   - [ ] Modal closes, page refreshes
   - [ ] New product appears in Products tab

4. **View Stock Balances**
   - [ ] Switch to "Stock Balances" tab
   - [ ] See table with: Product, SKU, Unit, On-Hand, Status
   - [ ] For BAK-001: should show 98 (after physical count adjustment)
   - [ ] For BAK-002: should show 0 (no stock receipt yet)

5. **Edit Product**
   - [ ] Click "Edit" button on a product
   - [ ] Modal opens with populated fields
   - [ ] Update: description to "Delicious pastry"
   - [ ] Update: minimum threshold to "5"
   - [ ] Click "Save changes"
   - [ ] Modal closes

6. **Low Stock Alerts**
   - [ ] Switch to "Low Stock Alert" tab
   - [ ] Should show products with quantity ≤ minimum threshold
   - [ ] After editing threshold, products may appear/disappear from this tab

7. **Archive Product**
   - [ ] In Products tab, click "Archive" on a product
   - [ ] Confirm dialog appears
   - [ ] Click OK to confirm
   - [ ] Product disappears from Products tab (archived products don't show)

8. **Error Handling**
   - [ ] Try to create product with duplicate SKU
   - [ ] Error message appears: "A product with this SKU already exists"
   - [ ] Stop API server
   - [ ] Try to refresh data
   - [ ] Error notification shows: "API unavailable"

**Status**: ✅ PASS if all manual tests pass.

---

### Test 7: Authorization Matrix

**Objective**: Verify role-based access control.

```powershell
# Create different users with different roles
# (This requires SQL or additional API endpoints not yet built)

# For now, verify owner role has full access:
# - catalog.read ✓
# - catalog.manage ✓
# - inventory.read ✓
# - inventory.receive ✓

# Test via API:
curl -H "Authorization: Bearer $token" \
  http://127.0.0.1:8080/api/v1/products
# Should work (owner has catalog.read)

curl -X POST http://127.0.0.1:8080/api/v1/products \
  -H "Authorization: Bearer $token" \
  -H "Content-Type: application/json" \
  -d '{...}'
# Should work (owner has catalog.manage)
```

**Status**: ✅ PASS if authorization checks work for owner role.

---

### Test 8: Data Integrity

**Objective**: Verify negative stock prevention and idempotency.

```powershell
# 1. Try to create movement that would make stock negative
curl -X POST http://127.0.0.1:8080/api/v1/inventory/movements \
  -H "Authorization: Bearer $token" \
  -H "Content-Type: application/json" \
  -d "{
    \"product_id\": \"$productId\",
    \"kind\": \"receipt\",
    \"quantity_delta\": -150,
    \"reason\": \"Trying to create negative stock\",
    \"idempotency_key\": \"$(New-Guid)\"
  }"
# Expected: 409 CONFLICT - "Movement would make stock negative"

# 2. Test idempotency: Create receipt twice with same key
$idempKey2 = (New-Guid).Guid
$resp1 = curl -X POST http://127.0.0.1:8080/api/v1/inventory/movements \
  -H "Authorization: Bearer $token" \
  -H "Content-Type: application/json" \
  -d "{
    \"product_id\": \"$productId\",
    \"kind\": \"receipt\",
    \"quantity_delta\": 50,
    \"reason\": \"Test idempotency\",
    \"idempotency_key\": \"$idempKey2\"
  }" | ConvertFrom-Json

# Save ID
$movementId = $resp1.id

# Send same request again
$resp2 = curl -X POST http://127.0.0.1:8080/api/v1/inventory/movements \
  -H "Authorization: Bearer $token" \
  -H "Content-Type: application/json" \
  -d "{
    \"product_id\": \"$productId\",
    \"kind\": \"receipt\",
    \"quantity_delta\": 50,
    \"reason\": \"Test idempotency\",
    \"idempotency_key\": \"$idempKey2\"
  }" | ConvertFrom-Json

# Compare IDs
if ($resp1.id -eq $resp2.id) {
  Write-Host "✓ Idempotency works: same ID returned" -ForegroundColor Green
} else {
  Write-Host "✗ Idempotency failed: different IDs returned" -ForegroundColor Red
}

# 3. Verify balance increased only once
curl -H "Authorization: Bearer $token" \
  http://127.0.0.1:8080/api/v1/inventory/balances | ConvertFrom-Json
# Quantity should include +50, not +100
```

**Status**: ✅ PASS if negative stock is prevented and idempotency works.

---

### Test 9: Audit Trail

**Objective**: Verify audit events are logged.

```sql
-- In psql connected to complexe_sms_dev
SELECT action, target_type, target_id, created_at 
FROM audit_events 
ORDER BY created_at DESC 
LIMIT 10;

-- Expected events:
-- - identity.bootstrap
-- - catalog.create (for created products)
-- - inventory.archive_product
-- - inventory.reactivate_product
-- - inventory.physical_count
```

**Status**: ✅ PASS if audit events exist for all operations.

---

### Test 10: Database Constraints

**Objective**: Verify database-enforced constraints work.

```powershell
# Try to insert duplicate SKU (should fail)
# This would require psql access or direct DB test

# Via inventory API, duplicate SKU is already tested in Test 6

# Verify CHECK constraints in SQL:
psql -U postgres -d complexe_sms_dev -c "
  INSERT INTO products(sku, name, category, unit, selling_price, tax_rate)
  VALUES('TEST', 'Test', 'Test', 'piece', -100, 0);
"
# Expected: ERROR - violates CHECK constraint "products_selling_price_check"
```

**Status**: ✅ PASS if constraints prevent invalid data.

---

## Summary Report

After completing all tests above, fill in:

| Test | Status | Notes |
|------|--------|-------|
| 1. Database Migrations | ⏳ | |
| 2. Backend Unit Tests | ⏳ | |
| 3. TypeScript Type Checking | ⏳ | |
| 4. Rust Code Quality | ⏳ | |
| 5. API Integration | ⏳ | |
| 6. Frontend UI | ⏳ | |
| 7. Authorization | ⏳ | |
| 8. Data Integrity | ⏳ | |
| 9. Audit Trail | ⏳ | |
| 10. Database Constraints | ⏳ | |

**Overall Result**: 🟡 IN PROGRESS → (fill as tests complete)

---

## Troubleshooting

### PostgreSQL Connection Failed
```
Error: could not connect to server: Connection refused
```
**Solution**:
```powershell
Get-Service postgresql-x64-16 | Start-Service
# Wait 5 seconds
Test-NetConnection localhost -Port 5432
```

### Migrations Already Applied (Idempotent)
```
Error: relation "products" already exists
```
**Solution**: This is expected! Migrations are idempotent (they check IF NOT EXISTS). Safe to re-run.

### Port 8080 Already in Use
```
Error: Address already in use
```
**Solution**:
```powershell
# Find process using port 8080
netstat -ano | findstr :8080
# Kill process
taskkill /PID <PID> /F
# Or use different port:
$env:BIND_ADDR = "127.0.0.1:8081"
```

### TypeScript Compilation Error
```
error TS2304: Cannot find name 'React'
```
**Solution**:
```powershell
cd apps/desktop
rm -r node_modules pnpm-lock.yaml
pnpm install
pnpm typecheck
```

### Cargo Build Fails
```
error: could not compile `sms-api`
```
**Solution**:
```powershell
cd services/api
cargo clean
cargo build
# Check for missing dependencies or Rust version issues
```

---

## Success Criteria

✅ **Phase 1 Verification is COMPLETE** when:

1. All 10 tests above pass
2. No compilation errors
3. All API endpoints respond correctly
4. Frontend UI loads and functions
5. Data integrity safeguards work
6. Audit trail logs operations
7. Authorization checks work
8. No diagnostics/warnings in code

---

**Verification Checklist Created**: November 2026  
**Estimated Duration**: 2-3 hours  
**Status**: Ready for local testing
