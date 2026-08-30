# Privacy & tenancy

Audit-v38 **C02 L24** (score-3, committed 2026-08-30).

## Threat model (local single-operator)

sharecli supervises processes for **one operator / one machine** (or one trust
domain). It is **not** a multi-tenant SaaS control plane.

| Concern | Current posture |
|---------|-----------------|
| Tenant isolation | N/A — single trust domain; projects share one serve process |
| PII | Process names/env may appear in logs/metrics; treat hosts as trusted |
| GDPR "processor" role | Not claimed; operators remain data controllers for workloads they run |
| Project limits | CPU/mem caps isolate noisy projects (`ProjectLimits`), not tenants |

## Single-tenant commitment

sharecli is designed as a **single-tenant local supervisor**. The product
explicitly does **not** implement multi-tenant isolation primitives:

- No namespace / tenant-key boundary on `serve` endpoints.
- No per-tenant resource quota system beyond `ProjectLimits` (which scopes
  resources per *project*, not per *operator*).
- No GDPR-data-processor responsibilities claimed (operators retain
  data-controller status for the workloads they run).
- No row-level security or per-operator data partitioning in audit logs.

This commitment is enforced at the architecture level: see
[`BOUNDARY.md`](../../BOUNDARY.md) for the process-orchestration trust
boundary and [`THREAT_MODEL.md`](../../THREAT_MODEL.md) for the STRIDE
threat model that defines which threats are in-scope vs. out-of-scope.

## Project-level resource isolation

The only isolation primitive currently shipped is `ProjectLimits`
(`src/config.rs:ProjectLimitsConfig`), which caps:

- **CPU% per project** — prevents noisy projects from starving others.
- **Memory MB per project** — caps total RSS.
- **Pool idle timeout** — reclaims idle project resources.

This is per-**project** isolation, not per-**tenant** isolation. Operators
who need true multi-tenant isolation must add a reverse proxy with its own
authentication layer; sharecli does not provide this.

## Operator rules

1. Bind `serve` to loopback (default) unless behind an authenticated proxy.
2. Set `SHARECLI_SERVE_TOKEN` when the port is reachable beyond localhost.
3. Keep audit JSONL (`SHARECLI_AUDIT_*`) on disk with OS ACLs appropriate to the host.
4. Treat the host as a single trust domain — do not run `sharecli` on a
   multi-user system without OS-level user isolation.
5. PII (process names, env values) may appear in logs/metrics — review
   retention settings before exporting logs beyond the operator's host.

## Out of scope

| Item | Reason |
|------|--------|
| Multi-tenant AuthZ / namespaces | Would need ADR + architecture change |
| Per-tenant data partitioning in audit log | Conflicts with single-tenant commitment |
| KMS / sealed secrets / hardware keys | Until networked multi-tenant mode exists |

See also: [`crypto-keys.md`](crypto-keys.md) (L22).