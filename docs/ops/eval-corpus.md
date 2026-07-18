# Eval corpus fixtures (soft) — sharecli

Audit-v38 **C08 L71**. Synthetic process-supervisor scenarios live under
`docs/eval/corpus/` (ADR-in-scope; Harbor/SWE-bench agent corpora remain N/A per
[ADR 0002](../adr/0002-eval-surface-out-of-scope.md)).

## Fixtures layout

```
docs/eval/corpus/
├── README.md              # scenario index table
└── scenarios/
    ├── empty-pool.json      # CORPUS-1 — empty pool baseline
    ├── single-idle.json     # CORPUS-2 — one idle process
    └── thermal-red-deny.json # CORPUS-3 — gate DENY under thermal red
```

Each scenario JSON **must** include:

| Field | Required | Purpose |
|-------|----------|---------|
| `id` | yes | Stable ID (`CORPUS-N`) for tables and CI logs |
| `name` | yes | Short slug (filename stem) |
| `expect` | yes | Object with at least `expect.health` **or** `expect.gate` |
| `seed_processes` | no | Future harness: synthetic pool rows |
| `thermal` | no | `green` / `yellow` / `red` for gate fixtures |
| `notes` | no | Human context for auditors |

Example (`expect.health` + `expect.gate`):

```json
{
  "id": "CORPUS-1",
  "name": "empty-pool",
  "expect": {
    "process_count": 0,
    "health": "ok",
    "gate": "ADMIT"
  }
}
```

## Local validation

```bash
just eval-corpus
# or: bash scripts/eval/run-corpus.sh
```

Optional live probe (serve must be running):

```bash
SHARECLI_CORPUS_LIVE=1 SHARECLI_BASE_URL=http://127.0.0.1:9000 bash scripts/eval/run-corpus.sh
```

Rust unit tests in `src/commands/serve.rs` map `expect.health` → `healthz_json()`
and `expect.gate` + `thermal` → `gate_decision()` for fixtures on disk.

## Eval corpus CI workflow

Workflow: [`.github/workflows/eval-corpus-soft.yml`](../../.github/workflows/eval-corpus-soft.yml)

| Trigger | Paths |
|---------|-------|
| `pull_request` | any |
| `push` to `main` | `docs/eval/corpus/**`, `scripts/eval/**`, workflow file |

Job `corpus` runs `bash scripts/eval/run-corpus.sh` on `ubuntu-24.04` with
`continue-on-error: true` (soft gate). Pool-level HTTP probes are separate:
[`live-pool-soft.yml`](../../.github/workflows/live-pool-soft.yml).

## How to add golden cases

1. **Add** `docs/eval/corpus/scenarios/<slug>.json` with the next `CORPUS-N` id.
2. **Register** the row in [`docs/eval/corpus/README.md`](../eval/corpus/README.md).
3. **Run** `just eval-corpus` locally; fix schema errors before push.
4. **Assert** (optional but preferred): if the scenario encodes a stable
   supervisor invariant, add or extend a `#[test]` in `serve.rs` that reads
   `expect.health` or `expect.gate` from the new file (see
   `corpus_health_fixtures_match_healthz` / `corpus_thermal_gate_fixtures_match_gate_decision`).
5. **CI** re-runs automatically when corpus paths change.

Golden cases here are **deterministic JSON fixtures**, not screenshot snapshots
(C10 `tests/golden/` is a separate visual gate). Keep scenarios small and
supervisor-scoped; do not add agent-eval task corpora without an ADR supersede.
Harbor Phase 2 stub: [`harbor-eval-stub.md`](./harbor-eval-stub.md); Phase 3 soak plan:
[`harbor-phase3-soak.md`](./harbor-phase3-soak.md) (cross-ref
[ADR 0005](../adr/0005-agent-eval-supersede.md)).
