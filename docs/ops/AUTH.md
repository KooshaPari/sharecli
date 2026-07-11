# sharecli serve AuthN

## Model

| Mode | When | Behavior |
|------|------|----------|
| **Open** (default) | No `SHARECLI_SERVE_TOKEN` and no `config.serve.bearer_token` | All routes open. Intended for loopback-only binds. |
| **Bearer** | Token set via env (preferred) or config | Non-probe routes require `Authorization: Bearer <token>`. |

Public without a token even in Bearer mode:

- `GET /healthz` (liveness)
- `GET /readyz` (readiness)

## Enable

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

Env wins over config when both are set.

## Audit log

Security-relevant events append JSON lines to:

- `SHARECLI_AUDIT_LOG` if set, else
- `$XDG_STATE_HOME/sharecli/audit.jsonl` / `~/.local/state/sharecli/audit.jsonl`
- Windows: `%LOCALAPPDATA%/sharecli/audit.jsonl`

Events include `auth_enabled`, `auth_disabled`, `auth_ok`, `auth_fail`, `serve_start`, `serve_stop`.
