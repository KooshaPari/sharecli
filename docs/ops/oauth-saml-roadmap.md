# OAuth / SAML governance (soft)

Audit-v38 **C02 L21** — residual federated AuthN beyond JWT resource-server mode.

## Current scope (shipped)

| Mode | Status | Evidence |
|------|--------|----------|
| Open (loopback) | Done | `docs/ops/AUTH.md` |
| Bearer shared secret | Done | `src/serve_auth.rs`, `SHARECLI_SERVE_TOKEN` |
| JWT / JWKS (OAuth2 **resource server**) | Done | `[serve.jwt]` in config; `tests/fr012_serve_jwt_auth.rs` |

sharecli validates **Bearer access tokens** issued by an external IdP. It does **not** run an OAuth authorization server or SAML service provider.

See [`AUTH.md`](./AUTH.md) for operator configuration (issuer, audience, JWKS path, probe routes).

## Deferred (hard)

| Capability | Rationale | Effort |
|------------|-----------|--------|
| OAuth 2.0 Authorization Code (+ PKCE) login UI | Out of product boundary — local supervisor, not a web app IdP client | L |
| SAML 2.0 SP (ACS / metadata / logout) | Enterprise SSO for `serve` UI not in MVP; JWT/JWKS covers API tokens today | L |
| Unix-socket peer credentials | Platform-specific; loopback HTTP suffices for current threat model | M |
| OAuth actor attribution in audit JSONL | Blocked on spawn rows + stable `sub` → project mapping | M |

Documented in `THREAT_MODEL.md` and `SECURITY.md` as explicitly out of scope until a networked multi-tenant `serve` mode ships.

## Soft wiring (this roadmap)

1. Keep **JWT resource-server** as the only federated path; IdP owns login/consent.
2. Cross-link operator docs: this file ↔ `AUTH.md` ↔ `docs/ops/spawn-audit.md`.
3. When spawn audit rows land (`docs/ops/spawn-audit.md`), extend audit schema with optional `actor_sub` from validated JWT — no new login flows.
4. Revisit SAML/OAuth code-flow only if `GAP-QA-MATRIX` C02 L21 gap is promoted from **Closed** (JWT) to **multi-protocol**.

## Spawn audit tie-in

`docs/ops/spawn-audit.md` (C02 L28) targets JSONL `spawn` / `stop` events. Deferred hard items there include **SIEM export / OAuth actor attribution**.

Governance sequence:

1. Ship spawn rows (`project`, `capability`, `outcome`) to audit JSONL.
2. For JWT-mode `serve`, propagate `sub` from `auth_ok` into spawn rows when the same request context spawns a harness.
3. SAML/OAuth code-flow remains deferred — attribution uses existing JWT `sub`, not a new IdP integration.

## Soft follow-up

| Item | Status |
|------|--------|
| Operator JWT guide | Done (`AUTH.md`) |
| Residual OAuth/SAML roadmap | Done (this file) |
| Spawn → audit actor linkage | Deferred (see `spawn-audit.md`) |
| Authorization Code / SAML SP | Deferred |

See also: [`AUTH.md`](./AUTH.md), [`spawn-audit.md`](./spawn-audit.md), [`privacy-tenant.md`](./privacy-tenant.md).
