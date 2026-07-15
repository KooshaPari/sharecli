# Privacy & tenancy (soft)

Audit-v38 **C02 L24**.

## Threat model (local single-operator)

sharecli supervises processes for **one operator / one machine** (or one trust
domain). It is **not** a multi-tenant SaaS control plane.

| Concern | Current posture |
|---------|-----------------|
| Tenant isolation | N/A — single trust domain; projects share one serve process |
| PII | Process names/env may appear in logs/metrics; treat hosts as trusted |
| GDPR "processor" role | Not claimed; operators remain data controllers for workloads they run |
| Project limits | CPU/mem caps isolate noisy projects (`ProjectLimits`), not tenants |

## Operator rules

1. Bind `serve` to loopback (default) unless behind an authenticated proxy.
2. Set `SHARECLI_SERVE_TOKEN` when the port is reachable beyond localhost.
3. Keep audit JSONL (`SHARECLI_AUDIT_*`) on disk with OS ACLs appropriate to the host.

## Soft follow-up

| Item | Status |
|------|--------|
| Explicit single-tenant policy | Done (this file) |
| Multi-tenant AuthZ / namespaces | Out of scope (would need ADR) |

See also: [`crypto-keys.md`](crypto-keys.md) (L22).
