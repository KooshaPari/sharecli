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
