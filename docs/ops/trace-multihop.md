# Multi-hop trace propagation (soft)

Audit-v38 **C05 L44**.

## Today (one hop)

| Boundary | Propagation |
|----------|-------------|
| Inbound HTTP → serve | `traceparent` extracted (`http_observability_middleware`) |
| Serve → OTLP | Optional export when `OTEL_EXPORTER_OTLP_ENDPOINT` set |
| Outbound HTTP from serve | Injects `traceparent` when making client calls from middleware path |

See [`otel.md`](otel.md).

## Soft multi-hop map

| Hop | Status |
|-----|--------|
| CLI command → serve HTTP | Soft: CLI should forward `traceparent` when calling local serve (operator/env) |
| CLI → supervised child env | Soft wired: `ProcessPool::spawn` injects `TRACEPARENT` (`src/otel.rs`, `src/runtime.rs`) |
| IPC daemon → supervised child env | Soft wired: IPC handler uses `ProcessPool::spawn` (same inject path as CLI) |
| Tray / desktop → IPC daemon | Soft wired: `sharecli-ffi` `sharecli_ipc_start` injects `TRACEPARENT` on sidecar spawn (`src/otel.rs`, `crates/sharecli-ffi`) |
| Tray / desktop → serve HTTP | Soft future: pass W3C headers on dashboard fetches |

## Operator tip

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
curl -H 'traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01' \
  http://127.0.0.1:9000/healthz
```

Hard lift remains wiring tray dashboard HTTP injectors and CLI→serve header forwarding end-to-end.
