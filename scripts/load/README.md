# Load scripts (macrobench tier)

| Script | Target | Env |
|--------|--------|-----|
| `healthz_burst.sh` | `GET /healthz` | `SHARECLI_LOAD_URL`, `SHARECLI_LOAD_N` |
| `soak_healthz.sh` | `GET /healthz` (5m soft loop) | `SHARECLI_LOAD_URL`, `SHARECLI_SOAK_SEC`, `SHARECLI_SOAK_INTERVAL_SEC` |
| `chaos_restart.sh` | kill + restart serve; recover `/healthz` | `SHARECLI_LOAD_URL`, `SHARECLI_SERVE_BIND`, `SHARECLI_SERVE_BIN`, `SHARECLI_CHAOS_RECOVER_SEC` |

Requires a running `sharecli serve` (except `chaos_restart.sh`, which manages serve). Repro notes: [`docs/eval/REPRO.md`](../../docs/eval/REPRO.md).
Soft CI: [`.github/workflows/load-soft.yml`](../../.github/workflows/load-soft.yml) (C05 L50); [`.github/workflows/soak-soft.yml`](../../.github/workflows/soak-soft.yml) (C05 L47). Chaos restart is local-only (see [`docs/ops/soak-chaos.md`](../../docs/ops/soak-chaos.md)).

```bash
bash scripts/load/healthz_burst.sh
# or:
just load-soft

bash scripts/load/soak_healthz.sh
# or:
just load-soak

bash scripts/load/chaos_restart.sh
# or:
just chaos-soft
```
