# Eval governance (C08 L80)

Process-supervisor evaluation policy for sharecli. Supersedes ad-hoc bench
additions; Harbor/SWE-bench agent corpora remain out of scope until product
scope changes.

## In-scope eval family

| Tier | Path | Purpose |
|------|------|---------|
| Microbench | `benches/*.rs` | Criterion hot paths (config, pool, metrics, auth) |
| Macro / load | `scripts/load/` | HTTP burst against live `sharecli serve` |
| Latency toolbelt | `scripts/bench/` | Hyperfine harnesses (e.g. `/healthz`) |
| Repro | `docs/eval/REPRO.md` | SHA, lockfile, toolchain, `SHARECLI_BENCH_SEED` |
| Baselines | `docs/eval/baselines/` | Committed Criterion means + optional hyperfine JSON |
| Trends | `docs/eval/TRENDS.md` | Nightly Criterion artifact contract |
| SLO budgets | `docs/ops/SLO.md` | BENCH-* / LOAD-* rows linked to harnesses |

**Current Criterion family (supervisor tasks):** `config_parse`, `pool_list`,
`prometheus_render`, `jwt_auth_validate` (FR-012 JWT validate).

## N/A surfaces (ADR map)

Governed by [`docs/adr/0002-eval-surface-out-of-scope.md`](../adr/0002-eval-surface-out-of-scope.md).

| Pillar | Surface | Status |
|--------|---------|--------|
| L75 | Cross-language (Py/Go/TS) eval parity | N/A |
| L76 | Harbor / portage / Terminal-Bench | N/A |
| L77 | Compression / spec-extraction benches | N/A |
| L78 | LLM token-burn / cost tracking | N/A |
| L71 (agent corpora) | SWE-bench / SWE-RL task sets | N/A |

Auditors score L75–L78 as **deferred / N/A**, not missing product. C08 lifts
come from supervisor benches + load + REPRO + CI gate + this doc.

## How to add a bench

1. **Pick a real hot path** — config parse, pool list, metrics render, JWT
   validate, audit emit, etc. Prefer public APIs in `sharecli::` crates; do
   not bench private internals.
2. **Add `benches/<name>.rs`** + `[[bench]]` in root `Cargo.toml` (`harness =
   false`).
3. **Document** — header comment with draft SLO; row in `docs/ops/SLO.md`
   (`BENCH-N`); link from `docs/eval/README.md`.
4. **Baseline** — run locally with pins from `REPRO.md`, then
   `python3 scripts/seed-bench-baseline.py` and commit
   `docs/eval/baselines/criterion-baseline.json` with ~2× headroom for CI
   variance.
5. **Wire CI** — `jwt_auth_validate` is included in `.github/workflows/bench.yml`
   (`criterion`, `bench-gate`, `bench-nightly`) with baseline key `jwt_validate_rs256`.
   Merge gate ownership: **C08 lane /
   perf maintainers**; flake quarantine per `docs/testing/flake-policy.md`.

Load scripts follow the same pattern under `scripts/load/` with `LOAD-*` SLO
rows and `scripts/load/README.md`.

## CI gate ownership

| Job | Owner intent | Gate? |
|-----|--------------|-------|
| `criterion` (soft) | Advisory Criterion on PR/push | No (`continue-on-error`) |
| `bench-gate` | `check-bench-baseline.py` vs committed JSON | **Yes** (50% max regression) |
| `bench-nightly` | `export-trend.py` + hyperfine JSON → Actions artifacts | No (longitudinal) |
| `hyperfine-healthz` (soft) | Live `/healthz` hyperfine JSON on PR/push | No (`continue-on-error`) |

Hyperfine JSON (`SHARECLI_HYPERFINE_OUT`) is produced locally and in CI
artifacts (`hyperfine-healthz-<sha>.json`); see `TRENDS.md` and
`scripts/bench/README.md`.

## References

- Lane evidence: `audit/.lane-c08/C08.md`
- Rubric: `audit/rubric/audit-30-pillar/audit-30-pillar-L71-L80-eval-coverage.md`
- FR traceability: `docs/specs/TRACEABILITY.md` (FR-002 config, FR-012 JWT)
