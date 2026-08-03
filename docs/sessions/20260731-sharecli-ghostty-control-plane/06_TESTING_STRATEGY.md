# Testing Strategy

Verified in this worktree:

- `CARGO_BUILD_JOBS=2 cargo check --workspace --locked --offline` (pass;
  existing workspace warnings only)
- `cargo test -p sharecli-session --offline` (baseline session, discovery,
  layout/state suites; the live RPC slice now has 29 library tests, including
  bounded event streaming)
- `cargo test -p sharecli-fuse --locked --offline` (36 unit + 11 integration passed)
- `cargo test -p sharecli-ipc --locked --test handler_dispatch` (10 passed,
  including layout save/list/inspect)
- `cargo test -p sharecli --locked --test session_cli` (layout save/list plus
  missing-native-socket watch fail-open check)
- `cargo test -p sharecli-session --offline -- --nocapture` also covers exact
  sidecar mappings, PID recycling fail-closed behavior, malformed JSONL, and
  append serialization; `session_cli` covers the registrar command.
- `swift test --package-path contrib/ghostty-control` (17 tests: dispatcher,
  strict input/size validation, token enforcement, MainActor provider crossing,
  Unix listener round-trip, notification suppression, RFC3339 timestamp parity,
  and bounded LiveIO sequence/overflow/subscription behavior)
- `cargo test --test session --offline` (the live client path includes event
  decoding/unsubscribe and rejects invalid subscription limits before connect)
- `sharecli surface layout-snapshot [--output PATH]` and
  `sharecli surface layout-restore <INPUT>` must be covered with CLI tests for
  atomic output, malformed/duplicate snapshot rejection before socket connect,
  and explicit missing-provider degradation; neither command may shell out or
  mutate panes without a native provider.
- `cargo run -p sharecli-fuse --bin fuse-runtime-probe` (read-only host evidence)
- `cargo fmt --all -- --check`
- `git diff --check`

Required before claiming full feature completion: Ghostty fork/socket provider
integration, native pane discovery/layout application, macOS FUSE KEXT/FSKit
mount smoke, crash/restart chaos, and a clean installed-dogfood run. The
ShareCLI-side live broker/client contract is now covered; the remaining gates
are app-side/runtime evidence and end-to-end crash recovery.
