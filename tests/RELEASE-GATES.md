# Release acceptance matrix

| Area | Required scenario | Expected result | Status |
|---|---|---|---|
| Authentication | Invalid password / locked account | Generic denial; rate limit; no secret in logs | Not run |
| RBAC | Cashier attempts stock adjustment approval | Server returns 403; audit/security event recorded | Not run |
| Procurement | Partial supplier delivery | Received quantity, discrepancy reason, cost and lot are traceable | Not run |
| Inventory | Concurrent consumption of final stock | No negative stock or duplicate movement | Not run |
| Production | Complete approved batch | Actual ingredient consumption and finished output post atomically | Not run |
| POS | Two simultaneous sales of final unit | At most one sale succeeds | Not run |
| POS | Retry same idempotency key | Same result; no duplicate ticket/payment/movement | Not run |
| Payments | Mobile Money declared but unverified | Remains pending; not marked paid | Not run |
| Refund | Cashier requests own refund approval | Approval denied; requester cannot approve own request | Not run |
| Cash | Closing count differs from expected | Variance remains visible with explanation and reviewer | Not run |
| Audit | Attempt to update/delete posted movement | Database rejects operation | Not run |
| Recovery | Restore encrypted backup to clean server | Counts and key business totals reconcile | Not run |
| Desktop | Clean Windows install and first launch | App installs, connects to configured server and shows health state | Not run |
| Network | LAN client and remote VPN client | TLS-authenticated API only; PostgreSQL not exposed | Not run |

Do not change a status to Pass without retaining test evidence, environment, build identifier and reviewer.
