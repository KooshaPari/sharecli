# ShareCLI Ghostty session control specification

## Product contract

ShareCLI must discover agent-bearing terminal surfaces, expose authenticated
single-host live pane I/O, persist recovery evidence to disk, and restore known
agent sessions after a terminal crash. It is an OS-adjacent control plane, not a
replacement terminal emulator and not a wrapper around vendor agent CLIs.

## Architecture

```
Ghostty existing panes -- native adapter --+
                                           +-- session observations -- SQLite WAL ledger
ShareCLI managed PTYs -- broker adapter --+                              |
                                                                          +-- recovery plan/executor
local Unix RPC <--- CLI / IPC / tray / dashboard <-----------------------+
```

The Ghostty adapter provides pane identity, layout, foreground process, working
directory, capability state, bounded output observation, and explicitly scoped
input dispatch. It must not use clipboard paste as its primary transport.

The managed-PTY adapter is optional for pre-existing Ghostty panes and required
only for sessions ShareCLI launches when lossless buffered I/O and exact restart
metadata are desired. zmx is an adapter candidate, not a required dependency.

## Ledger and recovery rules

The ledger uses SQLite WAL with atomic observation writes. A record includes a
stable surface ID, terminal adapter, parent surface/layout identity, cwd,
process fingerprint, detected harness, confidence-scored session ID, shell-free
resume recipe, last-observed time, and capability/health state.

Only a verified adapter may write a resume recipe. A low-confidence or ambiguous
record is visible in the operator UI but never auto-resumed. Recovery restores
the layout where the terminal adapter supports it, then resumes sessions with
bounded concurrency and per-session structured outcomes.

## Local RPC

The control plane uses ShareCLI's local Unix-socket IPC first. It exposes list,
inspect, observe, send, plan, recover, and cancel verbs. The service validates
peer ownership and applies per-pane serialization, output backpressure, message
size limits, and audit events. NATS is reserved for a future multi-host fleet
bridge and is not needed for a single Mac.

## FUSE policy

FUSE is never the session persistence layer. Its ordered optional policy is:

1. macFUSE kernel-extension backend.
2. FSKit backend when its extension is available and approved.
3. Non-FUSE control and recovery path.

Unavailable FUSE must yield a typed capability result and continue with the
non-FUSE path. It must not silently attempt an unavailable or mislabeled
backend.

## Acceptance evidence

- A Ghostty-managed agent pane can be discovered without tmux.
- Its cwd, process/harness, and verified session identity persist across a
  ShareCLI restart.
- Local RPC can read bounded output and send scoped input only to the selected
  pane.
- A crash recovery dry run produces an ordered, shell-free plan; execution
  resumes verified sessions and reports unresolved ones without guessing.
- The same flows work with FUSE unavailable.
