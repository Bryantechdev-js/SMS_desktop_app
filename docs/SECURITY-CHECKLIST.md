# Production security checklist — not yet complete

- [ ] Argon2id password hashes; no plaintext passwords.
- [ ] Revocable, rotated, expiring sessions; privileged MFA.
- [ ] Server-side RBAC and least privilege for every endpoint.
- [ ] Rate limiting, login throttling, inactivity lock and session revocation.
- [ ] TLS on LAN; strict CORS; bind only intended interfaces; Windows Firewall allowlist.
- [ ] PostgreSQL inaccessible from client workstations.
- [ ] Audit log with actor, action, entity, timestamp and correlation ID; redact secrets.
- [ ] Tauri capability allowlist and restrictive CSP.
- [ ] Signed installers and updater; protect signing private key.
- [ ] Encrypted backups, integrity checks, offsite copy and tested restores.
- [ ] Threat model and OWASP ASVS 5.0 Level 2 review.
- [ ] Review applicable Cameroon data-protection requirements with qualified local counsel.

## Additional desktop and operations controls to implement

- [ ] Explicit Tauri capabilities per window; deny unused shell, process and filesystem access.
- [ ] Strict Content Security Policy; no remote content in privileged windows.
- [ ] Validate IPC and API inputs at boundaries; enforce request size/time limits.
- [ ] Import/export file validation, safe paths, no macro execution, audit sensitive exports.
- [ ] API origin allowlist, TLS and CSRF protections appropriate to the selected session model.
- [ ] Dedicated least-privilege Windows service identity; no DB superuser for application runtime.
- [ ] Secrets stored in OS-protected storage or server-side secret manager, never frontend/local config.
- [ ] Security event monitoring, correlation IDs, dependency review and SBOM.
- [ ] Restore drill, signed update verification, staged rollout and rollback evidence.
- [ ] Screen privacy mode and automatic lock for shared terminals / public dashboard displays.
- [ ] Review against OWASP ASVS 5.0 Level 2 plus a desktop-specific threat model; record exceptions and evidence.

## Identity foundation (implemented in this development milestone; integration tests still required)
- [x] Argon2 password hashing; no plaintext password persistence.
- [x] One-time initial owner bootstrap protected by a configured secret and a PostgreSQL advisory transaction lock.
- [x] Opaque random bearer tokens; only SHA-256 token digests are stored in the database.
- [x] Server-side session expiry (12 hours), revocation and active-user checks.
- [x] Login lockout after repeated failed attempts (15-minute lock after 8 failures).
- [x] Server-side permission lookup for catalog read/manage and inventory read/receive/adjust routes.
- [x] Default-deny behavior when no role grants the required permission.
- [ ] Add administrator-managed user lifecycle, password change/reset and MFA enrollment/recovery.
- [ ] Add integration tests against PostgreSQL for bootstrap races, login lockout, expiry, revocation and permission denial.
- [ ] Add rate limiting and security event alerting at the API edge before any LAN deployment.

The bearer token is held in frontend memory and is not persisted to browser local storage. The current login implementation is an initial identity milestone, not a complete identity-management product or production deployment approval.
