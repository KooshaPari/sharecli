# Harbor Phase 3 soak checklist log (soft)

Audit-v38 **C08 L76** — execution evidence for ADR 0005 Phase 3.
Plan: [`docs/ops/harbor-phase3-soak.md`](../../docs/ops/harbor-phase3-soak.md).

**Clock:** starts on first `main` push after soak execution scaffold merges. Count only
`harbor-eval-stub-soft.yml` runs where job `Harbor eval stub (corpus preflight)` logs
`STUB PASS: corpus valid`.

## Seven consecutive `main` runs

| # | recorded_at_utc | git_sha | workflow_run | stub_pass | notes |
|---|-----------------|---------|--------------|-----------|-------|
| 1 | _pending_ | — | — | — | post-merge row 1 |
| 2 | _pending_ | — | — | — | |
| 3 | _pending_ | — | — | — | |
| 4 | _pending_ | — | — | — | |
| 5 | _pending_ | — | — | — | |
| 6 | _pending_ | — | — | — | |
| 7 | _pending_ | — | — | — | Phase 3 maintainer sign-off gate |

Append local parity rows with:

```bash
SHARECLI_HARBOR_SOAK_LOG=audit/.lane-c08/harbor-phase3-soak-log.md \
  SHARECLI_HARBOR_SOAK_SOURCE=local \
  bash scripts/eval/harbor_soak.sh
```

## Phase 3 checklist (manual)

- [ ] Seven consecutive `main` workflow runs green for `harbor-eval-stub-soft.yml`.
- [ ] No merged PRs broke `scripts/eval/run-corpus.sh` preflight on `main` during the window.
- [x] `just harbor-soak` / `harbor_soak.sh` reproduces CI locally on branch HEAD (scaffold PR).
- [ ] Cross-repo pins table filled with recorded portage / pheno-harness refs.
- [ ] Maintainer acknowledges Phase 3 complete — **does not** mark ADR 0002 superseded (Phase 4).

## Cross-repo pins (recorded)

| Asset | Org repo | Recorded pin | sharecli touchpoint |
|-------|----------|--------------|---------------------|
| Harbor env provisioning | `phenotype-org/portage` | _pending_ | `scripts/eval/harbor_stub.sh` |
| SWE-bench / Terminal-Bench tasks | `phenotype-org/pheno-harness` | _pending_ | `docs/eval/GOVERNANCE.md` N/A rows |
| Adapter DAG | portage `adapters/pheno_harness_to_portage.py` | _pending_ | L71/L76 rubric cross-ref |

## Partial evidence (T-520 scaffold)

| Evidence | Status |
|----------|--------|
| `scripts/eval/harbor_soak.sh` local parity runner | Done (this PR) |
| `harbor-soak-exec-soft.yml` CI soft job | Done (this PR) |
| Checklist log template (this file) | Done (this PR) |
| Seven-day `main` soak window | Open (post-merge) |
| L76 score bump | Deferred until soak completes + Phase 4 discussion |
