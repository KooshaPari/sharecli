# Traceability Index — sharecli

> Authoritative mapping between Functional Requirements, source code, and
> acceptance tests. Updated by the spec+test+traceability workflow.

**How to use this file:**

1. Every FR in `docs/specs/FR.md` has a row here.
2. The `Source` column lists the canonical Rust module(s) that implement
   the requirement.
3. The `Tests` column lists every acceptance test file that covers the
   requirement. Each test carries an `// FR-XXX` comment in its source.
4. The `Status` column tracks whether the FR has at least one passing
   acceptance test.

**Phase:** 3+ runtime thesis (FR-006..011) · operator AuthN (FR-012)
**Last updated:** 2026-07-19 (thesis re-enum from origin export)

---

## FR ↔ Source ↔ Tests Matrix

| FR ID   | Title                                | Source                                                                                       | Tests                                                                                              | Status |
|---------|--------------------------------------|----------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------|--------|
| FR-001  | Managed Process Lifecycle            | `src/main.rs`, `src/commands/mod.rs`, `src/runtime.rs`                                      | `tests/fr001_process_lifecycle.rs`, `tests/fr001_stop_filter.rs`                                  | ACCEPTED |
| FR-002  | TOML Configuration Management        | `src/config.rs`, `src/commands/mod.rs`                                                       | `tests/fr002_config_load.rs`, `tests/fr002_config_init.rs`                                         | ACCEPTED |
| FR-003  | Project Registry                     | `src/config.rs`, `src/commands/mod.rs`                                                       | `tests/fr003_project_registry.rs`, `tests/fr003_project_discover.rs`                              | ACCEPTED |
| FR-004  | Process & Pool Health Status         | `src/runtime.rs`, `src/monitoring.rs`, `src/commands/mod.rs`                                 | `tests/fr004_status_health.rs`, `tests/fr004_pool_status.rs`                                      | ACCEPTED |
| FR-005  | Per-Project Resource Limits          | `src/runtime.rs`, `src/commands/mod.rs`                                                      | `tests/fr005_project_limits.rs`, `tests/fr005_resource_check.rs`                                  | ACCEPTED |
| FR-006  | Agent Detection (proc scan)          | `crates/sharecli-core/src/detect.rs`, `proc_scan.rs`                                         | `tests/fr006_agent_detection.rs`, `tests/fr006_proc_tree.rs`                                      | ACCEPTED |
| FR-007  | Resource & Syscall-Relevant Watch    | `sharecli-core`, `sharecli-fleet`, `src/monitoring.rs`                                       | `tests/fr007_resource_thermal_watch.rs`                                                           | ACCEPTED |
| FR-008  | Speculative Coalesce / Debounce / Queue | `sharecli-ipc` (cache+queue+nocache), `sharecli-core` Hypervisor                          | `tests/fr008_coalesce_mesh.rs`                                                                    | ACCEPTED |
| FR-009  | FUSE IO Intercept                    | `crates/sharecli-fuse`                                                                       | `tests/fr009_fuse_intercept.rs`                                                                   | ACCEPTED |
| FR-010  | Agent Mesh / Shared Substrate        | `sharecli-fleet` registry, `crates/sharecli-mesh` MaildirQueue                               | `tests/fr010_mesh_substrate.rs`                                                                   | ACCEPTED |
| FR-011  | Thermal Contention Gate              | `sharecli-fleet` thermal, `sharecli-core` FakeThermalGate                                    | `tests/fr011_thermal_gate.rs`, `tests/fr008_coalesce_mesh.rs`                                     | ACCEPTED |
| FR-012  | Serve HTTP Federated AuthN           | `src/serve_auth.rs`, `src/config.rs` (`ServeJwtConfig`), `src/commands/serve.rs`            | `tests/fr012_serve_jwt_auth.rs`                                                                   | ACCEPTED |

**Coverage summary:** 12 FRs mapped (FR-001..012), acceptance tests present, 0 gaps.

---

## Per-FR Detail

### FR-001 — Managed Process Lifecycle

| AC        | Test file                              | Test function                          |
|-----------|----------------------------------------|----------------------------------------|
| AC-001.1  | `tests/fr001_process_lifecycle.rs`     | `fr001_start_records_pid_in_pool`      |
| AC-001.2  | `tests/fr001_process_lifecycle.rs`     | `fr001_ps_table_columns_present`       |
| AC-001.3  | `tests/fr001_process_lifecycle.rs`     | `fr001_ps_filter_by_project`           |
| AC-001.4  | `tests/fr001_stop_filter.rs`           | `fr001_stop_all_terminates_everything` |
| AC-001.5  | `tests/fr001_stop_filter.rs`           | `fr001_stop_without_selector_errors`   |

### FR-002 — TOML Configuration Management

| AC        | Test file                          | Test function                            |
|-----------|------------------------------------|------------------------------------------|
| AC-002.1  | `tests/fr002_config_init.rs`       | `fr002_init_creates_default_toml`        |
| AC-002.2  | `tests/fr002_config_init.rs`       | `fr002_validate_reports_project_count`   |
| AC-002.3  | `tests/fr002_config_load.rs`       | `fr002_show_prints_projects_and_runtime` |
| AC-002.4  | `tests/fr002_config_load.rs`       | `fr002_load_roundtrips_projects_map`     |
| AC-002.5  | `tests/fr002_config_load.rs`       | `fr002_runtime_config_default_values`    |

### FR-003 — Project Registry

| AC        | Test file                            | Test function                              |
|-----------|--------------------------------------|--------------------------------------------|
| AC-003.1  | `tests/fr003_project_registry.rs`    | `fr003_project_add_inserts_and_persists`   |
| AC-003.2  | `tests/fr003_project_registry.rs`    | `fr003_project_list_prints_registered`     |
| AC-003.3  | `tests/fr003_project_registry.rs`    | `fr003_project_show_resolves_path`         |
| AC-003.4  | `tests/fr003_project_discover.rs`    | `fr003_project_discover_finds_git_repos`   |
| AC-003.5  | `tests/fr003_project_registry.rs`    | `fr003_project_remove_drops_entry`         |

### FR-004 — Process & Pool Health Status

| AC        | Test file                          | Test function                            |
|-----------|------------------------------------|------------------------------------------|
| AC-004.1  | `tests/fr004_status_health.rs`     | `fr004_status_prints_harness_table`      |
| AC-004.2  | `tests/fr004_pool_status.rs`       | `fr004_pool_reports_node_and_bun`        |
| AC-004.3  | `tests/fr004_pool_status.rs`       | `fr004_health_reports_healthy_or_degraded` |
| AC-004.4  | `tests/fr004_status_health.rs`     | `fr004_health_status_marks_unhealthy`    |
| AC-004.5  | `tests/fr004_status_health.rs`     | `fr004_process_stats_idle_threshold`     |

### FR-005 — Per-Project Resource Limits

| AC        | Test file                          | Test function                            |
|-----------|------------------------------------|------------------------------------------|
| AC-005.1  | `tests/fr005_project_limits.rs`    | `fr005_project_limits_default_values`    |
| AC-005.2  | `tests/fr005_project_limits.rs`    | `fr005_limits_set_persists_values`       |
| AC-005.3  | `tests/fr005_project_limits.rs`    | `fr005_get_limits_returns_default_for_unknown` |
| AC-005.4  | `tests/fr005_resource_check.rs`    | `fr005_resource_check_overall_ok_logic`  |
| AC-005.5  | `tests/fr005_resource_check.rs`    | `fr005_check_prints_status_lines`        |

### FR-006 — Agent Detection

| AC        | Test file                          | Test function                            |
|-----------|------------------------------------|------------------------------------------|
| AC-006.1..3 | `tests/fr006_agent_detection.rs` | (see file)                               |
| AC-006.4..6 | `tests/fr006_proc_tree.rs`       | (see file)                               |

### FR-007 — Resource Watch

| AC        | Test file                                | Notes |
|-----------|------------------------------------------|-------|
| AC-007.1..3 | `tests/fr007_resource_thermal_watch.rs` | thermal as watch signal |

### FR-008 — Coalesce

| AC        | Test file                        | Notes |
|-----------|----------------------------------|-------|
| AC-008.1..4 | `tests/fr008_coalesce_mesh.rs` | command_key / with_lock / thermal / cache hit |
| AC-008.5 | `tests/fr008_coalesce_mesh.rs` (`fr008_ttl_stale_entry_is_miss`); `sharecli-ipc` `ttl_lookup_miss_and_evict_on_store` | TTL miss + eviction |
| AC-008.6 | `tests/fr008_coalesce_mesh.rs` (`fr008_debounce_waits_and_shares`); `sharecli-ipc` `debounce_shares_recent_store` | debounce share window |
| queue / nocache | — | still TBD |

### FR-009 — FUSE

| AC        | Test file                        | Notes |
|-----------|----------------------------------|-------|
| AC-009.1..2 | `tests/fr009_fuse_intercept.rs` | construct + mount API; no privileged mount |
| AC-009.3 | `tests/fr009_fuse_intercept.rs`; `inode_map` unit tests | inode map / path resolution |
| AC-009.4 | `tests/fr009_fuse_intercept.rs`; `read_cache` unit tests | read coalesce hit/miss meters |
| AC-009.5 | `tests/fr009_fuse_intercept.rs`; `write_serialize` unit tests | path lock + CoW stubs; write no ENOSYS |

### FR-010 — Mesh

| AC        | Test file                        | Notes |
|-----------|----------------------------------|-------|
| AC-010.1..3 | `tests/fr010_mesh_substrate.rs` | registry primitives |

### FR-011 — Thermal Gate

| AC        | Test file                     | Notes |
|-----------|-------------------------------|-------|
| AC-011.1..3 | `tests/fr011_thermal_gate.rs` | also AC-008.3 |

### FR-012 — Serve JWT AuthN

| AC        | Test file                        | Test function |
|-----------|----------------------------------|---------------|
| AC-012.1..3 | `tests/fr012_serve_jwt_auth.rs` | (see file) |

---

## Change log

- **2026-07-19 — FR-009 A+ recovery:** InterceptFs passthrough + inode map +
  read coalesce meters + write-serialize scaffold (AC-009.3..5). CoW
  commit/discard remain TODO stubs.
- **2026-07-19 — thesis re-enum:** Published FR-006..FR-011 from origin export
  (paste + agent-mesh WBS + harness). Traceability matrix expanded to 12 FRs.
- **2026-07-12 — T-220:** Landed `tests/fr004_status_health.rs` + `tests/fr004_pool_status.rs` (AC-004.1..004.5).
- **2026-07-12 — T-210:** Landed `tests/fr003_project_registry.rs` + `tests/fr003_project_discover.rs` (AC-003.1..003.5). Also T-260 claim-lock + T-270 loop budgets.
- **2026-07-12 — T-200:** Landed `tests/fr002_config_init.rs` + `tests/fr002_config_load.rs` (AC-002.1..002.5). FR-002 Status remains ACCEPTED with files on disk. Governance: `docs/ops/governance/{WBS-PHASED,GAP-QA-MATRIX}.md`.
- **2026-06-15 — Phase 3 initial:** 5 FRs published; 10 acceptance test files added; full FR→source→test matrix established.
