# Hermetic builds (soft)

Audit-v38 **C06 L54**. Goal: reproducible offline builds after dependency fetch.

## Soft contract (today)

1. **`Cargo.lock` + `--locked`** on CI builds (already required).
2. **Fetch then offline:** `cargo fetch --locked` then `cargo build --locked --offline`.
3. Soft CI: `.github/workflows/hermetic-soft.yml` (`continue-on-error`).
4. Local: `just hermetic` (same fetch → offline build).

This is **not** a network-blocked runner or vendored `vendor/` tree yet. Soft gate
proves the lockfile is complete enough for offline compile after a one-shot fetch.

## Hard follow-up

| Control | Status |
|---------|--------|
| `cargo vendor` + committed or artifact vendor dir | Not required yet |
| Network-blocked Actions step / hermetic builder | Deferred (SLSA L3) |
| Bit-identical SOURCE_DATE_EPOCH (see `docs/slsa.md`) | Seeded on release |

## Commands

```bash
cargo fetch --locked
cargo build --locked --offline -p sharecli
# or:
just hermetic
```
