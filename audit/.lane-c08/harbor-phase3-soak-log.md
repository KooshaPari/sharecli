# Harbor Phase 3 soak checklist log (soft)

Audit-v38 **C08 L76** — execution evidence for ADR 0005 Phase 3.
Plan: [`docs/ops/harbor-phase3-soak.md`](../../docs/ops/harbor-phase3-soak.md).

**Clock:** starts on first `main` push after soak execution scaffold merges. Count only
`harbor-eval-stub-soft.yml` runs where job `Harbor eval stub (corpus preflight)` logs
`STUB PASS: corpus valid`.

**Progress (2026-07-19 UTC):** `main` soak days complete **0 / 7** · remaining **7**.
Local soft parity is allowed for evidence but does **not** decrement the seven-day `main` clock.

**Backfill (T-650):** Disallowed. Clock starts on the first `main` push *after* soak scaffold
merge (#333 / `219b8d0`). Candidate historical `Harbor eval stub (soft)` runs
[29639034239](https://github.com/KooshaPari/sharecli/actions/runs/29639034239),
[29632606178](https://github.com/KooshaPari/sharecli/actions/runs/29632606178),
[29616634071](https://github.com/KooshaPari/sharecli/actions/runs/29616634071) all logged
`STUB PASS: corpus valid`, but 29616634071 / 29632606178 are pre-scaffold and 29639034239
*is* the scaffold merge push — none are post-merge consecutive soak days. Leave **0/7**.

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

## Local soft parity (does not count toward 7/7)

| # | recorded_at_utc | git_sha | source | stub_pass | notes |
|---|-----------------|---------|--------|-----------|-------|
| L1 | 2026-07-19T01:47:13Z | 94ad489 | local | yes | `harbor_soak.sh` + `run-corpus.sh` green on `feat/aplus-sota-ownership` (W14.2 progress) |

Append further local parity rows with:

```bash
SHARECLI_HARBOR_SOAK_LOG=audit/.lane-c08/harbor-phase3-soak-log.md \
  SHARECLI_HARBOR_SOAK_SOURCE=local \
  bash scripts/eval/harbor_soak.sh
```

> Prefer editing this **Local soft parity** table manually after a run; the scaffold script
> appends a raw row at EOF and may not preserve table sections.

## Phase 3 checklist (manual)

- [ ] Seven consecutive `main` workflow runs green for `harbor-eval-stub-soft.yml`.
- [ ] No merged PRs broke `scripts/eval/run-corpus.sh` preflight on `main` during the window.
- [x] `just harbor-soak` / `harbor_soak.sh` reproduces CI locally on branch HEAD (scaffold + 2026-07-19 soft soak).
- [ ] Cross-repo pins table filled with recorded portage / pheno-harness refs.
- [ ] Maintainer acknowledges Phase 3 complete — **does not** mark ADR 0002 superseded (Phase 4).

## Cross-repo pins (recorded)

| Asset | Org repo | Recorded pin | sharecli touchpoint |
|-------|----------|--------------|---------------------|
| Harbor env provisioning | `phenotype-org/portage` | _pending_ | `scripts/eval/harbor_stub.sh` |
| SWE-bench / Terminal-Bench tasks | `phenotype-org/pheno-harness` | _pending_ | `docs/eval/GOVERNANCE.md` N/A rows |
| Adapter DAG | portage `adapters/pheno_harness_to_portage.py` | _pending_ | L71/L76 rubric cross-ref |

## Partial evidence (T-520 scaffold + W14.2)

| Evidence | Status |
|----------|--------|
| `scripts/eval/harbor_soak.sh` local parity runner | Done |
| `harbor-soak-exec-soft.yml` CI soft job | Done |
| Checklist log template (this file) | Done |
| Local soft soak row L1 (2026-07-19) | Done |
| Seven-day `main` soak window | Open — **0/7 complete · 7 remaining** |
| L76 score bump | Deferred until soak completes + Phase 4 discussion |
