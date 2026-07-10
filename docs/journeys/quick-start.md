# Quick Start

CLI-first journey for sharecli (FR-001, FR-002, FR-003).

| Step | Action | FR |
|------|--------|----|
| 1. Install | Build or install the binary | NFR-001 |
| 2. Configure | Init TOML config | FR-002 |
| 3. Register | Add a project path | FR-003 |
| 4. Run | Start / list / stop a process | FR-001 |
| 5. Verify | Status / health | FR-004 |

**Estimated duration:** ~5 minutes

## Installation

```bash
# From a git checkout
cargo build --release
./target/release/sharecli --version

# Or install
cargo install sharecli
# cargo binstall sharecli
```

## Configuration

```bash
sharecli config init
sharecli config validate
sharecli config show
```

Config lives under the platform config directory
(`$XDG_CONFIG_HOME/sharecli/config.toml` or OS equivalent). See FR-002.

## Register a project

```bash
sharecli project add demo /path/to/repo
sharecli project list
sharecli project show demo
```

## Run & verify

```bash
sharecli start demo --harness node
sharecli ps --project demo
sharecli status
sharecli stop --all
```

## Friction

If a step fails with a generic error, log it in
[`docs/friction-log.md`](../friction-log.md) and cite the FR above.
