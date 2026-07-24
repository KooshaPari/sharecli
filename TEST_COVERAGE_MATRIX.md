# Test Coverage Matrix

**Project**: sharecli  
**Document Version**: 1.7
**Last Updated**: 2026-07-24 (post-#606 coverage re-measure — 81.22%)

---

## Coverage Summary

| Metric | Value |
|--------|-------|
| Functional Requirements (Phase 3) | 5 (`FR-001`..`FR-005`) |
| FR acceptance test files on disk | 8 (`fr001_*`..`fr004_*`) + 1 CLI smoke |
| Integration / cast / coordination test files | 7 |
| Test functions in `tests/` | 72 (`#[test]` / `#[tokio::test]`) + #399 lift suites |
| Unit-ish tests in `src/` + `crates/` | ~1500+ (includes generated/large suites) |
| Coverage Target | 85% (see `.github/workflows/quality-gate.yml` `COVERAGE_THRESHOLD`) |
| Current Coverage | **81.22% lines** (broad workspace; see pin below) |

---

## Measured coverage pin

| Field | Evidence |
|-------|----------|
| Source revision | `28bfb101ecf4523131cd1dfb71950b46189b9e65` (post-#606 fuser/climb-2 re-measure) |
| Retained snapshot | CI run [30083201303](https://github.com/KooshaPari/sharecli/actions/runs/30083201303) |
| Measured line percentage | **81.22%** (40,392 lines; 32,806 covered) |
| Functions / regions | 84.17% functions · 82.75% regions (same snapshot) |
| Meets 85% unit gate? | **No** for broad workspace (`meets_lines_target: false`); PR hard gate uses scoped `--lib` ignores in `quality-gate.yml` |
| CI artifact parity | Coverage run [30083201303](https://github.com/KooshaPari/sharecli/actions/runs/30083201303) uploaded `coverage-snapshot-28bfb101ecf4523131cd1dfb71950b46189b9e65` |

### Prior pin (superseded)

| Field | Evidence |
|-------|----------|
| Prior revision | `5d8dc08928c7258110f8a20c7e0fafd9f474f22e` (post-remeasure / #580) |
| Prior snapshot | `audit/coverage-snapshots/5d8dc08.coverage-snapshot.json` (retained) |
| Prior line percentage | **80.51%** (40,077 lines; 32,266 covered) |
| Note | Superseded by post-#606 re-measure pin **81.22%** @ `28bfb10` — prior snapshot retained |

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
below 85% makes the `unit-tests` job fail. The broader workspace snapshot in
`coverage.yml` is evidence/reporting and does not replace that hard gate; its
`meets_lines_target` field makes target drift machine-readable.

### Codecov supplementary policy

`codecov.yml` remains **supplementary** to the hard gates:

| Layer | Target | Scope |
|-------|--------|-------|
| `quality-gate.yml` | **85% lines** (hard) | `--lib` unit scope with documented filename ignores |
| `coverage.yml` snapshot | **81.22% lines** (pinned evidence) | `--workspace --all-features` broad measurement |
| Codecov `project` | 70% + 1% threshold | `src/**/*.rs` upload from quality-gate LCOV |
| Codecov `patch` | 80% + 1% threshold | PR diff only |

The pinned broad-workspace percentage does not lower the 85% unit gate; it documents
fleet-visible coverage debt for prioritization.

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
| — | #399 coverage lift suites | `tests/c01_coverage_lift.rs` (+ related FR-003 surfaces) | **Landed**; broad % pinned at **81.22%** (post-#606) |

Canonical AC ↔ function map: [`docs/specs/TRACEABILITY.md`](docs/specs/TRACEABILITY.md).  
Root FR stories: [`FUNCTIONAL_REQUIREMENTS.md`](FUNCTIONAL_REQUIREMENTS.md).

---

## Coverage Gaps

### Critical Gaps
1. Homebrew bottle sha still PLACEHOLDER (`WORK_DAG` Wave4 / C11).
2. Broad-workspace line coverage **81.22%** is below the 85% unit gate target (gap for prioritization; unit gate unchanged).

---

## Recommendations

### Immediate Actions
1. Raise broad-workspace llvm-cov toward 85% without inventing pin percentages.
2. Keep FR annotations (`//! FR: FR-NNN`) on every new acceptance test.
3. Sync status tokens in `docs/ops/governance/GAP-QA-MATRIX.md` + `WBS-PHASED.md`.

### Short-term Actions
1. Claim Wave4 packaging (brew sha after `v*` attach) or residual C10 hex drift (T-692).

---

**Last Updated**: 2026-07-24 (T-692 re-measure — 81.22% @ `28bfb10`)
