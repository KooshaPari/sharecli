# Bench trend store

Nightly Criterion means are exported by `scripts/bench/export-trend.py` and
uploaded as CI artifacts from `.github/workflows/bench.yml` (`bench-nightly`
job, cron `0 6 * * *` UTC).

## Local export

```bash
cargo bench --locked --bench config_parse -- --sample-size 10 --warm-up-time 1 --measurement-time 2
python3 scripts/bench/export-trend.py \
  --criterion-dir target/criterion \
  --out docs/eval/trends/local-$(date -u +%Y%m%d).json
```

## Artifact layout

Each nightly run uploads `bench-trend-<sha>.json` containing:

| Field | Meaning |
|-------|---------|
| `ts` | UTC timestamp |
| `sha` | Git commit |
| `seed` | `SHARECLI_BENCH_SEED` |
| `means_ns` | Criterion mean point estimates (nanoseconds) keyed by group/bench |

Download from the Actions run → Artifacts. Per-PR regression gating remains
`scripts/check-bench-baseline.py` against `docs/eval/baselines/criterion-baseline.json`
(50% threshold) — trends are longitudinal, not the merge gate.

## Hyperfine /healthz JSON

Nightly (and soft PR) runs also upload `hyperfine-healthz-<sha>.json` from
`scripts/bench/hyperfine-healthz.sh` (LOAD-2). Soft job:
`hyperfine healthz (soft)` (`continue-on-error`).

## Related

- `docs/eval/REPRO.md` — pins / seed contract
- `docs/testing/flake-policy.md` — Criterion flake quarantine
- `docs/ops/SLO.md` — BENCH + LOAD budgets
- `scripts/bench/README.md` — hyperfine harness
