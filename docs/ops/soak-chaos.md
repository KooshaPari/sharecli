# Soak & chaos (soft)

Audit-v38 **C05 L47** — long-run stability evidence. Chaos hard-gate promotion:
[`chaos-restart-hard-gate.md`](chaos-restart-hard-gate.md) (C05 L50 · T-630).

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

**CI hard soak (phase 2):** [`.github/workflows/chaos-restart-hard.yml`](../../.github/workflows/chaos-restart-hard.yml)
runs `chaos_restart.sh` on PR/push `main` **without** `continue-on-error`. Branch protection
and `ci-success` wiring remain deferred per [`chaos-restart-hard-gate.md`](chaos-restart-hard-gate.md).

Local / `workflow_dispatch` fallback when debugging recovery:

```bash
just chaos-hard
```

```bash
# against an already-running serve:
SHARECLI_LOAD_URL=http://127.0.0.1:7700/healthz bash scripts/load/soak_healthz.sh
# or local serve + 5m soft soak:
just load-soak
```
