# Concurrency & lock ordering (sharecli)

Soft guidance for audit-v38 **C00 L7**. Formal loom/TSan gates are still soft.

## Lock / coordination map

| Resource | Mechanism | Held while | Notes |
|----------|-----------|------------|-------|
| Serve singleton | `serve_lock` + `fs2` file lock | Process lifetime of `sharecli serve` | Fail fast if another serve owns the lock |
| Circuit breaker | `AtomicU32` / `AtomicU64` (SeqCst) | Brief CAS loops | `harness-native` strategies |
| Process pool | Interior mutexes in pool types | Spawn/list/stop paths | Prefer short critical sections; no I/O under pool lock |

## Ordering rule

Acquire **serve lock → pool lock → finer atomics**. Never take the serve lock while holding a pool mutex.

## Soft CI

Optional Miri / race tooling may run `continue-on-error` on PRs. Hard loom coverage for `ProcessPool` remains a follow-up.
