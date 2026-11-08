# Architecture decisions

- Modular monolith: one Rust API process, clear domain/application/infrastructure boundaries.
- Tauri desktop clients communicate with API; no DB credentials in clients.
- PostgreSQL is authoritative. Stock is represented by immutable signed movement rows and balances are calculated from the ledger.
- Money uses PostgreSQL `NUMERIC` / Rust decimal, never binary floating point.
- Completed sales/refunds must be append-only and reversal-based; not implemented in this foundation.
- Any production LAN deployment must use TLS, authenticated sessions, server-side RBAC, strict CORS, firewall restrictions, backups and tested restore procedures.
- Current API is development-only: permissive CORS, no authentication, no audit identity, no rate limiting. Keep it bound to loopback.
