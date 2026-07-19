# Config validator property tests (C07 L66)

Audit-v38 **C07 L66**. Property tests span thermal-tui, config validation, and cast registry/address.

## Coverage

| Target | Property | Status |
|--------|----------|--------|
| `project_limits.max_processes` | `1..=10_000` TOML roundtrip + no field error | `prop_config_toml_roundtrip_max_processes_valid` |
| `monitoring.health_check_interval_secs` | `1..=3600` accepted; `0` / `>3600` fail | `prop_health_check_interval_boundary` + `prop_health_check_interval_out_of_range_fails` |
| `pool.idle_timeout_secs` | `1..=3600` accepted | `prop_pool_idle_timeout_boundary` |
| `spawn_policy.max_concurrent_builds` | `>=1` TOML roundtrip | `prop_spawn_policy_concurrent_builds_valid` |
| `cast::registry` | valid pane names + TOML map roundtrip + register/list | `prop_*` in `src/cast/registry.rs` |
| `cast::address` | machine chars + display/parse roundtrip + peel indices | `prop_*` in `src/cast/address.rs` |
| `sharecli-thermal-tui` | slot ratio, compact width, thermal gate | `prop_*` in `crates/sharecli-thermal-tui/src/lib.rs` |

## Boundary + shrinking + replay (score 3)

- **Boundary:** explicit `0`, `1`, `3600`, `3601` ranges via dedicated strategies (`prop_oneof`, bounded ranges).
- **Shrinking:** proptest default shrinker on all `proptest!` blocks.
- **Replay:** `src/proptest_util.rs` wires `FileFailurePersistence::SourceParallel("proptest-regressions")`; committed seeds under `proptest-regressions/` (e.g. `config_validator.txt` replays `max_processes = 1`).

## Local run

```bash
cargo test -p sharecli --bin sharecli config_validator::tests::prop_
cargo test -p sharecli --lib prop_
cargo test -p sharecli-thermal-tui prop_
cargo test -p sharecli --test c07_l66_proptest_expand
```

## Task refs

- T-410 — root `proptest` dev-dep + config roundtrip (#329) — DONE
- T-650 — expand boundary props + registry + replay seeds — DONE
