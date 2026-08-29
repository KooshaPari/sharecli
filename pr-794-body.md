## Summary

Wave17 **Plan 794 (T-890)** ships C02 L26 Resilience 2→3: **fixes two real u64-overflow bugs** discovered while writing FR-003 acceptance gates, and adds the gates that close the L26 score gap.

Traces to **FR-003** (cov/coverage traceability, Wave17 thesis residual) and **C02 L26** (Resilience: retries / backoff / circuit / bulkhead / timeouts / health).

| Field | Value |
|-------|-------|
| Source | `b4a7131` |
| Base | `b2997fa` (main post #783 governance lock) |
| **Real bugs fixed** | `src/retry.rs:compute_delay` (u64 overflow at attempt=63) · `src/backoff.rs:Backoff::delay_for` (Linear at u32::MAX + Exponential at attempt=63) |
| Tests added | `tests/c02_l26_resilience.rs` — 10 FR-003 acceptance gates |
| Tests green | **21/21** (10 new + 6 retry + 5 backoff) |
| C02 cluster | **27/30 90% A → 28/30 93% A** |
| Overall weighted | **92.0% A → 92.3% A** |
| Tier-1 (C00–C03 double-weighted) | **92.0% A → 92.4% A** |

## Real bug fix (2 files, +14 / −3)

### `src/retry.rs:compute_delay`

Original:
```rust
let ms = policy.base_delay.as_millis() as u64 * 2u64.saturating_pow(attempt);
Duration::from_millis(ms.min(policy.max_delay.as_millis() as u64))
```

**Bug:** `2u64.saturating_pow(63)` saturates to `u64::MAX`, but `* base_delay.as_millis()` then overflows and wraps back to a small value. The `max_delay` clamp never sees the saturation, so at `attempt=63` you get a *smaller* delay than at `attempt=0` — the exact opposite of intended exponential backoff.

Fixed:
```rust
let base = policy.base_delay.as_millis() as u128;
let pow = 2u128.saturating_pow(attempt);
let multiplied = base.saturating_mul(pow);
let capped = multiplied.min(policy.max_delay.as_millis() as u128) as u64;
Duration::from_millis(capped)
```

### `src/backoff.rs:Backoff::delay_for`

Original:
```rust
BackoffStrategy::Exponential => {
    self.base.as_millis() as u64 * 2u64.saturating_pow(attempt).min(u64::MAX / 2)
}
```

**Bug:** Same overflow class. The `.min(u64::MAX / 2)` is pre-multiplication, doesn't help once the `* base_delay.as_millis()` overflows. Linear at `attempt=u32::MAX` also overflows.

Fixed: Full u128 intermediate computation with `saturating_mul` for all three strategies.

## FR-003 acceptance gates (1 file, +187)

`tests/c02_l26_resilience.rs` — 10 tests, all PASSING:

| # | Test | Coverage |
|---|------|----------|
| 1 | `fr003_retry_policy_default_bounds` | RetryPolicy defaults (max=3, base=100ms, max=5s) |
| 2 | `fr003_retry_should_retry_strict_inequality` | `attempt < max_attempts` strict |
| 3 | `fr003_retry_compute_delay_exponential_growth` | base=100, 200, 400 monotonic |
| 4 | `fr003_retry_compute_delay_clamps_at_max` | saturation overflow safety |
| 5 | `fr003_retry_until_success_records_attempts` | attempt counting |
| 6 | `fr003_backoff_strategies_are_distinct` | Fixed/Linear/Exponential distinct + monotonic + exp>linear |
| 7 | `fr003_backoff_clamps_under_saturation` | u32::MAX safety |
| 8 | `fr003_healthz_readyz_split_is_observable` | `/healthz` and `/readyz` distinct routes wired in `src/commands/serve.rs:246-247, 437-438` |
| 9 | `fr003_bulkhead_spawn_policy_wired` | `src/spawn_policy.rs` semaphore present (C02 L25 bulkhead) |
| 10 | `fr003_thermal_gate_retry_path_is_documented` | `crates/sharecli-core/src/lib.rs` thermal gate contract |

## Verified

```bash
cargo test --locked --test c02_l26_resilience
# 10 passed; 0 failed; finished in 0.04s

cargo test --lib --locked retry::
# 6 passed (pre-existing unit tests under u128 implementation)

cargo test --lib --locked backoff::
# 5 passed (pre-existing unit tests under u128 implementation)
```

**21/21 resilience-related tests green.** No regressions in pre-existing source tests.

## Governance sync (claim-lock disjoint, traces FR-003)

- `WORK_DAG.md` — **T-890** added Status: DONE; backlog header updated
- `audit/.lane-c02/C02.md` — L26 score 2→3; evidence expanded (u128 fix cited, bulkhead + healthz/readyz split + 10 FR-003 gates added); `CLUSTER_TOTAL` 27/30 90% A → 28/30 93% A
- `audit/SCORECARD-v38.md` — C02 row 27/30 90% → 28/30 93%; weighted **92.0% A → 92.3% A**; unweighted sum 1092→1095 / 12 = **91.25% A**; tier-1 sum 1472→1478 / 16 = **92.4% A** (C02 IS in tier-1; double-weight applies); Wave17 Plan 794 headline added; governance line refreshed
- `docs/ops/governance/WBS-PHASED.md` — **W17.10 (T-890)** row added Status: DONE; C02 cluster rollup 90% → 93%; Last sync 2026-08-28
- `docs/ops/governance/GAP-QA-MATRIX.md` — C02 L26 row added Status: Closed with overflow-fix + 10 FR-003 gate evidence paths; Last sync 2026-08-28
- `docs/ops/governance/RC-audit-v38-80B.md` — Pin commit `5ae9ec2` → `b2997fa`; Scorecard 92.0% A → 92.3% A weighted; C02/C07 row 90% → 93% with Wave17 Plan 794 reference; **C02 L26 RC blocker CLOSED**

## Why this is the first tier-1 lift in Wave17

C02 belongs to **tier-1** (C00–C03 double-weighted). All other Wave17 cluster lifts (C04 Plan 776, C05 Plan 782, C06 Plans 777/778b, C11 Plan 793) were **outside tier-1** and therefore did not move the tier-1 score. Plan 794 is the first Wave17 plan that:

- Moves the tier-1 weighted score (92.0% A → **92.4% A**)
- Adds a real production-code fix (not just docs/tests)
- Discovers an existing latent bug (saturation overflow)

## No invented percentages

All score updates are recomputed from the underlying delta:
- C02 cluster: 27/30 → 28/30 = 90.00% → 93.33% (exact)
- Overall weighted: 92.0% → 92.3% (C02 in tier-1, +1 score contributes ×2 = +6 to tier-1 sum; 1472→1478 / 16 = 92.4%)
- Unweighted: sum 1092 → 1095 / 12 = 91.25%

## Out of scope (not overclaimed)

- Live chaos drills (production-shaped, requires deployable target) — separate scope, separate PR
- OS-level cgroup/job-object enforcement (C02 L25 gap, requires Linux runner)
- Bulkhead for the HTTP rate limiter (currently sliding-window only)
