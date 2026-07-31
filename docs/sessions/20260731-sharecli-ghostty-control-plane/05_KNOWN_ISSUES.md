# Known Issues and Open Gates

- `GhosttySurfaceAdapter::discover` is intentionally a capability contract;
  stock Ghostty has no proven external pane enumeration API in this checkout.
- `SHARECLI_FUSE_FSKIT_APPROVED` is a conservative approval input, not a full
  MFMount entitlement probe. macOS install/approval and mount smoke are still
  required before enabling it by default.
- IPC tests use the default local session database and should be isolated from
  a concurrently running production daemon before a full CI parallelization.
- Recovery launch reports process spawn, not completion or readiness; a future
  supervisor should add readiness/health events to the ledger.
