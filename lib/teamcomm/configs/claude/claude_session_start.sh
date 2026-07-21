#!/usr/bin/env bash
# claude_session_start.sh — register this Claude Code CLI instance with the teamcomm daemon.
set -euo pipefail

SOCKET="${TEAMCOMM_SOCKET:-/tmp/teamcomm.sock}"
AGENT_TYPE="claude"

META=$(cat <<JSON
{
  "host":"$(hostname)",
  "cwd":"${CLAUDE_CWD:-$PWD}",
  "model":"${CLAUDE_MODEL:-unknown}",
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
    echo "[teamcomm-claude-start] client call failed (non-fatal)" >&2
else
  echo "[teamcomm-claude-start] teamcomm_client not on PATH; skipping registration" >&2
fi
