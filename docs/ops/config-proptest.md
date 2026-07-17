# Config validator property tests (soft)

Audit-v38 **C07 L66**. Thermal-tui already has proptest; config is next.

## Soft plan

| Target | Property idea | Status |
|--------|---------------|--------|
| `project_limits.max_processes` | values in `1..=10_000` never emit that field error alone | Soft backlog |
| `monitoring.health_check_interval_secs` | `1..=3600` accepted; `0` and `>3600` fail | Covered by unit tests today |
| `spawn_policy.max_concurrent_builds` | `>=1` | Unit tests |

## Local experiment (optional)

```bash
# When proptest is added to sharecli [dev-dependencies]:
cargo test -p sharecli --lib config_validator -- --nocapture
```

Hard lift: add `proptest` to root `Cargo.toml` `[dev-dependencies]` and shrink-test
`validate_config` invariants without rewriting `Cargo.lock` mid-merge conflicts.
