# Test Coverage Matrix

**Project**: sharecli  
**Document Version**: 1.2  
**Last Updated**: 2026-07-12 (T-220)

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
| Current Coverage | Measured in CI (`coverage.yml` / llvm-cov); not pinned in-repo — run `just coverage` |

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
1. No outside-in `*_journey_*` test mapping user journeys → FR IDs yet (see `WORK_DAG.md` T-240).
2. Unhappy-path friction suites still open (`WORK_DAG.md` T-300).

### Partial Coverage
1. **FR-001** library acceptance is strong; CLI binary coverage is smoke-level (`integration_cli.rs`), not full AC-001.* via the binary.
2. Cast FRs are tested but not yet promoted into `docs/specs/FR.md` as FR-006+.

---

## Recommendations

### Immediate Actions
1. Claim `T-240` for outside-in journey coverage.
2. Keep FR annotations (`//! FR: FR-NNN`) on every new acceptance test.
3. Sync status tokens in `docs/ops/governance/GAP-QA-MATRIX.md` + `WBS-PHASED.md`.

### Short-term Actions
1. Add journey + unhappy-path tests (`T-240`, `T-300`).
2. Publish a measured coverage % from `just coverage` into this summary after CI green.

---

**Last Updated**: 2026-07-13
