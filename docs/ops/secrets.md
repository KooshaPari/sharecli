# Secret management (soft)

Audit-v38 **C01 L18**.

## Sources of secrets

| Secret | How to supply | Scanned? |
|--------|---------------|----------|
| `SHARECLI_SERVE_TOKEN` | Env / process manager | gitleaks rules cover bearer-like tokens |
| `SHARECLI_AUDIT_*` paths | Env | N/A (paths, not secrets) |
| OTel / Pyroscope endpoints | Env | Prefer non-secret URLs |
| Codesign / notarize | Org Actions secrets | L112 runbook |

## Rules

1. Never commit real tokens; `.env.example` stays placeholder-only.
2. Prefer OS secret stores / CI secrets for `SHARECLI_SERVE_TOKEN` in shared hosts.
3. gitleaks CI (`security.yml`) + `gitleaks.toml` are the soft gate for accidental commits.
4. Rotate serve tokens by restarting `serve` with a new env value.

## Soft follow-up

| Item | Status |
|------|--------|
| Scanning + example env | Done |
| Runtime OS keyring helper | Deferred |
| Sealed config file format | Deferred |

See also: [`crypto-keys.md`](crypto-keys.md), [`privacy-tenant.md`](privacy-tenant.md).
