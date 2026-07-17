# Harbor eval stub (soft) — sharecli

Audit-v38 **C08 L71 / L76**. Phase 2 stub for the agent-eval supersede pathway in
[ADR 0005](../adr/0005-agent-eval-supersede.md). Harbor / portage / Terminal-Bench
task runs remain **N/A** per [ADR 0002](../adr/0002-eval-surface-out-of-scope.md)
until Phase 4 supersede.

## Soft contract (Phase 2)

| Step | Action | Hard gate? |
|------|--------|------------|
| 1 | Validate supervisor JSON under `docs/eval/corpus/scenarios/` | No |
| 2 | Print **STUB PASS** — no Harbor env provisioned in-repo | No |
| 3 | CI: `harbor-eval-stub-soft.yml` with `continue-on-error: true` | No |

The stub reuses [`scripts/eval/run-corpus.sh`](../../scripts/eval/run-corpus.sh) as
preflight for a future Harbor harness. Do **not** add agent-eval task corpora here;
follow the supersede process in ADR 0005 §Decision.

## Local run

```bash
just harbor-stub
# or: bash scripts/eval/harbor_stub.sh
```

Expected tail output:

```
STUB PASS: corpus valid; Harbor task runner not wired (Phase 2 soft)
```

## CI workflow

Workflow: [`.github/workflows/harbor-eval-stub-soft.yml`](../../.github/workflows/harbor-eval-stub-soft.yml)

| Trigger | Paths |
|---------|-------|
| `pull_request` | any |
| `push` to `main` | `docs/eval/corpus/**`, `scripts/eval/**`, `docs/ops/harbor-eval-stub.md`, workflow file |

## Phase map (ADR 0005)

| Phase | Deliverable | Status |
|-------|-------------|--------|
| 0 | Supervisor corpus + ADR 0002 | Done |
| 1 | ADR 0005 supersede plan | Done |
| **2** | **This doc + `harbor_stub.sh`** | **Done (soft)** |
| 3 | [`harbor-phase3-soak.md`](./harbor-phase3-soak.md) + seven-day soft soak on `main` | In progress (plan landed) |
| 4 | Mark ADR 0002 superseded; GOVERNANCE + lane re-score | Deferred |

Phase 3 soak evidence plan: [`harbor-phase3-soak.md`](./harbor-phase3-soak.md) (cross-repo
pins, seven-day checklist). Cross-repo assets stay **out of repo** until Phase 4.

**Status:** soft stub (Phase 2) · **FR:** FR-003 traceability · **Last sync:** 2026-07-17
