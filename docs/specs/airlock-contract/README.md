# Airlock Coordination Contract (pinned reference)

This directory preserves the interface contract of the `airlock` Go MCP
coordination daemon (`github.com/adamorad/airlock/v2`), captured during the
airlock→sharecli convergence (ADR 0008). It is a **specification**, not a port:
sharecli does not import Go, and no first-party MCP server is introduced (per
ADR 0004).

Source of truth at capture time: `airlock@HEAD` `internal/mcp/tools.go`.

## Capability groups (22 tools)

| Group | Tools |
|---|---|
| Locks | `lock_resource`, `unlock_resource`, `renew_lock`, `list_locks`, `lock_resources` |
| Notes | `set_note`, `get_note`, `list_notes`, `delete_note`, `set_note_if` |
| Counters | `increment_counter` |
| Agents | `register_agent`, `unregister_agent`, `list_agents` |
| Events | `signal_event`, `wait_for_event`, `clear_event` |
| Tasks | `push_task`, `claim_next_task`, `complete_task`, `fail_task`, `list_tasks` |

## Semantics summary

- **Locks** are exclusive, TTL-based, and atomically acquirable in bulk
  (`lock_resources` = all-or-nothing). Bearers identified by `lock_token` /
  `agent_id`.
- **Notes** are a shared key/value store with optional `ttl_seconds` and an
  atomic compare-and-swap (`set_note_if`) for lock-free coordination.
- **Counters** are named, atomically incremented, auto-created.
- **Agents** register with a heartbeat TTL; lapsed agents auto-release their
  locks.
- **Events** are generation-counter based: `wait_for_event` takes
  `last_seen_generation` to avoid missing signals between polls.
- **Tasks** are leased from priority queues with a `lease_token`;
  `claim_next_task` auto-requeues on lease expiry or agent loss.

## Ownership / provenance

- Upstream: `adamorad/airlock` (Go). Local fork had a dangling origin (404).
- KooshaPari-authored content (the Rust `phenovcs-airlock-v2` crate) was
  reconciled home to `PhenoVCS/crates/airlock-v2` in a separate action; it was
  **not** part of the MCP daemon contract captured here.