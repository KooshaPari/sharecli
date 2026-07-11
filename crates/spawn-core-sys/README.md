# spawn-core-sys

FFI / platform bindings for the Zig `spawn_core` hot path (semaphore + spawn + waitpid).

## Platforms

| Target | Backend |
|--------|---------|
| Linux / macOS | `zig build` → `libspawn_core.a`, linked from `build.rs` |
| Windows | **No Zig.** `build.rs` skips Zig; `src/windows_stub.rs` provides the public API |

### Windows gotcha

Zig `spawn_core` uses POSIX `fork` / `waitpid` / pthread and does not compile on Windows.
On `CARGO_CFG_TARGET_OS=windows`, this crate:

1. Skips `zig build` (do not require Zig on PATH for Windows targets).
2. Exposes a Rust stub: counting semaphore works; `zig_spawn` / `zig_waitpid` return `ErrorKind::Unsupported`.

`SpawnPolicy` concurrency throttling still works on Windows via the stub semaphore; the Zig spawn/QoS path does not.

Cross-compile note: `build.rs` keys off `CARGO_CFG_TARGET_OS`, not `cfg!(windows)` (host vs target).
