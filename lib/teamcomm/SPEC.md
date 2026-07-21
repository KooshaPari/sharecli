# SPEC.md — phenotype-teamcomm

## Purpose

Inter-agent coordination infrastructure for the Phenotype ecosystem. Provides a shared, durable, and observable communication layer that lets multiple AI-driven coding agents (and humans) cooperate on the same repository without conflicting edits, stale context, or lost messages.

## Scope

### In Scope (MVP)

1. **Session Management** — Register, deregister, heartbeat, and list active agent sessions.
2. **File Reservations** — Advisory locking: claim, release, and list reservations with TTL-based expiry.
3. **Inbox** — Pull-based messaging: post, list, and read messages between agents.
4. **Live State** — Publish and query agent focus (file, branch, worktree, status).
5. **Discovery** — Query active sessions by path, branch, capability.
6. **Event Bus** — Append-only hook events for lifecycle signals (session start/end, file changes, reservations).

### Out of Scope (Future Milestones)

- Thread-first-class entities (M1)
- Structured message types (M1)
- 5-state status enum migration (M1)
- SQLite-backed persistence (M1)
- MCP tool-call surface (M2)
- Cross-machine networking (M3)

## Architecture

- **Hub-and-Spoke**: A single `teamcomm-daemon` process acts as the central hub. Agents connect as lightweight spokes via JSON-RPC 2.0 over a Unix-domain socket.
- **In-Memory State**: M0 stores all state in memory (HashMap-based). M1 will migrate to SQLite.
- **Pull-Based Inbox**: Agents poll for unread messages; daemon marks them as read upon delivery.
- **Advisory Locks**: Cooperative reservations tracked by daemon but not enforced by OS kernel.

## Wire Protocol

JSON-RPC 2.0 over newline-delimited lines on a Unix-domain socket.

### Core Methods

| Method | Description |
|--------|-------------|
| `session.register` | Register a new agent session |
| `session.deregister` | Remove a session |
| `session.heartbeat` | Refresh session lease |
| `session.list` | List all active sessions |
| `session.get` | Get details for a single session |
| `reservation.claim` | Claim an advisory lock on a path |
| `reservation.release` | Release a reservation |
| `reservation.list` | List active reservations |
| `inbox.post` | Post a message to another session |
| `inbox.list` | List messages (unread or all) |
| `inbox.read` | Read a single message by id |
| `state.set` | Publish live state (focus, status) |
| `state.get` | Get live state for a session |
| `discover.agents` | Query sessions by filters |

## Quality Gates

- `cargo check --workspace` must pass
- `cargo test --workspace` must pass
- `cargo fmt --all` must run clean
- `cargo clippy --workspace --all-targets -- -D warnings` must pass

## References

- `docs/SDD.md` — Software Design Document
- `AGENTS.md` — Agent working guidelines
- `README.md` — Project overview
