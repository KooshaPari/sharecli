# PRD — sharecli

## Overview

`sharecli` is an OS/kernel-adjacent **agent runtime** for multi-agent hosts. It
detects running agents by process scan and known patterns (it does **not** wrap
vendor agent binaries), watches resource and syscall-relevant activity, and
under high concurrency coalesces redundant work across agents. An agent mesh
coordinates that shared substrate.

The declarative TOML process supervisor, HTTP serve surface, and tray/dashboard
are operator UX on top of the hypervisor stack (`sharecli-core`,
`sharecli-fuse`, `sharecli-ipc`, `sharecli-fleet`).

**Out of product scope:** Harbor / SWE-bench / Terminal-Bench agent-eval
harnesses. Suite-facing soft eval lives in `phenotype-tooling/crates/benchora`
(`harbor-soft/`). Harbor fork/env lives in `KooshaPari/portage-temp`.

## Goals

- Detect agents via proc scan + pattern match (no vendor-bin wrap)
- Watch CPU, memory, network, FDs, and IO/syscall-relevant activity
- Speculatively coalesce concurrent agent work under contention
- Coordinate agents through an agent mesh / shared substrate
- Expose operator supervise/serve/tray surfaces for lifecycle and observability

## Epics

### E1 — Agent detection & mesh

- E1.1 Proc-scan discovery of known agent patterns
- E1.2 Mesh membership / substrate coordination across detected agents
- E1.3 No wrapping of Claude / other vendor agent binaries

### E2 — Watch & thermal gate

- E2.1 CPU / MEM / Net / FD observation for managed and detected agents
- E2.2 Thermal / contention gate before speculative coalesce
- E2.3 Operator status/health surfaces (CLI + HTTP)

### E3 — Coalesce & IO intercept

- E3.1 Lock-wait / read coalesce across agents (`sharecli-ipc` CoalesceCache)
- E3.2 Optional FUSE cwd intercept for shared build-cache reads
- E3.3 Speculative merge of redundant concurrent checkout/IO work

### E4 — Operator supervise surface (supporting)

- E4.1 Declarative TOML lifecycle (start/stop/restart)
- E4.2 Config hot-reload, health checks, metrics
- E4.3 Tray / dashboard / plugins

## Acceptance Criteria

- Agents appear in discovery without wrapping their binaries
- Resource watch reports CPU/MEM/Net/FD (and IO path where FUSE enabled)
- Under multi-agent contention, coalesce paths reduce redundant IO/work
- Mesh/substrate state is observable via status/serve
- Harbor soft CI is not required in-repo for product A+
