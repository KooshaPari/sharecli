# ShareCLI Implementation Plan

> **Agent backlog:** claimable tasks live in [`WORK_DAG.md`](WORK_DAG.md)
> (FR-linked, effort ≤ M). This file is the human roadmap summary only.

## Overview

Build a shared CLI process manager for multi-project agent orchestration.

## Current focus (C03 Agent Readiness)

Lift L30 Agent Readiness toward grade **C** by making FRs machine-readable,
tasks claimable, and agent entrypoints complete. See `WORK_DAG.md` tasks
T-100..T-160 (this wave) and T-200..T-310 (next).

## Phases (historical roadmap)

### Phase 1: Process Management — FR-001, FR-005
- Process spawning and monitoring
- Resource limits (CPU, memory)
- Process lifecycle + signal handling

### Phase 2: Multi-Project Support — FR-002, FR-003
- Project configuration / registry
- Cross-project isolation boundaries
- Shared state via config persistence

### Phase 3: Health & Orchestration — FR-004
- Pool / health status surfaces
- Agent-facing status commands
- Traceability: `docs/specs/FR.md` + `TRACEABILITY.md`

### Phase 4: CLI / DX polish
- Completions, journeys, friction log
- Golden output / visual fixtures
- FR-006+ (workflows, cast promotion)

## How to pick work

1. Open [`WORK_DAG.md`](WORK_DAG.md).
2. Claim a **READY** task whose predecessors are **DONE**.
3. Reference the FR ID in the PR body (enforced by `pr-lint.yml`).
