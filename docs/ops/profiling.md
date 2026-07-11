# Continuous profiling (sharecli serve)

Opt-in CPU profiling for audit-v38 **L45**. Disabled by default so local binds
stay quiet unless you ask for samples.

## Enable

```bash
export SHARECLI_PPROF=1
# optional: require bearer when exposing beyond loopback
# export SHARECLI_SERVE_TOKEN=...
sharecli serve --bind 127.0.0.1:9000
```

## Capture a flamegraph

```bash
# 10s sample (default); hard-capped at 60s
curl -o profile.svg "http://127.0.0.1:9000/debug/pprof/profile?seconds=10"
# with auth:
curl -o profile.svg -H "Authorization: Bearer $SHARECLI_SERVE_TOKEN" \
  "http://127.0.0.1:9000/debug/pprof/profile?seconds=15"
```

Response: `image/svg+xml` flamegraph (Unix builds via the `pprof` crate).

Windows builds return **501** for this route — use an external sampler instead:

```bash
# example external tools
samply record -- sharecli serve --bind 127.0.0.1:9000
# or: cargo install flamegraph && cargo flamegraph --bin sharecli -- serve
```

## Safety

- Route is **not** public: when `SHARECLI_SERVE_TOKEN` / `config.serve.bearer_token`
  is set, Bearer auth is required (same as `/metrics/prometheus`).
- Without `SHARECLI_PPROF=1`, the handler returns **404**.
- Prefer loopback binds while profiling; sampling adds CPU overhead.

## Related

- OTel traces: `docs/ops/otel.md`
- RED metrics: `GET /metrics/prometheus`
- OpenAPI: `docs/openapi/serve.yaml` → `/debug/pprof/profile`
