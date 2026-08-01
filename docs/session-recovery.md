# Session recovery

ShareCLI records agent processes and emits shell-free resume recipes. The watcher
persists state and never launches agents. With a native Ghostty socket, `session
watch` also records surface identity, process argv, capabilities, and
evidence-backed harness session ids.

Install the optional macOS watcher:

```sh
mkdir -p "$HOME/Library/LaunchAgents"
cp contrib/com.phenotype.sharecli.session-watch.plist "$HOME/Library/LaunchAgents/"
launchctl bootstrap "gui/$UID" "$HOME/Library/LaunchAgents/com.phenotype.sharecli.session-watch.plist"
```

Inspect or recover explicitly:

```sh
sharecli session recovery-plan
sharecli session recover             # dry-run
sharecli session recover --execute   # explicit launch
sharecli session watch --once        # one native Ghostty inventory pass
```

The plist targets the current Cargo install at `/Users/kooshapari/.cargo/bin/sharecli`;
edit `ProgramArguments` for another installation path. Ghostty remains the stable
daily terminal. zmx provides durable PTYs. The fork-friendly native contract is
implemented in `contrib/ghostty-control` and can bind Ghostty's app-side
surface/PTY objects without AppleScript. Its socket must be owner-only (`0600`)
and may require `SHARECLI_GHOSTTY_TOKEN`; a missing socket or provider
capability is degraded, never a recovery blocker.

For a read-only FUSE capability report (no kext load, approval prompt, or mount):

```sh
cargo run -p sharecli-fuse --bin fuse-runtime-probe -- /tmp/sharecli-probe
```

The selector is deterministic: macFUSE KEXT first, approved FSKit only under
`/Volumes`, then the non-FUSE path.
