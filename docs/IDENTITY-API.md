# Identity API — initial implementation

All identity endpoints use JSON. The API must remain bound to loopback in this development release.

## One-time owner bootstrap

`POST /api/v1/auth/bootstrap`

```json
{
  "bootstrap_secret": "the-one-time-secret-configured-in-the-api-environment",
  "username": "owner",
  "display_name": "Bakery Owner",
  "password": "a-unique-password-with-at-least-12-characters"
}
```

Bootstrap is disabled unless `BOOTSTRAP_SECRET` is configured with at least 24 characters. It can only create the first account. A PostgreSQL transaction advisory lock prevents concurrent bootstrap requests from creating multiple first owners. The first owner receives the `owner` role and grants defined in the database migration.

## Login

`POST /api/v1/auth/login`

```json
{"username":"owner","password":"the-owner-password"}
```

Returns an opaque bearer token once. Only its SHA-256 digest is stored in PostgreSQL. Passwords are hashed with Argon2id (Argon2 default variant and parameters). Sessions expire after 12 hours. Eight failed attempts trigger a 15-minute lockout for that account.

## Authenticated requests

Send `Authorization: Bearer <access_token>`.

- `GET /api/v1/auth/me` — current user and role codes.
- `POST /api/v1/auth/logout` — revoke current session.
- `GET /api/v1/products` — requires `catalog.read`.
- `POST /api/v1/products` — requires `catalog.manage`.
- `GET /api/v1/inventory/balances` — requires `inventory.read`.
- `POST /api/v1/inventory/movements` — currently accepts receipt movements only and requires `inventory.receive`.

Stock adjustments are deliberately rejected until an approval workflow exists. Sales, production consumption/output, waste and returns are deliberately rejected by this generic movement endpoint; their dedicated workflows must own their validation and atomic posting.

## Security limitations still open

This initial slice does not yet provide administrator-managed staff accounts, password change/reset, MFA enrollment/recovery, API-edge rate limiting, brute-force protection for unknown usernames, session/device management UI, CSRF protections for browser deployments, TLS/LAN configuration, audit monitoring, or complete endpoint authorization because most business endpoints do not yet exist. Do not treat it as production-ready.
