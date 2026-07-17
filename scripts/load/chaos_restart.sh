#!/usr/bin/env bash
# Soft chaos: start sharecli serve, kill it, restart, verify /healthz recovers within 30s.
# See docs/ops/soak-chaos.md and docs/eval/REPRO.md.
set -euo pipefail

URL="${SHARECLI_LOAD_URL:-http://127.0.0.1:9000/healthz}"
BIND="${SHARECLI_SERVE_BIND:-127.0.0.1:9000}"
BIN="${SHARECLI_SERVE_BIN:-./target/release/sharecli}"
RECOVER_SEC="${SHARECLI_CHAOS_RECOVER_SEC:-30}"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl required" >&2
  exit 1
fi

if [[ ! -x "$BIN" ]]; then
  echo "serve binary not found or not executable: $BIN" >&2
  exit 1
fi

if ! [[ "$RECOVER_SEC" =~ ^[0-9]+$ ]] || [[ "$RECOVER_SEC" -lt 1 ]]; then
  echo "SHARECLI_CHAOS_RECOVER_SEC must be a positive integer" >&2
  exit 1
fi

serve_pid=""

cleanup() {
  if [[ -n "$serve_pid" ]]; then
    kill "$serve_pid" 2>/dev/null || true
    wait "$serve_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

wait_healthz() {
  local limit_sec="$1"
  local deadline_s=$(( $(date +%s) + limit_sec ))
  while [[ "$(date +%s)" -lt "$deadline_s" ]]; do
    if curl -fsS -o /dev/null --max-time 2 "$URL"; then
      return 0
    fi
    sleep 1
  done
  return 1
}

start_serve() {
  "$BIN" serve --bind "$BIND" &
  serve_pid=$!
}

echo ">> chaos: starting serve on $BIND"
start_serve
if ! wait_healthz 30; then
  echo "chaos: serve did not become healthy within 30s" >&2
  exit 2
fi

echo ">> chaos: healthy; killing pid=$serve_pid"
kill -9 "$serve_pid" 2>/dev/null || true
wait "$serve_pid" 2>/dev/null || true
serve_pid=""

# Allow the bind address to be released before restart.
sleep 1

echo ">> chaos: restarting serve"
start_serve
recover_start_s=$(date +%s)
if ! wait_healthz "$RECOVER_SEC"; then
  echo "chaos: /healthz did not recover within ${RECOVER_SEC}s" >&2
  exit 3
fi
recover_elapsed_s=$(( $(date +%s) - recover_start_s ))

echo "sharecli chaos: url=$URL recover_sec=$recover_elapsed_s limit_sec=$RECOVER_SEC"
