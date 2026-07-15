# HTTP / CLI error envelope (soft)

Audit-v38 **C00** (API contract) / serve surface.

## Soft JSON error shape (serve)

Auth and client errors should converge on:

```json
{
  "error": {
    "type": "authentication_error",
    "code": "unauthorized",
    "message": "missing or invalid bearer token",
    "request_id": null
  }
}
```

| Field | Meaning |
|-------|---------|
| `type` | Stable machine class (`authentication_error`, `validation_error`, …) |
| `code` | Stable snake_case code |
| `message` | Human English (see ADR-0003) |
| `request_id` | Optional; filled when request-id middleware lands |

Probe routes (`/healthz`, `/readyz`) stay bare `{ "status": ... }` by design.

## Current mapping

| Status | Today | Soft target |
|--------|-------|-------------|
| 401 Bearer | JSON body from `serve_auth` | Match envelope above |
| 4xx validation | Mixed | Envelope |
| 5xx | tracing + optional JSON | Envelope without leaking internals |

## Soft follow-up

1. Shared `ErrorEnvelope` type in `src/` used by auth + handlers.
2. OpenAPI component schema for `ErrorEnvelope`.
3. Golden tests for 401 envelope bytes.

Until then this doc is the contract seed (no breaking change required for probes).
