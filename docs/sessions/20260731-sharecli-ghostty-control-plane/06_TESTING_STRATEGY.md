# Testing Strategy

Verified in this worktree:

- `CARGO_BUILD_JOBS=2 cargo check --workspace --locked --offline` (pass;
  existing workspace warnings only)
- `cargo test -p sharecli-session --offline` (22 unit + 7 discovery + 3 layout integration passed)
- `cargo test -p sharecli-fuse --locked --offline` (36 unit + 11 integration passed)
- `cargo test -p sharecli-ipc --locked --test handler_dispatch` (10 passed,
  including layout save/list/inspect)
- `cargo test -p sharecli --locked --test session_cli` (layout save/list plus
  missing-native-socket watch fail-open check)
- `swift test` in `contrib/ghostty-control` (native dispatcher contract)
- `cargo run -p sharecli-fuse --bin fuse-runtime-probe` (read-only host evidence)
- `cargo fmt --all -- --check`
- `git diff --check`

Required before claiming full feature completion: Ghostty fork/socket
integration, native pane discovery/layout application, macOS FUSE KEXT/FSKit
mount smoke, crash/restart chaos, and a clean installed-dogfood run. The
ShareCLI-side layout and surface-control unit tests are covered above; the
remaining gates are app-side/runtime evidence.
