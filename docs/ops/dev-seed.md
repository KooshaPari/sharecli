# Dev seed fixture (C07 L70)

Audit-v38 **C07 L70** — one command + verify + seed-data for reproducible local dev.

## Contract

| Step | Command | Evidence |
|------|---------|----------|
| Bootstrap | `just dev` | Builds, `--help` smoke, then seed verify |
| Seed fixture | `fixtures/dev-seed/config.toml` | Committed valid `Config` + `validate_config` pass |
| Verify gate | `scripts/dev/verify_seed.sh` | Runs `tests/c07_l70_dev_seed.rs` fixture test |
| CI parity | `cargo test c07_l70_dev_seed_fixture_valid` | Same gate in PR test matrix |

## Usage

```bash
just dev
# or only the seed gate:
bash scripts/dev/verify_seed.sh
```

The fixture is intentionally small: pool/monitoring/spawn limits suitable for laptop dev
without touching a developer's live `~/.config/sharecli/config.toml`.

## Hard follow-up

- Optional `sharecli config init --seed fixtures/dev-seed/config.toml` helper
- Seed runtime state dir + sample project registry for e2e tier (C07 L64)

**Status:** L70 score-3 path live · **FR:** FR-003 · **Last sync:** 2026-07-19
