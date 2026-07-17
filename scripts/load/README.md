# Load scripts (macrobench tier)

| Script | Target | Env |
|--------|--------|-----|
| `healthz_burst.sh` | `GET /healthz` | `SHARECLI_LOAD_URL`, `SHARECLI_LOAD_N` |
| `soak_healthz.sh` | `GET /healthz` (5m soft loop) | `SHARECLI_LOAD_URL`, `SHARECLI_SOAK_SEC`, `SHARECLI_SOAK_INTERVAL_SEC` |

Requires a running `sharecli serve`. Repro notes: [`docs/eval/REPRO.md`](../../docs/eval/REPRO.md).
Soft CI: [`.github/workflows/load-soft.yml`](../../.github/workflows/load-soft.yml) (C05 L50).

```bash
bash scripts/load/healthz_burst.sh
# or:
just load-soft

bash scripts/load/soak_healthz.sh
# or:
just load-soak
```
