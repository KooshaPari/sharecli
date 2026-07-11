# Test Coverage Matrix

**Project**: sharecli  
**Document Version**: 1.1  
**Last Updated**: 2026-07-10

---

## Coverage Summary

| Metric | Value |
|--------|-------|
| Functional Requirements (Phase 3) | 5 (`FR-001`..`FR-005`) |
| FR acceptance test files on disk | 2 (`fr001_*`) + 1 CLI smoke |
| Integration / cast / coordination test files | 7 |
| Test functions in `tests/` | 57 (`#[test]` / `#[tokio::test]`) |
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
| FR-002 | TOML Configuration Management | Target: `tests/fr002_config_load.rs`, `tests/fr002_config_init.rs` | **Gap** — listed in TRACEABILITY; files not in tree (claim T-200) |
| FR-003 | Project Registry | Target: `tests/fr003_project_registry.rs`, `tests/fr003_project_discover.rs` | **Gap** (claim T-210) |
| FR-004 | Process & Pool Health Status | Target: `tests/fr004_status_health.rs`, `tests/fr004_pool_status.rs` | **Gap** (claim T-220) |
| FR-005 | Per-Project Resource Limits | Target: `tests/fr005_project_limits.rs`, `tests/fr005_resource_check.rs` | **Gap** (claim T-230) |
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
1. **FR-002..FR-005** acceptance files referenced by TRACEABILITY are missing on disk — highest agent-readiness risk for autonomous FR closure.
2. No outside-in `*_journey_*` test mapping user journeys → FR IDs yet (see `WORK_DAG.md` T-240).

### Partial Coverage
1. **FR-001** library acceptance is strong; CLI binary coverage is smoke-level (`integration_cli.rs`), not full AC-001.* via the binary.
2. Cast FRs are tested but not yet promoted into `docs/specs/FR.md` as FR-006+.

---

## Recommendations

### Immediate Actions
1. Claim `T-200..T-230` in `WORK_DAG.md` to land FR-002..005 acceptance suites.
2. Keep FR annotations (`//! FR: FR-NNN`) on every new acceptance test.

### Short-term Actions
1. Add journey + unhappy-path tests (`T-240`, `T-300`).
2. Publish a measured coverage % from `just coverage` into this summary after CI green.

---

**Last Updated**: 2026-07-10
