# Soak & chaos (soft)

Audit-v38 **C05 L47** — long-run stability evidence.

## Soft plan

| Phase | Script / workflow | Pass criteria |
|-------|-------------------|---------------|
| Load burst | `load-soft.yml` + `healthz_burst.sh` | p95 < 500ms @ 50 rps |
| Soak 5m (soft) | [`.github/workflows/soak-soft.yml`](../../.github/workflows/soak-soft.yml) · [`scripts/load/soak_healthz.sh`](../../scripts/load/soak_healthz.sh) · `just load-soak` | 0 non-2xx /healthz |
| Chaos kill | [`scripts/load/chaos_restart.sh`](../../scripts/load/chaos_restart.sh) · `just chaos-soft` | serve recovers < 30s |

## Metrics

- RED series from `otel.md` / Grafana dashboard
- RSS soft gate: `rss-soft.yml`

Soft CI: `soak-soft.yml` (60s soak, `continue-on-error`). Local `just load-soak` runs 5m. L47 **2** (soak CI + chaos script on disk).

### Chaos restart (local)

`chaos_restart.sh` starts `sharecli serve`, SIGKILLs it, restarts, and polls `/healthz` until recovery or `SHARECLI_CHAOS_RECOVER_SEC` (default 30).

```bash
cargo build --locked --release -p sharecli
just chaos-soft
# or against custom bind/url:
SHARECLI_LOAD_URL=http://127.0.0.1:7700/healthz \
  SHARECLI_SERVE_BIND=127.0.0.1:7700 \
  bash scripts/load/chaos_restart.sh
```

**CI skip:** chaos restart is intentionally **not** wired in GitHub Actions — port reuse and kill/restart timing can flake on shared runners. Run locally or in `workflow_dispatch` soak jobs when debugging L47 recovery.

```bash
# against an already-running serve:
SHARECLI_LOAD_URL=http://127.0.0.1:7700/healthz bash scripts/load/soak_healthz.sh
# or local serve + 5m soft soak:
just load-soak
```
