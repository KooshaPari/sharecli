# Load scripts (macrobench tier)

| Script | Target | Env |
|--------|--------|-----|
| `healthz_burst.sh` | `GET /healthz` | `SHARECLI_LOAD_URL`, `SHARECLI_LOAD_N` |

Requires a running `sharecli serve`. Repro notes: [`docs/eval/REPRO.md`](../../docs/eval/REPRO.md).
Soft CI: [`.github/workflows/load-soft.yml`](../../.github/workflows/load-soft.yml) (C05 L50).

```bash
bash scripts/load/healthz_burst.sh
# or:
just load-soft
```
