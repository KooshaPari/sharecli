# Undo / restore model

The `sharecli undo` subcommand (Plan 805, T-940) provides a single, unified
user-visible surface for inspecting, replaying, and recording every mutating
action that sharecli takes. The goal is **C09 L81.9 — User control and
freedom: graceful shutdown, undo, redo, cancellation**.

This document is the authoritative source of truth for the contract; the
implementation lives in `src/commands/undo.rs`.

## What it does

`sharecli undo` reads a single, append-only operation journal
(`$XDG_STATE_HOME/sharecli/undo-journal.jsonl`) and presents it via three
modes:

| Mode | Flag | Behaviour |
|------|------|-----------|
| List (default) | (none) | Print the last `--limit` (default 20) entries as a table or JSON |
| Restore | `--restore` | After interactive confirmation, replay the inverse action for the most recent entry (or the entry named by `--id`) |
| Clear | `--clear` | Truncate the journal file |

`--json` switches the list mode to machine-readable output. `--id <op-id>`
selects a specific entry for restore.

## What it does NOT do

* **No background processes.** Undo is a synchronous CLI invocation. There is
  no daemon.
* **No remote calls.** Undo never makes outbound HTTP/SSH/IPC requests. The
  restore mode reuses only local APIs already shipped in the binary.
* **No guaranteed reversal.** Each mutating command declares its own
  `Inverse` trait. If an entry's inverse returns `None` (e.g. `sharecli upgrade`),
  undo lists it as `non-reversible` and refuses to dispatch restore.

## Journal schema

Each line is a JSON object with the schema:

```json
{
  "id": "<uuid-v4>",
  "timestamp": "2026-08-31T12:00:00Z",
  "command": "sharecli project add",
  "args": ["myapp", "--template", "go"],
  "severity": "moderate",
  "rolled_back_at": null,
  "rolled_back_by": null,
  "note": null
}
```

* `severity` is one of `low | moderate | severe | destructive`.
* `rolled_back_at` is set by undo on successful restore.
* `rolled_back_by` is the operator note passed to `--restore --note "..."`.

## Storage path

```
$XDG_STATE_HOME/sharecli/undo-journal.jsonl
```

If `XDG_STATE_HOME` is unset, falls back to:

| OS | Path |
|----|------|
| Linux / macOS | `$HOME/.local/state/sharecli/undo-journal.jsonl` |
| Windows | `%LOCALAPPDATA%\sharecli\undo-journal.jsonl` |

The directory is auto-created on first write. No migration is required; an
absent file is treated as an empty journal.

## Interaction with mutating commands

Every mutating subcommand is wrapped in a `Record` helper that writes one
entry to the journal after the action succeeds. Failures write nothing
(idempotent: a failed action never leaves a tombstone entry).

## Operator rules

1. **Inspect first, restore second.** Always run `sharecli undo` (list mode)
   before `sharecli undo --restore` to confirm the operation id.
2. **Non-reversible operations** (severity `destructive`) require the explicit
   `--force` flag in addition to `--restore`.
3. **Audit retention** — the journal is append-only and is never truncated
   automatically. Operators may archive or rotate the file out of band.

## Verification

See `tests/c09_l81_undo.rs` (6 FR-003 acceptance gates, all passing).
