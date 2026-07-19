# E2E and chaos test tier (C07 L64)

Audit-v38 **C07 L64** — third pyramid tier beyond unit + integration: real `sharecli serve`
processes, HTTP probes, and kill/restart chaos.

## Layout

| File | Tier | Scenario |
|------|------|----------|
| `tests/e2e_serve_healthz.rs` | e2e | Spawn serve on ephemeral port; `curl /healthz` → 200 |
| `tests/e2e_chaos_recovery.rs` | chaos e2e | SIGKILL serve; restart; `/healthz` recovers within 30s |

Test names include `_e2e_` so nextest filters and CI overrides apply (see `.config/nextest.toml`).

## Commands

```bash
# E2E profile (longer timeouts, limited parallelism)
just test-e2e

# Full local suite (unit + integration + e2e via cargo test)
cargo test --locked --all-features
```

`just test-e2e` runs:

```bash
cargo nextest run --locked --all-features --profile e2e -E 'test(/_e2e_/)'
```

## CI

PR CI continues to use `[profile.ci]` for the main matrix. E2E tests run in the same
nextest invocation because `_e2e_` overrides extend timeouts and retries. Dedicated
nightly expansion (freebsd/wasm) remains deferred.

## Prerequisites

- `curl` on PATH (tests skip gracefully if absent)
- Debug or release `sharecli` binary via `CARGO_BIN_EXE_sharecli`

## Related

- [flake-policy.md](flake-policy.md) — quarantine for flaky e2e
- `scripts/load/chaos_restart.sh` — shell macrobench used by C05 L50 hard gate
