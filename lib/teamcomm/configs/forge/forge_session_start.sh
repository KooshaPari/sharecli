#!/usr/bin/env bash
# forge_session_start.sh — register this Forge agent with the teamcomm daemon.
#
# Usage: invoked by the Forge hook runner when a forge agent starts.
# Idempotent: re-running just refreshes the heartbeat on the existing session_id.
set -euo pipefail

SOCKET="${TEAMCOMM_SOCKET:-/tmp/teamcomm.sock}"
AGENT_TYPE="forge"
METADATA_KEYS=(
  "host=$(hostname)"
  "cwd=${FORGE_CWD:-$PWD}"
  "fork=${FORGE_FORK:-unknown}"
  "started_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
)

# Build metadata JSON object
META="{"
for k in "${METADATA_KEYS[@]}"; do
  META+="\"${k%%=*}\":\"${k#*=}\","
done
META="${META%,}}"

PAYLOAD=$(cat <<JSON
{"method":"teamcomm.session.register","params":{"agent_type":"$AGENT_TYPE","pid":$$,"metadata":$META},"id":1}
JSON
)

# Talk to the daemon via the teamcomm_client binary (preferred), or curl/socat as fallback.
if command -v teamcomm_client >/dev/null 2>&1; then
  echo "$PAYLOAD" | teamcomm_client --socket "$SOCKET" || \
    echo "[teamcomm-forge-start] client call failed (non-fatal)" >&2
else
  echo "[teamcomm-forge-start] teamcomm_client not on PATH; skipping registration" >&2
fi
