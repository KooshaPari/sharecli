# Research

## Ghostty

- The installed Ghostty 1.3.1 binary exposes no documented external control
  socket. The upstream macOS scripting dictionary does expose window/tab/
  terminal identity, working directory, PID, TTY, split/focus/close, and input
  actions. AppleScript is therefore a useful degraded discovery/control path,
  but it is TCC-gated and does not provide a supported PTY/screen readback
  stream: https://ghostty.org/docs/features/applescript
- Upstream discussion favors narrowly scoped platform IPC and calls out the
  security implications of screen readback:
  https://github.com/ghostty-org/ghostty/discussions/2353
- ShareCLI consequently keeps a transport-neutral Unix JSON-RPC client and
  capability-gates readback. A Ghostty-side socket/fork is still required for
  AppleScriptless live PTY I/O and atomic layout operations.

## FUSE

- macFUSE documents KEXT/VFS and an explicit `backend=fskit` selector. FSKit has
  documented limitations and must not silently replace the KEXT path.
- ShareCLI's selector is deterministic: loaded KEXT -> approved FSKit -> no
  interception. The last state is functional fail-open, not a mount failure
  that blocks session recovery.

## Harness evidence

Resume recipes are generated from adapter state first, persisted state second,
and exact argv patterns third. Heuristic/ambiguous evidence is retained for
inspection but is never auto-launched.
