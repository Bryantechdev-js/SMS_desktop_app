# Implementation Plan and Definition of Done

This plan deliberately separates a working vertical slice from feature checkboxes. A module is complete only when its UI, API, domain rules, database constraints, permissions, audit events, localization, tests and recovery paths work together.

## Phase 0 — Baseline and decisions
- Preserve original client documents and record each requirement as Must/Should/Could with source reference.
- Replace permissive development CORS with loopback-only development configuration; add config validation and fail-closed production startup.
- Pin toolchain/dependencies; make CI build/test frontend, Rust workspace, migrations and installer.
- Record unresolved client policies and make them configurable/disabled until approved.

## Phase 1 — Identity, security and foundation
- Users, invitations/password setup, Argon2id, session rotation/revocation, MFA for privileged roles.
- Permission catalogue, role assignment, scope enforcement in API middleware/handlers, approval policy engine.
- Audit events for authentication, role changes, prices, stock, cash, exports, approvals and settings.
- Health/readiness, structured redacted logs, request IDs, rate limits, TLS deployment, secret management.

## Phase 2 — Master data and inventory
- Product/ingredient/packaging catalogue, bilingual labels, barcode, categories, units/conversions, locations.
- Suppliers and opening balance import with validation and signed-off reconciliation.
- Immutable movement ledger, lots/expiry, receipt/issue/count/adjustment, reservation and reorder rules.
- Concurrency and ledger invariant tests.

## Phase 3 — Procurement and receiving
- Requisitions, purchase orders, supplier delivery, partial receiving, quality/quarantine, discrepancies, cost and payable references.
- Approval and separation-of-duties policies.

## Phase 4 — Production
- Versioned recipes, yield/unit validation, allergens, work orders, capacity/availability checks.
- Batch states and operator checklists; actual consumption, output, rejects, waste and lot genealogy.
- Production costing and variance reporting.

## Phase 5 — POS and cash
- Scan/search/cart, server totals, configured tax/discount, tenders/split tender, receipt and idempotency.
- Atomic sale/payment/stock/audit commit; verified payment status; reversal/refund and approval workflows.
- Cash session open/close, expenses, denomination count, variance, manager review and reconciliation.

## Phase 6 — Reports and collaboration
- Role-scoped dashboards, exports, scheduled reports, internal task comments/approvals, notification centre.
- Optional email/SMS providers with consent, retry, deduplication and delivery monitoring.

## Phase 7 — Desktop experience and hardware
- Full English/French parity, light/dark theme, keyboard-first navigation, accessible UI, guided tour and contextual help.
- Windows printer queues, barcode HID, optional label printer; model-specific scale integration only after hardware test.
- Read-only TV/projector dashboard mode.

## Phase 8 — Reliability and release
- Encrypted automated backups, off-host copy, monitoring and restore drills.
- Signed updater, release signing, rollback and database migration compatibility.
- Windows clean-install tests, multi-PC LAN/VPN tests, security review, performance test, role-based UAT and acceptance sign-off.

## Definition of done per workflow
- No placeholder button or fake success.
- UI state and server state agree; pending/offline is explicit.
- Server enforces authorization; DB constraints protect invariants.
- Errors are actionable and do not leak secrets or sensitive data.
- English and French strings exist and are reviewed.
- Unit, integration and end-to-end tests cover normal, boundary, failure, concurrency and permission cases.
- Audit record exists for sensitive changes.
- Documentation and operator guidance are updated.
