# Memory discipline (sharecli serve)

Soft guidance for audit-v38 **C00 L8**.

## Current posture

- Zig spawn-core documents explicit allocator / no-panic alloc failure (`spawn_policy`).
- Tray packages use size-oriented `opt-level = "z"` where configured.
- The long-running Rust `serve` binary still uses the default system allocator.

## Soft budgets (operator)

| Surface | Soft budget | How to sample |
|---------|-------------|----------------|
| `sharecli serve` RSS (idle) | < 64 MiB on linux CI runners | `ps -o rss=` / `/proc/self/status` |
| `sharecli serve` RSS (32 procs) | < 256 MiB | same |

These are **not** hard merge gates yet.

## Follow-ups

- Optional `jemallocator` / `tikv-jemallocator` behind a feature for `serve`.
- `dhat` or heaptrack sample job (soft) on main — see [`alloc-profiling.md`](alloc-profiling.md).
- Soft RSS sample: `just rss-soft` / `.github/workflows/rss-soft.yml`.
