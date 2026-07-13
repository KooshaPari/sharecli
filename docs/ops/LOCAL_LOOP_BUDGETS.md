# Local loop timing budgets (L30.10 / T-270)

Measured on a developer workstation / CI-class runner. Use these as soft
budgets for agent loops — if you exceed them, prefer `just test-nextest` or
targeted `--test` filters.

| Loop | Command | Budget (wall) | Notes |
|------|---------|---------------|-------|
| Format check | `just fmt-check` / `cargo fmt --all -- --check` | ≤ 30s | Stable rustfmt |
| Lint | `just lint` / `cargo clippy … -D warnings` | ≤ 4 min cold / ≤ 90s warm | Full workspace |
| Unit + integration | `just test` | ≤ 8 min cold / ≤ 3 min warm | Prefer nextest in CI |
| Fast parallel tests | `just test-nextest` | ≤ 3 min warm | CI profile |
| FR acceptance slice | `cargo test --test fr00N_*` | ≤ 30s | After compile |
| Perf gate (local) | `cargo bench --bench <name> -- --sample-size 10` | ≤ 2 min / bench | Soft; CI has `bench-gate` |

## Agent guidance

1. After a behavior change under `src/`, run the matching `fr00N_*` tests first.
2. Before opening a PR: `just fmt-check` + `just lint` + targeted tests (or
   `just test-nextest` if time allows).
3. Do not block on full Criterion suites in FR lanes — owned by agent-c08.

**Status:** DONE (T-270) · **Last sync:** 2026-07-12
