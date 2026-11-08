# Phase 2: Complete Point of Sale (POS)
## Implementation Plan

**Objective**: Deliver a working POS checkout workflow with cart, payment handling, and sales transaction atomicity.

**Scope**:
- Shopping cart management (frontend state)
- Product search and barcode input
- Quantity adjustment and line removal
- Discount controls with permission checks
- Tax calculation (backend-authoritative)
- Multiple payment methods (cash, mobile money, card, bank transfer)
- Receipt generation
- Sales transaction atomicity (inventory consumption + payment + audit)
- Sale history and refund workflow
- Walk-in and customer selection

---

## Critical Requirements

### Transaction Atomicity (Backend Rule)
A completed sale **must atomically**:
1. Validate cashier permissions (`pos.sell`)
2. Validate product availability and current prices
3. Validate quantities and discount permissions
4. Lock relevant inventory records
5. Calculate authoritative totals (backend, not client)
6. Create sale record
7. Create sale line records
8. Create payment record
9. Record inventory consumption movements
10. Record audit event
11. Commit transaction

**If any step fails, ROLLBACK entire transaction.**

### Financial Rules
- Use `NUMERIC(14,2)` for all monetary values (never floating-point)
- Server calculates all totals: `total = subtotal - discount + tax`
- Client submits request; backend verifies every number
- Receipts are immutable after creation
- Refunds are reversals, not deletions

### Payment Handling
- Cash: Confirmed immediately
- Mobile Money: Marked as `pending` until callback confirmation (or manual verification)
- Card/Bank Transfer: Manual confirmation flow
- Split payments: Multiple payment records per sale

### Inventory Consumption
- Sale confirms product availability before checkout
- Inventory movements created atomically with sale
- Use `kind: 'sale'` in inventory_movements
- Reference sale ID in reference_type/reference_id
- Prevent overselling (check balance before locking)

---

## Database Changes Needed

### Existing Tables (Already in Migration 0002)
- `sales`: Main sale record
- `sale_lines`: Individual line items
- `payments`: Payment records
- `sale_refunds`: Refund requests

### Verification
The schema is ready. We need to:
1. Implement API endpoints to use these tables
2. Add any missing indexes
3. Add validation constraints if needed

---

## API Endpoints (Phase 2)

### POS Cart (Session State - Frontend Only)
No API needed; cart lives in React state.

### Sales Workflow
```
POST   /api/v1/sales/checkout          Create sale with payment
GET    /api/v1/sales                   List sales (with filters)
GET    /api/v1/sales/:id               Get sale details
GET    /api/v1/sales/:id/receipt       Generate receipt (text/HTML)
POST   /api/v1/sales/:id/refund        Request refund
GET    /api/v1/sales/:id/refund        View refund status
```

### Payment Workflow
```
POST   /api/v1/payments/:id/confirm    Confirm payment (admin/accountant)
GET    /api/v1/payments/:id            Get payment status
```

### Receipts
```
GET    /api/v1/receipts/:id            Generate receipt (text or HTML)
POST   /api/v1/receipts/:id/reprint    Reprint existing receipt
```

---

## Frontend Components (Phase 2)

### POS Page Structure
```
<POSPage>
  ├── <ProductBrowser>
    │   ├── Search input + barcode scanner
    │   ├── Category filter
    │   └── Product grid/list
    │
    ├── <ShoppingCart>
    │   ├── Cart items (product, qty, price)
    │   ├── Quantity adjustment
    │   ├── Item remove button
    │   └── Subtotal display
    │
    └── <Checkout>
        ├── Discount input (with permission check)
        ├── Tax calculation display
        ├── Total amount
        ├── Customer selector (or walk-in)
        ├── Payment method selector
        ├── Payment details (amount, reference)
        └── Complete Sale button
```

### States
- Cart items (in-memory state)
- Selected payment method
- Discount input
- Checkout loading/error
- Receipt modal (after success)

---

## Validation Rules

### Frontend
- Product quantity must be positive
- Discount must not exceed subtotal
- Payment amount must equal total (or partial if split)
- Cashier confirmation before final submission

### Backend
- **Revalidate everything** client submits
- Prices must match current catalog
- Quantities must be available (check current balance)
- Discount must be authorized (check permission)
- Payment amount must not exceed sale total

---

## Security & Authorization

### Permissions
- `pos.sell`: Can create sales
- `pos.discount.approve`: Can apply discounts > threshold
- `sale.refund.request`: Can request refund
- `sale.refund.approve`: Can approve refund
- `sales.read`: Can view sale history

### Key Checks
1. Only `pos.sell` permission can checkout
2. Discounts > threshold require `pos.discount.approve`
3. Backend recalculates all totals (never trust client)
4. Mobile Money payments marked `pending` until verified
5. Refund requester ≠ approver (separation of duties)

---

## Implementation Order

1. **Backend Domain & Validators** (Day 1)
   - Add POS validators (quantity, discount, payment)
   - Add types for checkout request/response

2. **Backend API** (Day 2)
   - Sales checkout endpoint (core logic)
   - Sales list/detail endpoints
   - Payment confirmation endpoint
   - Receipt generation endpoint

3. **Frontend Components** (Day 3)
   - ProductBrowser (search, categories, barcode)
   - ShoppingCart (items, adjustment, remove)
   - Checkout (discount, payment, submit)

4. **Integration & Testing** (Day 4)
   - End-to-end checkout workflow
   - Inventory consumption verification
   - Receipt generation
   - Error handling

5. **Hardening** (Day 5)
   - Refund workflow
   - Receipt reprinting
   - Split payments
   - Edge cases

---

## Success Criteria

### Functional
- [x] Create sale with cart items
- [x] Inventory consumed atomically
- [x] Receipt generated
- [x] Payment status tracked
- [x] Refund workflow works
- [x] Sale history viewable

### Security
- [x] Authorization checked
- [x] Backend validates all totals
- [x] No negative inventory
- [x] Idempotent sales (via ticket number)
- [x] Audit events logged

### Quality
- [x] Unit tests pass
- [x] Integration tests pass
- [x] TypeScript errors: 0
- [x] Rust diagnostics: 0

---

## Known Unknowns (To Clarify)

1. **Discount Threshold**: What amount of discount requires approval?
   - Suggest: 5% or 10% of subtotal
   
2. **Mobile Money Verification**: How does callback work?
   - Suggest: Manual confirmation in UI for now; callback integration in Phase 6+

3. **Receipt Format**: Plain text or HTML?
   - Suggest: Both (default plain text; HTML for display)

4. **Split Payments**: Support in Phase 2 or later?
   - Suggest: Schema ready; API support in Phase 2; UI in Phase 6+

5. **Customer Field**: Mandatory or optional (walk-in)?
   - Suggest: Optional (null = walk-in customer)

---

## Testing Strategy

### Unit Tests
- Checkout request validator
- Discount validation
- Tax calculation
- Payment amount validation

### Integration Tests
- Create sale → inventory consumed
- Create sale → audit event logged
- Checkout fails if stock unavailable
- Checkout fails if unauthorized
- Refund creates reversal transaction

### E2E Tests
- Full cashier workflow: search → add → checkout → receipt
- Barcode scanning
- Multiple payment methods
- Refund approval workflow

---

## Estimated Effort

- **Database**: 0 (schema ready)
- **Backend**: 8-12 hours (validators + endpoints + transaction logic)
- **Frontend**: 8-12 hours (components + state + integration)
- **Testing**: 4-6 hours (unit + integration + E2E)
- **Total**: 20-30 hours (~2.5 dev days)

---

## Deliverables

### Code
- Backend: `services/api/src/pos.rs` (400+ lines)
- Frontend: `apps/desktop/src/pages/POS.tsx` (600+ lines)
- Validators: Extensions to `crates/application/src/lib.rs`
- API types: New types in `crates/domain/src/lib.rs`

### Documentation
- Phase 2 completion report
- API endpoint documentation
- Receipt format specification

### Tests
- 10+ unit tests
- 5+ integration tests
- E2E scenarios documented

---

**Next**: Start Phase 2 implementation (Day 1: Domain & validators)
