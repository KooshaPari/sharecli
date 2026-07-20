# Feb harness recovery (A+ path)

Pointer for agents recovering Feb→sharecli lineage. Policy: [ADR 0006](../adr/0006-feb-harness-recovery-lineage.md). Harbor stays out of scope ([ADR 0002](../adr/0002-eval-surface-out-of-scope.md)).

## Ranked recovery paths

Prefer higher ranks before inventing new mesh/FUSE/strategy code:

| Rank | Source | Use for |
|------|--------|---------|
| **1** | Live sharecli crates (this repo) | Current tray/serve UX, supervisor surface, any already-ported strategy/FUSE/mesh |
| **2** | `thegent/src/thegent/mesh/` | Feb mesh substrate absorbed into thegent (donor after Jun 21 boundary flip) |
| **3** | `~/Downloads/files/` | Archived Feb/Mar harness and related artifacts |

## Status on `main` (2026-07-19/20)

| Item | Status | Evidence |
|------|--------|----------|
| Feb recovery lineage / ADR wiring | **Landed** | PR [#397](https://github.com/KooshaPari/sharecli/pull/397) |
| CoW commit/discard + `smart_merge` / worktree pool | **Landed** | PR [#400](https://github.com/KooshaPari/sharecli/pull/400) (`sharecli-fuse` / `sharecli-mesh`) |

## Remaining (highest leverage)

1. **C04 L34 — Verified commits** — ruleset `main-signed-commits` (**19181236**) is active; L34 stays **2** until a maintainer-signed commit shows **Verified** on `main`. Operator guide: [`gpg-verified-commits-l34.md`](gpg-verified-commits-l34.md) (see also [`signed-commits.md`](signed-commits.md), [`ruleset-checklist.md`](ruleset-checklist.md)).
2. **Optional privileged FUSE mount e2e** — unit/CoW helpers are on main; full privileged mount e2e remains optional depth.
3. **C11 L112 — codesign/notarize secrets** — still blocked (zero repo secrets); hard signed dmg/msi path deferred until secrets land ([`codesign-notarize.md`](codesign-notarize.md)).

## Notes

- **Mar 25** greenfield orphan root is a new DAG — not a wipe of recovery donors above.
- **Jun 21** made sharecli canonical; strategy + FUSE + mesh recovery continues here for optional depth only.
- Tray/serve is current UX; Feb TUI is historical reference only.
