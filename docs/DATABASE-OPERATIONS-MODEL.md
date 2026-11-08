# Database operations model

## Core invariants

- PostgreSQL is reachable only by the API service. Desktop clients must never receive database credentials or connect directly to port 5432.
- Money is stored as `NUMERIC(14,2)`; quantities as `NUMERIC(14,3)`. Never use floating-point values for financial calculations.
- `inventory_movements` is the stock ledger. Balances are derived from posted movement deltas, not from a mutable quantity field.
- Posted inventory movements and audit events are append-only. Corrections use compensating entries with a reason and actor.
- Sales have a unique ticket number and idempotency key. A sale, sale lines, confirmed payment records, stock movements and audit event must be committed atomically by one backend transaction.
- Electronic payments remain pending until verified against an official provider response or controlled reconciliation evidence. A cashier declaration is not a confirmation.
- Refund requester and approver must be different users. The API must also check the `sale.refund.approve` permission.
- A cash session is unique per cashier while active. A variance is recorded and reviewed, never silently overwritten.
- Lot expiry is optional at schema level because not every product is perishable; the operational policy must be configured per item.

## Important implementation note

The migration establishes the persistence model and constraints. It does not by itself implement API authorization, workflow transitions, stock concurrency protection, payment-provider verification, or atomic transaction use cases. Those controls belong in the API/application layer and must be tested there.
