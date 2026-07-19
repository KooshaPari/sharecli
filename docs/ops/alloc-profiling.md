# Allocation profiling (soft)

Audit-v38 **C00 L8**. Operator memory budgets live in [`memory.md`](memory.md).

## Wired paths

### jemalloc (serve production)

`tikv-jemallocator` behind feature `jemalloc` for Linux/macOS `sharecli serve` deployments.
Windows MSVC builds keep the system allocator.

```bash
cargo build --locked --release -p sharecli --features jemalloc
just serve-jemalloc
```

### dhat (dev heap profile)

Feature `dhat-heap` installs `dhat::Alloc` as the global allocator and emits
`dhat-heap.json` when the process returns from `main` (Profiler `Drop`).

Do **not** use `sharecli --help` / `--version` for sampling: clap exits via
`process::exit`, which skips destructors and leaves no JSON artifact. The soft
script runs `sharecli completions bash` so `main` returns normally.

```bash
just dhat-soft
# or
bash scripts/ops/dhat_soft.sh
```

Do **not** enable `jemalloc` and `dhat-heap` together (both require `#[global_allocator]`).

## Soft CI

| Step | Status |
|------|--------|
| RSS budget docs | Done (`memory.md`) |
| Soft idle RSS sample | Done (`rss-soft.yml` / `scripts/ops/rss_soft.sh`) |
| Feature-gated jemallocator | Done (`src/alloc.rs`, `jemalloc` feature) |
| Soft dhat sample job | Done (`dhat-soft.yml` / `scripts/ops/dhat_soft.sh`) |

Do not enable global jemalloc on all bins until spawn/Zig hot-core alloc ownership is reviewed;
container + documented serve builds use the feature explicitly.
