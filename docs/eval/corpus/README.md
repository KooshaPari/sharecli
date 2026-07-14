# Synthetic process-supervisor eval corpus (C08 L71)

Harbor / SWE-bench style agent task corpora remain **out of scope**
([ADR 0002](../adr/0002-eval-surface-out-of-scope.md)). This corpus is the
**in-scope** synthetic stand-in: deterministic scenarios for the process
supervisor surface (spawn/list/stop/health), exercised by integration tests
and load scripts.

## Scenarios

| ID | File | Intent |
|----|------|--------|
| CORPUS-1 | [`scenarios/empty-pool.json`](scenarios/empty-pool.json) | Empty pool list / health baseline |
| CORPUS-2 | [`scenarios/single-idle.json`](scenarios/single-idle.json) | One registered idle process |
| CORPUS-3 | [`scenarios/thermal-red-deny.json`](scenarios/thermal-red-deny.json) | Gate DENY under thermal red |

Run contract: scenarios are fixtures for docs + future harness wiring; CLI
acceptance remains `tests/fr00*.rs` + Criterion/hyperfine (see [`../README.md`](../README.md)).
