# Implementation Strategy

- Keep the session model in `sharecli-session` so the CLI, IPC daemon, and
  future Ghostty adapter share one typed contract.
- Use SQLite WAL plus `BEGIN IMMEDIATE` for serialized append/materialization;
  compact only the observation history, never the materialized session rows.
- Keep Ghostty transport behind a small Unix JSON-RPC client with an optional
  bearer-like token. The server-side dispatcher enforces that token and applies
  owner-only socket permissions. AppleScript may supply identity and input
  fallback, but it is explicitly degraded and never treated as PTY readback
  truth.
- Use `sharecli-session::SurfaceObservationScanner` for one-pass discovery:
  per-surface capability failures are isolated, known harness argv/state is
  resolved conservatively, and unknown processes are recorded without a resume
  recipe. `sharecli session watch` is the durable CLI loop around that contract.
  Exact launch-time mappings come from the append-only JSONL sidecar selected by
  `--state-sidecar`/`SHARECLI_SESSION_SIDECAR`; the latest record wins, PID
  mismatches fail closed, and malformed input degrades the pass. `session
  register` is the owner-only append path for launch wrappers.
- Keep the native Ghostty bridge in `contrib/ghostty-control` as a standalone
  Swift package. It defines the provider boundary and JSON-RPC dispatcher so a
  Ghostty fork can bind native split/pane/PTY objects without carrying ShareCLI's
  Rust workspace into the app.
- Keep layout persistence in ShareCLI and make the Ghostty adapter responsible
  only for applying a validated snapshot. This permits recovery when Ghostty
  is unavailable and avoids coupling the ledger to an app-specific tree API.
- Use a bounded batch executor and `spawn`, so one long-running agent cannot
  block recovery of all other panes.
- Treat FUSE as an optimization/observation aid. Session persistence and
  recovery remain correct with no filesystem interception.
