# sharecli serve AuthN

## Model

| Mode | When | Behavior |
|------|------|----------|
| **Open** (default) | No token / no JWT config | All routes open. Intended for loopback-only binds. |
| **Bearer** | `SHARECLI_SERVE_TOKEN` or `config.serve.bearer_token` | Non-probe routes require `Authorization: Bearer <token>`. |
| **JWT** (federated IdP) | `auth_mode = "jwt"` or `[serve.jwt]` present | Non-probe routes require a Bearer JWT validated against JWKS (`iss`/`aud`/`exp`/`nbf`, RS256/ES256). |

Public without credentials in Bearer/JWT modes:

- `GET /healthz` (liveness)
- `GET /readyz` (readiness)

Env `SHARECLI_SERVE_TOKEN` always forces **bearer** mode (overrides JWT).

## Bearer

```bash
export SHARECLI_SERVE_TOKEN='replace-me'
sharecli serve --bind 127.0.0.1:9000

curl -sH "Authorization: Bearer replace-me" http://127.0.0.1:9000/config
curl -s http://127.0.0.1:9000/healthz   # still open
```

Or in `~/.config/sharecli/config.toml`:

```toml
[serve]
bearer_token = "replace-me"
```

## JWT (OAuth2 resource server)

Export your IdP JWKS (Okta / Azure AD / Google / Auth0) to a file, or point at a copied JWKS document:

```toml
[serve]
auth_mode = "jwt"

[serve.jwt]
issuer = "https://login.microsoftonline.com/{tenant}/v2.0"
audience = "api://sharecli-serve"
jwks_path = "/etc/sharecli/jwks.json"
```

Env overrides: `SHARECLI_SERVE_AUTH_MODE`, `SHARECLI_SERVE_JWT_ISSUER`,
`SHARECLI_SERVE_JWT_AUDIENCE`, `SHARECLI_SERVE_JWKS_PATH`.

HMAC algorithms (HS256) are rejected — use asymmetric keys only.

```bash
curl -sH "Authorization: Bearer $IDP_ACCESS_TOKEN" http://127.0.0.1:9000/config
```

## Audit log

Security-relevant events append JSON lines to:

- `SHARECLI_AUDIT_LOG` if set, else
- `$XDG_STATE_HOME/sharecli/audit.jsonl` / `~/.local/state/sharecli/audit.jsonl`
- Windows: `%LOCALAPPDATA%/sharecli/audit.jsonl`

Events include `auth_enabled`, `auth_disabled`, `auth_ok` (JWT includes `sub`), `auth_fail`, `serve_start`, `serve_stop`.
