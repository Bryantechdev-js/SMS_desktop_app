# Procurement Integration Tests - Phase 4

## Test Coverage Matrix

### Supplier Management
- [x] Create supplier with valid data
- [x] Create supplier with invalid email
- [x] List suppliers (pagination)
- [x] Archive supplier (soft delete)
- [x] Cannot archive already archived supplier
- [x] List includes only active suppliers by default

### Purchase Order Workflow
- [x] Create PO with valid line items
- [x] Create PO validates supplier exists
- [x] Create PO validates product exists
- [x] Submit PO (changes status to submitted)
- [x] Cannot submit already-submitted PO
- [x] Cancel PO (changes status to cancelled)
- [x] Cannot cancel already-cancelled PO
- [x] List POs with status filtering
- [x] Get PO detail with line items

### Goods Receipt Workflow
- [x] Create receipt for submitted PO
- [x] Cannot create receipt for draft PO
- [x] Cannot create receipt for cancelled PO
- [x] Receipt quantity must not exceed PO quantity
- [x] Receipt triggers inventory ledger entries
- [x] Partial receipt allowed for single PO
- [x] Multiple receipts allowed for same PO
- [x] Receipt updates inventory balances

### Permission Enforcement
- [x] Manager can create suppliers
- [x] Storekeeper cannot create suppliers (403)
- [x] Manager can create purchase orders
- [x] Storekeeper cannot create purchase orders (403)
- [x] Storekeeper can create receipts
- [x] Cashier cannot create receipts (403)

### Data Integrity
- [x] Supplier immutability - cannot modify after creation
- [x] PO immutability - line items locked after submission
- [x] Receipt immutability - cannot modify after creation
- [x] Atomic transaction for receipt + inventory update
- [x] No negative stock from partial receipts
- [x] Ledger entries are auditable and immutable

### Scenarios Covered

#### Scenario 1: Happy Path - Full Supplier to Receipt Workflow
```
1. Create supplier "Boulangerie Artisanale"
2. Create product "Flour 25kg" (if not exists)
3. Create PO with 10 bags of flour @ 5000 XAF each
4. Submit PO (status → submitted)
5. Create receipt for 10 bags
6. Verify inventory updated: +10 bags, total cost = 50,000 XAF
7. Verify ledger entries created with timestamps
```

#### Scenario 2: Partial Delivery
```
1. Create PO for 50kg sugar
2. Submit PO
3. Receive partial receipt: 30kg
4. Verify balance shows 30kg received, 20kg pending
5. Create second receipt: 20kg (final)
6. Verify total 50kg received, inventory updated
```

#### Scenario 3: Permission Denial
```
1. Login as storekeeper
2. Attempt to create supplier → 403 Forbidden
3. Attempt to create PO → 403 Forbidden
4. Attempt to create receipt → 200 OK (allowed)
5. Login as cashier
6. Attempt to create receipt → 403 Forbidden
```

#### Scenario 4: Quality Discrepancy
```
1. Create PO for 100 kg flour
2. Receive 95kg with quality_status = "rejected" for 5kg
3. Verify receipt created with discrepancy_notes captured
4. Verify inventory only credits 95kg
5. Verify rejection reason audit trail
```

#### Scenario 5: Data Immutability
```
1. Create supplier
2. Attempt to UPDATE supplier via PUT → 400 or 404 (not supported)
3. Attempt to DELETE supplier directly → 400 (immutable, use archive)
4. Verify supplier audit ledger shows creation only
```

## Test Status: PENDING
Tests are documented but not yet implemented in Rust test harness.
Implementation requires: sqlx test database, test fixtures, HTTP client.

## Next Phase
Implement as part of `tests/integration_test.rs` with:
- PostgreSQL test database (test transaction rollback)
- API HTTP client
- Test fixtures (products, users, roles)
- Bearer token auth for permission tests
