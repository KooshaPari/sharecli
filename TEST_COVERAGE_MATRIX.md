# Test Coverage Matrix

**Project**: sharecli  
**Document Version**: 1.4
**Last Updated**: 2026-07-19 (T-625 numeric pin)

---

## Coverage Summary

| Metric | Value |
|--------|-------|
| Functional Requirements (Phase 3) | 5 (`FR-001`..`FR-005`) |
| FR acceptance test files on disk | 8 (`fr001_*`..`fr004_*`) + 1 CLI smoke |
| Integration / cast / coordination test files | 7 |
| Test functions in `tests/` | 72 (`#[test]` / `#[tokio::test]`) |
| Unit-ish tests in `src/` + `crates/` | ~1500+ (includes generated/large suites) |
| Coverage Target | 85% (see `.github/workflows/quality-gate.yml` `COVERAGE_THRESHOLD`) |
| Current Coverage | **83.48% lines** (broad workspace; see pin below) |

---

## Measured coverage pin

| Field | Evidence |
|-------|----------|
| Source revision | `d3cb7c4c34fab7a21478616e61869e03cd55a5ec` (`origin/main` post-#353) |
| Retained snapshot | `audit/coverage-snapshots/d3cb7c4.coverage-snapshot.json` |
| Measured line percentage | **83.48%** (29,143 lines; 24,328 covered) |
| Functions / regions | 85.05% functions · 85.48% regions (same snapshot) |
| Meets 85% unit gate? | **No** for broad workspace (`meets_lines_target: false`); PR hard gate uses scoped `--lib` ignores in `quality-gate.yml` |
| CI artifact parity | `coverage.yml` uploads `coverage-snapshot-${{ github.sha }}` for 30 days |

The compact snapshot records covered/count/percentage totals for lines, functions,
regions, and branches, plus the source SHA, Actions run ID, 85% target, and whether
the measured line percentage meets that target. A future documentation update may
pin a numeric percentage only from one of these retained snapshots.

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
| `coverage.yml` snapshot | **83.48% lines** (pinned evidence) | `--workspace --all-features` broad measurement |
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

Canonical AC ↔ function map: [`docs/specs/TRACEABILITY.md`](docs/specs/TRACEABILITY.md).  
Root FR stories: [`FUNCTIONAL_REQUIREMENTS.md`](FUNCTIONAL_REQUIREMENTS.md).

---

## Coverage Gaps

### Critical Gaps
1. Homebrew bottle sha still PLACEHOLDER (`WORK_DAG` Wave4 / C11).

---

## Recommendations

### Immediate Actions
1. Claim Wave4 packaging (brew sha after `v*` attach) or T-310 C03 polish.
2. Keep FR annotations (`//! FR: FR-NNN`) on every new acceptance test.
3. Sync status tokens in `docs/ops/governance/GAP-QA-MATRIX.md` + `WBS-PHASED.md`.

### Short-term Actions
1. Optionally close T-310 after lane evidence sweep.
2. Pin the next measured percentage from the retained `coverage-snapshot-<sha>` artifact.

---

**Last Updated**: 2026-07-18 (T-620)
