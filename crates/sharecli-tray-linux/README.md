# sharecli-tray-linux

Linux system-tray client for sharecli — the third native tray alongside the
macOS Swift tray (`desktop/`) and the Windows WinUI 3 tray (`windows/`).

It renders a [StatusNotifierItem] tray via the [`ksni`] crate and shows the same
data as the other trays: managed-process count, memory usage, health, and a
per-process kill action. All data comes from the shared `sharecli-ipc` daemon
over its Unix socket (`process.list`, `health.status`, `process.kill`,
`process.kill_all`) — the same NDJSON-RPC contract the macOS/Windows trays use.

## Build & run

```bash
cargo build -p sharecli-tray-linux --release
# start the IPC daemon (if not already running), then the tray:
cargo run -p sharecli-ipc &
./target/release/sharecli-tray
```

The tray polls the daemon every 3 seconds and re-renders. If the daemon is not
reachable it shows an "offline" state rather than exiting.

## Requirements

- A running StatusNotifierItem / AppIndicator host (KDE Plasma, or GNOME with the
  AppIndicator/KStatusNotifierItem extension, or any tray that speaks SNI).
- The `sharecli-ipc` daemon (started separately, e.g. by your session manager).

## Configuration

- `SHARECLI_IPC_SOCK` — override the IPC socket path (defaults to
  `$XDG_DATA_HOME/sharecli/ipc.sock`, matching `sharecli-ipc`).
- `RUST_LOG` — tracing filter (defaults to `sharecli_tray=info`).

## Platform note

The crate compiles on all targets so `cargo build --workspace` stays green
everywhere, but the tray only functions on Linux (SNI is freedesktop-only). On
macOS/Windows the binary prints a pointer to the native tray and exits non-zero.

[StatusNotifierItem]: https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/
[`ksni`]: https://crates.io/crates/ksni
