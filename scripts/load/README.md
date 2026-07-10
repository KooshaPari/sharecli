# Load scripts (macrobench tier)

| Script | Target | Env |
|--------|--------|-----|
| `healthz_burst.sh` | `GET /healthz` | `SHARECLI_LOAD_URL`, `SHARECLI_LOAD_N` |

Requires a running `sharecli serve`. Repro notes: [`docs/eval/REPRO.md`](../../docs/eval/REPRO.md).

```bash
bash scripts/load/healthz_burst.sh
```
