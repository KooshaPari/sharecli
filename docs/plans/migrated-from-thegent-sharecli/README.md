# Migrated from `thegent-sharecli` (archived 2026-08-08)

This directory preserves project-specific content that lived only in the
archived `thegent-sharecli` repository, brought into canonical `sharecli`
as historical context. (Note: `thegent-sharecli` was itself described as
"DEPRECATED duplicate of sharecli".)

## Provenance

| File | Source blob SHA | Source HEAD | Source path |
|---|---|---|---|
| `AGILEPLUS_SPEC-thegent-sharecli.md` | `98a3d1819042` | `2e1d734067c5` | `thegent-sharecli/docs/plans/AGILEPLUS_SPEC.md` |
| `KILO_GASTOWN_SPEC-thegent-sharecli.md` | `6c4c61bed4f5` | `2e1d734067c5` | `thegent-sharecli/docs/plans/KILO_GASTOWN_SPEC.md` |
| `../.agileplus/migrated-from-thegent-sharecli/worklog.md` | `da86d7966986` | `2e1d734067c5` | `thegent-sharecli/.agileplus/worklog.md` |

## Timeline

- **2026-04-02**: Initial commits in `thegent-sharecli` documenting how
  the rig applied the canonical AgilePlus + Kilo/Gastown methodologies.
- **2026-07-22**: `thegent-sharecli` archived with tag "DEPRECATED duplicate
  of sharecli — absorb into thegent when runtime stabilizes".
- **2026-08-08**: cherry-picked `2e1d734` (coordination_contract.py) into
  `thegent/sharecli/`. Project-specific methodology docs migrated to both
  `thegent` and `sharecli` (this repo).

## Why these files were NOT replaced with canonical versions

This `sharecli` repo does not have its own `docs/plans/AGILEPLUS_SPEC.md`
or `docs/plans/KILO_GASTOWN_SPEC.md` — those exist in the `thegent`
project. The `thegent-sharecli` versions captured *project-specific
bindings* of the methodology to a multi-rig agent fleet. They are
preserved here for historical reference, not as canonical methodology.

## Recoverability

If these files are ever needed in their original context, the full git
history of `thegent-sharecli` is preserved in the bundle:

```
/tmp/gh-backup-2026-07-28-thegent-sharecli.bundle
```

This bundle contains every commit, blob, and tag from the
thegent-sharecli repository up to its archival on 2026-08-08.
To restore the entire repo from this bundle:

```bash
git clone /tmp/gh-backup-2026-07-28-thegent-sharecli.bundle thegent-sharecli-restored
cd thegent-sharecli-restored
git checkout main
```
