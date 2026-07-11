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

## Capture a profile

Returns Google [`profile.proto`](https://github.com/google/pprof/blob/master/proto/profile.proto)
bytes (pprof protobuf) — compatible with `go tool pprof` / Pyroscope ingest.

```bash
# 10s sample (default); hard-capped at 60s
curl -o profile.pb "http://127.0.0.1:9000/debug/pprof/profile?seconds=10"
# with auth:
curl -o profile.pb -H "Authorization: Bearer $SHARECLI_SERVE_TOKEN" \
  "http://127.0.0.1:9000/debug/pprof/profile?seconds=15"

# render locally (requires go tool pprof)
go tool pprof -http=:8081 profile.pb
# or: go tool pprof -svg profile.pb > profile.svg
```

Windows builds return **501** for this route — use an external sampler instead:

```bash
samply record -- sharecli serve --bind 127.0.0.1:9000
# or: cargo install flamegraph && cargo flamegraph --bin sharecli -- serve
```

## Safety

- Route is **not** public: when `SHARECLI_SERVE_TOKEN` / `config.serve.bearer_token`
  is set, Bearer auth is required (same as `/metrics/prometheus`).
- Without `SHARECLI_PPROF=1`, the handler returns **404**.
- Prefer loopback binds while profiling; sampling adds CPU overhead.
- Flamegraph SVG is **not** served in-process (avoids `inferno` → vulnerable
  `quick-xml`); convert with `go tool pprof` offline.

## Related

- OTel traces: `docs/ops/otel.md`
- RED metrics: `GET /metrics/prometheus`
- OpenAPI: `docs/openapi/serve.yaml` → `/debug/pprof/profile`
