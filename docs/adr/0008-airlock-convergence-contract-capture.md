# ADR 0008 — Absorb Airlock as Contract Capture (Parity Mapping), Not Code Splice

**Status:** Accepted
**Date:** 2026-09-02
**Deciders:** KooshaPari (operator "converge airlock to sharecli")
**Supersedes:** —
**Related:** [0004-no-mcp-server.md](0004-no-mcp-server.md), [0007-ports-adapters-hexagonal.md](0007-ports-adapters-hexagonal.md)

---

## Context

`airlock` (`github.com/adamorad/airlock/v2`) is a **Go MCP-over-HTTP
coordination daemon**: agents hold named locks, share key/value notes,
increment counters, register presence, signal/wait on events, and lease tasks
from queues — 22 MCP tools total, persisted to SQLite, surfaced as an MCP HTTP
server with capability tokens.

`sharecli` is a **Rust hexagonal** agent-runtime monorepo. ADR 0007 locks the
ports/adapters boundary, and ADR 0004 explicitly decides **no first-party MCP
server in sharecli**.

The operator directed: *"converge airlock to sharecli."* The naive reading is a
code splice (Go daemon → a new sharecli crate). That is **wrong** on two
structural grounds:

1. **Language boundary** — airlock is Go, sharecli is Rust. A Go binary/crate
   inside a Cargo workspace is a foreign body with no `go.mod` ancestor.
2. **ADR 0004 boundary** — sharecli explicitly forecloses a first-party MCP
   *server*. Airlock *is* an MCP server. Reintroducing it as a crate would
   revoke ADR 0004 without a deliberate decision.

## Decision

Convergence is performed as **contract capture + parity mapping**, not code
splice. Concretely:

1. **Pin airlock's 22-tool contract** as a reference specification
   (`docs/specs/airlock-contract/`) so the coordination *interface* is
   preserved even after the Go fork is archived.
2. **Map each airlock capability to an existing (or to-be-extracted) sharecli
   primitive**, recording gaps where sharecli has no equivalent yet.
3. **Do not** import Go source, do not add a `go.mod`, do not add an MCP server
   crate. The MCP transport remains out of scope per ADR 0004; if an MCP
   transport is ever required, ADR 0004 is re-opened as a deliberate, separate
   decision.

This is the boundary policy's `CONVERT_TO_SPEC` / `PARITY_MAP` shape: absorb
the *interface*, drop the *implementation*, defer the *transport*.

## Parity mapping (airlock capability → sharecli primitive)

| Airlock capability (tools) | sharecli primitive | Status |
|---|---|---|
| Locks: `lock_resource`, `unlock_resource`, `renew_lock`, `list_locks`, `lock_resources` | `sharecli-core::CoalesceCache` (lock/coalesce semantics), `sharecli-core::SlotQueue` (Lock-Wait-Cache), `sharecli-ipc` (Lock-Wait-Cache dedup) | **Mapped** — coalescing ≈ resource contention; need TTL + token bearer parity |
| Notes: `set_note`, `get_note`, `list_notes`, `delete_note`, `set_note_if` | shared state store (no current equivalent) | **Gap** — key/value store with CAS (`set_note_if`) |
| Counters: `increment_counter` | atomic counter (no current equivalent) | **Gap** — trivial to add |
| Agents/presence: `register_agent`, `unregister_agent`, `list_agents` | `sharecli-session` / heartbeat | **Partial** — presence + TTL + lock-release-on-lapse |
| Events: `signal_event`, `wait_for_event`, `clear_event` | `sharecli-mesh` (event generation/broadcast) | **Partial** — generation-counter semantics (`last_seen_generation`) not yet modeled |
| Tasks: `push_task`, `claim_next_task`, `complete_task`, `fail_task`, `list_tasks` | `sharecli-mesh` queue (priority lease) | **Partial** — lease token + auto-requeue parity |

**Outcome:** lock/coalescing and task-queue primitives already exist in
sharecli; the concrete gaps are (a) key/value state with CAS, (b) atomic
counters, (c) explicit generation-counter event semantics, (d) lease-token
task lifecycle. These are captured as a future `sharecli-coordination` crate
extraction, **not** a Go port.

## Consequences

- Go `adamorad/airlock` fork may be archived after this contract is captured;
  no sharecli code depends on it.
- The coordination interface is preserved forever as a pinned spec, so a
  future `sharecli-coordination` crate can implement it in Rust behind the
  existing hexagonal boundary.
- ADR 0004 remains intact (no first-party MCP server).
- Costs: the parity gaps listed above are not yet implemented; this ADR only
  captures the *contract*, it does not ship Rust implementations of them.