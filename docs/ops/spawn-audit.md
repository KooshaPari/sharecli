# Spawn audit events (soft)

Audit-v38 **C02 L28** — governance trace for process spawn.

**Status:** Wired (soft) — `ProcessPool` emits `spawn` / `stop` rows via `audit_log::emit_if_configured` when `SHARECLI_AUDIT_LOG` is set.

## Event shape

```json
{"ts":…,"event":"spawn","service":"sharecli","project":"alpha","pid":1234,"capability":"claude","outcome":"ok"}
{"ts":…,"event":"stop","service":"sharecli","project":"alpha","pid":1234,"capability":"claude","outcome":"ok"}
```

`capability` is the harness tag when present, otherwise the spawned program name. `outcome` is `ok` or `denied`.

## Current evidence

- `src/runtime.rs` — `ProcessPool::spawn` / `kill` / `kill_all` audit rows
- `src/spawn_policy.rs` — build harness throttle before spawn
- `src/audit_log.rs` — JSONL sink (`SHARECLI_AUDIT_LOG`); rotation in `docs/ops/AUTH.md`
- `tests/spawn_audit.rs` — FR-004 spawn/stop JSONL acceptance
- FR annotations in spawn tests (`tests/fr001_process_lifecycle.rs`)

## Soft wiring

1. Set `SHARECLI_AUDIT_LOG` to an append-only JSONL path.
2. `spawn` + `stop` rows include `project`, `capability`, `outcome` (`ok` / `denied`).
3. Retention + rotation: `SHARECLI_AUDIT_MAX_BYTES` / `SHARECLI_AUDIT_RETAIN` (see `docs/ops/AUTH.md`).

## Deferred (hard)

- Signed audit envelopes
- SIEM export / OAuth actor attribution
