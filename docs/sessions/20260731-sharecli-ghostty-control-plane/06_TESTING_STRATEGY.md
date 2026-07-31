# Testing Strategy

Verified in this worktree:

- `cargo check --workspace --locked`
- `cargo test -p sharecli-session --locked` (14 passed)
- `cargo test -p sharecli-fuse backend::tests --locked` (2 passed)
- `cargo test -p sharecli --test session --locked` (5 passed)
- `cargo test -p sharecli --test session_cli --locked` (1 passed)
- `cargo test -p sharecli-ipc --locked --test handler_dispatch` (9 passed)
- `cargo fmt --all -- --check`
- `git diff --check`

Required before claiming full feature completion: Ghostty fork/socket
integration, native pane discovery, layout restore, macOS FUSE KEXT/FSKit
mount smoke, crash/restart chaos, and a clean installed-dogfood run.
