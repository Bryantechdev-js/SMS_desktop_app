# Phase 4: Procurement - Requirements Specification

**Phase**: 4 of 15  
**Module**: Procurement and Supplier Management  
**Status**: Specification (Requirements Phase)  
**Date Created**: November 8, 2026  
**Estimated Implementation Time**: 3-4 hours  

---

## BUSINESS OBJECTIVE

Enable bakeries to:
- Manage supplier relationships and contact information
- Create and track purchase orders for raw materials and packaging
- Record partial and complete goods receipts
- Match purchase orders to physical deliveries
- Track purchase history and supplier performance
- Manage purchase costs and maintain supplier pricing

---

## USER PERSONAS AND NEEDS

### Storekeeper (Primary User)
- **Responsibility**: Receiving goods, verifying deliveries, recording stock receipt
- **Needs**:
  - Quick supplier lookup and search
  - Simple purchase order creation and tracking
  - Ability to receive partial shipments
  - Clear visibility of what's outstanding
  - Easy receipt verification (quantity, quality, discrepancies)

### Manager (Approver)
- **Responsibility**: Approve purchase orders, manage supplier relationships
- **Needs**:
  - View pending purchase orders
  - Approve/reject orders before sending to supplier
  - View supplier performance (delivery timeliness, quality)
  - Compare supplier pricing for raw materials

### Accountant (Reviewer)
- **Responsibility**: Verify costs and maintain financial records
- **Needs**:
  - View purchase order costs
  - Verify received quantity matches invoiced quantity
  - Track supplier invoices
  - Export purchase history for reconciliation

### Owner (Business Intelligence)
- **Responsibility**: Monitor procurement spend and supplier relationships
- **Needs**:
  - View total spend by supplier
  - Identify procurement trends
  - Monitor for duplicate suppliers or conflicting pricing
  - Dashboard showing outstanding orders

---

## FUNCTIONAL REQUIREMENTS

### FR1: Supplier Management

**FR1.1: Create Supplier**
- User can create a new supplier record with:
  - Supplier name (required, unique)
  - Contact person name (required)
  - Phone number (required)
  - Email address (required)
  - Physical address (optional)
  - Payment terms (e.g., "Net 30", "COD", "Prepay")
  - Tax/VAT ID (optional)
  - Active status (default: true)
- System validates:
  - Supplier name is not duplicated
  - Phone and email are valid formats
- Server stores supplier in `suppliers` table

**FR1.2: View Supplier**
- User can view supplier details including:
  - All contact information
  - Payment terms
  - List of active purchase orders
  - List of recent receipts (last 5)
  - Total lifetime purchases
- User can view supplier's pricing history for products

**FR1.3: Update Supplier**
- User can edit supplier details:
  - Contact information
  - Payment terms
  - Active status
- System prevents modification of supplier name (to maintain history)
- All changes logged with timestamp and user who made change

**FR1.4: Archive/Reactivate Supplier**
- User can archive supplier (soft delete)
- System prevents creating POs with archived suppliers
- User can reactivate archived supplier
- All historical data preserved

**FR1.5: Search and Filter Suppliers**
- User can search by:
  - Supplier name (partial match)
  - Contact person name
  - Phone number
- User can filter by:
  - Active/archived status
  - Payment terms

### FR2: Purchase Order Management

**FR2.1: Create Purchase Order**
- User can create a PO with:
  - Supplier (required, dropdown)
  - Expected delivery date (optional)
  - PO number (auto-generated, format: PO-YYYY-MM-001)
  - Line items:
    - Product (required, dropdown, only active products)
    - Quantity (required, positive decimal)
    - Unit price (required, non-negative)
    - Notes (optional, e.g., "needs special packaging")
  - Total amount (calculated by system: sum of line items)
  - Status (default: "draft")
- System validates:
  - Supplier exists and is active
  - Products exist and are active
  - Quantities and prices are positive
  - Total is sum of all line items
- System prevents creation of duplicate POs within same day for same supplier

**FR2.2: View Purchase Order**
- User can view PO details:
  - All header information (supplier, date, status)
  - All line items with quantities, prices, subtotals
  - Total PO amount
  - Expected delivery date
  - Current receipt status (pending, partial, complete)
  - List of associated receipts with quantities received

**FR2.3: Update Purchase Order (Draft Only)**
- User can edit PO only if status is "draft":
  - Add/remove line items
  - Change quantities or unit prices
  - Change expected delivery date
  - Change supplier (if no receipts yet)
- System prevents editing if status is "submitted", "received", or "cancelled"
- All edits logged with timestamp and user

**FR2.4: Submit Purchase Order**
- User can change PO status from "draft" to "submitted"
- System validates:
  - PO has at least one line item
  - All quantities and prices are positive
- Once submitted, PO cannot be edited (only cancelled)
- System records submission timestamp and user

**FR2.5: Cancel Purchase Order**
- User can cancel PO if status is "draft" or "submitted"
- Cancellation reason (required)
- System prevents cancellation if receipts already exist for this PO
- Cancelled POs remain in history but marked as "cancelled"

**FR2.6: Search Purchase Orders**
- User can search by:
  - PO number
  - Supplier name
  - Date range
- User can filter by:
  - Status (draft, submitted, partial, complete, cancelled)
  - Supplier
  - Expected delivery date (overdue, upcoming, etc.)
- System shows:
  - PO number, supplier, total amount
  - Current receipt status
  - Days outstanding

### FR3: Goods Receipt Management

**FR3.1: Create Receipt (Partial or Complete)**
- User can create a receipt for a PO with:
  - Purchase Order (required, must exist and be submitted or already receiving)
  - Receipt date (default: today)
  - Receipt number (auto-generated, format: RCP-YYYY-MM-001)
  - For each PO line item:
    - Quantity received (required, <= ordered quantity - already received)
    - Quality status (good/damaged/missing)
    - Notes (e.g., "damaged package", "short by 5 units")
  - Supplier invoice number (optional, for reconciliation)
  - Supplier invoice date (optional)
- System validates:
  - Quantities match or are less than ordered
  - Quantities don't exceed (ordered - already received)
  - Receipt date is not in future
- System creates immutable receipt record
- System automatically updates product stock (adds to inventory) via stock movement ledger

**FR3.2: View Receipt**
- User can view receipt details:
  - Receipt number, date, PO number
  - Supplier name and invoice info
  - Line by line: ordered, received, quality status
  - Total received vs. total ordered for this PO
  - List of all receipts for this PO
  - Link to update PO receipt status

**FR3.3: Record Discrepancies**
- User can note quality issues:
  - Damaged goods
  - Missing items (short delivery)
  - Wrong product received
- System stores discrepancy with:
  - What was ordered vs. what received
  - Description of issue
  - Timestamp and user
- Discrepancies trigger low-stock alerts if quantity short

**FR3.4: Complete PO Receipt**
- When all items of a PO have been received:
  - System automatically marks PO as "complete"
  - Or user can manually mark as complete if all items received
- System prevents further receipts for completed PO unless reopened

**FR3.5: Reopen Receipt**
- Manager can reopen a completed PO:
  - Reason (required)
  - Allows recording additional late deliveries
- Reopened PO status becomes "partial" again

---

## NON-FUNCTIONAL REQUIREMENTS

### NFR1: Data Integrity
- All receipts are immutable (no edit/delete after creation)
- All PO status changes are logged with timestamp and user
- All supplier information changes are logged with audit trail
- Stock movements from receipts are atomically committed with receipt creation

### NFR2: Performance
- Supplier search returns results in < 100ms
- PO creation/update completes in < 500ms
- Receipt creation completes in < 1000ms (includes stock movement)
- List pages (POs, receipts) load in < 500ms

### NFR3: Accuracy
- Total amounts always calculated server-side (never trust client math)
- Quantities tracked to 3 decimal places (e.g., 5.125 kg)
- Prices tracked to 2 decimal places (NUMERIC(14,2))
- Totals use NUMERIC type (never floating-point)

### NFR4: Usability
- Supplier and product dropdowns show search as user types
- Estimated delivery dates show day-of-week
- Outstanding orders highlighted by how many days overdue
- Quick actions: "Create Receipt" button visible on PO detail

---

## ACCEPTANCE CRITERIA

### Supplier Management
- ✅ User can create supplier with validation
- ✅ Supplier names are unique (case-insensitive)
- ✅ User can view all supplier details
- ✅ User can update supplier (except name)
- ✅ User can archive/reactivate suppliers
- ✅ Search and filter work correctly
- ✅ All changes logged to audit trail

### Purchase Order Management
- ✅ PO numbers are auto-generated and unique
- ✅ User can create PO with multiple line items
- ✅ Totals calculated correctly (server-side validation)
- ✅ Only active products can be added to POs
- ✅ User can edit draft POs
- ✅ User cannot edit submitted POs
- ✅ User can submit, cancel, or archive POs
- ✅ Search and filter work correctly
- ✅ All status changes logged

### Goods Receipt Management
- ✅ Receipts are immutable after creation
- ✅ Receipt quantities cannot exceed ordered quantities
- ✅ Receipts automatically update inventory (stock movement ledger)
- ✅ User can record quality discrepancies
- ✅ PO status updates correctly based on receipts (draft → partial → complete)
- ✅ Can record partial receipts
- ✅ Outstanding receipts visible on PO detail
- ✅ All receipts logged to immutable ledger

### Security and Authorization
- ✅ Only authorized users can create/edit suppliers (manager or above)
- ✅ Only authorized users can create/edit POs (manager or above)
- ✅ Only authorized users can record receipts (storekeeper or above)
- ✅ All operations logged with user ID and timestamp
- ✅ Audit trail accessible to accountants and managers

---

## USER STORIES

### Storekeeper Workflow

**Story 1: Receive Partial Delivery**
```
As a storekeeper
I want to record a partial delivery of flour
So that I can track what we've received and what's still outstanding

Acceptance:
- Find the PO for the flour order (5 bags)
- Click "Create Receipt"
- Record that 3 bags were received
- Note that 2 bags are damaged
- Submit receipt
- See inventory increased by 3 bags
- See PO status still shows 2 bags outstanding
```

**Story 2: Resolve Delivery Discrepancy**
```
As a storekeeper
I want to note that we received the wrong item
So that the manager can contact the supplier to correct the issue

Acceptance:
- Create receipt showing items received
- Add discrepancy note: "Received BAK-005 instead of BAK-003"
- Submit receipt
- Manager gets alerted to discrepancy
- PO shows pending resolution
```

### Manager Workflow

**Story 3: Create and Approve Purchase Order**
```
As a manager
I want to create and approve a purchase order
So that the storekeeper knows what to expect

Acceptance:
- Search for supplier (e.g., "Flour Mills Inc")
- Click "New Purchase Order"
- Add line items: flour (100kg @ 50 XAF/kg), sugar (50kg @ 80 XAF/kg)
- See total automatically calculated (9,000 XAF)
- Set expected delivery date
- Submit order
- Order number generated (e.g., PO-2026-11-001)
- Status shows "submitted"
```

**Story 4: Monitor Outstanding Orders**
```
As a manager
I want to see all outstanding purchase orders
So that I can follow up with suppliers on late deliveries

Acceptance:
- View list of all submitted POs
- Filter by status (outstanding, overdue, complete)
- See days outstanding for each PO
- See total amount outstanding
- Click PO to view details
```

### Accountant Workflow

**Story 5: Reconcile Purchase Costs**
```
As an accountant
I want to match supplier invoices to purchase receipts
So that I can verify costs and manage cash flow

Acceptance:
- View receipt details
- See supplier invoice number and date
- See total amount for this receipt
- Compare with supplier's invoice
- Record any discrepancies
- Export purchase history for reconciliation
```

---

## TECHNICAL CONSTRAINTS

### Database
- All supplier data must fit in `suppliers` table
- All PO data must fit in `purchase_orders` and `purchase_order_items` tables
- All receipt data must fit in `purchase_receipts` and `purchase_receipt_items` tables
- Stock movements from receipts recorded in existing `inventory_movements` table
- No direct inventory updates allowed (only via stock movement ledger)

### API
- All calculations server-side (no client-side math for totals or quantities)
- All status changes must be atomic transactions
- Receipt creation must atomically create receipt + stock movement
- All timestamps in UTC, stored as TIMESTAMPTZ

### Frontend
- Supplier dropdowns searchable (autocomplete)
- Product dropdowns searchable
- Dates shown with day-of-week (e.g., "Monday, Nov 8, 2026")
- Outstanding orders highlighted (red if overdue, yellow if due soon)

---

## DEPENDENCIES

### Input Dependencies (from Phase 1 & 2)
- Product catalog (ACTIVE products only)
- Stock movement ledger (to record receipts)
- User roles and permissions (manager, storekeeper, accountant)
- Inventory balances (to prevent over-receipt)

### Output Dependencies (to Phase 5 & 6)
- Purchase history (for cost analysis in Phase 5 production)
- Supplier pricing (for purchasing optimization in Phase 7 reporting)
- Delivery performance (for supplier rating in future phases)

---

## SCOPE EXCLUSIONS (Phase 4)

- ❌ Supplier rating/performance metrics (Phase 7)
- ❌ Automated reorder suggestions (Phase 7)
- ❌ Multi-currency support (Phase 4+ future)
- ❌ Supplier portal/API (Phase 8+ future)
- ❌ EDI/XML order transmission (Phase 9+ future)
- ❌ Purchase requisition approval workflows (Phase 4+ future with Phase 6)

---

## SUCCESS METRICS

✅ All suppliers can be created and searched  
✅ All purchase orders can be created, submitted, and tracked  
✅ All goods receipts can be recorded and linked to inventory  
✅ Stock automatically updates when receipts recorded  
✅ All outstanding orders visible at a glance  
✅ Discrepancies captured and tracked  
✅ All data immutable after recording  
✅ Full audit trail for all operations  
✅ Zero TypeScript/Rust diagnostics errors  
✅ All acceptance criteria pass  

---

## NEXT PHASE (After Requirements Approved)

1. Create database schema (migration 0006)
2. Define API contracts (routes, request/response types)
3. Design UI components and pages
4. Create detailed task breakdown
5. Begin implementation (Phase 4 Design & Implementation)

