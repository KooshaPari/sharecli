# Functional Requirements — sharecli

> Agent-facing FR index using stable `FR-NNN` IDs. Full acceptance criteria,
> source maps, and AC tables live in [`docs/specs/FR.md`](docs/specs/FR.md)
> and [`docs/specs/TRACEABILITY.md`](docs/specs/TRACEABILITY.md).

**Conventions**

- IDs are stable: `FR-NNN`. Do not renumber published entries.
- Each FR below uses: title, role story, MUST statement, acceptance test refs.
- Phase 3 covers **FR-001..FR-005** (supervise surface). **FR-006..FR-008**
  capture the OS-adjacent runtime thesis (detect / watch / coalesce+mesh).
- PRs that change behavior MUST cite at least one `FR-NNN` in the PR body.

**Legacy alias map** (pre-FR-NNN root doc → current IDs)

| Legacy ID | Current ID |
|-----------|------------|
| FR-PROC-001 / FR-PROC-002 | FR-001 |
| FR-PROC-003 | FR-004 |
| FR-PROC-004 | FR-005 |
| FR-SESSION-001..003 | FR-001 / FR-003 (session via project + process group) |
| FR-CFG-001..003 | FR-002 / FR-005 |

---

## FR-001 — Managed Process Lifecycle (start / list / stop)

**As a** multi-project agent operator, **I want** to start, list, and stop
managed CLI processes by project/harness/PID, **so that** long-running agent
workloads stay supervised without manual `kill`.

**MUST:** The CLI starts a named process for a project/harness, lists running
managed processes (with optional filters), and stops them by PID, project,
harness, or `--all`.

**Acceptance:**

- `tests/fr001_process_lifecycle.rs` — AC-001.1..AC-001.3
- `tests/fr001_stop_filter.rs` — AC-001.4..AC-001.5
- `tests/integration_cli.rs` — CLI smoke (`--help`, `ps`, surfaces)

**Source:** `src/runtime.rs`, `src/commands/mod.rs`, `src/main.rs`  
**Detail:** [`docs/specs/FR.md#fr-001`](docs/specs/FR.md)

---

## FR-002 — TOML Configuration Management

**As a** sharecli user, **I want** config init/validate/show against a TOML
file in the platform config directory, **so that** project and runtime settings
persist across invocations.

**MUST:** Load, initialize, validate, and display TOML config from
`$XDG_CONFIG_HOME/sharecli/config.toml` (or OS equivalent) and persist project
registrations.

**Acceptance:**

- `tests/fr002_config_init.rs` — AC-002.1..AC-002.2
- `tests/fr002_config_load.rs` — AC-002.3..AC-002.5

**Source:** `src/config.rs`, `src/commands/mod.rs`  
**Detail:** [`docs/specs/FR.md#fr-002`](docs/specs/FR.md)

---

## FR-003 — Project Registry

**As a** fleet operator, **I want** to add/list/show/discover/remove named
projects mapped to filesystem paths, **so that** agents can target the right
workspace without hard-coded paths.

**MUST:** Maintain `[projects]` name→path registry with add, list, show,
remove, and discover (recursive `.git` scan).

**Acceptance:**

- `tests/fr003_project_registry.rs` — AC-003.1..AC-003.3, AC-003.5
- `tests/fr003_project_discover.rs` — AC-003.4

**Source:** `src/config.rs`, `src/commands/mod.rs`  
**Detail:** [`docs/specs/FR.md#fr-003`](docs/specs/FR.md)

---

## FR-004 — Process & Pool Health Status

**As an** operator, **I want** status/pool/health reports for managed processes
and the shared runtime pool, **so that** I can detect crashes and memory
pressure before agents fail.

**MUST:** Report per-harness counts/memory, shared pool idle/in-use, system
memory, and HEALTHY/DEGRADED health.

**Acceptance:**

- `tests/fr004_status_health.rs` — AC-004.1, AC-004.4..AC-004.5
- `tests/fr004_pool_status.rs` — AC-004.2..AC-004.3

**Source:** `src/runtime.rs`, `src/monitoring.rs`, `src/commands/mod.rs`  
**Detail:** [`docs/specs/FR.md#fr-004`](docs/specs/FR.md)

---

## FR-005 — Per-Project Resource Limits

**As a** multi-tenant agent host, **I want** per-project memory and process-count
limits with a check command, **so that** one project cannot starve the host.

**MUST:** Set/get per-project limits and report whether running processes comply
(`OK` / `EXCEEDED`).

**Acceptance:**

- `tests/fr005_project_limits.rs` — AC-005.1..AC-005.3
- `tests/fr005_resource_check.rs` — AC-005.4..AC-005.5

**Source:** `src/runtime.rs`, `src/commands/mod.rs`  
**Detail:** [`docs/specs/FR.md#fr-005`](docs/specs/FR.md)

---

## FR-006 — Agent Detection (proc scan, no bin wrap)

**As a** multi-agent host operator, **I want** sharecli to discover running
agents by scanning processes and matching known patterns, **so that** agents
are observed without wrapping vendor binaries (for example Claude Code).

**MUST:** Detect agents via process/pattern scan; MUST NOT require wrapping or
replacing the agent executable as the primary detection path.

**Acceptance:**

- `tests/fr006_agent_detection.rs` — AC-006.1..AC-006.3
- `crates/sharecli-core/src/detect.rs` — pattern registry unit tests

**Source:** `crates/sharecli-core` (`detect`, Hypervisor)  
**Detail:** PRD E1; `docs/specs/FR.md` may lag — root FR doc is authoritative for FR-006.

---

## FR-007 — Resource & Syscall-Relevant Watch

**As an** operator under agent load, **I want** CPU, memory, network, FD, and
IO/syscall-relevant activity watched for detected/managed agents, **so that**
contention is visible before coalesce fires.

**MUST:** Expose resource watch signals (CPU/MEM/Net/FD at minimum) and, when
FUSE intercept is enabled, IO paths used by coalesce.

**Acceptance:**

- `tests/fr007_resource_thermal_watch.rs` — AC-007.1..AC-007.3

**Source:** `crates/sharecli-core`, `crates/sharecli-fleet`, `src/monitoring.rs`  
**Detail:** PRD E2.

---

## FR-008 — Speculative Coalesce & Agent Mesh

**As a** host running many agents on overlapping worktrees, **I want**
redundant concurrent IO/work coalesced and coordinated on a shared substrate,
**so that** agents do not thrash the same files/locks unnecessarily.

**MUST:** Provide coalesce primitives (for example CoalesceCache) and a mesh /
substrate coordination path under contention; thermal gate before speculative
work.

**Acceptance:**

- `tests/fr008_coalesce_mesh.rs` — AC-008.1..AC-008.4

**Source:** `crates/sharecli-ipc`, `crates/sharecli-core`, `crates/sharecli-fuse`  
**Detail:** PRD E3.

---

## Related surfaces

Cast/pane tests use `FR-CAST-00N` annotations in `tests/cast_*.rs` (extension
FRs). Harbor/agent-eval soft harness is **out of sharecli FRs** — see ADR 0002
and `phenotype-tooling/crates/benchora/harbor-soft`.

## NFR notes

- **NFR-001** Platform: Linux, macOS, Windows (`#[cfg(unix)]` / `#[cfg(windows)]`).
- **NFR-002** Observability: structured `tracing` logs (`--verbose` / `--quiet`).
- **NFR-003** Errors: commands return `anyhow::Result<()>`; missing config → default.
- **NFR-004** Eval boundary: Harbor soak is not a sharecli product acceptance
  criterion (ADR 0002).
