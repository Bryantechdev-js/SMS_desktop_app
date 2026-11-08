# Complexe SMS Bakery Operations Research

**Prepared for:** Sinior.dev / Complexe SMS  
**Status:** Engineering research baseline — operational facts still require on-site validation  
**Scope:** Bakery production, procurement, inventory, point of sale, cash control, staff coordination and food-safety traceability.

## 1. Evidence boundary

The Complexe SMS discovery pack explicitly says its documents are proposals and hypotheses, not proof that the bakery's real processes, volumes, legal obligations or policies have been verified. The product must therefore support configurable workflows and record approved decisions. It must not hard-code assumed tax rates, recipes, shelf lives, supplier terms, payment providers or staff authority limits.

The source documents establish this target flow: receive and inspect purchased materials; record supplier, quantity, cost and evidence; plan production from recipes; record actual consumption and waste; create finished-product lots; sell against validated tickets; record unsold, damaged and expired goods; reconcile declared payments against actual receipts at close.

## 2. Typical bakery operating lifecycle

A common bread/pastry workflow is:

1. Demand and production planning from sales history, pre-orders, seasonality, available staff, equipment capacity and opening hours.
2. Procurement planning from approved recipes, stock on hand, reserved quantities, reorder points, supplier lead times and minimum order quantities.
3. Supplier delivery and receiving: match purchase order to delivery note/invoice; inspect packaging, quality, quantity, lot/batch, expiry/best-before and storage requirements; quarantine rejected or uncertain goods.
4. Put-away: assign storage location; record unit and conversion; apply FIFO or FEFO according to item policy; keep allergen and contamination controls visible.
5. Preparation: issue ingredients to a production batch; verify recipe version, quantities, scales, equipment, staff assignment and sanitation checklist.
6. Mixing and dough handling: scaling/weighing, sieving where required by SOP, mixing/kneading, bulk fermentation when used, dividing, rounding, resting/intermediate proof, moulding and final proof.
7. Baking and finishing: load oven, record relevant time/temperature or process checks when required, bake, depann/de-pan, cool, slice/decorate/package, perform quality checks.
8. Finished goods: create a traceable lot with product, quantity, production time, responsible operator, recipe version, rejects, packaging and expiry/sell-by policy. Move accepted output into finished-goods stock.
9. Dispatch and sale: reserve/preorder allocation, counter sale, barcode or quick-key lookup, server-authoritative price/tax/discount calculation, payment capture and receipt.
10. End-of-day: count physical cash by denomination/payment channel; reconcile cash, Mobile Money/card/bank references, expenses, refunds and voids; explain variance; manager review; close shift.
11. Waste and recall: record unsold, damaged, expired, quality-rejected or returned products with reason, lot and quantity; authorize adjustment; preserve audit and traceability.

This is a reference process, not a claim that every step is currently used by Complexe SMS. The bakery's own SOPs take precedence once validated.

## 3. Main actors and separation of duties

| Actor | Main responsibilities | Controls / limits to configure |
|---|---|---|
| Owner / Director | Policy, price and role approval, high-value exceptions, reporting | MFA; approval for sensitive policy and financial changes |
| Bakery manager | Daily coordination, production plan, staff assignment, exception approval | Cannot erase completed transactions; approval limits |
| Baker / production operator | Follow recipe, record batch steps, actual ingredient use, output and rejects | Only assigned production; cannot approve own high-risk adjustment |
| Storekeeper | Receive, inspect, put away, issue ingredients, cycle counts | Receiving discrepancies and adjustments require reason and review |
| Procurement officer | Suppliers, quotations, purchase orders and follow-up | Separation from receiving/approval for material purchases |
| Cashier / sales operator | POS, payments, receipt, shift opening/closing | Cannot approve own refunds/voids above limit or edit prices |
| Accountant / finance | Reconciliation, expenses, review and reports | Read/approve finance scope; no stock or recipe mutation by default |
| Auditor / read-only manager | Review records and exception history | Read-only access; exports audited |
| System administrator | User/device/system setup, backups and technical health | No automatic business approval privilege; privileged access logged |

Small teams may combine roles, but the system should make conflicts explicit and require a second approver for configured high-risk operations.

## 4. Data the application must model

### Master data
- Products: SKU/barcode, display names in English/French, category, type (ingredient, packaging, finished good, service), base unit, allowed units, sale price, tax profile, active status, allergen declarations, storage and shelf-life policy.
- Units and conversions: each, loaf, tray, kg, g, litre, ml, bag, carton, etc. Conversion must be dimensionally valid and traceable; avoid silent conversions based on free text.
- Suppliers: contact, terms, approved status, lead time and supplier-specific item codes.
- Recipes: versioned ingredient lines, yield, expected process loss, instructions, allergens, equipment, time/temperature SOP references and effective date.
- Locations: receiving, dry store, cold store, freezer, production, finished goods, dispatch, quarantine and waste.
- Staff, roles, permissions, approval limits, work shifts and assigned stations.

### Operational records
- Purchase requisition/order, supplier delivery, accepted/rejected quantities and discrepancy evidence.
- Immutable stock movement ledger with source document, lot, location, quantity, unit, cost, actor, timestamp and reason.
- Production order/batch, recipe version, planned and actual inputs, process checks, output, rejects and waste.
- Sales, lines, price/tax/discount snapshots, payments, refunds/reversals, receipt and idempotency key.
- Cash sessions, opening float, collections by tender, expenses, cash counts, variance and approval.
- Audit events, notifications, backup/restore records, device registrations and synchronization state.

## 5. Important calculations and invariants

- Money is stored as integer minor units or exact decimal (`NUMERIC` / Rust decimal); never IEEE floating point.
- Recipe scaling uses exact decimal quantities and controlled rounding only at defined unit/scale boundaries.
- Expected ingredient requirement = recipe quantity per yield × planned yield, adjusted only by an approved, versioned factor.
- Available stock = accepted on-hand ledger balance − active reservations − quarantine/blocked quantities.
- Production yield and variance must be visible: actual good output, rejects, waste and unexplained difference.
- Theoretical cash = opening float + cash receipts − cash refunds − approved cash expenses − cash drops (according to configured policy).
- Stock is never edited by overwriting a balance. Receipts, consumption, output, sale, waste, return and adjustment create ledger movements.
- A completed sale is immutable. Corrections use a reversal/refund/credit-note workflow as approved by the client and applicable law.
- Payment marked as Mobile Money/card/bank is not “confirmed” merely because the cashier typed a reference. Confirmation requires a verified provider callback/API or a documented, authorized reconciliation process.
- The sale transaction is atomic: validate actor, price, availability and idempotency; write sale, lines, tenders, stock movements and audit record in one database transaction.
- No destructive delete for financially or operationally material records; use inactive/void/reversal status and audit.

## 6. Food safety, quality and traceability

The application should support, not replace, trained food-safety procedures. Configure checklists and evidence for receiving inspection, cleaning/sanitation, allergen handling, temperature checks where applicable, pest-control records, personal hygiene, equipment maintenance, product quality release, expiry and recall. Do not present generic software defaults as legal certification or HACCP compliance.

Traceability should answer: which supplier delivery/ingredient lots were consumed in which production batches, and which finished-goods lots were sold or dispatched to which tickets/customers (where customer identity is lawfully collected). A recall report should identify affected lots and current stock/dispatch history.

## 7. Physical tools and workstation patterns

- USB barcode scanners commonly emulate keyboard input (HID); support keyboard-wedge input and configurable suffix/prefix, with a dedicated scan focus and duplicate-scan protection.
- Thermal receipt printers should use installed Windows printer queues first; support configurable paper width, test print and fallback PDF. ESC/POS cash-drawer pulses only when the selected printer model and permissions are verified.
- Label printers require model-specific templates and driver tests.
- Scales are not generic: integrate only after model/protocol/serial/USB driver and calibration requirements are known. Do not assume Bluetooth support means a scale can be read safely.
- Projectors and smart TVs should display a dedicated read-only dashboard with no edit controls, no personal/financial detail by default, a privacy mode and automatic session timeout.
- Bluetooth is for supported peripherals, not for multi-PC database synchronization. PCs communicate over wired LAN/Wi-Fi through authenticated API; remote access uses a managed VPN or secure gateway.

## 8. Daily screens and interaction principles

- Dashboard: date/shift, branch, service status, today's validated sales, production status, low-stock alerts, orders due, pending approvals and backup status. Label stale/offline data clearly.
- POS: scan-first flow, large touch targets, clear cart and totals, explicit tender state, keyboard shortcuts, receipt preview and no hidden price overrides.
- Production board: batches grouped by status/time; recipe and allergen warnings; ingredient availability; simple step completion; actual output and waste capture.
- Inventory: searchable catalogue, stock by location/lot/expiry, receive/issue/count/adjust actions, units visible and reasons required.
- Procurement: supplier, PO, expected delivery, received versus ordered, discrepancies and approvals.
- Cash: current cashier/terminal, opening float, tender totals, expenses, cash count and variance explanation.
- Help: contextual plain-language guidance, examples, keyboard shortcuts and a guided first-run tour in the selected language.

Every operation should have visible states: Draft, Pending approval, In progress, Completed, Rejected, Failed, Offline pending, or Reconciliation required. Avoid success messages until the server commits the operation.

## 9. Reference sources

- FSSAI, *Food Safety Supervisor Course — Bakery Level 2*, manufacturing flow for bread and cake, receiving/quality/storage/issue/weighing/mixing/fermentation/dividing/proofing/moulding/baking/cooling/packaging/quality clearance/dispatch: https://fostacold.fssai.gov.in/fostac/doc/Traning%20Manual_Bakery%20%20Level%202.pdf
- FICSI, *Training Handbook: Bakery*, material flow and hygiene controls (mixing, processing, packaging/storage, FIFO): https://www.ficsi.in/upload/participant_handbook/4875b7a7ed5dccd9ce85cf580cf98cbb.pdf
- Complexe SMS source pack: `01_Proposition_strategique_Complexe_SMS.docx`, `02_PRD_PRS_Boulangerie_SMS.docx`, `03_SRS_Systeme_Gestion_Boulangerie_SMS.docx`, `06_Kit_reunion_controle_transactions_SMS.docx`.

## 10. Validation required on site

Confirm product catalogue, recipes/yields, unit conversions, ingredient and finished-good storage, expiry rules, actual equipment, work shifts, roles, opening hours, production cut-offs, stock deduction timing, taxes/pricing, tender providers, refunds/discounts/credit, approval thresholds, report formats, data retention and emergency paper process. Capture sample anonymized tickets, delivery notes, stock cards and daily cash reports with client authorization.
