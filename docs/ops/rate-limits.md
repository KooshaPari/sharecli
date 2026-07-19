# HTTP rate limits — sharecli serve (C02 L25)

Audit-v38 **C02 L25**. Sliding-window rate limiting on `sharecli serve` HTTP routes.

## Contract

| Surface | Behavior |
|---------|----------|
| `/healthz`, `/readyz` | Always exempt (orchestrator probes) |
| All other HTTP routes | Count toward limit when enabled |
| Disabled | Default when `rate_limit_max` unset or `0` |

## Configuration

`config.toml`:

```toml
[serve]
rate_limit_max = 120
rate_limit_window_secs = 60
```

Environment overrides (take precedence):

| Variable | Meaning |
|----------|---------|
| `SHARECLI_SERVE_RATE_LIMIT_MAX` | Max requests per window (`0` = off) |
| `SHARECLI_SERVE_RATE_LIMIT_WINDOW_SECS` | Window length (default `60`) |

## Response

When saturated, serve returns **429 Too Many Requests** with the unified JSON envelope:

```json
{
  "error": {
    "type": "rate_limit_error",
    "code": "rate_limited",
    "message": "HTTP rate limit exceeded; retry later",
    "request_id": null
  }
}
```

`Retry-After` header is set to seconds until the oldest request in the window expires.

## Related limits (unchanged)

- `ProjectLimits` / `max_memory_mb` — per-project resource caps in the runtime
- `SpawnPolicy` — build harness concurrency semaphore
- OS cgroup / job-object enforcement — deferred (see lane C02 L25 gaps)

## Evidence

- `src/serve_rate_limit.rs` — resolver + probe exemption
- `src/commands/serve.rs` — `serve_rate_limit_middleware`
- `tests/c02_serve_rate_limit.rs` — FR-003 gate

**Status:** hard (in-process) · **FR:** FR-003 · **Last sync:** 2026-07-19
