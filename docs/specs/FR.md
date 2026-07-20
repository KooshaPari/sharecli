# Functional Requirements — sharecli

> Canonical FR index for the sharecli CLI. Each requirement is traceable to source
> code, tests, and acceptance criteria via `docs/specs/TRACEABILITY.md`.

**Scope:** This document defines the *minimum* set of Functional Requirements (FRs)
that the sharecli binary MUST satisfy to be considered Phase 3 complete. Additional
NFRs and design notes live in `SPEC.md` and `PRD.md`; this file is the spec-of-record
for the CLI surface.

**Conventions:**

- FR IDs are stable: `FR-NNN`. They MUST NOT be renumbered once published.
- Each FR has a single **MUST** statement, plus acceptance criteria (AC) that are
  independently testable.
- The `Source` column points at the canonical Rust source file(s) that implement
  the requirement. The `Test` column points at the acceptance test file(s) that
  cover it.
- Phase 3 only covers FR-001..FR-005. Runtime thesis FRs FR-006..FR-011 and
  operator FR-012 MAY be appended but MUST NOT renumber or rewrite existing
  entries.

---

## FR-001 — Managed Process Lifecycle (start / list / stop)

**Statement:** The CLI MUST be able to start a named process associated with a
project and harness type, list the running managed processes with optional
filtering, and stop them by PID, project, harness, or `--all`.

**Source:**

- `src/main.rs:38-91` — `Commands::Ps`, `Commands::Start`, `Commands::Stop` enum variants
- `src/commands/mod.rs:25-138` — `ps`, `start`, `stop` command implementations
- `src/runtime.rs:44-156` — `ProcessPool::spawn`, `ProcessPool::list`, `ProcessPool::kill`, `ProcessPool::kill_all`

**Acceptance Criteria:**

- **AC-001.1:** `sharecli start <project> --harness <harness>` records a process
  in the in-memory `ProcessPool` and returns a non-zero PID.
- **AC-001.2:** `sharecli ps` prints a table with columns `PID`, `NAME`, `MEM(MB)`,
  `PROJECT`, `HARNESS` plus a totals footer.
- **AC-001.3:** `sharecli ps --project <p>` returns only processes whose
  `project` field equals `<p>`.
- **AC-001.4:** `sharecli stop --all` terminates every managed process and
  reports `All processes stopped.`
- **AC-001.5:** `sharecli stop` with no selector exits with an error message
  instructing the user to specify `--pid`, `--project`, `--harness`, or `--all`.

**Test refs:** `tests/fr001_process_lifecycle.rs`, `tests/fr001_stop_filter.rs`

---

## FR-002 — TOML Configuration Management

**Statement:** The CLI MUST load, initialize, validate, and display its TOML
configuration from the platform config directory
(`$XDG_CONFIG_HOME/sharecli/config.toml` or OS equivalent), and it MUST persist
project registrations across invocations.

**Source:**

- `src/config.rs:1-119` — `Config`, `RuntimeConfig`, `Config::load`, `Config::init`, `Config::save`
- `src/commands/mod.rs:194-222` — `config` command (Init, Validate, Show, Get, Set)

**Acceptance Criteria:**

- **AC-002.1:** `sharecli config init` creates the config directory if missing
  and writes a default TOML file that round-trips through `Config::load`.
- **AC-002.2:** `sharecli config validate` reports the number of registered
  projects on success.
- **AC-002.3:** `sharecli config show` prints the serialized TOML containing
  a `[projects]` and `[runtime]` table.
- **AC-002.4:** A `Config` deserialized from TOML preserves the `projects`
  `HashMap<String, String>` and the `RuntimeConfig` fields
  (`node_path`, `bun_path`, `max_memory_mb`, `max_processes`).
- **AC-002.5:** `RuntimeConfig::default()` returns `max_memory_mb = Some(4096)`
  and `max_processes = Some(100)`.

**Test refs:** `tests/fr002_config_load.rs`, `tests/fr002_config_init.rs`

---

## FR-003 — Project Registry (add / list / show / discover / remove)

**Statement:** The CLI MUST maintain a registry of named projects (each mapping
to a filesystem path) under the `[projects]` table, and MUST support add, list,
show, remove, and discover (recursive scan for git repos) operations.

**Source:**

- `src/config.rs:8-68` — `Config.projects`, `default_projects`
- `src/commands/mod.rs:225-313` — `project` command (Add, Remove, List, Show, Discover, Generate)

**Acceptance Criteria:**

- **AC-003.1:** `sharecli project add <name> <path>` inserts a new entry into
  `Config.projects` and persists the change via `Config::save`.
- **AC-003.2:** `sharecli project list` prints one `name -> path` line per
  registered project, or the empty-state hint if none are registered.
- **AC-003.3:** `sharecli project show <name>` prints the resolved path and
  whether the path currently exists on disk.
- **AC-003.4:** `sharecli project discover [path]` scans the given directory
  and reports any subdirectory that contains a `.git` directory.
- **AC-003.5:** `sharecli project remove <name>` removes the entry from the
  `Config.projects` map and persists the change.

**Test refs:** `tests/fr003_project_registry.rs`, `tests/fr003_project_discover.rs`

---

## FR-004 — Process & Pool Health Status

**Statement:** The CLI MUST be able to report the health of managed processes
(per-harness counts, per-harness memory, system memory) and of the shared
runtime pool (node/bun totals, idle count, in-use count, max-per-type), and
MUST report per-process resource compliance.

**Source:**

- `src/runtime.rs:152-356` — `system_memory_usage`, `SharedRuntime::status`, `SharedRuntime::health_check`, `PoolStatus`, `RuntimeHealth`
- `src/monitoring.rs:1-118` — `HealthStatus`, `ProcessStats`, `MonitoringReport`, `ProcessStats::is_idle`
- `src/commands/mod.rs:140-191` — `status` command
- `src/commands/mod.rs:325-396` — `pool_status`, `health` commands

**Acceptance Criteria:**

- **AC-004.1:** `sharecli status` prints a per-harness table of `(count,
  memory_mb)` totals, followed by a shared-runtime pool table, and a
  system-memory line.
- **AC-004.2:** `sharecli pool` reports node and bun pool totals, idle
  counts, and the `max_per_type` ceiling.
- **AC-004.3:** `sharecli health [--harness <h>]` reports
  `HEALTHY`/`DEGRADED` based on whether every pooled process is still alive
  and under the 1 GB high-memory threshold.
- **AC-004.4:** `HealthStatus::mark_unhealthy(reason)` increments
  `checks_failed` and emits a `Health check failed: <reason>` message to
  stderr.
- **AC-004.5:** `ProcessStats::is_idle(threshold)` returns `true` only when
  the process has been up longer than `threshold` seconds AND `cpu_percent < 1.0`.

**Test refs:** `tests/fr004_status_health.rs`, `tests/fr004_pool_status.rs`

---

## FR-005 — Per-Project Resource Limits

**Statement:** The CLI MUST be able to set per-project memory and
max-process-count limits, persist them in-memory across calls within a single
process lifetime, and check whether the currently running processes for a
project are within those limits.

**Source:**

- `src/runtime.rs:358-455` — `ProjectLimits`, `ProjectResources::set_limits`,
  `ProjectResources::get_limits`, `ProjectResources::check_limits`, `ResourceCheck`
- `src/commands/mod.rs:398-447` — `set_limits`, `check_limits` commands

**Acceptance Criteria:**

- **AC-005.1:** `ProjectLimits::default()` returns
  `memory_limit_mb = 1024`, `max_processes = 10`, `cpu_affinity = None`.
- **AC-005.2:** `sharecli limits <project> --memory <mb> --processes <n>` sets
  the project's limits and prints a confirmation.
- **AC-005.3:** `ProjectResources::get_limits` returns the most recently
  set limits, or `ProjectLimits::default()` for unknown projects.
- **AC-005.4:** `ResourceCheck::overall_ok` is `true` only when both
  `memory_ok` and `processes_ok` are `true`.
- **AC-005.5:** `sharecli check <project>` prints memory, process count, and
  per-axis status (`OK` / `EXCEEDED`) plus an overall verdict line.

**Test refs:** `tests/fr005_project_limits.rs`, `tests/fr005_resource_check.rs`

---

## FR-006 — Agent Detection (proc scan, no bin wrap)

**Statement:** The runtime MUST discover agents via process / pattern scan
(including process-tree ancestor walk) and MUST NOT require wrapping or
replacing vendor agent executables as the primary detection path.

**Source:**

- `crates/sharecli-core/src/detect.rs` — known-agent pattern registry
- `crates/sharecli-core/src/proc_scan.rs` — `/proc` + ancestor walk
- `crates/sharecli-core` Hypervisor — observation path (direct argv)

**Acceptance Criteria:**

- **AC-006.1:** Known agent names / path basenames resolve to agent families.
- **AC-006.2:** Ordinary shells / tools are not classified as agents.
- **AC-006.3:** Hypervisor executes argv directly (no wrap).
- **AC-006.4:** Process scan lists agent PIDs only.
- **AC-006.5:** Child tool under an agent walks to the agent family.
- **AC-006.6:** Human tool under a non-agent shell is not an agent path.
- **AC-006.7:** `sharecli ps` table includes an `AGENT` column derived from
  proc-scan ancestor walk.
- **AC-006.8:** `sharecli ps --all` lists host-detected agent processes via
  [`scan_host_agents`](crates/sharecli-fleet/src/proc_scan.rs).
- **AC-006.9:** `sharecli thermal` polls [`scan_host_agents`](crates/sharecli-fleet/src/proc_scan.rs)
  on each redraw and renders a DetectedAgent inventory panel.
- **AC-006.10:** Detected agent PIDs carry live per-process resource samples
  via [`AgentResourceSample::capture_for_pid`](crates/sharecli-fleet/src/resource_watch.rs)
  / [`watch_detected_agents`](crates/sharecli-fleet/src/resource_watch.rs); thermal
  TUI agent rows and `sharecli ps --all` MUST show RSS (FD on Linux when exposed);
  dead PIDs MUST be omitted rather than silent zero.
- **AC-006.11:** `sharecli proc` lists host-detected agents with live RSS/FD samples
  and thermal gate section (parity with `ps --all` agent inventory).
- **AC-006.12:** Aggregate watched-agent RSS from [`watch_host_agents`](crates/sharecli-fleet/src/resource_watch.rs)
  escalates [`AgentAwareThermalGate`](crates/sharecli-core/src/lib.rs) via
  [`combined_agent_contention_tier`](crates/sharecli-fleet/src/agent_contention.rs)
  (default warn ≥16GiB, refuse ≥32GiB total agent RSS).
- **AC-006.13:** `sharecli proc --json` and `sharecli status --json` emit structured
  detected-agent rows (`pid`, `family`, `comm`, `mem_rss_bytes`, `fd_count`) plus
  gate fields including `agent_total_rss_bytes`.
- **AC-006.14:** Ambiguous agent `comm` names (`forge`, `goose`, `gemini`) require
  cmdline fingerprint markers in [`match_known_agent`](crates/sharecli-fleet/src/detect.rs)
  to avoid false positives from unrelated tooling.
- **AC-006.15:** `sharecli proc --watch N` (N ≥ 1) clears the terminal and
  re-renders the host agent inventory every N seconds until Ctrl-C; MUST honor
  `--json` (pretty JSON each refresh) and print a `[watch]` footer with the
  refresh interval.

**Test refs:** `tests/fr006_agent_detection.rs`, `tests/fr006_proc_tree.rs`, `tests/fr006_ps_agent_column.rs`, `tests/fr006_thermal_tui_agents.rs`, `tests/fr006_agent_pid_watch.rs`, `tests/fr006_proc_cli.rs`, `tests/fr006_proc_fingerprints.rs`, `tests/fr006_agent_rss_gate.rs`, `tests/fr006_proc_watch.rs`

---

## FR-007 — Resource & Syscall-Relevant Watch

**Statement:** The runtime MUST expose resource watch signals (CPU / MEM /
Net / FD at minimum) and, when FUSE intercept is enabled, IO paths used by
coalesce. Thermal watch signals MAY surface via FR-011.

**Source:**

- `crates/sharecli-core`, `crates/sharecli-fleet`, `src/monitoring.rs`

**Acceptance Criteria:**

- **AC-007.1:** Mock thermal levels are visible via poll (watch signal).
- **AC-007.2:** Fake thermal gate decisions are stable for a level.
- **AC-007.3:** Idle heuristic encodes CPU + uptime watch signal.
- **AC-007.4:** FD watch samples the current process open descriptor count
  (`sample_self_fds` / `ResourceWatchSample::capture`); MUST fail loudly on
  unsupported OS rather than returning silent zero.
- **AC-007.5:** Host net RX/TX watch returns byte counters via
  `sample_host_net` / `ResourceWatchSample::capture`; MUST fail loudly on
  unsupported OS rather than returning silent zero.
- **AC-007.6:** [`Hypervisor::run`](crates/sharecli-core/src/lib.rs) captures
  a live [`ResourceWatchSample`](crates/sharecli-fleet/src/resource_watch.rs)
  on every invocation and attaches it to [`SpawnOutcome::resource_watch`](crates/sharecli-core/src/lib.rs).
- **AC-007.7:** RSS watch samples current process resident memory via
  `sample_self_rss_bytes` / `ResourceWatchSample::capture`; MUST fail loudly on
  unsupported OS rather than returning silent zero.
- **AC-007.8:** Host 1-minute load average is sampled via
  `sample_host_load_1m` / `ResourceWatchSample::capture`; MUST fail loudly on
  unsupported OS rather than returning silent zero.
- **AC-007.9:** `sharecli status` and `sharecli thermal` surface FUSE
  read-coalesce hit/miss meters via [`global_read_cache_meters`](crates/sharecli-fuse/src/read_cache.rs)
  / [`format_status_section`](crates/sharecli-fuse/src/read_cache.rs).
- **AC-007.10:** `sharecli status` captures a live
  [`ResourceWatchSample::capture`](crates/sharecli-fleet/src/resource_watch.rs)
  after the system-memory section and prints
  [`format_status_section`](crates/sharecli-fleet/src/resource_watch.rs)
  with open FD count, RSS bytes, 1-minute load average, and host net RX/TX
  counters; MUST fail loudly via `?` when sampling is unsupported or errors.
- **AC-007.11:** `sharecli thermal` polls a live
  [`ResourceWatchSample::capture`](crates/sharecli-fleet/src/resource_watch.rs)
  on each redraw and renders host FD/RSS/load plus FUSE read-coalesce meters
  in dedicated TUI panels (Feb harness dashboard slice); capture failure MUST
  render an explicit unavailable message (no silent zero panel).

**Test refs:** `tests/fr007_resource_thermal_watch.rs`, `tests/fr007_thermal_tui_watch.rs`, `tests/fr004_status_health.rs`

---

## FR-008 — Speculative Coalesce / Debounce / Queue

**Statement:** Identical concurrent invocations MUST coalesce via
Lock-Wait-Cache (`CoalesceCache`) with configurable TTL (default 300s) and
optional debounce window. Mutating argv matching `nocache_args` (Feb harness)
MUST route through an N-slot [`SlotQueue`] / [`PriorityQueue`] and MUST NOT
be served from the coalesce cache.

**Source:**

- `crates/sharecli-ipc` — `command_key`, `CoalesceCache::{new,with_ttl,with_options}`,
  `SlotQueue` / `PriorityQueue`, `has_nocache_arg` / `DEFAULT_NOCACHE_ARGS`
- `crates/sharecli-core` — `Hypervisor::{run,queue,nocache_args}`

**Acceptance Criteria:**

- **AC-008.1:** Identical argv / cwd / env → same `command_key`; different argv → different.
- **AC-008.2:** `CoalesceCache::with_lock` runs the miss path once per key.
- **AC-008.3:** Thermal `Refuse` fails loudly before speculative coalesce / spawn.
- **AC-008.4:** Second identical `Allow` run is served from coalesce cache.
- **AC-008.5:** Entries older than configured TTL are treated as miss; `store` evicts stale `*.json`.
- **AC-008.6:** When debounce is set, a miss waits then shares an in-window sibling store instead of re-running.
- **AC-008.7:** Argv containing a configured `nocache_args` flag (`--fix`, `--force`, `--write`, …) MUST bypass coalesce detection (`should_bypass_coalesce`).
- **AC-008.8:** `SlotQueue` with `max_concurrent=1` MUST serialize concurrent lane work.
- **AC-008.9:** `Hypervisor::run` with nocache argv MUST execute via `SlotQueue` and MUST NOT set `from_cache` on replay.
- **AC-008.10:** End-to-end Hypervisor nocache MUST (a) re-execute identical mutating argv (side-effect counter increments per run), (b) serialize concurrent nocache runs through the Hypervisor queue (`max_concurrent=1`), and (c) remain isolated from a seeded coalesce cache hit on a read-only twin argv.
- **AC-008.11:** `sharecli status` and `sharecli thermal` surface Hypervisor coalesce hit/miss/nocache counters via [`global_coalesce_meters`](crates/sharecli-ipc/src/lib.rs) / [`CoalesceMeters::format_status_section`](crates/sharecli-ipc/src/lib.rs); counters MUST increment on lookup hits, [`CoalesceCache::with_lock_detailed`](crates/sharecli-ipc/src/lib.rs) outcomes, and nocache queue runs.
- **AC-008.12:** `sharecli status` and `sharecli thermal` surface Hypervisor SlotQueue acquire/wait/timeout counters via [`global_slot_queue_meters`](crates/sharecli-fleet/src/slot_queue_meters.rs) / [`SlotQueueMeters::format_status_section`](crates/sharecli-fleet/src/slot_queue_meters.rs); counters MUST increment on successful [`SlotQueue::with_slot`](crates/sharecli-ipc/src/queue.rs) acquisitions, wait-loop iterations, and queue timeouts.

**Test refs:** `tests/fr008_coalesce_mesh.rs`; `tests/fr008_coalesce_status.rs`; `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs`; `tests/e2e_hypervisor_nocache.rs`; `sharecli-ipc` unit tests for TTL/debounce/queue/nocache/meters.

---

## FR-009 — FUSE IO Intercept

**Statement:** On Linux and macOS, sharecli MUST provide a FUSE attach point
(`InterceptFs` / `mount`) over a backing path as the hypervisor IO intercept
extension point. Other platforms MUST return a clear unsupported error.
Core VFS ops (lookup, getattr, open, read, write, readdir, mkdir, unlink,
rmdir, rename) MUST forward to the backing filesystem via an inode map.
In-process read content cache MUST key by path+mtime with hit/miss meters.
Writes to the same path MUST serialize. Staging CoW (`stage_bytes` →
`commit_pending` / `discard_pending`) MUST promote or drop staging copies
without silent no-ops; commit/discard with no pending entry MUST return
`NoPending`. Successful `write_rel` and `commit_rel` MUST stamp write
provenance extended attributes (`user.sharecli.session`,
`user.sharecli.written_at`) on the backing file. Missing-path lookups MUST be
remembered in a TTL negative dentry cache and skipped on subsequent probes
until create / mkdir / rename-into invalidates the entry.

**Source:**

- `crates/sharecli-fuse/src/lib.rs` — `InterceptFs`, `mount`
- `crates/sharecli-fuse/src/inode_map.rs` — `InodeMap`
- `crates/sharecli-fuse/src/read_cache.rs` — `ReadContentCache`
- `crates/sharecli-fuse/src/neg_dentry.rs` — `NegativeDentryCache`
- `crates/sharecli-fuse/src/write_serialize.rs` — `WriteSerialize`
- `crates/sharecli-fuse/src/provenance.rs` — write provenance xattrs
- `crates/sharecli-fuse/src/mount_smoke.rs` — opt-in privileged mount smoke
- `src/commands/fuse.rs` — `sharecli fuse provenance`

**Acceptance Criteria:**

- **AC-009.1:** `InterceptFs::new` constructs over an existing backing path
  without requiring a privileged mount (unit / pure construct).
- **AC-009.2:** `mount` is publicly exported; on unsupported platforms it
  returns an error mentioning unsupported (no silent success).
- **AC-009.3:** `InodeMap` resolves nested parent/child paths and allocates
  stable inodes without a privileged mount.
- **AC-009.4:** In-process read coalesce cache records a miss then a hit for
  identical path+mtime; meters expose hits/misses (no privileged mount).
- **AC-009.5:** Per-path write serialization is available; passthrough write
  succeeds (no ENOSYS); `stage_bytes` + `commit_pending` promotes staging to
  backing; `stage_bytes` + `discard_pending` leaves backing unchanged;
  commit/discard with no pending returns `NoPending`. InterceptFs exposes
  `stage_rel` / `commit_rel` / `discard_rel` for FR tests without mount.
- **AC-009.6:** `write_rel` and `commit_rel` stamp `user.sharecli.session`
  and `user.sharecli.written_at` on the backing path; session id is readable
  via `read_provenance` / `InterceptFs::session_id` (no privileged mount).
- **AC-009.7:** `exists_rel` / FUSE `lookup` remember ENOENT in
  `NegativeDentryCache` (miss then hit within TTL); `invalidate_neg_rel` /
  mkdir / rename-into clear the entry so a newly created path is visible
  without waiting for TTL expiry (no privileged mount).
- **AC-009.8:** With `SHARECLI_FUSE_MOUNT_SMOKE=1`, `run_mount_smoke` performs
  a read/write round-trip through a live FUSE mount over a temp backing tree and
  verifies write provenance xattrs on the backing file after the FUSE write;
  default `cargo test` skips without failure when the env var is unset.
- **AC-009.9:** `sharecli status` and `sharecli thermal` surface FUSE
  negative-dentry hit/miss meters via [`global_neg_dentry_meters`](crates/sharecli-fuse/src/neg_dentry.rs)
  / [`NegDentryMeters::format_status_section`](crates/sharecli-fuse/src/neg_dentry.rs).
- **AC-009.10:** `sharecli status` and `sharecli thermal` surface FUSE
  write-serialize / CoW counters (passthrough writes, stages, commits, discards) via
  [`global_write_serialize_meters`](crates/sharecli-fuse/src/write_serialize_meters.rs)
  / [`WriteSerializeMeters::format_status_section`](crates/sharecli-fuse/src/write_serialize_meters.rs);
  counters MUST increment on `write_rel`, `stage_rel` / `stage_bytes`, `commit_rel` /
  `commit_pending`, and `discard_rel` / `discard_pending`.
- **AC-009.11:** `sharecli fuse provenance <path>` reads write-provenance xattrs via
  [`read_provenance`](crates/sharecli-fuse/src/provenance.rs) on a backing file (no live
  mount required); `--json` emits `{path,session_id,written_at_unix}` or `null` when absent;
  missing paths and directories fail loudly.
- **AC-009.12:** On Hypervisor cache-miss spawns, FUSE intercept mounts MUST use
  [`fuse_session_id_for_command_key`](crates/sharecli-core/src/lib.rs) derived from the
  coalesce [`CommandKey`](crates/sharecli-ipc/src/lib.rs) (`hv-{first 16 hex}`) via
  [`sharecli_fuse::mount_with_session`](crates/sharecli-fuse/src/lib.rs); writes through
  the intercept layer MUST stamp that session id (parity with AC-009.6).
- **AC-009.13:** [`SpawnOutcome::fuse_session_id`](crates/sharecli-core/src/lib.rs) MUST be
  `Some(session)` when a cache-miss spawn had an active FUSE intercept mount
  ([`SpawnOutcome::fuse_intercept_active`](crates/sharecli-core/src/lib.rs)); MUST be `None`
  for coalesce cache hits, nocache queue routing, or cache misses where FUSE did not mount.
  The session string MUST match [`fuse_session_id_for_command_key`] for the coalesce key.
- **AC-009.14:** When FUSE intercept is active, [`SpawnOutcome`](crates/sharecli-core/src/lib.rs)
  MUST expose `fuse_backing` (original cwd) and `fuse_mountpoint` (ephemeral intercept mount);
  [`SpawnOutcome::remap_fuse_path`](crates/sharecli-core/src/lib.rs) and
  [`sharecli_fuse::remap_mount_to_backing`](crates/sharecli-fuse/src/path_remap.rs) MUST
  translate paths under the mount to backing equivalents (prefix-safe, `None` outside subtree).
  [`FuseGuard`](crates/sharecli-core/src/lib.rs) MUST remain mounted for the full coalesce
  spawn window and force-unmount on drop after the child exits.

**Test refs:** `tests/fr009_fuse_intercept.rs`; `tests/fr009_fuse_cli.rs`; `tests/fr009_fuse_hypervisor_session.rs`; `tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs`; `sharecli-fuse` unit tests.

---

## FR-010 — Agent Mesh / Shared Substrate

**Statement:** sharecli MUST expose mesh / substrate coordination primitives
for participating hosts (registry subject namespace + device records), a
Maildir-style filesystem task queue (`enqueue` / `claim` / `ack`) as the
execution-substrate port of `thegent.mesh.task_queue`, plus Jun mesh ports
for smart three-way merge (`SmartMerger`) and git worktree pooling
(`WorktreePool`). Operators MUST be able to inspect queue depth
([`MaildirQueue::status`] / `sharecli mesh status`) and reclaim stranded
in-flight work for a given owner ([`MaildirQueue::reclaim_owner`] /
`sharecli mesh reclaim`).

**Source:**

- `crates/sharecli-fleet/src/registry.rs` — `FleetRegistry`, `DeviceRecord`
- `crates/sharecli-mesh` — `MaildirQueue::{enqueue,claim,ack,nack,status,reclaim_owner}`
- `crates/sharecli-mesh` — `SmartMerger`, `WorktreePool`
- `src/commands/mesh.rs` — `sharecli mesh status|reclaim`

**Acceptance Criteria:**

- **AC-010.1:** Disconnected registry uses default subject prefix
  `sharecli.fleet`.
- **AC-010.2:** `subject_for(device_id)` yields `{prefix}.devices.{device_id}`.
- **AC-010.3:** `DeviceRecord` JSON round-trips with required keys; register
  without NATS fails loudly (no silent publish).
- **AC-010.4:** `MaildirQueue::enqueue` writes via `tmp/`→`new/`; `claim` moves
  `new/`→`cur/`; `ack` removes from `cur/`.
- **AC-010.5:** Lower `priority` values are claimed before higher ones.
- **AC-010.6:** `nack` returns a claimed task from `cur/` to `new/` for retry.
- **AC-010.7:** `SmartMerger::merge` falls back to `git merge-file --diff3`
  when mergiraf is unavailable; clean non-overlapping edits succeed;
  conflicting edits set `success=false` and still write output.
- **AC-010.8:** `WorktreePool::allocate` / `release` create and remove git
  worktrees under a pool root; opening a non-git repo fails with
  `NotGitRepo` (loud, no directory-slot silent fallback).
- **AC-010.9:** `MaildirQueue::status` reports `ready` (`new/`), `in_flight`
  (`cur/`), and `pending` (= sum); `sharecli mesh status --queue` / `--json`
  exposes the same counts.
- **AC-010.10:** `MaildirQueue::reclaim_owner` moves matching `cur/` tasks
  back to `new/` (count returned); non-matching owners reclaim 0;
  `sharecli mesh reclaim --queue --owner` performs the same; empty owner
  fails loudly.
- **AC-010.11:** `sharecli status` and `sharecli thermal` surface Maildir
  queue depth (`ready` / `in_flight` / `pending`) via
  [`capture_maildir_status`](crates/sharecli-mesh/src/operator_status.rs)
  / [`MaildirStatus::format_status_section`](crates/sharecli-mesh/src/operator_status.rs)
  when `SHARECLI_MESH_QUEUE` or the default `{state_dir}/mesh/queue` exists.

**Test refs:** `tests/fr010_mesh_substrate.rs`; `tests/fr010_mesh_cli.rs`;
`tests/fr004_status_health.rs`; `tests/fr007_thermal_tui_watch.rs`;
`sharecli-mesh` unit tests.

---

## FR-011 — Thermal Contention Gate

**Statement:** Host pressure MUST map to Green / Yellow / Red and gate
speculative coalesce via Allow / Warn / Refuse semantics.

**Source:**

- `crates/sharecli-fleet/src/thermal.rs` — `ThermalGovernor`, `ThermalLevel`
- `crates/sharecli-core` — `FakeThermalGate`, `ThermalDecision`

**Acceptance Criteria:**

- **AC-011.1:** `ThermalGovernor::with_mock` returns the configured level on poll.
- **AC-011.2:** `FakeThermalGate` Refuse / Allow decisions are stable.
- **AC-011.3:** Hypervisor Refuse path errors with thermally throttled
  (see also AC-008.3).
- **AC-011.4:** Production [`Hypervisor`](crates/sharecli-core/src/lib.rs) wraps
  [`SystemThermalGate`](crates/sharecli-core/src/lib.rs) with
  [`AgentAwareThermalGate`](crates/sharecli-core/src/lib.rs): live proc-scan agent
  count escalates Allow→Warn at `warn_at` (default 4) and any decision→Refuse at
  `refuse_at` (default 8); `sharecli thermal` gate panel uses
  [`effective_gate_decision`](crates/sharecli-fleet/src/agent_contention.rs) so
  agent Refuse shows DENY even when thermal is Green.
- **AC-011.5:** `sharecli status` polls live thermal level and proc-scan agent
  inventory and prints [`format_gate_status_section`](crates/sharecli-fleet/src/agent_contention.rs)
  with thermal level, detected agent count, contention tier, and ADMIT/DENY gate
  decision (parity with thermal TUI gate panel).
- **AC-011.6:** `sharecli report` (text + JSON) and `sharecli ps --all` expose the
  same gate fields via [`gate_status_snapshot`](crates/sharecli-fleet/src/agent_contention.rs)
  / [`format_gate_status_section`](crates/sharecli-fleet/src/agent_contention.rs)
  (detected agent count, contention tier, ADMIT/DENY).
- **AC-011.7:** `sharecli pool` and `sharecli health` print the same live
  [`format_gate_status_section`](crates/sharecli-fleet/src/agent_contention.rs)
  after pool/runtime health output (parity with `status` / AC-011.5).

**Test refs:** `tests/fr011_thermal_gate.rs`, `tests/fr011_agent_thermal_gate.rs`, `tests/fr004_status_health.rs`, `tests/fr011_report_gate.rs`, `tests/fr011_pool_health_gate.rs`, `tests/fr008_coalesce_mesh.rs`

---

## FR-012 — Serve HTTP Federated AuthN

**Statement:** When `sharecli serve` is configured with `auth_mode = "jwt"` (or an
equivalent `[serve.jwt]` block), non-probe HTTP routes MUST reject requests whose
`Authorization: Bearer` JWT fails signature or `iss` / `aud` / `exp` validation
against the configured JWKS. `/healthz` and `/readyz` MUST remain public.

**Source:**

- `src/serve_auth.rs` — JWT / JWKS validation middleware
- `src/config.rs` — `ServeConfig` / `ServeJwtConfig`
- `src/commands/serve.rs` — AuthN wiring at serve startup
- `docs/ops/AUTH.md` — operator guide

**Acceptance Criteria:**

- **AC-012.1:** Valid RS256 JWT with matching `iss`/`aud` is authorized (returns
  `sub` from claims).
- **AC-012.2:** Expired JWT is rejected with reason `jwt_expired`.
- **AC-012.3:** Wrong `aud` / `iss` are rejected; probe paths stay public
  (see unit tests for `/healthz` `/readyz`).

**Test refs:** `tests/fr012_serve_jwt_auth.rs`, `src/serve_auth.rs` unit tests

---

## NFR Notes

- **NFR-001 Platform Support:** The CLI MUST build and run on Linux, macOS,
  and Windows. Process-pool tests are gated with `#[cfg(unix)]` /
  `#[cfg(windows)]` blocks to honour this.
- **NFR-002 Observability:** The CLI MUST emit structured logs via
  `tracing`/`tracing-subscriber`, gated on `--verbose` / `--quiet`.
- **NFR-003 Error Handling:** All commands MUST return `anyhow::Result<()>`
  and MUST NOT panic on missing config (FR-002 covers the missing-file case
  by returning `Config::default()`).
- **NFR-004 Eval boundary:** Harbor soak is not a sharecli product acceptance
  criterion (ADR 0002).
