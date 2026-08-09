# AgilePlus Methodology Spec — thegent-sharecli

## Overview

`thegent-sharecli` is a Rust CLI process manager that serves as a centralized control plane for multi-project agent orchestration within thegent infrastructure. It implements a shared runtime pool model to reduce memory overhead across agent processes.

## AgilePlus Application

### Work Decomposition

AgilePlus work packages (WPs) for this rig decompose as follows:

| WP Type | Description | Examples in sharecli |
|---|---|---|
| `feat` | New runtime capabilities | Shared pool, project limits |
| `fix` | Process lifecycle corrections | Graceful stop, orphan cleanup |
| `docs` | Specification and documentation | This spec, AGENTS.md |
| `chore` | Tooling and sync | Dependency updates, CI |
| `refactor` | Internal improvements | Interface consolidation |

### Development Methodology

All feature and chain workflow changes follow the **TDD + BDD + SDD** triad:

- **TDD (Test-Driven Development):** Core lifecycle logic (`start`, `stop`, `ps`) requires unit tests in `src/` preceding implementation.
- **BDD (Behavior-Driven Development):** CLI interface changes require `.feature` scenarios describing user-facing behavior.
- **SDD (Specification-Driven Development):** Architectural changes (e.g., shared pool design, project registry schema) require a written spec in `docs/plans/` before code.

### Phased WBS with DAG Dependencies

Work packages are organized in phases with explicit dependencies:

```
Phase 1: Foundation
  └── sharecli binary structure, basic commands

Phase 2: Core Runtime
  ├── depends on: Phase 1
  ├── shared runtime pool implementation
  └── project registry and limits

Phase 3: Operations
  ├── depends on: Phase 2
  ├── monitoring and health checks
  └── process lifecycle management

Phase 4: Ecosystem Integration
  ├── depends on: Phase 3
  └── thegent infrastructure integration
```

### Architectural Boundaries

Sharecli follows **Hexagonal + Clean + SOLID** principles:

- **Hexagonal:** Core business logic (`src/runtime.rs`, `src/projects.rs`) is isolated from CLI adapters (`src/commands/`) and infrastructure adapters (monitoring, config I/O).
- **Clean:** Explicit layer separation — `commands` → `lib` → `runtime/projects`.
- **SOLID:**
  - `S`: Each command (`start`, `stop`, `ps`, `status`, `config`, `project`) is a single-responsibility unit.
  - `O`: New subcommands extend `src/commands/mod.rs` without modifying existing command logic.
  - `L`: Project types and runtime types follow recognizable interface contracts.
  - `I`: Narrow interfaces for `Runtime`, `ProjectRegistry`, `Monitor`.
  - `D`: `lib.rs` exposes a clean public API; `main.rs` wires CLI parsing.

### Error Handling

**Explicit failures over silent degradation.** Sharecli uses `Result`-based error propagation:

- CLI commands return actionable error messages (e.g., "Project 'X' not found in registry" rather than silent no-op).
- Monitoring subsystem surfaces health check failures explicitly.
- Process start failures propagate with exit codes and diagnostics.

### Integration with AgilePlus Ecosystem

Sharecli participates in the broader AgilePlus town as a **shared infrastructure component**:

- **Work Package Tracking:** All work affecting sharecli is tracked as WPs in AgilePlus with explicit phase assignment.
- **Cross-Rig Dependencies:** When another rig (e.g., `portage`, `thegent-metrics`) depends on sharecli behavior, that dependency is declared in the WP metadata and enforced via convoy DAG ordering.
- **Convoy Coordination:** Multi-repo changes involving sharecli use convoys to ensure atomic cross-rig updates.

## Conventions

### Commit Messages

Format: `{type}: {short description}`

Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`

Example: `feat: add shared runtime pool and project limits`

### File Structure

```
src/
├── commands/       # CLI subcommands (one file per command)
├── lib.rs          # Public API exports
├── main.rs         # CLI entry point
├── config.rs       # Global configuration
├── runtime.rs      # Process spawning and lifecycle
├── projects.rs     # Project registry management
└── monitoring.rs   # Health checks and stats

docs/plans/         # AgilePlus specs and WBS documentation
```

### Code Standards

- Run `cargo clippy -- -D warnings` before committing.
- All public functions have doc comments.
- Error types implement `std::error::Error` and provide context.
- Tests live in `src/` alongside the code they test (module tests).

## Change Process

1. **Spec First:** For any `feat` or `refactor`, write the spec in `docs/plans/` and get it reviewed.
2. **TDD Cycle:** Write failing tests, implement to pass, refactor.
3. **Phase gates:** Phase N+1 work cannot merge until Phase N is stable.
4. **Convoy linking:** For cross-rig changes, link the WP bead to the owning convoy.
