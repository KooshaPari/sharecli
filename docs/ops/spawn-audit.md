# Spawn audit events (soft)

Audit-v38 **C02 L28** — governance trace for process spawn.

## Event shape (target)

```json
{"event":"spawn","project":"…","pid":1234,"capability":"terminal","ts":"…"}
```

## Current evidence

- `src/spawn_policy.rs` — capability gates before spawn
- `src/audit.rs` — JSONL audit sink (`SHARECLI_AUDIT_PATH`)
- FR annotations in spawn tests (`tests/fr001_process_lifecycle.rs`)

## Soft wiring

1. Log `spawn` + `stop` rows to audit JSONL when `SHARECLI_AUDIT_PATH` set.
2. Include `project`, `capability`, `outcome` (`ok` / `denied`).
3. Document retention + rotation in `docs/ops/logging.md`.

## Deferred (hard)

- Signed audit envelopes
- SIEM export / OAuth actor attribution
