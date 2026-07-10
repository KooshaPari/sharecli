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
| Soft CI | `.github/workflows/bench.yml` | Advisory; `continue-on-error: true` |
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
