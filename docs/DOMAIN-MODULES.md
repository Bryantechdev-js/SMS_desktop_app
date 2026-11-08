# Domain modules and business rules

## Procurement
Supplier records, purchase orders, partial deliveries, goods receiving, discrepancy resolution and supplier balances. Accepted quantities create inventory receipt movements only after authorized receiving.

## Inventory
Ingredient and finished-good catalog, units/conversions, batches/expiry, movement ledger, cycle counts, controlled adjustments and minimum-stock alerts. No direct editable stock quantity.

## Production
Versioned recipes, scaling, availability checks, planned/approved/in-progress/completed/cancelled states, actual ingredient consumption, output, rejects, waste and finished stock output movements.

## POS
Product lookup/barcode, cart, server-calculated totals, cash/mobile-money payment types, split payments, receipts, idempotency keys, immutable sale and reversal-based refunds. A sale, payments, stock deduction and audit event must commit in one DB transaction.

## Cash
Cashier sessions, opening float, payments, expenses, cash movements, closing count, variance explanation and manager approval.

## Shared invariants
- Server authoritative; clients never determine final prices or stock.
- Lock stock rows / serialize relevant operations to prevent concurrent overselling.
- Reject negative stock unless a deliberate policy is approved and tested.
- Every adjustment requires a reason and permission.
- Every privileged action creates an audit event.
- Offline queues must use unique idempotency identifiers and conflict reconciliation; unsafe operations must wait for server authority.
