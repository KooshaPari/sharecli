# Specifications

## Durable ledger

`session_observations` is append-only with an autoincrement sequence, UTC
observation timestamp, terminal surface JSON, optional session JSON,
capabilities JSON, and an observation kind. The `sessions` table is a
materialized index updated in the same immediate SQLite transaction.

## Recovery safety

Only `Exact` and `Corroborated` records with an absolute working directory and
non-empty argv can be launched. Commands are constructed with
`std::process::Command`; shell evaluation is forbidden. `--execute` is an
explicit opt-in; the default CLI/IPC path is dry-run.

## Surface capabilities

Read, write, resize, layout, and durable-PTY claims are independent fields.
Missing capability is an explicit degraded result, never inferred from an
empty output buffer.

## Surface control protocol

`sharecli-session::rpc::serve_surface_unix` serves newline-delimited JSON-RPC
2.0. `surface.io.send` accepts exactly one UTF-8 `text` or byte-vector
payload, `surface.io.read` is capped at 1 MiB, and `surface.io.resize` rejects
zero dimensions. `surface.list` is a typed discovery hook; the default
adapter returns an explicit degraded error instead of an empty inventory.

## Live I/O subscriptions

`surface.io.subscribe` creates a numeric subscription ID and returns the next
global sequence number plus negotiated limits. Events are newline-delimited
JSON-RPC notifications using `surface.io.event`; output bytes are base64-encoded
and each chunk is at most 64 KiB. Queue capacity is bounded to 256 entries.

`surface.io.unsubscribe` is idempotent and reports whether a live subscription
was removed. A full queue drops the oldest event and emits a `dropped` marker
with `resync_required=true`; callers must use the durable surface snapshot and
explicit read path to recover state. Notifications never receive response
lines, and all response/event writes on one Unix connection are serialized.

The timestamp field is an optional RFC3339 string so Rust, Swift, and future
Ghostty-native providers share one wire representation.
