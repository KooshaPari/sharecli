# System status visibility & error recovery

Maps to Nielsen H1 (status visibility) and H5/H9 (error prevention & recovery). Ties to **FR-004** health/status acceptance (`tests/fr004_*.rs`).

## Status surfaces (FR-004)

| Surface | Command / URL | What it shows |
|---------|---------------|---------------|
| CLI | `sharecli status` | Pool / process summary |
| HTTP | `GET /health` | Liveness JSON |
| HTTP | `GET /health/processes` | Process list for probes |
| HTTP | `GET /metrics/prometheus` | RED metrics |
| Dashboard | `GET /` (WebSocket `/ws`) | Live process table + connection/thermal state |
| TUI | Thermal gauge + build-slot bar | Poll count, last poll age, thermal level |

Use `--verbose` on CLI commands for additional stderr detail during troubleshooting.

## Degraded mode

| Trigger | User-visible behavior | Recovery hint |
|---------|----------------------|---------------|
| Invalid config | Field-path error on stderr, non-zero exit | Fix path noted in message; run `sharecli config validate` |
| Unknown subcommand | `error: unrecognized subcommand` + suggestion | Run `sharecli --help` or `sharecli <cmd> --help` |
| Force-kill without confirm | `Would force-kill …` preview; no processes killed | Re-run with `--yes` (e.g. `sharecli stop --all --force --yes`) |
| WebSocket down | Dashboard: `disconnected — reconnecting in 3s` | Ensure `sharecli serve` is running on port 9000 |
| Thermal poll failure | TUI assumes GREEN, continues | Check platform thermal APIs; see README thermal section |
| Auth failure on `serve` | HTTP 401 + metric `sharecli_http_unauthorized_total` | See `docs/ops/AUTH.md` |

Config validation prints field paths before exit (`src/config_validator.rs`). Integration tests cover unknown-subcommand recovery (`tests/integration_cli.rs`).

## Long-running operations

- `sharecli serve`: Ctrl-C triggers clean shutdown (`src/commands/serve.rs`).
- `sharecli report --watch`: Ctrl-C cancels watch mode.
- `sharecli stop` / `sharecli project stop` / `sharecli prune --force`: batch operations show an **indicatif** progress bar with ETA on stderr when stderr is a TTY and the batch has ≥3 items; piped/CI output stays one line per process (`src/progress.rs`).
- TUI: progress via gauge + footer poll counter (no silent blocking).

Use `--verbose` for additional stderr detail during troubleshooting.
