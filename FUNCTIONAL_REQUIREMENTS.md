# Functional Requirements — sharecli

> Agent-facing FR index using stable `FR-NNN` IDs. Full acceptance criteria,
> source maps, and AC tables live in [`docs/specs/FR.md`](docs/specs/FR.md)
> and [`docs/specs/TRACEABILITY.md`](docs/specs/TRACEABILITY.md).

**Conventions**

- IDs are stable: `FR-NNN`. Do not renumber published entries.
- Each FR below uses: title, role story, MUST statement, acceptance test refs.
- Phase 3 covers **FR-001..FR-005**. Append new IDs (FR-006+) in later phases.
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

- Target: `tests/fr002_config_load.rs`, `tests/fr002_config_init.rs`
  (see TRACEABILITY; files are claimable work if missing)

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

- Target: `tests/fr003_project_registry.rs`, `tests/fr003_project_discover.rs`

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

- Target: `tests/fr004_status_health.rs`, `tests/fr004_pool_status.rs`

**Source:** `src/runtime.rs`, `src/monitoring.rs`, `src/commands/mod.rs`  
**Detail:** [`docs/specs/FR.md#fr-004`](docs/specs/FR.md)

---

## FR-005 — Per-Project Resource Limits

**As a** multi-tenant agent host, **I want** per-project memory and process-count
limits with a check command, **so that** one project cannot starve the host.

**MUST:** Set/get per-project limits and report whether running processes comply
(`OK` / `EXCEEDED`).

**Acceptance:**

- Target: `tests/fr005_project_limits.rs`, `tests/fr005_resource_check.rs`

**Source:** `src/runtime.rs`, `src/commands/mod.rs`  
**Detail:** [`docs/specs/FR.md#fr-005`](docs/specs/FR.md)

---

## Related surfaces (non-Phase-3 IDs)

Cast/pane tests use `FR-CAST-00N` annotations in `tests/cast_*.rs`. Treat those
as extension FRs until promoted into `docs/specs/FR.md` as FR-006+.

## NFR notes

- **NFR-001** Platform: Linux, macOS, Windows (`#[cfg(unix)]` / `#[cfg(windows)]`).
- **NFR-002** Observability: structured `tracing` logs (`--verbose` / `--quiet`).
- **NFR-003** Errors: commands return `anyhow::Result<()>`; missing config → default.
