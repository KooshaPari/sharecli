# Allocation profiling (soft)

Audit-v38 **C00 L8**. Operator memory budgets live in [`memory.md`](memory.md).
This note seeds **jemalloc / dhat** follow-ups without wiring always-on allocators.

## Soft recipes

### dhat (dev profile)

```bash
# Example: profile a short serve smoke under dhat (nightly / feature-gated when added)
RUSTFLAGS='--cfg dhat_heap' cargo +nightly build -p sharecli
# Follow-up: feature `dhat-heap` behind cfg; upload dhat-heap.json as soft CI artifact
```

### jemalloc (serve optional)

Prefer documenting `tikv-jemallocator` behind feature `jemalloc` for Linux/macOS
`sharecli serve` only after RSS budgets in `memory.md` show pressure. Windows
stays system allocator.

## Soft CI follow-up

| Step | Status |
|------|--------|
| RSS budget docs | Done (`memory.md`) |
| Soft idle RSS sample | Done (`rss-soft.yml` / `scripts/ops/rss_soft.sh`) |
| Feature-gated jemallocator | Not wired |
| Soft dhat sample job | Deferred |

Do not enable global jemalloc on all bins until spawn/Zig hot-core alloc ownership is reviewed.
