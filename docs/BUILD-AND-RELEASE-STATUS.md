# Complexe SMS — build and release status

**Status: engineering work in progress; not approved for production or client use.**

## Verified starting point

- The supplied source archive contains a Tauri 2 + React/TypeScript desktop shell and a Rust/Axum + PostgreSQL API foundation.
- The current shell implements catalogue/inventory display and product creation, plus a first-run owner setup and sign-in screen. The API now implements initial identity/session handling and permission checks for the existing catalogue and inventory read/receipt endpoints. POS, production, procurement, cash and complete staff administration remain unfinished.
- The supplied PRD and SRS are explicitly marked as drafts awaiting validation by Complexe SMS. Product catalogue, prices, recipes, tax treatment, stock deduction timing, refunds, approvals, payment methods and cash-close rules remain business decisions.

## Environment limitations in this build session

- This build environment has Node.js 22.16.0 and Python 3.13.5.
- Rust/Cargo, pnpm, PostgreSQL client/server and Windows MSVC build tools are not installed in this environment.
- External package registry access is unavailable (DNS resolution failed for registry.npmjs.org).
- Therefore no dependency installation, Rust compilation, database migration execution, Tauri Windows bundling, installer signing, or Windows runtime test can honestly be reported as completed here. A local TypeScript check was attempted but cannot pass without the React/Vite packages; `cargo test` is blocked because Cargo is absent; PostgreSQL migration execution is blocked because neither `psql` nor Docker is installed. A static source-structure check and YAML parse passed, but these are not substitutes for compilation or database execution.

## Work added in this engineering pass

- Added `database/migrations/0002_operations_security.sql` with the initial relational structures for individual users, roles, permissions, revocable sessions, suppliers, purchase receipts, inventory lots, recipes, production batches, cash sessions, sales, sale lines, payment states, refunds, cash movements/counts and immutable audit/stock ledgers.
- Added database constraints for non-negative quantities/amounts, explicit lifecycle states, unique ticket/idempotency identifiers, unique provider references, separation of refund requester/approver, and immutable posted stock/audit records.
- Added a least-privilege role and permission catalogue. This is a permission vocabulary, not a completed authorization implementation; API middleware and endpoint enforcement are still required.
- Added a Windows CI workflow that runs the database migrations against PostgreSQL 16 on Linux, then builds the API and unsigned Tauri installers on a Windows runner. It has not yet been executed.
- Removed unused frontend imports and replaced implicit React namespace type references with explicit type imports.

## Release gates still open

1. Complete and review identity management: staff lifecycle, password change/reset, MFA, rate limiting, brute-force protection for unknown accounts, security event monitoring, and a full authorization matrix for every business route. The initial password/session/permission foundation exists but is not production-complete.
2. Implement transaction services for receiving, production, POS, refunds, cash reconciliation and audit event creation. Use serializable/row-locking or equivalent concurrency controls and idempotency.
3. Implement the matching desktop workflows, French/English localization, keyboard/accessibility behavior, printer/barcode configuration and safe network status handling.
4. Execute migrations against supported PostgreSQL versions; test rollback/upgrade paths and all business invariants.
5. Add unit, integration, API authorization matrix, concurrency, E2E, backup/restore and security tests.
6. Build on Windows 11 x64 with the approved Rust MSVC toolchain, sign the installer/update artifacts, and test clean install, upgrade, uninstall, firewall configuration and multi-client LAN/VPN operation.
7. Complete client UAT and approve business rules before loading real data.

A successful compile alone is not a release gate. Until every critical gate above is evidenced, the software must be treated as a development build and must not be used to process real sales, payments or stock.

## Rebuild pass — bakery research and foundation hardening

- Added `docs/BAKERY_OPERATIONS_RESEARCH.md`: reference bakery lifecycle, actors/separation of duties, data model, stock/recipe/cash invariants, food-safety traceability, device patterns and on-site validation questions. It explicitly distinguishes reference workflows from verified Complexe SMS procedures.
- Added `docs/TECHNICAL-RESEARCH-AND-DECISIONS.md`: Rust/Tauri stack decisions, central API topology, installation trust boundaries, offline safety, performance strategy, ASVS 5.0 baseline, AI boundaries, notifications, backup/PITR, signed updates and UI design system.
- Added `docs/IMPLEMENTATION-PLAN.md` with vertical-slice phases and a per-workflow definition of done.
- Added `docs/REQUIREMENTS-TRACEABILITY.md` to map 25 requested capabilities to their source, current state and required acceptance evidence.
- Added an explicit minimal Tauri capability for the main window (`core:default` only; no shell/filesystem plugin grants).
- Added a fail-closed loopback bind check so the current unauthenticated development API refuses non-loopback interfaces.
- Replaced permissive API CORS with a small explicit development-origin allowlist and added a database readiness endpoint. The API remains unauthenticated and loopback-only; this is not production security.
- Added pure application validators for positive quantities, non-negative money and bounded required text, with unit tests. These tests have not been compiled or executed in this environment.
- Updated the README to clearly state current limitations and prevent treating the starter as production-ready.

## Validation for this pass

Only source-level inspection and static checks can be run here. Rust compilation, Cargo tests, frontend dependency installation/typecheck, PostgreSQL migration execution and Windows Tauri installer build remain unverified. No installer has been produced by this pass.

## Identity milestone update (source implementation; execution not verified)
Implemented in source: one-time owner bootstrap, Argon2 password hashing, opaque bearer sessions with database-side expiry/revocation, failed-login lockout, current-user endpoint, logout, and server-side permission checks on existing catalogue and inventory endpoints; receipt-only stock movement with other movement types rejected until their dedicated workflows exist; a first-run bilingual-ready login shell (English only at present). A first-run sign-in/setup screen is now present in the desktop UI. This has not yet been compiled or exercised against PostgreSQL in this environment; treat it as implementation awaiting automated and manual verification. User lifecycle, MFA, rate limiting, bilingual UI, and full business verticals remain open.
