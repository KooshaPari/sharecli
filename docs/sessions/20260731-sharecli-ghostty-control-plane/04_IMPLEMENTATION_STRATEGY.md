# Implementation Strategy

- Keep the session model in `sharecli-session` so the CLI, IPC daemon, and
  future Ghostty adapter share one typed contract.
- Use SQLite WAL plus `BEGIN IMMEDIATE` for serialized append/materialization;
  compact only the observation history, never the materialized session rows.
- Keep Ghostty transport behind a small Unix JSON-RPC client with an optional
  bearer-like token. AppleScript remains a degraded control path, not a source
  of readback truth.
- Use a bounded batch executor and `spawn`, so one long-running agent cannot
  block recovery of all other panes.
- Treat FUSE as an optimization/observation aid. Session persistence and
  recovery remain correct with no filesystem interception.
