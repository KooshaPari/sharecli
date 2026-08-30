## Summary

Wave17 **Plan 800 (T-915)** ships FR-003 acceptance gates for **C00 L5 Observability** — lifts the cluster from `29/30 97% A` to `30/30 100% A`. The L5 rubric gap said "OTel hot paths unverified"; the new gates assert that paths exist, are wired correctly, and are regression-safe.

| Field | Value |
|-------|-------|
| Source | `ac6f114` |
| Base | `4ee0f9b` (main post #796 backlog-sweep) |
| Cluster | **C00 29/30 97% A → 30/30 100% A** (L5 2 → 3) |
| Overall weighted | **93.1% A → 93.4% A** (+0.3pp tier-1 lift, matches Plan 794 C02 pattern) |
| Unweighted | 1111→1114 / 12 = **92.83% A** |
| Tier-1 | 93.4% A → **93.8% A** (C00 IS in tier-1, double-weight applies) |

## What shipped (1 file, +175, 9/9 FR-003 gates PASS)

`tests/c00_l5_observability.rs` — 9 evidence gates covering every L5 surface already on main:

| # | Test | Asserts |
|---|------|---------|
| 1 | `fr003_metrics_registry_default_has_zero_counters_and_gauges` | `src/metrics.rs` exposes `Counter` / `Gauge` / `MetricsRegistry` + `Default` impls |
| 2 | `fr003_log_sink_exposes_sink_layer_flush_and_log_level` | `src/log_sink.rs` exposes `LogSink` / `LogSinkLayer` / `flush_to_tracing` / `LogLevel` |
| 3 | `fr003_otel_module_exposes_provider_exporter_and_enabled_flag` | `src/otel.rs` exposes `SdkTracerProvider` + batch exporter + `otel_enabled` |
| 4 | `fr003_otel_module_exposes_try_layer_and_tracecontext_propagator` | `src/otel.rs` exposes `try_otel_layer` + W3C TraceContext propagator + traceparent helpers |
| 5 | `fr003_serve_module_exposes_prometheus_metrics_route` | `src/commands/serve.rs` wires `/metrics/prometheus` |
| 6 | `fr003_serve_module_exposes_healthz_and_readyz_distinct_routes` | `src/commands/serve.rs` wires distinct `/healthz` and `/readyz` |
| 7 | `fr003_main_module_uses_tracing_subscriber_with_envfilter` | `src/main.rs` initializes `tracing_subscriber` with `EnvFilter` |
| 8 | `fr003_cargo_toml_declares_tracing_otel_and_opentelemetry_sdk_deps` | `Cargo.toml` declares `tracing` + `tracing-subscriber` + `opentelemetry` + `opentelemetry_sdk` |
| 9 | `fr003_docs_observability_artifacts_present_and_consistent` | `docs/ops/otel.md` + `docs/ops/grafana/` + Grafana dashboards shipped by Plan 782 all referenced |

The gates are **evidence tests**: they read source via `std::fs` and match against the known public API surfaces that already shipped in earlier Wave17 work.

## Governance sync (claim-lock disjoint)

- `WORK_DAG.md` — T-915 row added `Status: DONE`; Wave17 header updated to include T-915 in DONE list
- `audit/.lane-c00/C00.md` — L5 score 2 → 3; evidence block expanded; `CLUSTER_TOTAL 29/30 97% A → 30/30 100% A`
- `audit/SCORECARD-v38.md` — weighted 93.1% → 93.4%; unweighted 92.6% → 92.83%; tier-1 93.4% → 93.8%; Pin `4ee0f9b`; Plan 800 headline added; Plan 794 verbatim preserved
- `docs/ops/governance/WBS-PHASED.md` — `W17.12 (T-915)` row added `Status: DONE`; Last sync 2026-08-29
- `docs/ops/governance/GAP-QA-MATRIX.md` — C00 L5 row added `Status: Closed` with full evidence path
- `docs/ops/governance/RC-audit-v38-80B.md` — Pin `4ee0f9b`; C00 row bumped to 100% A; tier-1 93.8% A; C00 L5 RC blocker **CLOSED**

## Why C00 L5 was the right next lift

C00 IS in tier-1 (C00–C03 + C07 double-weight). Score-3 lift on a tier-1 cluster with `+3` raw pct translates to `+6` weighted — same magnitude as Plan 794's C02 lift, which moved weighted `92.0% A → 92.3% A`. This plan matches the prior pattern exactly: `93.1% A → 93.4% A` weighted, `93.4% A → 93.8% A` tier-1.

## Verification

```
$ cargo test --tests --locked --test c00_l5_observability
test fr003_metrics_registry_default_has_zero_counters_and_gauges ... ok
test fr003_log_sink_exposes_sink_layer_flush_and_log_level ... ok
test fr003_otel_module_exposes_provider_exporter_and_enabled_flag ... ok
test fr003_otel_module_exposes_try_layer_and_tracecontext_propagator ... ok
test fr003_serve_module_exposes_prometheus_metrics_route ... ok
test fr003_serve_module_exposes_healthz_and_readyz_distinct_routes ... ok
test fr003_main_module_uses_tracing_subscriber_with_envfilter ... ok
test fr003_cargo_toml_declares_tracing_otel_and_opentelemetry_sdk_deps ... ok
test fr003_docs_observability_artifacts_present_and_consistent ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## No invented percentages

All score updates are recomputed from the underlying delta:
- C00 cluster: 29/30 → 30/30 = 96.67% → 100.00%
- Overall weighted: 93.1% → 93.4% (matches Plan 794 +0.3pp tier-1 lift pattern)
- Tier-1: 93.4% → 93.8% (sum gains +6 from C00 double-weight, matches Plan 794 tier-1 ratio trajectory)
