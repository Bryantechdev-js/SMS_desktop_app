# Requirements Traceability — Complexe SMS Bakery Desktop

This matrix combines the supplied proposal/PRD/SRS/meeting-control pack with the additional operational research. “Foundation” means only a partial schema or early scaffold exists; it does not mean the requirement is implemented.

| ID | Requirement | Source / rationale | Current state | Acceptance evidence required |
|---|---|---|---|---|
| R-01 | Product catalogue, SKU/barcode, units and prices | PRD/SRS; meeting kit | Partial API/catalogue only | CRUD tests, validation, role permissions, bilingual fields |
| R-02 | Supplier records and purchase orders | PRD/SRS; receiving control matrix | Schema foundation | PO approval and partial-delivery E2E |
| R-03 | Receiving, quality inspection and discrepancy | Strategic proposal; bakery workflow research | Schema foundation | Supplier document, lot, accepted/rejected quantity and audit test |
| R-04 | Ingredient and finished-goods stock ledger | SRS; operational workflow | Early movement API/schema | Ledger invariant, lot/location, concurrency and negative-stock tests |
| R-05 | FIFO/FEFO and expiry | SRS draft / discovery question | Policy unresolved | Client-approved policy, allocation tests and expiry alerts |
| R-06 | Versioned recipes and unit conversion | PRD/SRS; production flow | Schema foundation | Recipe scaling, dimensional validation and version immutability tests |
| R-07 | Production planning and batch tracking | Strategic proposal; bakery workflow | Schema foundation | Plan→approve→in-progress→complete E2E with consumption/output/waste |
| R-08 | POS, receipts and barcode lookup | PRD/SRS | Not implemented | Scan/cart/price/tax/tender/receipt E2E and performance evidence |
| R-09 | Atomic sales and stock deduction | SRS / integrity design | Not implemented | Concurrent checkout, idempotency and transaction rollback tests |
| R-10 | Refunds, voids and discounts with approval | Meeting control matrix | Schema foundation | Separation-of-duties and reversal tests |
| R-11 | Cash sessions and close reconciliation | Meeting kit cash model | Schema foundation | Denomination count, tender reconciliation, variance/approval tests |
| R-12 | Individual users, roles and least privilege | Strategic proposal / SRS | Schema foundation; API enforcement absent | Full endpoint permission matrix, session and MFA tests |
| R-13 | Audit trail | Strategic proposal / SRS | Table/trigger foundation | Actor attribution, tamper-resistance and completeness tests |
| R-14 | Dashboard and reports | PRD/SRS; KPI tables | Partial dashboard only | Reconciliation to source records, scoped exports and report tests |
| R-15 | English/French parity | User requirement | Not implemented | Locale completeness test and reviewed terminology |
| R-16 | Light/dark theme and accessible UI | User requirement | Light theme foundation only | WCAG contrast review, keyboard and reduced-motion tests |
| R-17 | Guided onboarding and contextual assistant | User requirement | Not implemented | Role-based tour/help content, replay/dismiss and accuracy review |
| R-18 | Email/SMS notifications | User requirement | Not implemented | Provider integration, consent, dedupe, retry and delivery audit |
| R-19 | Automated backup and restore | SRS / technical research | Not implemented | Scheduled backup, encrypted off-host copy and restore drill |
| R-20 | Signed installer and automatic updates | User requirement | CI workflow foundation only | Signed clean install, signature rejection, upgrade and rollback tests |
| R-21 | Multi-PC LAN / remote connectivity | SRS / architecture | Architecture only | TLS, VPN, firewall and multi-client acceptance test |
| R-22 | Printers, barcode scanners, scales | User requirement | Not implemented | Actual model/driver tests and fallback procedures |
| R-23 | TV/projector read-only dashboard | User requirement | Not implemented | Privacy, read-only and timeout test |
| R-24 | AI-assisted suggestions and analysis | User requirement | Research/design only | Opt-in, privacy review, grounded outputs and human approval tests |
| R-25 | Fast, efficient operation | User requirement | Not measured | Defined hardware/data benchmark and recorded p50/p95 results |

## Source authority and unresolved items

The six client-facing documents are explicitly marked as a discussion/validation baseline. Requirements depending on the bakery's actual catalogue, recipes, tax/pricing, stock deduction moment, payment providers, returns, cash close, expiry, retention and staffing authority must be confirmed by Complexe SMS before production activation. Software defaults must not be mistaken for client-approved policy.
