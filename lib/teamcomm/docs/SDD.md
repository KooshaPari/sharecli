# TeamComm Software Design Document (SDD)

**Version:** 1.0
**Date:** 2026-06-12
**Status:** Draft
**Scope:** Inter-agent coordination infrastructure for the Phenotype ecosystem

---

## 1. Overview

`phenotype-teamcomm` is the inter-agent coordination substrate for the Phenotype engineering portfolio. It provides a shared, durable, and observable communication layer that lets multiple AI-driven coding agents (and the humans steering them) cooperate on the same repository without conflicting edits, stale context, or lost messages.

This document is the authoritative design specification. It normalizes terminology and patterns drawn from the AGSLAG research archive (`agslag-docs/`) and the current `phenotype-teamcomm` Rust implementation.

---

## 2. Core Design Patterns

### 2.1 Hub-and-Spoke Model

The system implements a **hub-and-spoke** topology for real-time coordination.

- **Hub:** A single `teamcomm-daemon` process runs on the local machine and acts as the central coordination server. It exposes a JSON-RPC 2.0 surface over a local domain socket or TCP port.
- **Spokes:** Each agent (forge, codex, claude, copilot, etc.) connects to the hub as a lightweight client. Agents send heartbeats, reservations, inbox messages, and state updates to the hub.
- **Non-hub traffic:** Direct agent-to-agent stdio, SSE, or WebSocket streams remain decentralized and are not routed through the daemon. The hub is the source of truth for coordination state only, not for data pipelines.

**Why this pattern:**
- Eliminates port conflicts (one known daemon endpoint).
- Simplifies configuration (agents only need the hub address).
- Provides a single point for monitoring, lease management, and conflict detection.
- Mirrors the AGSLAG `centralServer.js` / `WebSocketServerManager` architecture documented in `agslag-docs/architecture/hub-spoke-model.md` and `agslag-docs/agents/multi_agent/Cross-Agent Server Implementation in AGS.md`.

**Key terminology:**
- `daemon` — the central hub process.
- `session` — a spoke/agent registration with the hub.
- `heartbeat` — periodic liveness signal from spoke to hub.
- `auto-identify` — agents detect their own environment (forge, codex, claude, etc.) and self-register with a stable generated ID.

---

### 2.2 TeamComms

**TeamComms** is the subsystem name for all inter-agent coordination primitives. The AGSLAG research defines a `team-communications` MCP server with a structured communication protocol. `phenotype-teamcomm` is the Phenotype implementation of that concept.

The subsystem provides five functional domains:

| Domain | Responsibility | Key Types |
|--------|----------------|-----------|
| **Sessions** | Registered agent identity with heartbeat, role, and bounded lifetime | `Session`, `SessionRegistration`, `AgentType` |
| **File Reservations** | Exclusive claims over a glob/expiry pair so two agents never edit the same hunk concurrently | `Reservation`, `ClaimRequest`, `ClaimResult`, `ReservationMode` |
| **Inbox** | Durable, addressed messages for offline peers | `InboxMessage`, `InboxQuery`, `Priority` |
| **Live State** | Queryable "who is doing what, where" snapshot | `LiveState`, `AgentStatus`, `DiscoveryQuery` |
| **Hook Events** | Append-only lifecycle signals emitted by host agents so the daemon can react | `HookEvent`, `HookEventType` |

These five domains map directly to the AGSLAG communication primitives (`send_message`, `broadcast_message`, `get_messages`, `create_thread`, `report_status`, etc.) documented in `agslag-docs/research/core_prompt_engineering_framework.md` and `agslag-docs/tools/communication.md`.

---

### 2.3 Pull-Based Inbox

The inbox is explicitly **pull-based**, not push-based.

**Pattern:**
1. An agent periodically calls `check_new_messages` (or the equivalent `inbox.query` with `unread_only: true`).
2. If unread messages exist, the agent calls `get_messages` (or `inbox.fetch`) to retrieve them.
3. Retrieved messages are **automatically marked as read** by the daemon upon delivery.

**Why this pattern:**
- Agents are autonomous processes; a push model would require the daemon to maintain outbound connections to every agent, which is fragile when agents restart, sleep, or run in containers.
- Pull-based retrieval aligns with the MCP tool-call model where the agent initiates all communication.
- Matches the AGSLAG `check_new_messages` / `get_messages` flow documented in `agslag-docs/research/core_prompt_engineering_framework.md`.

**Inbox message structure:**
```rust
struct InboxMessage {
    message_id: String,      // UUID v4
    from_session: String,    // sender session id
    to_session: String,      // recipient session id ("*" for broadcast)
    subject: String,         // short single-line subject
    body: String,            // full message body (plain text)
    priority: Priority,      // low | normal | high
    ts: DateTime<Utc>,       // posted timestamp
    read: bool,              // true once delivered to recipient
}
```

**Query structure:**
```rust
struct InboxQuery {
    unread_only: bool,
    limit: u32,
}
```

---

### 2.4 Advisory Locks (Reservations)

The file reservation system is **advisory**, not mandatory.

**Pattern:**
- An agent requests a `Reservation` on a path with a `ReservationMode` (`Read`, `Write`, `Exclusive`) and a TTL.
- The daemon records the claim and returns either the granted `Reservation` or a list of `conflicts` (existing reservations that block the claim).
- The agent is expected to respect the conflict report voluntarily. The daemon does not enforce the lock at the OS kernel level.

**Why this pattern:**
- Agents are cooperative peers, not hostile processes. Advisory locking is simpler, faster, and does not require filesystem-level integration.
- The AGSLAG `file_locking.ts` and `anti_jam.ts` tools use the same cooperative model.
- The term "advisory" is explicitly used in the hook convention: "Scripts should be idempotent — the daemon is the source of truth, hooks are advisory notifications."

**Reservation mode ordering:**
```
Read < Write < Exclusive
```
A claim conflicts only with claims of **equal or stronger** mode on the same path.

**Reservation structure:**
```rust
struct Reservation {
    reservation_id: String,
    session_id: String,
    path: PathBuf,
    mode: ReservationMode,
    acquired_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}
```

---

### 2.5 5-State Status Enum

The canonical agent status enum is **5-state**.

**Canonical states:**
```
online  — agent is connected and responsive
idle    — agent is registered but not actively working on a task
busy    — agent is actively working (writing code, running tests, etc.)
blocked — agent is waiting on a human, another agent, or a reservation
offline — agent is disconnected or has not heartbeated within lease TTL
```

**Why 5-state:**
- The AGSLAG research defines `['online', 'idle', 'busy', 'blocked', 'offline']` as the standard agent status vocabulary in `agslag-docs/research/core_prompt_engineering_framework.md`.
- The `agent-user-status` project uses the same model.
- The current `phenotype-teamcomm` implementation uses a 4-state enum (`Idle`, `Working`, `Blocked`, `Done`). This SDD supersedes that implementation: the enum must be updated to the 5-state model.

**Live state snapshot:**
```rust
struct LiveState {
    session_id: String,
    focus_file: Option<PathBuf>,
    focus_branch: Option<String>,
    worktree: Option<PathBuf>,
    status: AgentStatus,
    last_heartbeat: DateTime<Utc>,
}
```

---

### 2.6 Threads as First-Class

**Threads are first-class entities**, not just message metadata.

**Pattern:**
- A `Thread` has its own identifier, title, creator, participant list, and creation timestamp.
- Threads support dynamic membership: agents can `join`, `add_participant`, or `remove_participant`.
- Messages are posted to a thread by including `thread_id` in the `InboxMessage` envelope.
- Thread metadata is queryable independently of message content.

**Why this pattern:**
- The AGSLAG `create_thread`, `join_thread`, `add_participant_to_thread`, `remove_participant_from_thread`, and `get_thread_details` tools treat threads as full objects with their own lifecycle.
- First-class threads enable scoped collaboration: a group of agents can spin up a thread for a specific task or subtask without polluting the global inbox.
- Threads map naturally to the `collaborative_problem_solving` workflow phases in AGSLAG.

**Required thread operations:**
- `thread.create`
- `thread.join`
- `thread.add_participant`
- `thread.remove_participant`
- `thread.get_details`
- `thread.list` (for an agent's active threads)

---

### 2.7 Structured Message Types

All messages carry a **typed `message_type` enum** in their envelope.

**Canonical message types (from AGSLAG research):**
```
status_update
  task_assignment
  query
  response
  blocker_report
  knowledge_share
  feedback
  request_assistance
  decision_log
```

**Why this pattern:**
- The AGSLAG `Structured Communication Protocol` explicitly requires a `message_type` field so that agents can filter, route, and prioritize messages without parsing natural language.
- The `message_type` enum is the contract between the `team-communications` MCP server and the agents that consume it.
- The current `phenotype-teamcomm` `InboxMessage` struct lacks a `message_type` field. This SDD supersedes that implementation: the field must be added.

**Enriched inbox message structure:**
```rust
struct InboxMessage {
    message_id: String,
    message_type: MessageType,   // <-- added per this SDD
    from_session: String,
    to_session: String,
    thread_id: Option<String>,
    subject: String,
    body: String,
    priority: Priority,
    ts: DateTime<Utc>,
    read: bool,
}
```

---

## 3. Wire Protocol

### 3.1 JSON-RPC 2.0 Envelope

All daemon methods use JSON-RPC 2.0 over a local transport (domain socket or TCP). The protocol crate defines pure envelope types:

```rust
enum RpcId { String(String), Number(u64) }
struct JsonRpcRequest { jsonrpc: String, id: Option<RpcId>, method: String, params: Value }
struct JsonRpcResponse { jsonrpc: String, id: RpcId, result: Value }
struct JsonRpcError { code: i32, message: String, data: Option<Value> }
struct JsonRpcErrorResponse { jsonrpc: String, id: RpcId, error: JsonRpcError }
```

- `id == None` denotes a notification; the server must not reply.
- The `params` and `result` fields are `serde_json::Value` so the protocol crate stays agnostic of the concrete method set.

### 3.2 Session Lifecycle

```
1. Agent starts -> calls session.register
2. Daemon assigns session_id + lease_ttl_sec
3. Agent begins heartbeat loop (recommended 30s interval)
4. Daemon reaps session if no heartbeat within 90s (lease TTL)
5. Agent calls session.deregister on clean exit
```

### 3.3 Heartbeat and Lease

| Constant | Value | Purpose |
|----------|-------|---------|
| `HEARTBEAT_INTERVAL_SEC` | 30 | Recommended interval between agent heartbeats |
| `LEASE_TTL_SEC` | 90 | Maximum silence before daemon may reap the session |

The `session.heartbeat` response includes `next_heartbeat_sec` so the daemon can adjust cadence dynamically without redeploying agents.

---

## 4. Supporting Patterns

### 4.1 Capability-Based Task Assignment

Agents register free-form capability tags (e.g., `["rust", "git:write", "network"]`). The daemon's `DiscoveryQuery` supports `capabilities: Vec<String>` with AND semantics: a query returns only sessions that declare **all** requested tags.

This enables the AGSLAG pattern of matching tasks to agents by required capabilities.

### 4.2 Hook Event Stream

Agents emit typed `HookEvent` records into an append-only event stream. Other agents, the CLI, and the MCP surface can subscribe to filtered events.

**Canonical event types:**
```
SessionStarted
SessionEnded
FileRead
FileWritten
FileEdited
PlanAnnounced
FocusChanged
Heartbeat
ReservationClaimed
ReservationReleased
InboxMessagePosted
```

### 4.3 Discovery Query

The `discover_sessions` method accepts a `DiscoveryQuery` with optional filters:

```rust
struct DiscoveryQuery {
    path: Option<PathBuf>,         // focus_file under this path
    branch: Option<String>,        // focus_branch equals this
    repo: Option<PathBuf>,         // working_dir in this repo
    capabilities: Vec<String>,     // declare all of these tags
}
```

This is the "who else is working on what" primitive.

### 4.4 Traceability

All design requirements in this SDD must be linked to AgilePlus functional requirements via `trace.json` files conforming to `AgilePlus/traces/SCHEMA.md`.

---

## 5. Glossary

| Term | Definition |
|------|------------|
| **Hub-and-Spoke** | Central daemon (hub) with agent clients (spokes); real-time coordination is centralized, data traffic is decentralized |
| **TeamComms** | The inter-agent coordination subsystem (sessions, reservations, inbox, live state, hook events) |
| **Pull-Based Inbox** | Agents poll for unread messages; the daemon marks messages as read upon delivery |
| **Advisory Lock** | Cooperative reservation tracked by the daemon but not enforced by the OS kernel |
| **5-State Status** | `online`, `idle`, `busy`, `blocked`, `offline` — the canonical agent status vocabulary |
| **First-Class Thread** | A thread is an independent entity with its own ID, title, participants, and lifecycle operations |
| **Structured Message Type** | A typed enum (`status_update`, `task_assignment`, etc.) carried in every message envelope |
| **Session** | A registered agent identity with a heartbeat, role, and bounded lifetime |
| **Reservation** | An advisory claim on a filesystem path with a mode and TTL |
| **Hook Event** | An append-only lifecycle signal emitted by a host agent |
| **Discovery** | A queryable "who else is working on what" snapshot of all active sessions |

---

## 6. References

| Document | Path | Relevance |
|----------|------|-----------|
| AGSLAG Hub-and-Spoke Model | `agslag-docs/architecture/hub-spoke-model.md` | Centralized WebSocket hub architecture |
| AGSLAG Cross-Agent Server | `agslag-docs/agents/multi_agent/Cross-Agent Server Implementation in AGS.md` | WebSocket server, task delegation, agent registry |
| AGSLAG Core Prompt Engineering | `agslag-docs/research/core_prompt_engineering_framework.md` | Structured communication protocol, 5-state status, message types |
| AGSLAG Communication Tools | `agslag-docs/tools/communication.md` | `send_message`, `get_messages`, `create_thread`, `broadcast_message` schemas |
| AGSLAG Collaboration Tools | `agslag-docs/tools/collaboration.md` | `delegate_subtask`, `request_assistance`, `evaluate_contribution` |
| AGSLAG Project Management | `agslag-docs/tools/project_management.md` | `create_goal` with 5-state status enum |
| AGSLAG Progress Report | `agslag-docs/reports/progress_evaluation_report.md` | File locking, task delegation, workflow engine status |
| phenotype-teamcomm README | `phenotype-teamcomm/README.md` | Five functional domains (sessions, reservations, inbox, live state, hook events) |
| phenotype-teamcomm Protocol | `phenotype-teamcomm/crates/teamcomm-protocol/src/` | Rust wire types for inbox, state, reservations, sessions, RPC, discovery, hook events |
| phenotype-teamcomm Daemon | `phenotype-teamcomm/crates/teamcomm-daemon/src/handlers.rs` | JSON-RPC handlers, heartbeat, lease TTL |
| Agent User Status | `agent-user-status/README.md` | 5-state status model, session heartbeat, scoped messaging |
| Traceability Schema | `AgilePlus/traces/SCHEMA.md` | `trace.json` format for FR linkage |

---

## 7. Implementation Delta

This SDD identifies four gaps between the current `phenotype-teamcomm` implementation and the canonical design:

1. **Status enum:** `AgentStatus` is currently 4-state (`Idle`, `Working`, `Blocked`, `Done`). Must be updated to 5-state (`online`, `idle`, `busy`, `blocked`, `offline`).
2. **Message type:** `InboxMessage` lacks a `message_type` field. Must be added.
3. **Thread entity:** No `Thread` struct exists in the protocol crate. Must be added as a first-class type with CRUD operations.
4. **First-class thread operations:** The daemon does not yet implement `thread.create`, `thread.join`, `thread.add_participant`, or `thread.remove_participant`.

These deltas are tracked as follow-up work items in AgilePlus.
