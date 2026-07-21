#!/usr/bin/env bash
# copilot_session_end.sh — deregister this GitHub Copilot CLI instance from the teamcomm daemon.
set -euo pipefail

SOCKET="${TEAMCOMM_SOCKET:-/tmp/teamcomm.sock}"
SESSION_ID="${TEAMCOMM_SESSION_ID:-}"

if [[ -z "$SESSION_ID" ]]; then
  echo "[teamcomm-copilot-end] no TEAMCOMM_SESSION_ID set; nothing to deregister" >&2
  exit 0
fi

PAYLOAD=$(cat <<JSON
{"method":"teamcomm.session.deregister","params":{"session_id":"$SESSION_ID"},"id":1}
JSON
)

if command -v teamcomm_client >/dev/null 2>&1; then
  echo "$PAYLOAD" | teamcomm_client --socket "$SOCKET" || \
    echo "[teamcomm-copilot-end] deregister failed (non-fatal)" >&2
fi
