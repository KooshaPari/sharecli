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
| FR-006  | Agent Detection (proc scan)          | `crates/sharecli-fleet/src/proc_scan.rs`, `resource_watch.rs`                                | `tests/fr006_agent_detection.rs`, `tests/fr006_proc_tree.rs`, `tests/fr006_agent_pid_watch.rs`   | ACCEPTED |
| FR-007  | Resource & Syscall-Relevant Watch    | `sharecli-core`, `sharecli-fleet`, `src/monitoring.rs`                                       | `tests/fr007_resource_thermal_watch.rs`                                                           | ACCEPTED |
| FR-008  | Speculative Coalesce / Debounce / Queue | `sharecli-ipc` (cache+queue+nocache), `sharecli-core` Hypervisor                          | `tests/fr008_coalesce_mesh.rs`                                                                    | ACCEPTED |
| FR-009  | FUSE IO Intercept                    | `crates/sharecli-fuse`                                                                       | `tests/fr009_fuse_intercept.rs`                                                                   | ACCEPTED |
| FR-010  | Agent Mesh / Shared Substrate        | `sharecli-fleet` registry, `crates/sharecli-mesh` MaildirQueue / SmartMerger / WorktreePool, `sharecli mesh` CLI | `tests/fr010_mesh_substrate.rs` · `tests/fr010_mesh_cli.rs` | ACCEPTED |
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
| AC-006.7..8 | `tests/fr006_ps_agent_column.rs` | ps AGENT column + `--all` inventory      |
| AC-006.9    | `tests/fr006_thermal_tui_agents.rs`; `crates/sharecli-thermal-tui` | thermal TUI agent panel |
| AC-006.10   | `tests/fr006_agent_pid_watch.rs` | per-agent RSS/FD watch rows              |
| AC-006.11..13 | `tests/fr006_proc_cli.rs`; `src/commands/proc.rs` | `sharecli proc` + JSON status/proc |
| AC-006.14   | `tests/fr006_proc_fingerprints.rs`; `crates/sharecli-fleet/src/detect.rs` | cmdline fingerprints |
| AC-006.15   | `tests/fr006_proc_watch.rs`; `src/commands/proc.rs` | `sharecli proc --watch` live refresh |
| AC-006.16   | `tests/fr006_proc_tree_cli.rs`; `crates/sharecli-fleet/src/proc_scan.rs` (`build_agent_forests`); `src/commands/proc.rs` | `sharecli proc --tree` parent-child forests |
| AC-006.17   | `tests/fr006_proc_filters.rs`; `src/commands/proc.rs`; `crates/sharecli-fleet/src/resource_watch.rs` (`parse_rss_bytes`) | `sharecli proc --family` / `--min-rss` filters |
| AC-006.18   | `tests/fr006_proc_ndjson.rs`; `tests/fr006_proc_watch.rs`; `src/commands/proc.rs` | `sharecli proc --watch --json` NDJSON stream |
| AC-006.19   | `tests/fr006_proc_sort.rs`; `src/commands/proc.rs` | `sharecli proc --sort rss|fd|pid|state` |
| AC-006.20   | `tests/fr006_proc_fingerprints_ext.rs`; `tests/fr006_agent_detection.rs`; `crates/sharecli-fleet/src/detect.rs` | amp + expanded cmdline fingerprints |
| AC-006.21   | `tests/fr006_proc_limit.rs`; `src/commands/proc.rs` | `sharecli proc --limit N` caps inventory/tree roots |
| AC-006.22   | `tests/fr006_thermal_tui_agent_tree.rs`; `crates/sharecli-thermal-tui/src/lib.rs` | thermal TUI agent forests via `build_host_agent_forests` |
| AC-006.23   | `tests/fr006_proc_pid_detail.rs`; `src/commands/proc.rs`; `crates/sharecli-fleet/src/proc_scan.rs` (`lookup_proc`) | `sharecli proc --pid N` detail view |
| AC-006.24   | `tests/fr006_proc_csv.rs`; `src/commands/proc.rs` | `sharecli proc --csv` flat inventory export |
| AC-006.25   | `tests/fr006_proc_ppid.rs`; `src/commands/proc.rs`; `crates/sharecli-fleet/src/proc_scan.rs` (`lookup_proc`) | `sharecli proc --ppid N` parent-PID inventory filter |
| AC-006.26   | `tests/fr006_proc_tree_csv.rs`; `src/commands/proc.rs` | `sharecli proc --tree --csv` forest inventory CSV export |
| AC-006.27   | `tests/fr006_proc_filters.rs`; `src/commands/proc.rs`; `crates/sharecli-fleet/src/resource_watch.rs` (`parse_rss_bytes`) | `sharecli proc --max-rss` upper RSS bound filter |
| AC-006.28   | `tests/fr006_proc_filters.rs`; `src/commands/proc.rs` | `sharecli proc --min-fd` / `--max-fd` FD band filters |
| AC-006.29   | `tests/fr006_proc_comm.rs`; `src/commands/proc.rs` | `sharecli proc --comm` COMM substring filter |
| AC-006.30   | `tests/fr006_proc_cmdline.rs`; `src/commands/proc.rs` | `sharecli proc --cmdline` joined argv/cmdline substring filter |
| AC-006.31   | `tests/fr006_proc_state.rs`; `src/commands/proc.rs`; `crates/sharecli-fleet/src/proc_scan.rs` | `sharecli proc --state` process-state letter filter |
| AC-006.32   | `tests/fr006_proc_state_export.rs`; `src/commands/proc.rs` | `state` on flat `--json` / `--csv` and `--tree --csv` exports |
| AC-006.33   | `tests/fr006_proc_state_text.rs`; `tests/fr006_proc_pid_detail.rs`; `src/commands/proc.rs`; `src/commands/mod.rs` | `state` on flat text inventory and `proc --pid` detail |
| AC-006.34   | `tests/fr006_proc_tree_state.rs`; `src/commands/proc.rs` | `state` on `--tree` text nodes and `--tree --json` rows |
| AC-006.35   | `tests/fr006_proc_tree_state.rs`; `src/commands/proc.rs` (`collect_forest_pids`, `build_forest_state_map`) | live tree state for all forest PIDs (roots + children) |
| AC-006.36   | `tests/fr006_proc_sort.rs`; `src/commands/proc.rs` (`ProcSort::State`, `sort_watched_agents`, `sort_agent_forests`) | `sharecli proc --sort state` process-state letter ordering |
| AC-006.37   | `tests/fr006_proc_ndjson.rs`; `src/commands/proc.rs` (`emit_ndjson_line`, `AgentProcNdjsonLine`, `agent_row_from_watch`) | NDJSON watch agent rows include `state` (AC-006.32 parity); flushed stdout + stderr footer |
| AC-006.38   | `tests/fr006_proc_filters.rs`; `src/commands/proc.rs` | `sharecli proc --exclude-family` negates `--family` |
| AC-006.39   | `tests/fr006_thermal_tui_agent_tree.rs`; `crates/sharecli-thermal-tui/src/lib.rs`; `crates/sharecli-fleet/src/proc_scan.rs` (`build_host_forest_state_map`, `state_text_for_pid`) | thermal TUI agent-tree process state letters |
| AC-006.40   | `tests/fr006_thermal_tui_agents.rs`; `crates/sharecli-thermal-tui/src/lib.rs`; `crates/sharecli-fleet/src/proc_scan.rs` (`build_host_agent_state_map`, `state_text_for_pid`) | thermal TUI flat agent-lines process state letters |
| AC-006.12   | `tests/fr006_agent_rss_gate.rs`; `crates/sharecli-fleet/src/agent_contention.rs` | RSS-aware gate |

### FR-007 — Resource Watch

| AC        | Test file                                | Notes |
|-----------|------------------------------------------|-------|
| AC-007.1..3 | `tests/fr007_resource_thermal_watch.rs` | thermal as watch signal |
| AC-007.4    | `tests/fr007_resource_thermal_watch.rs` (`fr007_fd_watch_samples_self_fds`) | FD watch |
| AC-007.5    | `tests/fr007_resource_thermal_watch.rs` (`fr007_net_watch_samples_host_counters`) | net RX/TX watch |
| AC-007.6    | `tests/fr007_resource_thermal_watch.rs` (`fr007_hypervisor_run_carries_resource_watch`); `crates/sharecli-fleet/src/resource_watch.rs` | Hypervisor live watch path |
| AC-007.7    | `tests/fr007_resource_thermal_watch.rs` (`fr007_rss_watch_samples_self_rss`) | RSS watch |
| AC-007.8    | `tests/fr007_resource_thermal_watch.rs` (`fr007_load_watch_samples_host_load_1m`) | load average watch |
| AC-007.9    | `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs` (`fr007_thermal_tui_fuse_coalesce_lines`); `crates/sharecli-fuse/src/read_cache.rs` | FUSE read-coalesce in status + thermal TUI |
| AC-007.10   | `tests/fr007_resource_thermal_watch.rs` (`fr007_format_status_section`); `tests/fr004_status_health.rs` (`fr004_status_prints_harness_table`); `src/commands/mod.rs` (`status`) | Host resource watch in status |
| AC-007.11   | `tests/fr007_thermal_tui_watch.rs`; `crates/sharecli-thermal-tui` | Host watch + FUSE meters in thermal TUI |
| AC-007.12   | `tests/fr007_thermal_tui_watch.rs` (`fr007_thermal_tui_resource_watch_net_lines`); `crates/sharecli-thermal-tui/src/lib.rs` (`resource_watch_lines`) | thermal TUI host net RX/TX parity with status |
| AC-007.13   | `tests/fr007_proc_json_host_watch.rs`; `src/commands/proc.rs` (`AgentProcSnapshot::host_watch`); `src/monitoring.rs` (`HostResourceWatchJson`) | proc JSON host ResourceWatchSample parity |
| AC-007.14   | `tests/fr007_proc_text_csv_host_watch.rs`; `src/commands/proc.rs` (`print_host_watch_text_footer`, `append_host_watch_csv_companion`); `src/monitoring.rs` (`HostResourceWatchJson::format_text_section`, `format_csv_companion`) | proc text/CSV host ResourceWatchSample parity |
| AC-007.15   | `tests/fr007_proc_tree_json_host_watch.rs`; `src/commands/proc.rs` (`AgentTreeSnapshot::host_watch`) | proc tree JSON/NDJSON host ResourceWatchSample parity |
| AC-007.16   | `tests/fr007_proc_pid_json_host_watch.rs`; `src/commands/proc.rs` (`ProcDetailSnapshot::host_watch`, `render_proc_detail`) | proc pid JSON/text host ResourceWatchSample parity |
| AC-007.17   | `tests/fr007_proc_pid_gate.rs`; `src/commands/proc.rs` (`ProcDetailSnapshot::gate`, `render_proc_detail`); `crates/sharecli-fleet/src/agent_contention.rs` (`format_gate_status_from_snapshot`) | proc pid JSON/text thermal gate parity |
| AC-007.18   | `tests/fr007_proc_tree_json_gate.rs`; `src/commands/proc.rs` (`AgentTreeSnapshot::gate`) | proc tree JSON/NDJSON thermal gate parity |
| AC-007.19   | `tests/fr007_proc_text_csv_gate.rs`; `src/commands/proc.rs` (`append_proc_csv_companions`); `crates/sharecli-fleet/src/agent_contention.rs` (`GateStatusSnapshot::format_csv_companion`) | proc flat/tree CSV thermal gate companion parity |
| AC-007.20   | `tests/fr007_proc_tree_text_gate.rs`; `src/commands/proc.rs` (`render_once` tree text path) | proc tree text thermal gate section before host watch footer |

### FR-008 — Coalesce

| AC        | Test file                        | Notes |
|-----------|----------------------------------|-------|
| AC-008.1..4 | `tests/fr008_coalesce_mesh.rs` | command_key / with_lock / thermal / cache hit |
| AC-008.5 | `tests/fr008_coalesce_mesh.rs` (`fr008_ttl_stale_entry_is_miss`); `sharecli-ipc` `ttl_lookup_miss_and_evict_on_store` | TTL miss + eviction |
| AC-008.6 | `tests/fr008_coalesce_mesh.rs` (`fr008_debounce_waits_and_shares`, `fr008_hypervisor_debounce_waits_and_shares`); `sharecli-ipc` `debounce_shares_recent_store` | debounce share window |
| AC-008.7..9 | `tests/fr008_coalesce_mesh.rs` (`fr008_nocache_*`, `fr008_slot_queue_*`, `fr008_hypervisor_nocache_*`) | nocache bypass + SlotQueue + Hypervisor queue route |
| AC-008.10 | `tests/e2e_hypervisor_nocache.rs` | Hypervisor nocache e2e: re-exec, serialize, coalesce isolation |
| AC-008.11 | `tests/fr008_coalesce_status.rs`; `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs`; `sharecli-ipc` `global_coalesce_meters_record_hit_miss_and_nocache` | coalesce operator meters in status + thermal TUI |
| AC-008.12 | `tests/fr008_coalesce_status.rs`; `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs`; `crates/sharecli-fleet/src/slot_queue_meters.rs` | SlotQueue acquire/wait/timeout in status + thermal TUI |

### FR-009 — FUSE

| AC        | Test file                        | Notes |
|-----------|----------------------------------|-------|
| AC-009.1..2 | `tests/fr009_fuse_intercept.rs` | construct + mount API; no privileged mount |
| AC-009.3 | `tests/fr009_fuse_intercept.rs`; `inode_map` unit tests | inode map / path resolution |
| AC-009.4 | `tests/fr009_fuse_intercept.rs`; `read_cache` unit tests | read coalesce hit/miss meters |
| AC-009.5 | `tests/fr009_fuse_intercept.rs`; `write_serialize` unit tests | path lock + CoW stage/commit/discard |
| AC-009.6 | `tests/fr009_fuse_intercept.rs`; `provenance` unit tests | write provenance xattrs on write_rel/commit_rel |
| AC-009.7 | `tests/fr009_fuse_intercept.rs`; `neg_dentry` unit tests | negative dentry TTL hit/miss + invalidate |
| AC-009.8 | `tests/fr009_fuse_intercept.rs` (`fr009_privileged_mount_smoke`); `mount_smoke` unit tests | opt-in live FUSE mount + provenance xattrs (`SHARECLI_FUSE_MOUNT_SMOKE=1`) |
| AC-009.9 | `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs` (`fr009_thermal_tui_neg_dentry_lines`); `crates/sharecli-fuse/src/neg_dentry.rs` | FUSE neg dentry in status + thermal TUI |
| AC-009.10 | `tests/fr009_fuse_intercept.rs` (`fr009_global_write_serialize_meters`); `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs`; `crates/sharecli-fuse/src/write_serialize_meters.rs` | FUSE write-serialize / CoW in status + thermal TUI |
| AC-009.11 | `tests/fr009_fuse_cli.rs`; `src/commands/fuse.rs` | `sharecli fuse provenance` reads backing write xattrs |
| AC-009.12 | `tests/fr009_fuse_hypervisor_session.rs`; `sharecli-core` `fuse_session_id_for_command_key`; `sharecli-fuse` `mount_with_session` | Hypervisor FUSE session from coalesce CommandKey |
| AC-009.13 | `tests/fr009_fuse_hypervisor_session.rs` (`fr009_hypervisor_spawn_outcome_fuse_session_id`); `sharecli-core` `SpawnOutcome::fuse_session_id` | SpawnOutcome exposes FUSE session when intercept active |
| AC-009.14 | `tests/fr009_fuse_hypervisor_session.rs` (`fr009_remap_mount_to_backing_subtree`, `fr009_hypervisor_spawn_outcome_fuse_path_remap`); `sharecli-fuse` `path_remap.rs`; `sharecli-core` `SpawnOutcome::remap_fuse_path`, `FuseGuard` teardown | FUSE mount/backing remap + spawn/teardown lifecycle |

### FR-010 — Mesh

| AC        | Test file                        | Notes |
|-----------|----------------------------------|-------|
| AC-010.1..3 | `tests/fr010_mesh_substrate.rs` | registry primitives |
| AC-010.4..6 | `tests/fr010_mesh_substrate.rs` | MaildirQueue lifecycle |
| AC-010.7 | `tests/fr010_mesh_substrate.rs`; `smart_merge` unit tests | git merge-file fallback |
| AC-010.8 | `tests/fr010_mesh_substrate.rs`; `worktree_pool` unit tests | allocate/release + NotGitRepo |
| AC-010.9 | `tests/fr010_mesh_substrate.rs`; `tests/fr010_mesh_cli.rs` | status counts + `mesh status` CLI |
| AC-010.10 | `tests/fr010_mesh_substrate.rs`; `tests/fr010_mesh_cli.rs` | reclaim_owner + `mesh reclaim` CLI |
| AC-010.11 | `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs`; `sharecli-mesh` operator_status tests | Maildir depth in status + thermal TUI |

### FR-011 — Thermal Gate

| AC        | Test file                     | Notes |
|-----------|-------------------------------|-------|
| AC-011.1..3 | `tests/fr011_thermal_gate.rs` | also AC-008.3 |
| AC-011.4 | `tests/fr011_agent_thermal_gate.rs`; `sharecli-core` / `sharecli-fleet` agent_contention unit tests | agent-count thermal escalation |
| AC-011.5 | `tests/fr004_status_health.rs`; `sharecli-fleet` `format_gate_status_section` unit tests | status thermal+agent gate section |
| AC-011.6 | `tests/fr011_report_gate.rs`; `src/commands/report.rs` unit tests; `src/commands/mod.rs` (`ps --all`) | report + ps --all gate parity |
| AC-011.7 | `tests/fr011_pool_health_gate.rs`; `src/commands/mod.rs` (`pool`, `health`) | pool + health gate parity |

### FR-012 — Serve JWT AuthN

| AC        | Test file                        | Test function |
|-----------|----------------------------------|---------------|
| AC-012.1..3 | `tests/fr012_serve_jwt_auth.rs` | (see file) |

---

## Change log

- **2026-07-20 — FR-007 proc tree text gate ordering:** `sharecli proc --tree` text
  prints thermal gate section before host watch footer (AC-007.20); completes proc text
  gate coverage across flat/tree/pid surfaces.
- **2026-07-20 — FR-007 proc CSV gate companion:** `sharecli proc --csv` and
  `--tree --csv` append companion `gate` CSV record before `host` via
  `GateStatusSnapshot::format_csv_companion` (AC-007.19); completes proc CSV gate
  coverage across flat/tree surfaces.
- **2026-07-20 — FR-007 proc tree gate parity:** `sharecli proc --tree --json` and
  `--tree --watch --json` emit `gate` on every snapshot via
  [`AgentTreeSnapshot`](src/commands/proc.rs) (AC-007.18); completes proc JSON gate
  coverage across flat/tree/pid surfaces.
- **2026-07-20 — FR-007 proc pid gate parity:** `sharecli proc --pid N --json` emits
  `gate` on [`ProcDetailSnapshot`](src/commands/proc.rs) and text detail prints thermal
  gate section before host watch footer (AC-007.17); completes proc detail gate coverage.
- **2026-07-20 — FR-007 proc pid JSON host watch:** `sharecli proc --pid N --json` emits
  `host_watch` on [`ProcDetailSnapshot`](src/commands/proc.rs) and text detail appends host
  watch footer (AC-007.16); completes proc JSON `host_watch` coverage.
- **2026-07-20 — FR-007 proc tree JSON host watch:** `sharecli proc --tree --json` and
  `--tree --watch --json` emit `host_watch` on every snapshot via
  `AgentTreeSnapshot` (AC-007.15); parity with flat JSON from AC-007.13.
- **2026-07-20 — FR-007 proc text/CSV host watch:** `sharecli proc` text footer and
  `--csv` companion `host` record surface live FD/RSS/load/net via
  `HostResourceWatchJson` (AC-007.14); flat + `--tree` parity with JSON `host_watch`.
- **2026-07-20 — FR-006 thermal TUI flat agent state:** flat Detected Agents lines
  (full, compact, empty-forest fallback) show process state letters via
  `build_host_agent_state_map` / `state_text_for_pid` (AC-006.40).
- **2026-07-20 — FR-006 thermal TUI agent-tree state:** full-layout Detected Agents
  tree nodes show process state letters via `build_host_forest_state_map` (AC-006.39);
  forest helpers lifted to `sharecli-fleet` for proc/TUI parity.
- **2026-07-20 — FR-009 FUSE path remap + lifecycle:** SpawnOutcome exposes
  `fuse_backing` / `fuse_mountpoint`; `remap_mount_to_backing` + guard teardown
  on drop (AC-009.14).
- **2026-07-20 — FR-009 SpawnOutcome FUSE session:** cache-miss outcomes expose
  `fuse_session_id` when intercept mount is active (AC-009.13).
- **2026-07-20 — FR-009 Hypervisor FUSE session:** cache-miss mounts pass
  coalesce-derived session id to `mount_with_session` (AC-009.12).
- **2026-07-20 — FR-009 fuse provenance CLI:** `sharecli fuse provenance`
  reads backing write xattrs via `read_provenance` (AC-009.11).
- **2026-07-20 — FR-008 SlotQueue operator meters:** `global_slot_queue_meters`
  + status/thermal TUI panels (AC-008.12); acquire/wait/timeout on nocache lane.
- **2026-07-20 — FR-009 write-serialize operator meters:** `global_write_serialize_meters`
  + status/thermal TUI panels (AC-009.10); mirrors read-coalesce / neg-dentry surfacing.
- **2026-07-20 — FR-008 coalesce operator meters:** `global_coalesce_meters` +
  status/thermal TUI panels (AC-008.11); mirrors FUSE read-coalesce surfacing.
- **2026-07-20 — FR-011 pool/health gate parity:** `sharecli pool` and
  `sharecli health` print `format_gate_status_section` after runtime health
  (AC-011.7).
- **2026-07-20 — FR-011 report/ps gate parity:** `sharecli report` JSON/text and
  `sharecli ps --all` surface gate fields via `gate_status_snapshot` (AC-011.6).
- **2026-07-20 — FR-011 status gate section:** `sharecli status` prints
  `format_gate_status_section` with live thermal + agent inventory (AC-011.5).
- **2026-07-20 — FR-011 agent-aware thermal gate:** `AgentAwareThermalGate` wraps
  production Hypervisor gate; proc-scan agent count escalates spawn decisions
  (AC-011.4); thermal TUI gate panel uses `effective_gate_decision`.
- **2026-07-20 — FR-006 proc sort state:** `sharecli proc --sort state` orders flat
  inventory and tree root forests by process state letter with PID tie-break
  (AC-006.36).
- **2026-07-20 — FR-006 proc watch NDJSON state parity:** `proc --watch --json` NDJSON
  agent rows include `state` letter matching flat `--json` export (AC-006.37).
- **2026-07-20 — FR-006 proc exclude-family filter:** `sharecli proc --exclude-family <id>`
  drops matching agent families from flat inventory and `--tree` roots (negation of
  `--family`); mutually exclusive with `--family` (AC-006.38).
- **2026-07-20 — FR-006 proc tree forest state lookup:** `--tree` text/JSON/CSV resolve
  live process state for every forest PID (roots + children) via
  `build_forest_state_map` (AC-006.35).
- **2026-07-20 — FR-006 proc tree state surfaces:** `--tree` text nodes and
  `--tree --json` `AgentTreeNodeJson` rows include `state` letter (AC-006.34).
- **2026-07-20 — FR-006 proc state text surfaces:** flat text inventory and
  `proc --pid` detail expose process state letter (AC-006.33).
- **2026-07-20 — FR-006 proc state export:** flat `--json` agent rows and `--csv`
  / `--tree --csv` columns include `state` letter (AC-006.32).
- **2026-07-20 — FR-006 proc state filter:** `sharecli proc --state <letter>`
  filters flat inventory and `--tree` roots by Linux/sysinfo process state,
  composed with other proc filters (AC-006.31).
- **2026-07-20 — FR-006 proc cmdline filter:** `sharecli proc --cmdline <pattern>`
  case-insensitive joined argv/cmdline substring filter composes with other proc
  flags (AC-006.30).
- **2026-07-20 — FR-006 proc comm filter:** `sharecli proc --comm <pattern>`
  case-insensitive COMM substring filter composes with other proc flags (AC-006.29).
- **2026-07-20 — FR-006 proc min/max-fd:** `sharecli proc --min-fd N` and
  `--max-fd N` FD band filters compose with other proc flags (AC-006.28).
- **2026-07-20 — FR-006 proc max-rss:** `sharecli proc --max-rss <size>` upper RSS
  bound filter composes with `--min-rss` and other proc flags (AC-006.27).
- **2026-07-20 — FR-006 thermal TUI agent tree:** full-layout Detected Agents panel
  renders `build_host_agent_forests` subtrees with live RSS (AC-006.22).
- **2026-07-20 — FR-006 proc pid detail:** `sharecli proc --pid N` one-shot
  RSS/FD/cmdline/parent detail view with `--json` (AC-006.23).
- **2026-07-20 — FR-006 proc limit:** `sharecli proc --limit N` caps flat inventory
  rows and tree root forests after filters/sort (AC-006.21).
- **2026-07-20 — FR-006 extended fingerprints:** `amp` family plus codex/aider/cursor-agent
  wrapper argv markers and false-positive guards (AC-006.20).
- **2026-07-20 — FR-006 proc sort:** `sharecli proc --sort rss|fd|pid|state` orders
  inventory rows and tree root forests after filters (AC-006.19, AC-006.36).
- **2026-07-20 — FR-006 proc filters:** `sharecli proc --family` and `--min-rss`
  narrow inventory rows and tree root forests (AC-006.17); `parse_rss_bytes`
  accepts plain bytes or K/M/G suffixes.
- **2026-07-20 — FR-006 proc tree mode:** `sharecli proc --tree` parent-child agent
  forests via `build_agent_forests` (AC-006.16); `--tree --json` nested `forests` payload.
- **2026-07-20 — FR-006 proc watch mode:** `sharecli proc --watch N` live refresh
  for host agent inventory (AC-006.15); text + `--json` parity with one-shot proc.
- **2026-07-20 — FR-006 proc CLI + RSS gate:** `sharecli proc` / `--json`, status
  `--json` agent inventory (AC-006.11–006.13); cmdline fingerprints for ambiguous
  comms (AC-006.14); aggregate agent RSS escalates spawn gate (AC-006.12).
- **2026-07-20 — FR-006/007 per-agent PID watch:** `AgentResourceSample` +
  `watch_host_agents` attach RSS/FD samples to proc-scan agents (AC-006.10);
  thermal DetectedAgent panel + `ps --all` show per-agent RSS; `SpawnOutcome::agent_family`.
- **2026-07-20 — FR-006 thermal TUI agent panel:** `scan_host_agents` inventory
  in `sharecli thermal` DetectedAgent panel (AC-006.9); complements ps AGENT
  column from #416.
- **2026-07-20 — FR-009 negative dentry operator meters:** `global_neg_dentry_meters`
  + status/thermal TUI panels (AC-009.9); mirrors read-coalesce surfacing from #414.
- **2026-07-20 — FR-007 thermal TUI dashboard slice:** `sharecli thermal`
  polls ResourceWatchSample + FUSE read-coalesce meters each redraw
  (AC-007.9 TUI + AC-007.11); formalized AC-007.7..9 in FR.md.
- **2026-07-20 — FR-008 Hypervisor nocache e2e:** AC-008.10 side-effect
  re-exec + concurrent SlotQueue serialize + coalesce isolation
  (`tests/e2e_hypervisor_nocache.rs`).
- **2026-07-20 — FR-009 negative dentry cache:** `NegativeDentryCache` +
  `exists_rel` / FUSE lookup ENOENT TTL (AC-009.7).
- **2026-07-20 — FR-010 mesh CLI:** `MaildirQueue::status` + `reclaim_owner`
  operator surface; `sharecli mesh status|reclaim` (AC-010.9..10);
  TRACEABILITY AC-008.7..9 marked covered (was stale TBD).
- **2026-07-20 — FR-009 mount smoke provenance:** `run_mount_smoke` verifies
  write provenance xattrs on backing after live FUSE write (AC-009.6 × AC-009.8).
- **2026-07-20 — FR-009 mount smoke:** opt-in privileged FUSE read/write via
  `SHARECLI_FUSE_MOUNT_SMOKE=1` (`run_mount_smoke`, AC-009.8).
- **2026-07-20 — FR-009 write provenance:** `user.sharecli.session` /
  `user.sharecli.written_at` stamped on `write_rel` / `commit_rel` (AC-009.6).
- **2026-07-19 — FR-009/010 A+ closeout:** CoW `stage_bytes`/`commit_pending`/
  `discard_pending` (AC-009.5); `SmartMerger` + `WorktreePool` (AC-010.7..8).
- **2026-07-19 — FR-009 A+ recovery:** InterceptFs passthrough + inode map +
  read coalesce meters + write-serialize scaffold (AC-009.3..5).
- **2026-07-19 — thesis re-enum:** Published FR-006..FR-011 from origin export
  (paste + agent-mesh WBS + harness). Traceability matrix expanded to 12 FRs.
- **2026-07-12 — T-220:** Landed `tests/fr004_status_health.rs` + `tests/fr004_pool_status.rs` (AC-004.1..004.5).
- **2026-07-12 — T-210:** Landed `tests/fr003_project_registry.rs` + `tests/fr003_project_discover.rs` (AC-003.1..003.5). Also T-260 claim-lock + T-270 loop budgets.
- **2026-07-12 — T-200:** Landed `tests/fr002_config_init.rs` + `tests/fr002_config_load.rs` (AC-002.1..002.5). FR-002 Status remains ACCEPTED with files on disk. Governance: `docs/ops/governance/{WBS-PHASED,GAP-QA-MATRIX}.md`.
- **2026-06-15 — Phase 3 initial:** 5 FRs published; 10 acceptance test files added; full FR→source→test matrix established.
