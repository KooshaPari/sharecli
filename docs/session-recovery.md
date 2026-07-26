# Session recovery

ShareCLI records agent processes and emits shell-free resume recipes. The watcher
only persists state; it never launches agents.

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
```

The plist targets the current Cargo install at `/Users/kooshapari/.cargo/bin/sharecli`;
edit `ProgramArguments` for another installation path. Ghostty remains the stable
daily terminal. zmx provides durable PTYs, while a future Ghostty fork may add
capability-scoped Unix-socket surface control without changing this contract.
