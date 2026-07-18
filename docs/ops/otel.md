# OpenTelemetry (sharecli serve)

sharecli wires the OpenTelemetry Rust SDK + OTLP/HTTP exporter behind standard
env vars. Local and CI runs stay collector-free unless you opt in.

## Enable export

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
# optional overrides:
# export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://127.0.0.1:4318/v1/traces
# export OTEL_SERVICE_NAME=sharecli   # Resource still forces service.name=sharecli

sharecli serve --bind 127.0.0.1:9000
```

Without those env vars, `tracing` fmt logging still works and the serve HTTP
middleware continues to extract/inject W3C `traceparent` on every request.

## Propagation

| Boundary | Behavior |
|----------|----------|
| Inbound HTTP | Read `traceparent`; attach to `http.request` span |
| Outbound HTTP response | Echo inbound `traceparent`, or synthesize one |
| Tray dashboard HTTP | `traceparent_http_value` + `tray_http::get` / `sharecli_serve_get` FFI; dashboard HTML embeds `data-traceparent` for `fetch` |
| OTLP | Batch export when endpoint env is set (`src/otel.rs`) |

## Metrics (RED)

`GET /metrics/prometheus` includes:

- `sharecli_http_requests_total`
- `sharecli_http_errors_total` (status ≥ 500)
- `sharecli_http_request_duration_ms` histogram (+ `_sum` / `_count`)

Plus existing process/health gauges. Import `docs/ops/grafana/sharecli-serve.json`
into Grafana against a Prometheus scrape of that endpoint.

## Continuous profiling

Opt-in CPU flamegraphs: set `SHARECLI_PPROF=1` and hit
`GET /debug/pprof/profile?seconds=10`. See `docs/ops/profiling.md`.

## Collector smoke

```bash
# example: otelcol with OTLP HTTP receiver on :4318
docker run --rm -p 4318:4318 otel/opentelemetry-collector:latest
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318 sharecli serve
curl -sH 'traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01' \
  http://127.0.0.1:9000/healthz -D -
```
