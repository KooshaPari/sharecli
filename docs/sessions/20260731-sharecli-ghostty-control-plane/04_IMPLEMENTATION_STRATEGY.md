# Implementation Strategy

- Keep the session model in `sharecli-session` so the CLI, IPC daemon, and
  future Ghostty adapter share one typed contract.
- Use SQLite WAL plus `BEGIN IMMEDIATE` for serialized append/materialization;
  compact only the observation history, never the materialized session rows.
- Keep Ghostty transport behind a small Unix JSON-RPC client with an optional
  bearer-like token. AppleScript may supply identity and input fallback, but
  it is explicitly degraded and never treated as PTY readback truth.
- Keep layout persistence in ShareCLI and make the Ghostty adapter responsible
  only for applying a validated snapshot. This permits recovery when Ghostty
  is unavailable and avoids coupling the ledger to an app-specific tree API.
- Use a bounded batch executor and `spawn`, so one long-running agent cannot
  block recovery of all other panes.
- Treat FUSE as an optimization/observation aid. Session persistence and
  recovery remain correct with no filesystem interception.
