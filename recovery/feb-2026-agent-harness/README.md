# Feb 2026 agent-harness oracle (durable vault)

**Not a git clone of `agent-harness`.** GitHub Support cannot restore that remote
(past the 90-day deletion window). This directory is the permanent in-repo copy of
the Feb 12 Downloads dump used as the FUSE / PATH-proxy recovery oracle.

## What is here

| Path | Contents |
|------|----------|
| `artifacts/agent-harness.tar.gz` | Install tree only (~31KB): `bin/harness`, shims, `etc/`, install scripts. **No `.git`, no `fuse/` Rust sources.** |
| `artifacts/harness-fuse.elf` | Linux x86_64 prebuilt FUSE binary (`fuser` 0.14 era). Binary-only. |
| `artifacts/harness`, `core.sh`, plans | Loose Downloads copies |
| `etc/agents.conf` | Agent process-name patterns (substring match) |
| `etc/rules.conf` | Coalesce / queue / passthrough rules |
| `MANIFEST.sha256` | SHA-256 of every vaulted file |

## What is not here

- Full `agent-harness` git history
- `fuse/` crate sources (only the ELF + CLI strings / plans)
- Guaranteed macOS binary (ELF is Linux)

## Canonical product

Live FUSE / CoW / operator CLI: `crates/sharecli-fuse` + `sharecli fuse …`.
Lineage: [`docs/adr/0006-feb-harness-recovery-lineage.md`](../../docs/adr/0006-feb-harness-recovery-lineage.md),
[`docs/ops/feb-recovery.md`](../../docs/ops/feb-recovery.md).

## Restored sibling (not this oracle)

`KooshaPari/thegent-sharecli` (restored 2026-07-22) is the Mar 25 process-manager
twin / Python absorb stub — **not** Feb FUSE.
