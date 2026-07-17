# Hermetic builds (soft)

Audit-v38 **C06 L54**. Goal: reproducible offline builds after dependency fetch.

## Soft contract (today)

1. **`Cargo.lock` + `--locked`** on CI builds (already required).
2. **Fetch then offline:** `cargo fetch --locked` then `cargo build --locked --offline`.
3. Soft CI: `.github/workflows/hermetic-soft.yml` (`continue-on-error`), including a
   **poisoned-proxy** step (`HTTP_PROXY=http://127.0.0.1:9`) that still succeeds under
   `--offline` — soft stand-in for a network-blocked runner.
4. Local: `just hermetic` (same fetch → offline build).

This is **not** a fully network-blocked Actions runner or vendored `vendor/` tree yet.
The soft gate proves: lockfile completeness for offline compile + resilience to broken
egress during the offline phase.

## Hard follow-up

| Control | Status |
|---------|--------|
| `cargo vendor` + committed or artifact vendor dir | Not required yet |
| Network-blocked Actions step / hermetic builder | Deferred — see [slsa-l3-plan.md](./slsa-l3-plan.md) |
| Bit-identical SOURCE_DATE_EPOCH (see `docs/slsa.md`) | Seeded on release |

## Commands

```bash
cargo fetch --locked
cargo build --locked --offline -p sharecli
# or:
just hermetic
```
