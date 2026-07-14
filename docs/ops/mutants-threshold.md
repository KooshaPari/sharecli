# Mutation testing threshold (soft → hard path)

Audit-v38 **C07 L65**. Scoped to `sharecli-thermal-tui` pure helpers (`mutants.toml`).

## Current gate

| Mode | Workflow | Behavior |
|------|----------|----------|
| Soft | `.github/workflows/mutants.yml` | PR/cron/dispatch; **fail-on-survivors** for examine set; job `continue-on-error: true` |
| Local | `just mutants` | Scoped smoke matching CI examine file |
| Artifact | `mutants-soft-<sha>.json` | Upload on soft CI for triage |

## Threshold (soft enforced today)

| Metric | Soft (enforced) | Hard target |
|--------|-----------------|-------------|
| Scope | `examine_re` / `--file` thermal-tui lib | same |
| Outcome | **zero surviving mutants** in examine set | same + **required** check |
| Timeout | 60s per mutant | 60s |
| Fail mode | step fails → job soft-red | remove `continue-on-error` |

Hard promotion: remove `continue-on-error` once the examine set stays green on `main` for one week.

## Commands

```bash
just mutants
# or:
cargo mutants --timeout 60 --jobs 2 \
  --file 'crates/sharecli-thermal-tui/src/lib.rs' \
  -- --locked -p sharecli-thermal-tui
```
