# Eval reproducibility (sharecli)

Contract for repeating Criterion benches and load scripts with pinned inputs.
Harbor / SWE-bench are **out of scope** — see
[`../adr/0001-eval-surface-out-of-scope.md`](../adr/0001-eval-surface-out-of-scope.md).

## Pins

| Pin | Source | Notes |
|-----|--------|-------|
| **Git SHA** | `git rev-parse HEAD` | Record in SLO append rows and CI logs |
| **Lockfile** | `Cargo.lock` | Always `--locked` for benches |
| **Toolchain** | `rustc --version` (stable in CI) | Zig `0.14.1` required for `spawn-core-sys` |
| **Seed** | `SHARECLI_BENCH_SEED` (default `42`) | Reserved for future randomized load; Criterion uses its own sampling |
| **Env** | `CARGO_TERM_COLOR=always`, `RUSTFLAGS` unset for benches (do not use `-D warnings` for timing) | Soft `bench.yml` clears hard-fail flags |

## Surfaces

| Tier | Path | How to run |
|------|------|------------|
| Micro (Criterion) | `benches/*.rs` | `cargo bench --locked --bench pool_list` (etc.) |
| Macro / load | `scripts/load/healthz_burst.sh` | Start `sharecli serve`, then run script |
| Soft CI | `.github/workflows/bench.yml` (`criterion`) | Advisory; `continue-on-error: true` |
| Gate CI | `.github/workflows/bench.yml` (`bench-gate`) | Hard-ish; fails if mean > 1.5× committed baseline |
| SLO link | `docs/ops/SLO.md` § Bench-linked targets | Append-only measurement rows |

## Reproduce Criterion locally

```bash
export SHARECLI_BENCH_SEED=42
cargo bench --locked --bench config_parse
cargo bench --locked --bench pool_list
cargo bench --locked --bench prometheus_render
```

Optional wall-clock CLI latency (second toolbelt entry alongside Criterion):

```bash
# requires hyperfine: https://github.com/sharkdp/hyperfine
hyperfine --warmup 3 'cargo run --locked --quiet -- --help'
```

## Reproduce load burst

```bash
# terminal A
cargo run --locked -- serve --bind 127.0.0.1:7700

# terminal B
SHARECLI_LOAD_URL=http://127.0.0.1:7700/healthz \
  SHARECLI_LOAD_N=200 \
  bash scripts/load/healthz_burst.sh
```

## Recording results

Append a row under **Bench-linked targets** in `docs/ops/SLO.md` with:

1. ISO date  
2. Git SHA (short)  
3. Bench or script name  
4. Observed p50/p95 or success rate  
5. Host OS / runner label  

Do not rewrite historical rows.

## Baseline gate (C08 hard-ish)

Committed means live in
[`baselines/criterion-baseline.json`](baselines/criterion-baseline.json).
The soft `criterion` job stays advisory; the `bench-gate` job in
`.github/workflows/bench.yml` fails the workflow when any Criterion
`mean.point_estimate` exceeds `baseline_mean * (1 + threshold)`.

| Pin | Value |
|-----|-------|
| **Baseline file** | `docs/eval/baselines/criterion-baseline.json` |
| **Threshold** | `0.5` (50% regression) — generous for shared `ubuntu-latest` |
| **Checker** | `python3 scripts/check-bench-baseline.py` |
| **Seeder** | `python3 scripts/seed-bench-baseline.py` (after local/CI `cargo bench`) |

Local Criterion HTML compare (does **not** fail the process; gate still uses JSON):

```bash
cargo bench --locked --bench config_parse -- --save-baseline ci
cargo bench --locked --bench config_parse -- --baseline ci --noise-threshold 0.5
python3 scripts/check-bench-baseline.py --threshold 0.5
```

Refresh committed means only after a clean ubuntu-latest (or matching) run, then
append an SLO measurement row — do not rewrite older rows.
