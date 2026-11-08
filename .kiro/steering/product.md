# Product Steering - Complexe SMS

**Complexe SMS** is a professional bakery operations management desktop system for Windows workstations connected to a central PostgreSQL server.

## Business Purpose

Enable bakeries to:
- Manage raw ingredients and finished goods inventory with traceability
- Track production from recipes to finished output
- Conduct point-of-sale transactions with multiple payment methods
- Manage procurement and supplier relationships
- Reconcile cash and payments
- Generate operational and executive reports
- Make data-driven business decisions

## Product Positioning

- **Users**: Owners, managers, bakers, cashiers, storekeepers, accountants, administrators
- **Deployment**: Windows 11 x64 workstations (3-5) + central database server
- **Geographic focus**: Cameroon (Douala), French/English bilingual
- **Business model**: Commercial software license, hosted database
- **Competition**: Manual spreadsheets, generic retail systems, incomplete specialty software

## Core Principles

1. **Trust first**: Correct accounting and inventory tracking is non-negotiable
2. **Simple operations**: Complex workflows feel intuitive and guided
3. **Offline resilience**: Graceful handling of network interruptions
4. **Audit ready**: Every transaction is traceable and immutable
5. **Bilingual**: Full French and English support, no language barriers
6. **Safe growth**: Scalable from single-location to multi-site operations

## Feature Scope

### Phase 1-3 (Complete)
- Inventory management (products, stock movements, low-stock alerts)
- Point of sale (shopping cart, multiple payment methods, receipts)
- Basic internationalization (English/French UI)

### Phase 4-6 (Next)
- Procurement (suppliers, purchase orders, goods receipt)
- Production (recipes, batch tracking, ingredient consumption)
- Cash management (session opening/closing, reconciliation)

### Phase 7-8 (Future)
- Operational reporting (sales, inventory, production, procurement)
- Executive dashboards and decision support
- Cost analysis and profitability

### Phase 9-15 (Extended)
- Notifications and alerts
- User settings and preferences
- Onboarding and training
- Optional AI assistance
- Hardware integration (barcode scanners, receipt printers)
- Security hardening
- Packaging and automatic updates
- Multi-workstation deployment

## Out of Scope (Phase 0)

- E-commerce integration
- Loyalty programs
- Table service (restaurants)
- Catering management
- Workforce scheduling
- Accounting integration (different software domain)

## Success Metrics

- ✅ Zero data loss due to software failure
- ✅ All transactions immutable and auditable
- ✅ No negative stock (with explicit policies)
- ✅ Financial totals always match backend calculation
- ✅ User completion time < 20% of manual process
- ✅ Bilingual support (50% of interface in French)
- ✅ Installation on fresh Windows in < 30 minutes
- ✅ Support of 3+ concurrent users on modest server

## Release Strategy

- **Current**: Engineering build (phases 1-3 complete, not production-ready)
- **Target**: Phase 15 (fully featured, tested, production-ready)
- **Timeline**: 30-40 hours engineering (full build cycle from phases 1-15)
- **Quality gates**: See tests/RELEASE-GATES.md for acceptance criteria

## User Personas

### Owner (decision maker)
- Responsible for business strategy and profit
- Limited technical knowledge
- Needs: Dashboard, financial summary, exception alerts

### Manager (operations)
- Day-to-day business oversight
- Moderate technical knowledge
- Needs: Reports, exception management, staff oversight

### Cashier (transaction)
- Conducts customer sales
- Low technical knowledge
- Needs: Fast, intuitive checkout, receipt printing

### Storekeeper (inventory)
- Manages raw materials and finished goods
- Low technical knowledge
- Needs: Clear stock visibility, simple receipt workflows

### Baker (production)
- Executes production plans
- Low technical knowledge
- Needs: Clear recipe instructions, ingredient consumption tracking

### Accountant (financial)
- Reviews cash and financial records
- High technical knowledge
- Needs: Detailed reports, export options, audit trails

### Administrator (technical)
- System setup and maintenance
- High technical knowledge
- Needs: Configuration, user management, backup/restore

## Key Differentiators

1. **Immutable ledger**: Every stock movement and sale is logged and cannot be altered
2. **Offline support**: Works without constant network access
3. **Bilingual native**: Not English-first with translation bolted on
4. **Role-based access**: Fine-grained permissions per business function
5. **Audit ready**: Food safety, financial compliance built-in
6. **Decimal accuracy**: Never loses money due to rounding (NUMERIC type)

## Non-Functional Requirements

- **Availability**: Target 99.5% uptime (single-location deployment)
- **Performance**: <500ms for typical user action
- **Security**: OWASP ASVS Level 2 target (with documented verification)
- **Scalability**: Support 3-5 concurrent workstations without degradation
- **Accessibility**: WCAG 2.2 AA compliance where applicable to desktop

## Compliance and Risk

- **Cameroon data protection**: Law No. 2024/017 personal data protection
- **Food safety**: Expiry and lot traceability, waste tracking
- **Financial**: Accounting audit trails, cash reconciliation
- **Access control**: Employee role-based permissions, separation of duties

## Product Roadmap

| Phase | Focus | Timeline | Status |
|-------|-------|----------|--------|
| 1-3 | Core inventory + POS + i18n | Week 1 | ✅ Complete |
| 4-6 | Procurement + production + cash | Week 2 | ⏳ Next |
| 7-8 | Reporting and business intelligence | Week 3 | ⏳ Future |
| 9-15 | Polish, deployment, hardening | Week 4 | ⏳ Future |

## Stakeholder Communication

- **Daily standup**: 15 min, blockers and progress
- **Phase completion**: Evidence-based status only (no fabricated features)
- **Specification freeze**: Each phase specification locked before implementation
- **UAT gate**: Release gates must pass before phase completion

## Next Steps

1. Complete Phase 0 audit (DONE)
2. Create Phase 4 specification (Procurement)
3. Implement Phase 4 (2-3 hours)
4. Test Phase 4 end-to-end
5. Continue to Phase 5

---

**Product Owner**: Sinior.dev / Complexe SMS  
**Principal Architect**: Kiro (AI-assisted engineering)  
**Baseline Date**: November 8, 2026  
**Status**: Engineering Rebuild In Progress
