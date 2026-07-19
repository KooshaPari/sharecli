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
| **Notes** | Includes process/health gauges and HTTP RED series (`sharecli_http_requests_total`, `sharecli_http_errors_total`, `sharecli_http_request_duration_ms_*`). See `docs/ops/otel.md` and Grafana `docs/ops/grafana/sharecli-serve.json`. |

## SLO-4 — AuthN failure burn

| Field | Value |
|-------|-------|
| **Objective** | Protected routes reject bad credentials without flooding (401 rate bounded) |
| **Target** | `rate(sharecli_http_unauthorized_total) / rate(sharecli_http_requests_total)` ≤ 10% over 5m, sustained <10m |
| **Probe** | Prometheus series from `/metrics/prometheus` + JSONL `auth_fail` in audit log |
| **Error budget** | Short bursts OK (misconfigured clients); 10m sustained burn → investigate IdP/JWKS/token |
| **Alert** | `SharecliAuthFailBurn` in `docs/ops/alertmanager/sharecli.yml` |

## Mapping to probes

| Endpoint | Role | Success |
|----------|------|---------|
| `/healthz` | Liveness | 200 + `status=ok` |
| `/readyz` | Readiness | 200 while serving; 503 once shutdown requested |
| `/metrics/prometheus` | Scrape | 200 + required metric name prefixes |

## Error budget policy

Formal MWMB burn-rate policy, escalation tiers, and alert-pair mapping:
[`error-budget-policy.md`](error-budget-policy.md) (C05 L46).

## Out of scope (for now)

- Continuous profiling push agent (Pyroscope); opt-in pprof HTTP is in `docs/ops/profiling.md`
- Live PagerDuty routing keys committed to git (use env / secret store)
- Formal signed on-call roster

Alert **rule pack + MWMB burn pairs + severity routing + runbooks** ship in
`docs/ops/alertmanager/sharecli.yml`, [`error-budget-policy.md`](error-budget-policy.md),
and `docs/ops/alerting.md`.

## Bench-linked targets (C08)

Draft performance budgets tied to Criterion benches and load scripts.
Reproduce via [`docs/eval/REPRO.md`](../eval/REPRO.md). Harbor/SWE-bench remain
out of scope for sharecli per
[`docs/adr/0002-eval-surface-out-of-scope.md`](../adr/0002-eval-surface-out-of-scope.md);
soft Harbor CI/soak live in benchora `harbor-soft` / `portage-temp` (not this repo).

| ID | Surface | Budget | Harness |
|----|---------|--------|---------|
| BENCH-1 | `config_toml_from_str` | p95 < 1 ms | `benches/config_parse.rs` |
| BENCH-2 | `pool_new_and_list_empty` | p95 < 100 ms | `benches/pool_list.rs` |
| BENCH-3 | `prometheus_render_32` | p95 < 500 µs | `benches/prometheus_render.rs` |
| LOAD-1 | `GET /healthz` burst | ≥ 99% success over N=200 | `scripts/load/healthz_burst.sh` |
| LOAD-2 | `GET /healthz` latency | hyperfine p50 trend | `scripts/bench/hyperfine-healthz.sh` |

### Measurement log (append-only)

| Date | SHA | Surface | Observed | Host |
|------|-----|---------|----------|------|
| 2026-07-10 | _(pending first soft bench.yml run)_ | BENCH-1..3 + LOAD-1 | budgets declared; no baseline yet | ubuntu-latest (CI) |
| 2026-07-10 | _(feat/sharecli-w2-perf-gate)_ | BENCH-1..3 gate | seeded baseline JSON = SLO p95 budgets; `bench-gate` fails if mean > 1.5× baseline (50% regression); soft `criterion` job retained | ubuntu-latest (CI) |
| 2026-07-18 | _(feat/sharecli-c08-bench-tighten)_ | BENCH-1..4 gate | `default_max_regression` 0.50→0.25 (mean ≤ 1.25× baseline); justified by `criterion-trends.csv` max peak-to-peak 3.20% | seed CSV + docs |

### Baseline gate notes (append-only)

- **2026-07-10:** Hard-ish per-PR gate added as job `bench-gate` in
  `.github/workflows/bench.yml`. Compares Criterion
  `target/criterion/<bench>/new/estimates.json` means to
  `docs/eval/baselines/criterion-baseline.json` via
  `scripts/check-bench-baseline.py`. Default max regression **50%**.
  Seeded means equal draft SLO p95 budgets (BENCH-1..3) so the gate is
  non-flaky until real CI means are re-seeded with
  `scripts/seed-bench-baseline.py`. Criterion `--save-baseline ci-gate`
  is used in CI for local HTML compare artifacts; exit status comes from
  the Python checker, not Criterion itself.
- **2026-07-18:** Tightened default max regression **50% → 25%** from committed
  trend CSV peak-to-peak (max 3.20% on `config_toml_from_str`). See
  `docs/eval/TRENDS.md`. L74 score unchanged (already 3).
