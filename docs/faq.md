# sharecli — Frequently Asked Questions

User-facing troubleshooting for the sharecli CLI, `serve` dashboard, and fleet surfaces.
For accessibility posture see [`docs/a11y/README.md`](a11y/README.md). For install paths see
[`README.md`](../README.md).

## 1. How do I install sharecli?

Use one of:

- **crates.io:** `cargo install sharecli`
- **Prebuilt binary:** `cargo binstall sharecli` or download from [GitHub Releases](https://github.com/KooshaPari/sharecli/releases)
- **Homebrew (macOS):** `brew install --HEAD Formula/sharecli.rb` (see [`docs/ops/brew-bottle.md`](ops/brew-bottle.md))
- **Container:** `podman build -f Containerfile -t sharecli .`

Verify with `sharecli version` and `sharecli --help`.

## 2. Where is configuration stored?

Default paths (override with `SHARECLI_CONFIG` when documented in `sharecli config --help`):

| OS | Config directory |
|----|------------------|
| Linux / macOS | `~/.config/sharecli/` (`config.toml`, `pane-map.toml`) |
| Windows | `%APPDATA%\sharecli\` |

Runtime locks may live under `$XDG_RUNTIME_DIR/sharecli*` or the system temp directory while
`sharecli serve` is running. Use `sharecli uninstall --dry-run` to list paths before deleting.

## 3. How do I start the dashboard and health endpoints?

```bash
sharecli serve --bind 127.0.0.1:9000
```

Then open `http://127.0.0.1:9000/` for the HTML dashboard. Probes:

- **Liveness:** `GET /healthz`
- **Readiness:** `GET /readyz`
- **Metrics:** `GET /metrics/prometheus`

On WSL, bind `0.0.0.0:9000` and reach the dashboard from Windows at `http://127.0.0.1:9000/`.
See [`docs/deploy/FINALITY.md`](deploy/FINALITY.md).

## 4. How do I list, stop, or prune managed processes?

```bash
sharecli ps                    # table of managed processes
sharecli stop <name>           # stop one process
sharecli prune                 # dry-run idle cleanup
sharecli prune --force         # apply cleanup
```

If `ps` shows only headers, no processes are registered yet — start one with
`sharecli start` or your `process-compose.yaml` workflow (`sharecli proc-compose --help`).

## 5. Shell completions and man page?

```bash
sharecli completions bash > ~/.local/share/bash-completion/completions/sharecli
sharecli completions zsh  > ~/.zfunc/_sharecli
just man                      # writes share/man/man1/sharecli.1
man ./share/man/man1/sharecli.1   # local preview
```

Install the man page system-wide by copying `share/man/man1/sharecli.1` into your `MANPATH`
tree (e.g. `/usr/local/share/man/man1/`).

## More help

- **Quick start journey:** [`docs/journeys/quick-start.md`](journeys/quick-start.md)
- **Contributing:** [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- **All subcommands:** `sharecli list` and `sharecli --help`
