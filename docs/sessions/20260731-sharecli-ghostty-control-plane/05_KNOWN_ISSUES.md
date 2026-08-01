# Known Issues and Open Gates

- `GhosttySurfaceAdapter::discover` is intentionally a capability contract;
  stock Ghostty's AppleScript dictionary can enumerate terminal identity,
  cwd, PID, and TTY, but the installed app has no documented external socket
  for PTY/screen readback. Native layout and readback still require the
  Ghostty-side integration gate.
- `contrib/ghostty-control` is the native binding contract and tested
  dispatcher/listener, not a patched Ghostty.app. A fork must implement
  `SurfaceProvider` from Ghostty's live surface tree and PTY/screen model, then
  start the listener from the app lifecycle.
- The session watcher is intentionally a read-only consumer. A launcher or
  harness wrapper must call `session register` (or append the same
  `{surface_id,harness,session_id,pid}` record) before launch to provide exact
  identity; database/argv heuristics remain non-authoritative and will not be
  promoted into unattended recovery.
- `SHARECLI_FUSE_FSKIT_APPROVED` is a conservative approval input, not a full
  MFMount entitlement probe. macOS install/approval and mount smoke are still
  required before enabling it by default.
- `fuse-runtime-probe` is evidence-only. It does not attempt the privileged
  mount smoke; that remains an explicit operator gate.
- IPC tests use the default local session database and should be isolated from
  a concurrently running production daemon before a full CI parallelization.
- Recovery launch reports process spawn, not completion or readiness; a future
  supervisor should add readiness/health events to the ledger.
