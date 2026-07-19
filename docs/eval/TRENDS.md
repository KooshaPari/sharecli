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
(25% threshold; see below) — trends are longitudinal, not the merge gate.

## Committed multi-week CSV

In-repo seed (three weekly rows derived from measured baselines):

`docs/eval/trends/criterion-trends.csv`

Append nightly JSON rows over time; CSV is the human-readable longitudinal store
for L74 soft-goal evidence.

## Threshold tighten (2026-07-18, C08 L74)

Gate `default_max_regression` moved **0.50 → 0.25** using only committed CSV
means (no invented CI numbers). Peak-to-peak across the three seed weeks:

| Bench | Means (ns) | Peak-to-peak |
|-------|------------|--------------|
| `config_toml_from_str` | 20349 → 21000 → 20800 | **3.20%** |
| `pool_new_and_list_empty` | 54272000 → 55000000 → 54800000 | 1.34% |
| `prometheus_render_32` | 29971 → 30500 → 30100 | 1.77% |
| `jwt_validate_rs256` | 5000000 → 5100000 → 5050000 | 2.00% |

Max observed week-to-week swing is **3.20%**. A **25%** gate is ~8× that
noise floor while still far below the prior 50% slack (and still under the
~2× headroom already baked into each `mean_ns`). Score stays **L74 = 3** —
tighten closes the GAP row; it is not a 2→3 lift (already complete). Further
5–10% tighten waits on real `ubuntu-24.04` nightly artifact rows, not seeds.

## Hyperfine /healthz JSON

Nightly (and soft PR) runs also upload `hyperfine-healthz-<sha>.json` from
`scripts/bench/hyperfine-healthz.sh` (LOAD-2). Soft job:
`hyperfine healthz (soft)` (`continue-on-error`).

## Related

- `docs/eval/REPRO.md` — pins / seed contract
- `docs/testing/flake-policy.md` — Criterion flake quarantine
- `docs/ops/SLO.md` — BENCH + LOAD budgets
- `scripts/bench/README.md` — hyperfine harness
