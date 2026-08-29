# ADR 0007 — Ports & Adapters (Hexagonal) Architecture

**Status:** Accepted
**Date:** 2026-08-28
**Deciders:** sharecli maintainers
**Supersedes:** —
**Related:** [lib-sprawl-plan.md](../ops/lib-sprawl-plan.md) (Phases 2–4 crate extraction)

---

## Context

The C00 L0 audit gap (score 2, effort M) calls for a formal ports/adapters ADR
to lock the hexagonal architecture boundaries between domain core and
infrastructure I/O. Without a written contract, crate extraction Phases 2–4
risk ad-hoc dependency edges that leak infrastructure concerns into the domain
core.

sharecli's architecture already follows an implicit hexagonal layout:

| Layer | Owner | I/O direction |
|-------|-------|---------------|
| **Domain core** | `sharecli-core` (Hypervisor, ThermalGate, CoalesceCache) | None — pure business rules |
| **Application services** | `sharecli-runtime` (ProcessPool, SharedRuntime, scheduler) | Inbound from CLI commands |
| **Driving adapters** | `sharecli` binary, `sharecli-tray-*`, `sharecli-thermal-tui` | CLI + desktop GUI |
| **Driven adapters** | `sharecli-fuse` (FUSE), `sharecli-fleet` (device registry), `sharecli-ipc` (CoalesceCache), `spawn-core-sys` (Zig hot core) | OS, filesystem, network |
| **Infrastructure** | `sharecli-mesh` (Maildir queue), `sharecli-session` (persistence) | Disk, network |

The `lib-sprawl-plan.md` proposes Target crate boundaries with explicit
dependency arrows. This ADR formalizes those as architectural invariants.

## Decision

### 1. Domain core is dependency-free

`sharecli-core` (Hypervisor, ThermalGate) must **never** depend on:
- Tokio async runtime
- Axum HTTP framework
- Filesystem or network I/O
- Any `sharecli-*` infrastructure crate

Domain types use plain Rust enums and structs. Async wrappers live in
application services.

### 2. Dependency arrows are one-directional

```text
Driving adapters (CLI, tray, TUI)
    → Application services (runtime, config, serve)
        → Domain core (sharecli-core)
        → Driven adapters (fuse, fleet, ipc, session, spawn-core-sys)
            → OS / hardware
```

**Rule:** No upward or lateral arrows. Driven adapters never import from
driving adapters. Domain core never imports from any adapter.

### 3. Port traits for driven adapters

Each driven adapter exposes a trait (port) that the application service
depends on, not the concrete implementation:

```rust
// In sharecli-core or a shared types crate
pub trait ThermalSensor {
    fn read_pressure(&self) -> Result<u8, SensorError>;
}

pub trait ProcessSpawner {
    fn spawn(&self, spec: &ProcessSpec) -> Result<ManagedProcess, SpawnError>;
}
```

Adapters implement these traits. Runtime wires concrete adapters at startup.

### 4. Crate boundary enforcement

- `cargo-deny` bans cross-boundary dependency edges (Tier C → Tier A/B)
- CI `cargo-machete` catches unused transitive deps across crate boundaries
- Each crate's `Cargo.toml` documents its layer in a header comment

### 5. Error isolation

Each adapter owns its error type. Application services map adapter errors
to domain-level error variants. The `ErrorEnvelope` (serve HTTP) maps
domain errors to JSON at the boundary, not inside handlers.

## Consequences

- Adapters can be swapped (e.g., FUSE → native intercept) without changing
  domain core or application services.
- Unit testing domain core requires no async runtime or I/O mocks.
- New adapters (e.g., a different process spawner) only implement a trait
  and register at startup.
- Phases 2–4 of `lib-sprawl-plan.md` can be executed against these rules
  with CI enforcement.

## References

- lib-sprawl-plan.md: [`../ops/lib-sprawl-plan.md`](../ops/lib-sprawl-plan.md)
- Error envelope: [`../ops/error-envelope.md`](../ops/error-envelope.md)
- Concurrency docs: [`../ops/concurrency.md`](../ops/concurrency.md)
- ADR 0006 (Feb harness lineage): [`0006-feb-harness-recovery-lineage.md`](0006-feb-harness-recovery-lineage.md)
