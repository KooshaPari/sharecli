# Feb harness recovery (A+ path)

Pointer for agents recovering Feb→sharecli lineage. Policy: [ADR 0006](../adr/0006-feb-harness-recovery-lineage.md). Harbor stays out of scope ([ADR 0002](../adr/0002-eval-surface-out-of-scope.md)).

## Ranked recovery paths

Prefer higher ranks before inventing new mesh/FUSE/strategy code:

| Rank | Source | Use for |
|------|--------|---------|
| **1** | Live sharecli crates (this repo) | Current tray/serve UX, supervisor surface, any already-ported strategy/FUSE/mesh |
| **2** | `thegent/src/thegent/mesh/` | Feb mesh substrate absorbed into thegent (donor after Jun 21 boundary flip) |
| **3** | [`recovery/feb-2026-agent-harness/`](../../recovery/feb-2026-agent-harness/) | **Durable vault** of the Feb 12 Downloads dump (install tar, `harness-fuse.elf`, `agents.conf`, `rules.conf`, plans). **Not** a git clone — no `.git`, no `fuse/` Rust sources. |
| **4** | `~/Downloads/files/` | Original Downloads path (same content as rank-3 vault; prefer vault in-repo) |

**February included FUSE** (`agent-harness fuse` / `harness-fuse`). Recovery diffs
against rank-3 vault before claiming greenfield FUSE design.

**GitHub Support:** `agent-harness` remote is past the 90-day restore window and will not
return. Restored `thegent-sharecli` is the Mar 25 process-manager twin / Python stub — **not**
Feb FUSE.

## Recovery branch status (`feat/fuse-feb-recovery-complete`, 2026-07-21)

Intent on this branch: close Feb→sharecli lineage for **mesh**, **strategy**, and
**FUSE** library surfaces; finish operator-only blockers separately.

| Area | Status on branch | Notes |
|------|------------------|-------|
| **Mesh** (`sharecli-mesh`) | **Landed** | `task_queue`, `smart_merge`, `worktree_pool` — CoW commit/discard + mesh CLI substrate |
| **Strategy** | **Mostly landed** | `SlotQueue`, nocache argv routing, coalesce integration landed; **`cache_key` / semantic / per-rule nocache** completing on this branch |
| **FUSE** (`sharecli-fuse`) | **Landed + CoW agent parity** | Passthrough + inode map, read coalesce, write serialize, `sharecli fuse` CLI; **per-agent CoW** (`--cow` / `--agent` / `--agents-conf` / `--no-serialize`) |
| Feb recovery lineage / ADR wiring | **Landed** | PR [#397](https://github.com/KooshaPari/sharecli/pull/397) + ADR 0006 refresh |
| CoW commit/discard + `smart_merge` / worktree pool (mesh) | **Landed** | PR [#400](https://github.com/KooshaPari/sharecli/pull/400) |

## Mesh port (Jun boundary) — complete

Rust ports live under `crates/sharecli-mesh/`:

| Donor (`thegent/src/thegent/mesh/`) | sharecli |
|------------------------------------|----------|
| `task_queue.py` (Maildir) | `task_queue.rs` |
| `smart_merge.py` | `smart_merge.rs` |
| `git_parallelism.py` / worktree pool | `worktree_pool.rs` |

thegent Python modules remain **recovery donors / temporary shims** until adapters sunset (boundary contracts). New mesh work lands only in sharecli.

## Remaining (highest leverage)

1. **C04 L34 — GPG Verified commits (operator)** — ruleset `main-signed-commits` (**19181236**) is active; L34 stays **2** until a maintainer-signed commit shows **Verified** on `main`. Operator guide: [`gpg-verified-commits-l34.md`](gpg-verified-commits-l34.md) (see also [`signed-commits.md`](signed-commits.md), [`ruleset-checklist.md`](ruleset-checklist.md)).
2. **Privileged FUSE mount e2e (matrix)** — prefer `just fuse-smoke` /
   `mac_host_linux_colima` (no host macFUSE reboot). See
   [`fuse-mount-smoke-matrix.md`](fuse-mount-smoke-matrix.md). Opt-in cargo test still
   uses `SHARECLI_FUSE_MOUNT_SMOKE=1` (`fr009_privileged_mount_smoke`).
3. **C11 L112 — codesign/notarize secrets** — still blocked (zero repo secrets); hard signed dmg/msi path deferred until secrets land ([`codesign-notarize.md`](codesign-notarize.md)).

## UX / tray note

- **Tray/serve** is the **accepted current deploy UX** — not the Feb harness TUI.
- Feb operator surface was a **harness TUI dashboard with metrics** (thermal, coalesce, mesh meters).
- Optional later port of harness dashboard metrics into tray/serve does **not** block lineage closure on this branch.

## Lineage notes

- **Mar 25** greenfield orphan root is a **new DAG for GitHub sharecli** — not a wipe of recovery donors above.
- **Jun 21** flip: **canonical ownership → sharecli**; strategy + FUSE + mesh recovery continues here.
- Do not reopen Harbor eval scope; see ADR 0002.
