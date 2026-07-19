# Secret management (runtime contract)

Audit-v38 **C01 L18**.

## Sources of secrets

| Secret | How to supply | Scanned? |
|--------|---------------|----------|
| `SHARECLI_SERVE_TOKEN` | Env / process manager / CI secret | gitleaks rules cover bearer-like tokens |
| `SHARECLI_SERVE_AUTH_MODE` | Env (`bearer` / `jwt` / open) | N/A (mode selector) |
| `SHARECLI_SERVE_JWT_ISSUER` | Env | N/A |
| `SHARECLI_SERVE_JWT_AUDIENCE` | Env | N/A |
| `SHARECLI_SERVE_JWKS_PATH` | Env (filesystem path to JWKS JSON) | Path only — JWKS file must not be world-readable |
| `SHARECLI_AUDIT_*` paths | Env | N/A (paths, not secrets) |
| OTel / Pyroscope endpoints | Env | Prefer non-secret URLs |
| Codesign / notarize | Org Actions secrets | L112 runbook |

## Runtime contract (`serve`)

Implementation: `src/serve_auth.rs` · operator guide: [`AUTH.md`](AUTH.md).

### Bearer (shared secret)

1. **Supply:** set `SHARECLI_SERVE_TOKEN` in the process environment (systemd `Environment=`, launchd, Docker secret mount, or GitHub Actions secret injected at deploy). Config file `serve.bearer_token` is a dev fallback only.
2. **Precedence:** a non-empty `SHARECLI_SERVE_TOKEN` **always** forces bearer mode and overrides JWT config.
3. **Comparison:** token is compared via SHA-256 digest (constant-time) — never log the raw value.
4. **Rotation:** issue a new token, update the secret store / env, restart `sharecli serve`. No hot reload of auth secrets.
5. **Exposure:** bind `127.0.0.1` unless TLS terminates at a reverse proxy; see [`crypto-keys.md`](crypto-keys.md).

### JWT (federated IdP)

1. **Supply:** configure `[serve.jwt]` in TOML or env overrides (`SHARECLI_SERVE_JWT_ISSUER`, `SHARECLI_SERVE_JWT_AUDIENCE`, `SHARECLI_SERVE_JWKS_PATH`).
2. **JWKS file:** must contain asymmetric keys only (RS256/ES256). HS* algorithms are rejected.
3. **Precedence:** JWT mode applies when `auth_mode=jwt` (or `[serve.jwt]` present) **and** `SHARECLI_SERVE_TOKEN` is unset/empty.
4. **Rotation:** refresh JWKS from the IdP; restart serve when `jwks_path` changes on disk (no in-process JWKS polling).
5. **Probe routes:** `GET /healthz` and `GET /readyz` stay public in all modes.

### Forbidden

- Committing real tokens, JWKS private keys, or `.env` with production values.
- Checking secrets into `config.toml` on shared hosts (use env / secret store).
- Logging `Authorization` headers or raw bearer tokens.

## Rules

1. Never commit real tokens; `.env.example` stays placeholder-only.
2. Prefer OS secret stores / CI secrets for `SHARECLI_SERVE_TOKEN` in shared hosts.
3. gitleaks CI (`security.yml`) + `gitleaks.toml` are the soft gate for accidental commits.
4. Rotate serve tokens by restarting `serve` with a new env value.

## Soft follow-up

| Item | Status |
|------|--------|
| Scanning + example env | Done |
| Runtime contract (bearer + JWT) | Done (this file) |
| Runtime OS keyring helper | Deferred |
| Sealed config file format | Deferred |

See also: [`crypto-keys.md`](crypto-keys.md), [`gitleaks.md`](gitleaks.md), [`privacy-tenant.md`](privacy-tenant.md).
