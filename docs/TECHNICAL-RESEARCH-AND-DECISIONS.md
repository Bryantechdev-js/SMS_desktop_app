# Technical Research and Architecture Decisions

**Product:** Complexe SMS Bakery Operations Desktop  
**Owner:** Sinior.dev  
**Status:** Proposed engineering baseline; security and deployment acceptance pending implementation and tests.

## 1. Product architecture

Use a Rust-first modular monolith with a central service and thin desktop clients:

- Desktop shell: Tauri 2, Rust host, React + TypeScript UI in WebView2.
- API: Rust + Axum + Tokio; typed request/response contracts; versioned REST endpoints initially.
- Domain: Rust domain crate for business invariants, money/quantity types and state transitions.
- Application: use-case services, authorization policies, transaction boundaries and idempotency.
- Infrastructure: SQLx, PostgreSQL, filesystem/Windows integration, notification providers and telemetry.
- Database: PostgreSQL authoritative store; migrations are versioned, forward-only and tested on clean and upgrade paths.
- Communication: desktop clients call API over HTTPS; LAN clients never receive DB credentials. Remote clients use VPN/secure gateway.

Do not split into microservices at 3–5 concurrent workstations. A modular monolith is simpler to deploy, observe, back up and recover.

## 2. Technology choices

| Concern | Proposed technology | Reason / constraint |
|---|---|---|
| Desktop | Tauri 2 + Rust | Small native shell, narrow IPC and Windows integration |
| UI | React + TypeScript + Vite | Mature component ecosystem and fast iteration; strict typed contracts |
| Styling | CSS variables/design tokens + accessible component primitives | Consistent density, themes, contrast and maintainable visual language |
| API | Axum + Tokio | Async Rust service, clear middleware and typed handlers |
| Database | PostgreSQL 16+ (pin and test a supported major) | Transaction integrity, constraints, indexes, backups and multi-user concurrency |
| DB access | SQLx compile-time/query checking where practical | Typed SQL and migration integration |
| Validation | `validator`/typed domain constructors + DB constraints | Validate at UI, API and database boundaries |
| Serialization | Serde | Explicit stable API payloads |
| Observability | `tracing`, structured redacted logs, health/readiness checks | Diagnostics without leaking secrets or customer data |
| Tests | Rust unit/integration tests, `cargo test`, frontend tests, Playwright/Tauri smoke tests | Domain, API, UI and end-to-end coverage |
| Distribution | Tauri NSIS/MSI; signed releases | Windows deployment; choose one primary installer for pilot |
| Updates | Tauri signed updater or MSIX App Installer | Signed artifact verification is mandatory; never ship updater private key |
| CI | GitHub Actions Windows + PostgreSQL service | Reproducible build, migrations, tests and installer artifact |

Pin dependency versions and Rust toolchain in CI; update through reviewed dependency changes. Use cargo-deny/audit and frontend lockfile auditing; generate SBOM for releases.

## 3. Installation and topology

### Recommended first deployment

- One always-on office server/host runs the Rust API and PostgreSQL as managed services/containers, with reserved LAN IP, UPS and controlled administrator access.
- Each staff PC installs only the signed Tauri desktop client and connects to the API.
- API and database are not exposed directly to the public internet. Remote users connect through a managed VPN/secure gateway.
- If the existing server is not suitable, use a dedicated supported Windows/Linux host or managed cloud server after assessing reliability, backup and internet availability.

### Installer behavior

A desktop installer must not silently install services, change firewall rules, create privileged accounts or expose ports. The installer should install the client and prerequisite WebView2 if absent using a documented, signed, user-visible flow. Server provisioning is a separate elevated, explicitly approved setup wizard/script that validates machine suitability, creates a dedicated low-privilege service account, provisions database secrets, restricts network access and records the installation. Do not bundle an unpatched database executable in the client installer.

### Offline behavior

Support read-only cached reference data and drafts where safe. Queue explicitly eligible low-risk operations with globally unique IDs and visible Pending Sync state. Do not claim a sale, payment, stock movement or refund is final until the server has committed it. Conflict resolution must be deterministic and auditable; never silently merge conflicting stock or cash records.

## 4. Performance strategy

- Keep UI responsive; async API calls, cancellation, pagination, virtualization for large lists and debounced search.
- Use server-side filtering/sorting/pagination; avoid loading full movement history on dashboard.
- Batch writes within explicit transactions; use indexes based on measured query plans.
- Use bounded DB pool and request body limits; avoid unbounded background tasks.
- Use optimistic UI only for drafts; show server-confirmed status for financial/stock operations.
- Use caching for reference data with version/ETag invalidation; no stale cache for price, permissions or available stock during final sale validation.
- Define and measure cold start, dashboard load, POS scan-to-cart, checkout commit, report generation and backup duration on representative hardware.
- Do not claim performance targets until measured with realistic sample data and hardware.

## 5. Security baseline

Use OWASP ASVS 5.0 Level 2 as an API/application verification baseline, plus desktop-specific threat modeling. ASVS is not a certification and does not by itself cover every native desktop risk.

- Threat model: local malware, malicious/compromised workstation, LAN interception, credential theft, insider abuse, supply-chain compromise, tampered update, malicious import file, backup theft and server compromise.
- Tauri: minimal capabilities per window; deny unused shell/fs/process permissions; strict CSP; no remote content in privileged webviews; validate all IPC arguments; isolate untrusted content; never put secrets in frontend storage.
- API: TLS, authentication, short-lived/revocable sessions, Argon2id password hashing, MFA for privileged accounts, rate limiting, lockout/backoff, CSRF protection if cookie auth is used, secure headers and request-size/time limits.
- Authorization: deny by default; server-side permission + resource scope checks on every endpoint; separate duties; hide controls only as UX, never as authorization.
- Data: parameterized SQL; exact money; constraints; encryption in transit; protect backups and secrets at rest; minimize PII; redacted logs.
- Integrity: append-only financial/stock ledger, idempotency keys, transaction boundaries, concurrency-safe stock reservations and unique constraints.
- Files: validate import file size/type/encoding/schema; reject path traversal and macro execution; export to user-selected location with safe filenames.
- Updates: signed artifacts, protected private signing key, staged rollout, rollback plan and release provenance.
- Operations: least privilege service account, firewall allow-list, OS patching, dependency/SBOM review, security logs, restore drills and incident runbook.

## 6. Notifications and collaboration

- In-app notifications are the primary channel for approvals, low stock, delayed production, failed sync, backup failure and cash variance.
- Email/SMS are optional external channels, configured with provider credentials, consent and delivery logs. Never embed provider secrets in desktop clients. Retry with bounded backoff and deduplication; do not send sensitive financial or employee details in plain SMS.
- Internal messages should be task/approval comments attached to records, not an unrestricted chat system in the first release. Keep author, time, recipients and retention rules.

## 7. AI and automation boundaries

AI is optional assistance, never the authority for transactions or safety decisions.

Safe initial features:
- Local deterministic SKU/product/customer autocomplete and keyboard-first suggestions.
- Recipe scaling and stock/reorder calculations using validated business rules.
- Draft purchase suggestions from min/max levels and approved demand history.
- Production demand forecasts only after sufficient clean history; show confidence and explain assumptions.
- French/English UI translation from reviewed locale files; no live AI translation of critical labels without review.
- Contextual help assistant grounded in approved manuals; cite the exact help topic and offer escalation.
- Optional natural-language report query with strict read-only, scoped SQL/API tools and confirmation before exports.

AI must not autonomously approve refunds, change prices, create payroll, release food-safety holds, execute arbitrary SQL, send messages externally or post stock/cash transactions. Provide an opt-out, data-processing notice, no training on client data by default, retention controls and human review. Prefer local/offline deterministic autocomplete for low latency and privacy; cloud AI requires an explicit provider and legal/privacy review.

## 8. Notifications, backups, updates and recovery

- PostgreSQL backup plan: encrypted scheduled logical backups for simple restore, plus physical/base backup and WAL archiving/PITR where the recovery objectives justify it. Store a second encrypted copy off-host. Monitor success and age. Run restore drills; a backup that has never been restored is unverified.
- Define RPO/RTO with client. Initial proposal for discussion only: daily full logical backup plus frequent WAL archive where feasible; choose retention after capacity and legal review.
- Installer/update: signed NSIS/MSI or MSIX distribution. Tauri updater requires signature verification; private key must remain in a protected release secret store. Keep rollback package and database migration compatibility plan.
- Database migration: backup before schema changes; forward migration and rollback/restore plan; migrations run under deployment identity, not the desktop app's runtime DB user.
- Power/network failure: UPS, graceful service shutdown, health monitoring, documented manual continuity forms and controlled reconciliation after recovery.

## 9. UI/UX design system

### Visual direction

A calm, confident bakery operations workspace: warm neutral canvas, deep ink/navy for structure, restrained teal for primary action and positive status, amber for attention, red only for destructive/critical states. Avoid decorative gradients and excessive badges. Prioritize legibility, hierarchy, and consistent spacing over visual noise.

Suggested tokens (validate with accessibility checks):
- Ink `#172B3A`; deep navy `#102A3A`; teal `#137C70`; pale teal `#E8F4F1`.
- Canvas `#F5F7F8`; surface `#FFFFFF`; border `#E3E9EC`; muted text `#657784`.
- Warning `#A96312` on `#FFF4E5`; danger `#B42318` on `#FEF0EF`; success `#147D64` on `#E9F6F1`.

### Interaction rules

- One obvious primary action per screen; consistent back/cancel/save placement.
- Labels always visible; placeholders are examples, not labels.
- Use plain words, short sentences, local terminology and examples; provide French and English parity.
- Confirm destructive/high-risk actions with a summary and reason; do not ask confirmation for harmless navigation.
- Autosave drafts with visible Saved/Saving/Offline state; never silently auto-submit a sale or approval.
- Keyboard navigation, focus indicators, predictable shortcuts, high contrast, scalable text and touch targets.
- Empty states explain what the user can do next; errors explain cause, impact and safe recovery.
- Contextual help, guided tours and role-specific checklists; allow users to dismiss/replay.
- Avoid information overload: progressive disclosure for advanced options; show only role-relevant tasks.

## 10. Release acceptance gates

A release is not production-ready until all applicable gates pass:
1. Domain unit tests for money, units, state transitions, recipes, yield and cash reconciliation.
2. Database migration tests on clean DB and supported upgrade path; constraint and concurrency tests.
3. API tests for auth, RBAC, scope, idempotency, atomic POS, refunds, stock reservation, audit and rate limiting.
4. UI tests for keyboard, accessibility, localization, error/empty/loading/offline states and high-risk confirmation.
5. End-to-end tests with real PostgreSQL for procurement → receipt → production → output → sale → cash close → reports.
6. Windows install/uninstall/upgrade test on clean Windows 11, no developer tools present.
7. Multi-PC LAN test, remote VPN test, printer/scanner model test and outage/reconnect test.
8. Backup restore drill, update signature verification, rollback test and security review.
9. User acceptance by each real staff role using anonymized/approved operational data.

## 11. Current known implementation gap

The supplied starter/checkpoint is not a finished product. It currently has a basic desktop shell, limited product/inventory endpoints and an early database model. POS, production, procurement, cash, full authentication/RBAC enforcement, localization, notifications, backups, updater, hardware integration, AI features, automated end-to-end tests and installer verification remain incomplete. The current API has permissive CORS and no authentication and must remain loopback-only until security middleware is implemented and tested.

## 12. External references

- Tauri Windows installer: https://tauri.app/distribute/windows-installer/
- Tauri security capabilities: https://tauri.app/security/capabilities/
- Tauri updater and signing: https://tauri.app/plugin/updater/
- Microsoft MSIX automatic updates: https://learn.microsoft.com/en-us/windows/msix/app-installer/auto-update-and-repair--overview
- Microsoft package signing guidance: https://learn.microsoft.com/en-us/windows/msix/app-installer/installing-windows10-apps-web
- OWASP ASVS 5.0: https://asvs.owasp.org/
- PostgreSQL backup and restore: https://www.postgresql.org/docs/18/backup.html
- PostgreSQL WAL archiving / PITR: https://www.postgresql.org/docs/current/continuous-archiving.html
