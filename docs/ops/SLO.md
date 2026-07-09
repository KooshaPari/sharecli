# sharecli serve — draft SLOs

Draft service-level objectives for the `sharecli serve` HTTP surface.
Numbers are starting targets for local/daemon ops; tighten once scrape history exists.

## SLO-1 — Liveness availability

| Field | Value |
|-------|-------|
| **Objective** | `GET /healthz` returns HTTP 200 with `{"status":"ok"}` |
| **Target** | ≥ 99.5% successful probes over a rolling 30-day window |
| **Probe** | External or sidecar HTTP GET every 15–30s |
| **Error budget** | ~3.6 hours downtime / 30 days |
| **Notes** | Liveness only — process is up. Does not imply readiness for traffic. |

## SLO-2 — Controlled restart / readiness drain

| Field | Value |
|-------|-------|
| **Objective** | During intentional shutdown (Ctrl-C or thermal critical), `GET /readyz` flips to HTTP 503 before the listener exits |
| **Target** | ≤ 2 unplanned serve restarts per process per 24h (thermal-critical and crash-driven) |
| **Probe** | Compare `/readyz` vs `/healthz` during drain; alert on restart storms via notifier webhooks |
| **Error budget** | 2 restarts / day; burn rate > 1 restart / hour warrants investigation |
| **Notes** | Readiness is distinct from liveness (`/readyz` vs `/healthz` in `src/commands/serve.rs`). |

## SLO-3 — Metrics scrape freshness

| Field | Value |
|-------|-------|
| **Objective** | `GET /metrics/prometheus` returns Prometheus text exposition (content-type `text/plain; version=0.0.4`) including `sharecli_process_*` / `sharecli_health_check_*` series when processes are tracked |
| **Target** | ≥ 99% successful scrapes at ≤ 60s scrape interval |
| **Probe** | Prometheus (or equivalent) scrape job against `/metrics/prometheus` |
| **Error budget** | ~7 failed scrapes / day at 1/min cadence |
| **Notes** | Gauges today; RED/USE histograms are a follow-up, not part of this draft SLO. |

## Mapping to probes

| Endpoint | Role | Success |
|----------|------|---------|
| `/healthz` | Liveness | 200 + `status=ok` |
| `/readyz` | Readiness | 200 while serving; 503 once shutdown requested |
| `/metrics/prometheus` | Scrape | 200 + required metric name prefixes |

## Out of scope (for now)

- OpenTelemetry / W3C `traceparent` export
- PagerDuty / Alertmanager burn-rate multi-window alerts
- Formal error-budget policy signed by on-call
