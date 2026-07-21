#!/usr/bin/env bash
# codex_session_start.sh — register this Codex CLI instance with the teamcomm daemon.
set -euo pipefail

SOCKET="${TEAMCOMM_SOCKET:-/tmp/teamcomm.sock}"
AGENT_TYPE="codex"

META=$(cat <<JSON
{
  "host":"$(hostname)",
  "cwd":"${CODEX_CWD:-$PWD}",
  "model":"${CODEX_MODEL:-unknown}",
  "started_at":"$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
JSON
)

PAYLOAD=$(cat <<JSON
{"method":"teamcomm.session.register","params":{"agent_type":"$AGENT_TYPE","pid":$$,"metadata":$META},"id":1}
JSON
)

if command -v teamcomm_client >/dev/null 2>&1; then
  echo "$PAYLOAD" | teamcomm_client --socket "$SOCKET" || \
    echo "[teamcomm-codex-start] client call failed (non-fatal)" >&2
else
  echo "[teamcomm-codex-start] teamcomm_client not on PATH; skipping registration" >&2
fi
