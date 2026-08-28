# Test Coverage Matrix

**Project**: sharecli  
**Document Version**: 1.8
**Last Updated**: 2026-08-27 (Wave17 T-810 lib pin @ `fa887e9` — **77.34%** lines / **79.79%** funcs / **80.14%** regions; prior workspace pin **80.51%** @ `5d8dc08` retained as historical evidence)

---

## Coverage Summary

| Metric | Value |
|--------|-------|
| Functional Requirements (Phase 3) | 5 (`FR-001`..`FR-005`) |
| FR acceptance test files on disk | 8 (`fr001_*`..`fr004_*`) + 1 CLI smoke |
| Integration / cast / coordination test files | 7 |
| Test functions in `tests/` | 72 (`#[test]` / `#[tokio::test]`) + #399 lift suites + T-810 6 tests @ `e298e0f` |
| Unit-ish tests in `src/` + `crates/` | ~1500+ (includes generated/large suites) |
| Coverage Target | 85% (see `.github/workflows/quality-gate.yml` `COVERAGE_THRESHOLD`) |
| Current Coverage (--lib, Wave17 T-810) | **77.34% lines** (`fa887e9`; local llvm-cov run `local-lib-20260827`) |
| Retained Workspace Pin | **80.51% lines** (`5d8dc08` / run 29985746034; broad workspace pre-T-810 lifts) |

---

## Measured coverage pin

| Field | Evidence |
|-------|----------|
| Source revision | `fa887e9d8c89dbc6764dad0d06082dcb139eeaad` (Wave17 T-810 lib pin — fast-forwarded over #771 + #773 + #774) |
| Scope | `--lib --all-features --locked --ignore-run-fail` (cargo llvm-cov 0.9.0; local run `local-lib-20260827` 2026-08-27) |
| Retained snapshot | `audit/coverage-snapshots/fa887e9.coverage-snapshot.json` |
| Measured line percentage | **77.34%** (27,947 lines; 21,615 covered) |
| Functions / regions | 79.79% functions (3,439 / 2,744); 80.14% regions (49,622 / 39,765) |
| Meets 85% unit gate? | **No** (`meets_lines_target: false`); PR hard gate uses `--lib` scope per `quality-gate.yml` |
| Scope note | `--workspace --all-features broad` measurement blocked on Windows for this pin cycle by integration-test compatibility regressions in `tests/fr008_coalesce_mesh` (operator-env critical-timeout hang) and `tests/fr009_fuse_*` (FUSE-specific Linux/macOS code paths). The `--lib` scope is what `quality-gate.yml` hard-gates; the `--workspace` measurement is supplementary. The prior workspace pin at `5d8dc08` / **80.51%** remains the most recent broad measurement and is retained as historical evidence. |

### Prior pin (superseded for current cycle)

| Field | Evidence |
|-------|----------|
| Prior revision | `5d8dc08928c7258110f8a20c7e0fafd9f474f22e` (Wave16 T-730 pin refresh) |
| Prior scope | `--workspace --all-features --locked --ignore-run-fail` (broad workspace) |
| Prior snapshot | `audit/coverage-snapshots/5d8dc08.coverage-snapshot.json` (retained; **80.51%** lines @ `5d8dc08`) |
| Prior line percentage | **80.51%** (40,077 lines; 32,266 covered) |
| Functions / regions | 83.23% functions; 82.21% regions (same `5d8dc08` snapshot) |
| Note | Superseded for the Wave17 T-810 lib pin **77.34%** @ `fa887e9`; **80.51%** snapshot retained as historical evidence. Wave17 T-810 #771 added 6 tests for session + coordination coverage but the broad-workspace remeasure was blocked on Windows. |

### Wave15 / #399 → remeasure → #583 climb (2026-07-22..23)

| Field | Evidence |
|-------|----------|
| Coverage-lift merge | `922b4ae` — [#399](https://github.com/KooshaPari/sharecli/pull/399) |
| Empty-suite stall | Runs [29872308604](https://github.com/KooshaPari/sharecli/actions/runs/29872308604) / [29967745465](https://github.com/KooshaPari/sharecli/actions/runs/29967745465) failed guard before llvm-cov |
| Remeasure | [#580](https://github.com/KooshaPari/sharecli/pull/580) fixed `CARGO_TERM_COLOR=never` / empty-suite false positive; run [29985746034](https://github.com/KooshaPari/sharecli/actions/runs/29985746034) produced **80.51%** @ `5d8dc08` |
| Coverage climb | [#583](https://github.com/KooshaPari/sharecli/pull/583) @ `8c68bb5`; run [30005505196](https://github.com/KooshaPari/sharecli/actions/runs/30005505196) measured **81.17%** lines |
| Honest action | Pin **81.22%** at `28bfb10` from CI artifact — do **not** invent a higher percentage |

The compact snapshot records covered/count/percentage totals for lines, functions,
regions, and branches, plus the source SHA, Actions run ID, 85% target, and whether
the measured line percentage meets that target. Numeric percentages are pinned only
from retained llvm-cov snapshots.

### 85% enforcement

The 85% line target is a hard PR gate in
`.github/workflows/quality-gate.yml`: `COVERAGE_THRESHOLD=85` is passed to
`cargo llvm-cov --fail-under-lines` for the documented unit-test scope. A result
below 85% makes the `unit-tests` job fail. The lib pin (`fa887e9` 77.34%) and the
retained workspace pin (`5d8dc08` 80.51%) are both below 85%; the
`meets_lines_target: false` flag in each snapshot makes target drift machine-readable.

### Codecov supplementary policy

`codecov.yml` remains **supplementary** to the hard gates:

| Layer | Target | Scope |
|-------|--------|-------|
| `quality-gate.yml` | **85% lines** (hard) | `--lib` unit scope with documented filename ignores |
| `coverage.yml` snapshot (current) | **77.34% lines** @ `fa887e9` (lib pin) | `--lib --all-features --locked --ignore-run-fail` |
| `coverage.yml` snapshot (retained) | **80.51% lines** @ `5d8dc08` | `--workspace --all-features` broad measurement (historical) |
| Codecov `project` | 70% + 1% threshold | `src/**/*.rs` upload from quality-gate LCOV |
| Codecov `patch` | 80% + 1% threshold | PR diff only |

The pinned lib percentage does not lower the 85% unit gate; it documents the
current state of `--lib` measurement for prioritization. The retained 80.51%
workspace pin documents the most recent broad measurement pre-T-810 lifts.

---

## Test Categories

### Unit Tests
- **Location**: `src/**`, `crates/**` (`#[cfg(test)]` modules)
- **Purpose**: Test individual components in isolation
- **Coverage Target**: 90%

### Integration / Acceptance Tests
- **Location**: `tests/`
- **Purpose**: FR acceptance, CLI binary e2e, cast adapters, coordination
- **Coverage Target**: 75% of public CLI surface

### How to run
- `just test` — full locked suite
- `just test-nextest` — parallel nextest CI profile
- `just coverage` — llvm-cov (when tools installed via `just install-tools`)

---

## FR to Test Coverage Mapping

| FR ID | Description | Test Files | Coverage Status |
|-------|-------------|------------|-----------------|
| FR-001 | Managed Process Lifecycle | `tests/fr001_process_lifecycle.rs` (4), `tests/fr001_stop_filter.rs` (2), `tests/integration_cli.rs` (9, smoke) | **Covered** |
| FR-002 | TOML Configuration Management | `tests/fr002_config_init.rs` (2), `tests/fr002_config_load.rs` (3) | **Covered** (T-200 DONE) |
| FR-003 | Project Registry | `tests/fr003_project_registry.rs` (4), `tests/fr003_project_discover.rs` (1) | **Covered** (T-210 DONE) |
| FR-004 | Process & Pool Health Status | `tests/fr004_status_health.rs` (3), `tests/fr004_pool_status.rs` (2) | **Covered** (T-220 DONE) |
| FR-005 | Per-Project Resource Limits | `tests/fr005_project_limits.rs`, `tests/fr005_resource_check.rs` | **Covered** (T-230) |
| FR-CAST-001 | Pane Address Schema | `tests/cast_address.rs` (10) | **Covered** (extension) |
| FR-CAST-002 | Pane Registry | `tests/cast_registry.rs` (7) | **Covered** (extension) |
| FR-CAST-003 | Ghostty cast | `tests/cast_ghostty.rs` (7) | **Covered** (extension) |
| FR-CAST-004 | WezTerm cast | `tests/cast_wezterm.rs` (9) | **Covered** (extension) |
| FR-CAST-005 | Windows Terminal cast | `tests/cast_winterm.rs` (6) | **Covered** (extension) |
| — | Coordination helpers | `tests/coordination.rs` (3) | Supporting |
| — | #399 + T-810 coverage lift suites | `tests/c01_coverage_lift.rs` + `tests/session_cov.rs` + `tests/coordination_cov.rs` (T-810 6 tests @ `e298e0f`) | **Landed**; lib pin **77.34%** (Wave17 T-810 @ `fa887e9`); workspace pin **80.51%** retained (Wave16 T-730 @ `5d8dc08`) |

Canonical AC ↔ function map: [`docs/specs/TRACEABILITY.md`](docs/specs/TRACEABILITY.md).  
Root FR stories: [`FUNCTIONAL_REQUIREMENTS.md`](FUNCTIONAL_REQUIREMENTS.md).

---

## Coverage Gaps

### Critical Gaps
1. Homebrew bottle sha still PLACEHOLDER (`WORK_DAG` Wave4 / C11).
2. `--lib` line coverage **77.34%** is below the 85% unit gate target (gap for prioritization; unit gate unchanged).
3. `--workspace --all-features broad` remeasurement blocked on Windows by integration-test compatibility regressions (operator-env critical-timeout hang + FUSE path regressions); most recent broad pin **80.51%** @ `5d8dc08` retained.

---

## Recommendations

### Immediate Actions
1. Restore `--workspace --all-features broad` remeasurement on Linux/WSL runner (operator-env critical-timeout fix in `tests/fr008_coalesce_mesh` + FUSE cfg gate consistency in `tests/fr009_*`).
2. Keep FR annotations (`//! FR: FR-NNN`) on every new acceptance test.
3. Sync status tokens in `docs/ops/governance/GAP-QA-MATRIX.md` + `WBS-PHASED.md`.

### Short-term Actions
1. Claim Wave4 packaging (brew sha after `v*` attach) or residual C10 hex drift (T-692).

---

**Last Updated**: 2026-08-27 (Wave17 T-810 lib pin @ `fa887e9` — **77.34%**; prior workspace pin **80.51%** @ `5d8dc08` retained)
