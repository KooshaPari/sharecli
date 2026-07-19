# Mutation testing threshold (hard gate live)

Audit-v38 **C07 L65**. Scoped to `sharecli-thermal-tui` pure helpers (`mutants.toml`).
Hard-gate promotion (phase 4 live):
[`mutants-hard-gate.md`](mutants-hard-gate.md).

## Current gate

| Mode | Workflow | Behavior |
|------|----------|----------|
| Hard | `.github/workflows/mutants.yml` + `ci.yml` `mutants` | PR/cron/dispatch; **fail-on-survivors**; **no** `continue-on-error` |
| Aggregator | `ci-success` `needs: […, mutants]` | Red mutants blocks `CI Success` |
| Local | `just mutants` | Scoped smoke matching CI examine file |
| Artifact | `mutants-hard-<sha>.json` / `mutants-ci-<sha>.json` | Upload for triage |

## Threshold (hard enforced)

| Metric | Hard (enforced) |
|--------|-----------------|
| Scope | `examine_globs` / `--file` thermal-tui lib |
| Outcome | **zero surviving mutants** in examine set + **required** / aggregated check |
| Timeout | 60s per mutant |
| Fail mode | step fails → job fails → `ci-success` fails |

Do not reintroduce `continue-on-error` or `|| true` on the mutants step.

## Commands

```bash
just mutants
# or:
cargo mutants --timeout 60 --jobs 2 \
  -p sharecli-thermal-tui \
  -- --locked
```
