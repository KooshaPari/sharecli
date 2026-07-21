# configs/

Hook snippets for the agents that will integrate with `phenotype-teamcomm`.

Each hook script bridges lifecycle events from a host agent (forge, codex,
claude, copilot, …) into teamcomm's session/inbox/reservation/state
primitives via the daemon's Unix-socket JSON-RPC endpoint.

## Layout

| Subdir | Host agent                                   |
|--------|----------------------------------------------|
| `forge/`    | Phenotype forge agent                    |
| `codex/`    | Codex CLI                                |
| `claude/`   | Claude Code CLI                          |
| `copilot/`  | GitHub Copilot CLI                       |
| `droid/`    | Factory Droid (reserved — not yet populated) |

## Naming

One script per lifecycle event:

- `<agent>_session_start.sh`     — agent comes online (register session)
- `<agent>_session_end.sh`       — agent goes offline (deregister)
- `<agent>_reservation_claim.sh` — agent claims a file/area
- `<agent>_reservation_release.sh` — agent releases its claim

Scripts are idempotent — the daemon is the source of truth, hooks
are advisory notifications. Errors are logged but never abort the
calling tool.

## Wire protocol

All hooks POST a `teamcomm_client` JSON-RPC request to the daemon
Unix socket at `${TEAMCOMM_SOCKET:-/tmp/teamcomm.sock}`. The daemon
recognises a small set of `teamcomm.*` methods (separate from the
14-wire-protocol namespace which is for inter-session messaging).

| Method | Purpose |
|--------|---------|
| `teamcomm.session.register`   | Register the calling agent as a session |
| `teamcomm.session.deregister` | Deregister on agent exit |
| `teamcomm.reservation.claim`  | Reserve a file path |
| `teamcomm.reservation.release`| Release a file path |

Each script captures its owner's `TEAMCOMM_SESSION_ID` env var so
the daemon can correlate the agent's PID with its reservation.

## Activation (manual, until M3 chooses an installer)

```bash
# Forge hooks (example — wire into your hook runner):
ln -s "$(pwd)/configs/forge/" "$HOME/.forge/hooks/teamcomm"

# Codex CLI hook config (~/.codex/config.toml):
# [[hooks]]
#   event = "session.start"
#   command = "/path/to/phenotype-teamcomm/configs/codex/codex_session_start.sh"

# Claude Code CLI hook config (~/.claude/settings.json):
# {"hooks": {"SessionStart": [{"command": "/path/to/configs/claude/claude_session_start.sh"}]}}

# GitHub Copilot CLI hook config (~/.copilot/hooks.toml):
# [hooks.session-start]
# command = "/path/to/configs/copilot/copilot_session_start.sh"
```

The exact activation step is left to the agent pack that owns each
CLI; this directory is the source of truth for the scripts.

## Implementation status (2026-07-17)

| Subdir   | start | end | claim | release |
|----------|-------|-----|-------|---------|
| forge    | ✓     | ✓   | —     | —       |
| codex    | ✓     | ✓   | —     | —       |
| claude   | ✓     | ✓   | —     | —       |
| copilot  | ✓     | ✓   | —     | —       |
| droid    | —     | —   | —     | —       |

Reservation hooks (`claim` / `release`) are deferred until a host
agent actually calls them through its preferred invocation API.
