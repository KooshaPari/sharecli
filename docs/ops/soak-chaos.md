# Soak & chaos (soft)

Audit-v38 **C05 L47** — long-run stability evidence.

## Soft plan

| Phase | Script / workflow | Pass criteria |
|-------|-------------------|---------------|
| Load burst | `load-soft.yml` + `healthz_burst.sh` | p95 < 500ms @ 50 rps |
| Soak 30m | `scripts/load/soak_healthz.sh` (planned) | 0 non-2xx /healthz |
| Chaos kill | `scripts/load/chaos_restart.sh` (planned) | serve recovers < 30s |

## Metrics

- RED series from `otel.md` / Grafana dashboard
- RSS soft gate: `rss-soft.yml`

L47 stays **1** until soak script lands in CI (soft required check).
