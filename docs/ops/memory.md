# Memory discipline (sharecli serve)

Soft guidance for audit-v38 **C00 L8**.

## Current posture

- Zig spawn-core documents explicit allocator / no-panic alloc failure (`spawn_policy`).
- Tray packages use size-oriented `opt-level = "z"` where configured.
- `sharecli serve` may use **jemalloc** via the `jemalloc` Cargo feature (Unix, non-MSVC).
- Default CLI builds keep the system allocator unless the feature is enabled.

## Allocator features

| Feature | Use | Build |
|---------|-----|-------|
| `jemalloc` | Production serve / container images | `cargo build --release -p sharecli --features jemalloc` |
| `dhat-heap` | Dev heap profiling (mutually exclusive with `jemalloc`) | `cargo build --release -p sharecli --features dhat-heap` |

Implementation: `src/alloc.rs` (`#[global_allocator]` + `active_allocator_label()`).

Container images build with `--features jemalloc` (`Containerfile`).

## Soft budgets (operator)

| Surface | Soft budget | How to sample |
|---------|-------------|----------------|
| `sharecli serve` RSS (idle) | < 64 MiB on linux CI runners | `just rss-soft` / `scripts/ops/rss_soft.sh` |
| `sharecli serve` RSS (32 procs) | < 256 MiB | same |
| `sharecli serve` heap (dhat) | < 64 MiB total at idle smoke | `just dhat-soft` / `scripts/ops/dhat_soft.sh` |

These are **not** hard merge gates yet (soft CI uses `continue-on-error`).

## Follow-ups

- Hard RSS gate after jemalloc soak on main.
- Loom tests for `ProcessPool` / `serve_lock` (C00 L7).
- Soft RSS sample: `just rss-soft` / `.github/workflows/rss-soft.yml`.
- Soft dhat sample: `just dhat-soft` / `.github/workflows/dhat-soft.yml`.
