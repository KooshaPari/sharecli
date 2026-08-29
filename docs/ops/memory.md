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

## RSS budgets (operator)

| Surface | Budget | How to sample | Gate |
|---------|--------|---------------|------|
| `sharecli serve` RSS (idle) | < 64 MiB on linux CI runners | `just rss-soft` / `scripts/ops/rss_soft.sh` | soft |
| `sharecli serve` RSS (idle) | < 64 MiB on linux CI runners | `scripts/ops/rss_gate.sh` | **hard** |
| `sharecli serve` RSS (32 procs) | < 256 MiB | same | soft |
| `sharecli serve` heap (dhat) | < 64 MiB total at idle smoke | `just dhat-soft` / `scripts/ops/dhat_soft.sh` | soft |

RSS idle has both a soft gate (`rss-soft.yml`, `continue-on-error`) and a hard
gate (`rss.yml`, required check). The hard gate uses `rss_gate.sh` which exits
non-zero on budget breach.

## Follow-ups

- Loom tests for `ProcessPool` / `serve_lock` (C00 L7): `crates/sharecli-sync` + `just loom` / `ci.yml` `loom` job.
- Soft RSS sample: `just rss-soft` / `.github/workflows/rss-soft.yml`.
- Soft dhat sample: `just dhat-soft` / `.github/workflows/dhat-soft.yml`.
- Hard RSS gate: `.github/workflows/rss.yml` / `scripts/ops/rss_gate.sh`.
