# Tight perf budgets (soft)

Audit-v38 **C00 L6 / L8** — operator runbook for micro-bench gates, local-loop
wall clocks, and idle RSS sampling. This file tightens the narrative around
existing harnesses; it does **not** add new hard merge gates.

## Scope

| Pillar | Surface | Gate posture |
|--------|---------|--------------|
| **L6** | Criterion microbenches (`config_parse`, `pool_list`, `prometheus_render`) | Hard `bench-gate` (10% mean regression) + CI profiler artifacts |
| **L6** | Agent / FR local loops | Soft wall clocks — see [`LOCAL_LOOP_BUDGETS.md`](LOCAL_LOOP_BUDGETS.md) |
| **L8** | `sharecli serve` idle RSS | Soft sample — see [`memory.md`](memory.md) + `rss-soft.yml` |

Bench **SLO numbers** remain canonical in [`SLO.md`](SLO.md) (BENCH-1..3). C08 owns
eval corpus / load harness expansion; C00 owns the cross-linked operator map here.

## Criterion bench gates

### CI jobs (`.github/workflows/bench.yml`)

| Job | Required? | Behavior |
|-----|-----------|----------|
| `criterion` | No (`continue-on-error: true`) | Runs three benches with `--sample-size 10`; advisory on PR/main |
| `bench-gate` | Yes | Fails when Criterion mean exceeds **10%** over `docs/eval/baselines/criterion-baseline.json` |

Checker: `scripts/check-bench-baseline.py` against
`target/criterion/<bench>/new/estimates.json`. Seed pin: `SHARECLI_BENCH_SEED=42`.
Reproduce locally per [`docs/eval/REPRO.md`](../eval/REPRO.md).

### Tight budgets (target state)

Gate tolerates **10%** mean regression (tightened 2026-07-19 from 25% using
`criterion-trends.csv` peak-to-peak ≤ 3.20% headroom; see [`TRENDS.md`](../eval/TRENDS.md)).

| ID | Bench | SLO p95 (draft) | Gate today | Tight target |
|----|-------|-----------------|------------|--------------|
| BENCH-1 | `config_parse` | < 1 ms | mean ≤ 1.10× baseline | ≤ 10% regression |
| BENCH-2 | `pool_list` | < 100 ms | mean ≤ 1.10× baseline | ≤ 10% regression |
| BENCH-3 | `prometheus_render` | < 500 µs | mean ≤ 1.10× baseline | ≤ 10% regression |

### Local quick check

```bash
# One bench, soft sample (≤ 2 min wall per LOCAL_LOOP_BUDGETS)
cargo bench --locked --bench config_parse -- --sample-size 10 --warm-up-time 1 --measurement-time 2

# Full gate parity (after all three benches)
python3 scripts/check-bench-baseline.py \
  --baseline docs/eval/baselines/criterion-baseline.json \
  --criterion-dir target/criterion \
  --threshold 0.10
```

Do **not** block FR lanes on full Criterion suites — see agent guidance in
[`LOCAL_LOOP_BUDGETS.md`](LOCAL_LOOP_BUDGETS.md).

## LOCAL_LOOP_BUDGETS cross-ref

[`LOCAL_LOOP_BUDGETS.md`](LOCAL_LOOP_BUDGETS.md) (L30.10 / T-270) caps **wall-clock**
for fmt, lint, test, and a single-bench smoke. Use it when:

1. An agent loop exceeds **2 min/bench** — switch to targeted `fr00N_*` tests.
2. Pre-PR hygiene — `just fmt-check` + `just lint` + FR slice before `bench-gate`.
3. Deciding whether to re-seed baselines — only after a clean local `cargo bench`
   on the same host class as CI.

| Local loop | Budget | Links here |
|------------|--------|------------|
| Perf gate (local) | ≤ 2 min / bench | Criterion commands above |
| FR acceptance | ≤ 30s after compile | Skip full bench matrix |
| Full test | ≤ 3 min warm (`just test-nextest`) | Run before perf-sensitive PRs |

## RSS soft gate (L8)

Memory **budgets** live in [`memory.md`](memory.md). Sampling is soft-only:

| Surface | Soft budget | Workflow |
|---------|-------------|----------|
| `sharecli serve` idle | < 64 MiB RSS (linux CI) | `.github/workflows/rss-soft.yml` |
| `sharecli serve` @ 32 procs | < 256 MiB RSS | `scripts/ops/rss_soft.sh` |

```bash
just rss-soft   # local parity with CI soft job
```

`rss-soft.yml` uses `continue-on-error: true` — failures are evidence, not merge
blockers. Allocator follow-ups: [`alloc-profiling.md`](alloc-profiling.md).

## Operator checklist

1. **Behavior change under hot paths** (`config`, pool, metrics): run matching
   Criterion bench locally; confirm `bench-gate` green on PR.
2. **Serve / spawn memory change**: run `just rss-soft`; compare to `memory.md`
   table.
3. **Baseline drift on main**: inspect `bench.yml` artifacts; re-seed only with
   intentional perf work and SLO table update in `SLO.md`.

## CI posture summary

| Check | L6 / L8 | Hard? |
|-------|---------|-------|
| `bench-gate` | L6 | Yes (10% regression; was 25% until 2026-07-19) |
| `criterion` job | L6 | No |
| `rss-soft.yml` | L8 | No |
| Criterion profiler artifacts | L6 | Yes (`bench.yml` upload after gate) |

**Soft goal:** L8 hard RSS gate deferred; serve pprof HTTP documented in `docs/ops/profiling.md`.
