# Agent session recovery

Goal: deliver ShareCLI's macOS terminal/agent control plane: recover agent
conversations and live PTYs across Ghostty crashes without requiring tmux,
while preserving the daily Ghostty installation.

Ownership: ShareCLI owns the local control plane, durable session ledger,
process/surface discovery, recovery planning, and recovery execution. zmx is an
optional managed-PTY adapter. SessionLedger may remain a transcript provider,
but ShareCLI cannot depend on it for recovery.

Approved architecture: a hybrid adapter model. ShareCLI discovers and controls
existing Ghostty panes through a native capability-scoped adapter, while
ShareCLI-launched sessions may use a brokered PTY for the highest-fidelity
restart guarantee. Both paths feed the same SQLite WAL ledger and local Unix
RPC. Ambiguous session matches are never auto-targeted.

FUSE is an optional I/O accelerator only. Its required fail-open policy is
macFUSE kernel extension first, FSKit second, then the fully functional
non-FUSE recovery/control path. A failed mount must never prevent session
capture, live control, or recovery.
