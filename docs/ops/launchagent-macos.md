# macOS LaunchAgent sidecar (local deploy)

Installs a **KeepAlive** user LaunchAgent that runs `sharecli serve` on a fixed bind
(default `127.0.0.1:9000`), matching the Linux sample unit in
`docs/deploy/systemd/sharecli.service`.

## Install path (this Mac)

```bash
# From a sharecli checkout with the binary already on PATH:
cargo install --path . --locked --force   # → ~/.cargo/bin/sharecli
./scripts/install-sidecar.sh
```

Override bind:

```bash
SHARECLI_SERVE_BIND=127.0.0.1:9000 ./scripts/install-sidecar.sh
```

Uninstall:

```bash
./scripts/install-sidecar.sh --uninstall
```

## Prove it

```bash
launchctl list | rg sharecli
curl -sf http://127.0.0.1:9000/healthz
curl -sf http://127.0.0.1:9000/readyz
ls -la ~/Library/LaunchAgents/com.phenotype.sharecli.plist
tail -n 40 ~/.sharecli/sidecar.log ~/.sharecli/sidecar.err
```

Label: `com.phenotype.sharecli`. Logs: `~/.sharecli/sidecar.{log,err}`.

## Product note

Earlier sidecars ran a one-shot `fleet status` with `KeepAlive=false`. That does not
keep the dashboard up. The installer now uses `serve --bind … --on-conflict replace`
with `KeepAlive=true` / systemd `Restart=always`.
