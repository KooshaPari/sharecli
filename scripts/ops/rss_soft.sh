#!/usr/bin/env bash
# Soft idle RSS sample for sharecli serve (C00 L8).
# Budget: docs/ops/memory.md — serve idle < 64 MiB on linux CI runners.
set -euo pipefail

BIN="${SHARECLI_BIN:-./target/release/sharecli}"
BUDGET_KIB="${SHARECLI_RSS_BUDGET_KIB:-65536}" # 64 MiB
BIND="${SHARECLI_RSS_BIND:-127.0.0.1:9011}"
URL="http://${BIND}/healthz"

if [[ ! -x "$BIN" ]]; then
  echo "missing binary: $BIN (cargo build --release -p sharecli)" >&2
  exit 1
fi

"$BIN" serve --bind "$BIND" &
pid=$!
trap 'kill $pid 2>/dev/null || true' EXIT

for _ in $(seq 1 30); do
  if curl -sf -o /dev/null "$URL"; then
    break
  fi
  sleep 1
done
curl -sf -o /dev/null "$URL"

# RSS in KiB from /proc
rss_kib=$(awk '/VmRSS:/ {print $2}' "/proc/$pid/status")
echo "sharecli rss-soft: pid=$pid rss_kib=$rss_kib budget_kib=$BUDGET_KIB bind=$BIND"

if [[ "$rss_kib" -gt "$BUDGET_KIB" ]]; then
  echo "soft budget exceeded (non-blocking in CI via continue-on-error)" >&2
  exit 2
fi
