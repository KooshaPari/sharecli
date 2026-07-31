# Research

## Ghostty

- Official AppleScript controls windows, tabs, splits, and input but does not
  provide a supported external pane readback stream:
  https://ghostty.org/docs/features/applescript
- Upstream discussion considers scoped platform IPC and calls out security
  implications of screen readback:
  https://github.com/ghostty-org/ghostty/discussions/2353
- Therefore ShareCLI implements a transport-neutral Unix JSON-RPC client and
  advertises native readback only when that socket is explicitly configured.

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
