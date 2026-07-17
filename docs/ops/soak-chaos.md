# Soak & chaos (soft)

Audit-v38 **C05 L47** — long-run stability evidence.

## Soft plan

| Phase | Script / workflow | Pass criteria |
|-------|-------------------|---------------|
| Load burst | `load-soft.yml` + `healthz_burst.sh` | p95 < 500ms @ 50 rps |
| Soak 5m (soft) | [`.github/workflows/soak-soft.yml`](../../.github/workflows/soak-soft.yml) · [`scripts/load/soak_healthz.sh`](../../scripts/load/soak_healthz.sh) · `just load-soak` | 0 non-2xx /healthz |
| Chaos kill | `scripts/load/chaos_restart.sh` (planned) | serve recovers < 30s |

## Metrics

- RED series from `otel.md` / Grafana dashboard
- RSS soft gate: `rss-soft.yml`

Soft CI: `soak-soft.yml` (60s soak, `continue-on-error`). Local `just load-soak` runs 5m. L47 **1→2** (chaos restart still planned).

```bash
# against an already-running serve:
SHARECLI_LOAD_URL=http://127.0.0.1:7700/healthz bash scripts/load/soak_healthz.sh
# or local serve + 5m soft soak:
just load-soak
```
