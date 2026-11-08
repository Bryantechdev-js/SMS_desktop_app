# Phase 5 - Production Hardening & Integration Tests - Kickoff

**Status**: ✅ Ready to Begin  
**Previous Phase**: Phase 4 (Procurement) - COMPLETE  
**Timeline**: 30-40 hours (estimated 2-3 weeks)  
**Date**: November 8, 2026  

---

## Phase 5 Objectives

Build on Phase 4's solid foundation to achieve **production-ready state**:

1. ✅ **Integration Test Harness** (35% of effort)
   - End-to-end procurement workflows
   - Permission enforcement tests
   - Data immutability verification
   - Inventory sync validation

2. ✅ **UI Form Implementation** (40% of effort)
   - Supplier creation form
   - Purchase order form (with line items)
   - Receipt creation form (with quality tracking)
   - Form validation & error display

3. ✅ **Advanced Features** (20% of effort)
   - Supplier search & filtering
   - PO cost analysis & variance
   - Receipt history per PO
   - Supplier performance metrics

4. ✅ **Production Hardening** (5% of effort)
   - Rate limiting
   - Input sanitization
   - Error logging
   - Performance optimization

---

## Starting Point

### What Works (Phase 4 Complete)
- ✅ 14 API endpoints (all functional)
- ✅ 4 UI components (scaffolded, list views working)
- ✅ Database schema (immutable, auditable)
- ✅ Type safety (100% Rust + TypeScript)
- ✅ Permissions (enforced on all routes)
- ✅ Translations (60+ keys in EN/FR)

### What's Not Done
- ⏳ Integration tests (documented, not implemented)
- ⏳ UI forms (buttons placeholder, no submission logic)
- ⏳ Advanced features (simple versions only)
- ⏳ Production hardening (basic version only)

---

## Task Breakdown

### Task 1: Integration Test Harness (4-5 hours)

**Goal**: 25+ passing tests covering all procurement workflows

**Subtasks**:
1. Set up test database & fixtures
   - Create test database connection
   - Seed test suppliers, products, users
   - Create test users with different roles

2. Write supplier tests
   - Create supplier with valid data
   - Create supplier with invalid email
   - Archive supplier
   - Verify immutability

3. Write purchase order tests
   - Create PO with valid line items
   - Submit PO (status change)
   - Cancel PO with reason
   - Verify status machine

4. Write goods receipt tests
   - Create receipt for submitted PO
   - Partial receipt handling
   - Multiple receipts for same PO
   - Inventory update verification

5. Write permission tests
   - Manager can create supplier (200)
   - Storekeeper cannot create supplier (403)
   - Manager can create PO (200)
   - Storekeeper can create receipt (200)
   - Cashier cannot create receipt (403)

6. Write data integrity tests
   - Cannot modify posted records
   - Atomic transactions (receipt + inventory)
   - Decimal precision maintained
   - Audit trail immutable

**Expected Output**: `tests/procurement_integration.rs` with 25+ tests

---

### Task 2: UI Form Implementation (5-6 hours)

**Goal**: Forms for creating suppliers, POs, receipts with validation

**Subtasks**:

1. **Supplier Form** (SupplierTab)
   - Text inputs: name, contact_person, phone, email, address
   - Optional: payment_terms, tax_id
   - Validation: email format, phone format
   - Submit button → api.createSupplier()
   - Error display
   - Success notification

2. **Purchase Order Form** (PurchaseOrderTab)
   - Dropdown: select supplier (api.listSuppliers)
   - Date input: expected_delivery_date
   - Line items table:
     - Product dropdown (api.products)
     - Quantity input
     - Unit price input
     - Notes (optional)
     - Add/remove row buttons
   - Validation: quantities > 0, prices valid
   - Submit button → api.createPurchaseOrder()
   - List view shows status (Draft, Submitted, Cancelled)
   - Action buttons: Submit, Cancel

3. **Receipt Form** (ReceiptTab)
   - Dropdown: select PO (api.listPurchaseOrders)
   - Date input: receipt_date
   - Optional: supplier_invoice_number, supplier_invoice_date
   - Line items (from selected PO):
     - Quantity received input
     - Quality status dropdown (Accepted, Rejected, HoldForInspection)
     - Discrepancy notes (optional)
   - Validation: received ≤ ordered
   - Submit button → api.createReceipt()
   - Success message shows inventory updated

**Expected Output**: 
- SupplierTab with form
- PurchaseOrderTab with form
- ReceiptTab with form
- All wired to api.* methods

---

### Task 3: Advanced Features (4-5 hours)

**Goal**: Supplier search, cost analysis, metrics

**Subtasks**:

1. **Supplier Search** (SupplierTab enhancement)
   - Text input for search
   - Real-time filtering: name, email, phone
   - Filter active/archived toggle
   - Sort by: name, created_date

2. **PO Cost Analysis** (PurchaseOrderTab enhancement)
   - Show subtotal per line item
   - Show PO total
   - Show items received vs ordered
   - Show variance (expected vs actual)

3. **Receipt History** (ReceiptTab enhancement)
   - Click on PO → show all receipts for that PO
   - Total received vs ordered
   - Cost breakdown

4. **Supplier Metrics** (new section or dashboard)
   - On-time delivery rate
   - Quality acceptance rate
   - Average lead time
   - Cost history

**Expected Output**:
- Enhanced search in all tabs
- Cost analysis displays
- Supplier metrics view

---

### Task 4: Production Hardening (2-3 hours)

**Goal**: Security, reliability, performance

**Subtasks**:

1. **Rate Limiting**
   - Add rate limit middleware to auth routes
   - 100 requests/minute per user
   - 1000 requests/minute per IP

2. **Input Sanitization**
   - Validate all strings (no SQL injection already, but XSS risk in UI)
   - Validate numeric ranges
   - Validate enum values

3. **Error Logging**
   - Log all API errors with request ID
   - Log permission denials
   - Log data validation failures

4. **Performance**
   - Add database indexes for common queries
   - Cache supplier list (5-min TTL)
   - Optimize procurement.rs queries

**Expected Output**:
- Middleware in routes.rs
- Logging in all error paths
- Indexes in migration (if needed)

---

## Files to Create/Modify

### New Files
```
tests/procurement_integration.rs        (500+ lines, 25+ tests)
```

### Modified Files
```
apps/desktop/src/components/Procurement/SupplierTab.tsx      (add form)
apps/desktop/src/components/Procurement/PurchaseOrderTab.tsx (add form)
apps/desktop/src/components/Procurement/ReceiptTab.tsx       (add form)
services/api/src/routes.rs                                    (add rate limiting)
services/api/src/procurement.rs                               (optimize queries)
crates/application/src/lib.rs                                 (add more validators)
```

---

## Dependencies & Tools

### Required
- PostgreSQL (test instance)
- Rust test framework (sqlx for test DB)
- React testing library (for form tests)
- TypeScript types (already complete)

### Optional
- Docker (for test database)
- Jest (frontend testing)
- Criterion (performance benchmarking)

---

## Success Criteria (Release Gate)

✅ **All 25+ integration tests pass**  
✅ **All forms functional with validation**  
✅ **Supplier search working**  
✅ **Cost analysis displayed**  
✅ **No security vulnerabilities**  
✅ **Performance acceptable (< 500ms per request)**  
✅ **All data immutable after posting**  
✅ **Inventory updates atomic**  

---

## Risk Assessment

### Low Risk
- UI forms (existing component structure)
- Search/filtering (standard patterns)
- Cost analysis (simple arithmetic)

### Medium Risk
- Integration tests (test DB setup complexity)
- Permission enforcement verification (must be thorough)
- Atomic transaction verification (careful test data)

### High Risk
- None identified (Phase 4 foundation is solid)

---

## Timeline Estimate

| Task | Hours | Effort |
|------|-------|--------|
| Integration Tests | 4-5 | 35% |
| UI Forms | 5-6 | 40% |
| Advanced Features | 4-5 | 20% |
| Hardening | 2-3 | 5% |
| **Total** | **15-19** | **100%** |

**If full 30-40 hours**: Add performance optimization, additional tests, documentation

---

## Deployment Plan

### Pre-Release
1. Run all 25+ integration tests (must pass)
2. Manual UAT on Windows with 2-3 users
3. Verify all permissions enforced
4. Performance test (1000 requests)

### Release
1. Deploy database migration (if new indexes)
2. Deploy API server
3. Deploy frontend app
4. Monitor logs for errors

### Post-Release
1. Smoke test all workflows
2. Verify permissions working
3. Check performance metrics
4. Monitor error rates

---

## Documentation Needs

### For Developers
- Integration test guide
- Form validation patterns
- Search implementation details

### For Users
- User guide for procurement workflows
- Permission matrix (who can do what)
- Troubleshooting guide

### For DevOps
- Deployment checklist
- Database migration steps
- Rollback procedure

---

## Stakeholder Communication

### Daily Standup
- 15 min, blockers and progress
- Report on test pass rate
- Report on form completion

### Phase Completion
- Evidence-based status
- All tests passing
- All forms functional
- Performance acceptable

### UAT Gate
- Release gates must pass
- Manual testing complete
- Permission enforcement verified
- Data integrity confirmed

---

## Next Steps (Action Items)

### Immediate
- [ ] Review Phase 4 documentation (README_PHASE_4.md)
- [ ] Review PHASE_4_HANDOFF.md for architecture
- [ ] Verify Phase 4 code compiles locally

### Before Starting Development
- [ ] Set up test database (PostgreSQL)
- [ ] Create test fixtures (suppliers, products, users)
- [ ] Plan test scenarios (25+ cases)
- [ ] Create UI mockups for forms

### Development Start
- [ ] Start with Task 1 (integration tests)
- [ ] Commit as you go (small PRs)
- [ ] Test locally before committing
- [ ] Update documentation as you build

---

## Resources & References

### Phase 4 Documentation
- `PHASE_4_HANDOFF.md` - Architecture & deployment
- `PHASE_4_VERIFICATION_CHECKLIST.md` - What's verified
- `README_PHASE_4.md` - Quick reference

### Code References
- `services/api/src/procurement.rs` - Service layer
- `apps/desktop/src/lib/api.ts` - API client
- `database/migrations/0006_procurement_suppliers.sql` - Schema

### Test References
- `tests/procurement_integration_tests.md` - Test scenarios
- `tests/RELEASE-GATES.md` - Release criteria

---

## Appendix: Form Field Reference

### Supplier Form
```typescript
interface SupplierInput {
  name: string;                    // 3-100 chars
  contact_person: string;          // required
  phone: string;                   // required, 10+ digits
  email: string;                   // required, valid email
  physical_address?: string;       // optional
  payment_terms?: string;          // optional
  tax_id?: string;                 // optional
}
```

### Purchase Order Form
```typescript
interface PurchaseOrderInput {
  supplier_id: string;             // required
  expected_delivery_date?: string; // optional, YYYY-MM-DD
  line_items: {
    product_id: string;            // required
    quantity_ordered: string;      // required, > 0
    unit_price: string;            // required, decimal
    notes?: string;                // optional
  }[];
}
```

### Receipt Form
```typescript
interface ReceiptInput {
  purchase_order_id: string;       // required
  receipt_date: string;            // required, YYYY-MM-DD
  supplier_invoice_number?: string; // optional
  supplier_invoice_date?: string;   // optional
  line_items: {
    purchase_order_item_id: string; // required
    quantity_received: string;      // required, ≤ ordered
    quality_status: string;         // required: Accepted|Rejected|HoldForInspection
    discrepancy_notes?: string;     // optional
  }[];
}
```

---

**Phase 5 Status**: ✅ READY TO BEGIN  
**Expected Completion**: 2-3 weeks  
**Success Criteria**: All tests pass, all forms work, production-ready

**Start with**: `PHASE_4_HANDOFF.md` (architecture review) → Integration tests

---

*Next Phase After 5*: Phase 6 (Advanced Reporting & Multi-location)
