# Migrated from 10-repo override-deletion batch (2026-08-09)

This directory preserves **24 truly-lost source files** discovered during
the historical full-branch audit of the 10 repos slated for soft-deletion.
Each file's blob SHA appears in the source repo's git history but NOT in the
default branch (HEAD) — meaning they would be lost-on-delete WITHOUT this
migration.

## Audit methodology

For each of the 10 override-deletion candidates, performed:
- Enumerated ALL paths ever touched across ALL refs (`git log --all --name-only --diff-filter=ACMRT`)
- Enumerated paths reachable from default branch (`main` or `master` depending on repo)
- Computed orphan paths = paths in --all but NOT in default
- For each source-code orphan (.rs, .go, .ts): verified the blob SHA is **NOT** in default branch
- Confirmed blob SHAs exist exactly once (truly unique, not renames)

## 24 files migrated (211,696 bytes, 6,413 lines)

| Source repo | Files | Bytes | Language |
|---|---|---|---|
| KVirtualStage | 11 (.rs) | ~132 KB | Rust |
| KodeVibe | 6 (.go) | ~38 KB | Go |
| KDesktopVirt | 3 (.rs) | ~30 KB | Rust |
| KWatch | 2 (.go) | ~8 KB | Go |
| KodeVibeGo | 1 (.ts) | ~2 KB | TypeScript |
| KlipDot | 1 (.rs) | ~1 KB | Rust |
| **TOTAL** | **24** | **~211 KB** | mixed |

## Provenance (source commits)

| Source repo | Oldest commit (deepest history) | Bundle path |
|---|---|---|
| KVirtualStage | `f9c68cc3568c` | `/tmp/gh-backup-2026-08-09-KVirtualStage.bundle` |
| KodeVibe | `e822c5ddeacf` | `/tmp/gh-backup-2026-08-09-KodeVibe.bundle` |
| KDesktopVirt | `b64ae84449b3` | `/tmp/gh-backup-2026-08-09-KDesktopVirt.bundle` |
| KWatch | `718d19f41b1a` | `/tmp/gh-backup-2026-08-09-KWatch.bundle` |
| KodeVibeGo | `a59ff5776417` | `/tmp/gh-backup-2026-08-09-KodeVibeGo.bundle` |
| KlipDot | `b2da9d229738` | `/tmp/gh-backup-2026-08-09-KlipDot.bundle` |

## Why these files were NOT in default branch

These files exist only on **non-default branches** that diverged from main
during multi-agent development sprints (Q2-Q3 2026). They represent
in-progress work that was:
- Either abandoned (no PR merged)
- Or moved to different paths without proper migration
- Or contained in feature branches that were never merged

Their blob SHAs are unique (no rename/duplicate elsewhere), meaning the
deletion of the source repo would have orphaned this code permanently.

## Restorability

Full git history of all 10 source repos preserved in:
- `/tmp/gh-backup-2026-08-09-<name>.bundle` (10 bundles, ~222 MB total)

To restore any deleted repo from its bundle:
```bash
git clone /tmp/gh-backup-2026-08-09-<name>.bundle <name>-restored
cd <name>-restored
git checkout main
```

## Audit trail

- Bundle creation: `/Users/kooshapari/.forge/audit/2026-08-09-10-repo-batch-override.md`
- Historical sweep script: `/tmp/historical-sweep-final.py`
- Truly-lost report: `/tmp/truly-lost-summary.txt`
