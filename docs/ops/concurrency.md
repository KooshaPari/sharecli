# Concurrency & lock ordering (sharecli)

Guidance for audit-v38 **C00 L7**. ProcessPool pid registry is loom-verified;
full async `ProcessPool` + `serve_lock` remain documented with Miri soft.

## Lock / coordination map

| Resource | Mechanism | Held while | Notes |
|----------|-----------|------------|-------|
| Serve singleton | `serve_lock` + `fs2` file lock | Process lifetime of `sharecli serve` | Fail fast if another serve owns the lock |
| Process pool pid map | `sharecli-sync::PoolIndex` / `ProcessPool` `RwLock<HashMap<…>>` | insert/list/kill | Loom model: `crates/sharecli-sync` + `tests/loom_pool_index.rs` |
| Circuit breaker | `AtomicU32` / `AtomicU64` (SeqCst) | Brief CAS loops | `harness-native` strategies |
| Metrics counters | `AtomicU64` / `AtomicI64` | `inc` / `set` | Loom smoke in `tests/loom_pool_index.rs` |

## Ordering rule

Acquire **serve lock → pool lock → finer atomics**. Never take the serve lock while holding a pool mutex.

## Loom hard gate (C00 L7)

| Surface | Command | CI |
|---------|---------|-----|
| `PoolIndex` + relaxed counter | `just loom` | `ci.yml` job `loom` (required via `ci-success`) |

```bash
# Local parity with CI
just loom

# Equivalent
RUSTFLAGS="--cfg loom" cargo test --release --locked -p sharecli-sync --test loom_pool_index
```

`sharecli-sync::PoolIndex` mirrors the synchronous core of `runtime::ProcessPool`'s pid map
(`src/runtime.rs` `processes: RwLock<HashMap<u32, ManagedProcess>>`). Async
spawn/kill paths are not exhaustively modeled; Miri soft covers harness-native.

## Soft CI

Miri may run `continue-on-error` on PRs (`.github/workflows/miri-soft.yml`).
`serve_lock::decide` is pure policy — covered by unit tests in `serve.rs`.
