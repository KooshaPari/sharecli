#!/usr/bin/env bash
# Hard idle RSS gate for sharecli serve (C00 L8).
# Budget: docs/ops/memory.md — serve idle < 64 MiB on linux CI runners.
# Unlike rss_soft.sh, this script causes CI failure (exit 1) on breach.
set -euo pipefail

BIN="${SHARECLI_BIN:-./target/release/sharecli}"
BUDGET_KIB="${SHARECLI_RSS_BUDGET_KIB:-65536}" # 64 MiB
BIND="${SHARECLI_RSS_BIND:-127.0.0.1:9012}"
URL="http://${BIND}/healthz"

if [[ ! -x "$BIN" ]]; then
  echo "missing binary: $BIN (cargo build --release -p sharecli)" >&2
  exit 1
fi

"$BIN" serve --bind "$BIND" &
pid=$!
trap 'kill $pid 2>/dev/null || true' EXIT

for _ in $(seq 1 30); do
  if curl -sf -o /dev/null "$URL" 2>&1; then
    break
  fi
  echo "Waiting for sharecli health endpoint..." >&2
  sleep 1
done
if ! curl -sf -o /dev/null "$URL" 2>&1; then
  echo "::error::sharecli health endpoint not responding after 30s" >&2
  exit 1
fi

# RSS in KiB from /proc
rss_kib=$(awk '/VmRSS:/ {print $2}' "/proc/$pid/status")
echo "sharecli rss-gate: pid=$pid rss_kib=$rss_kib budget_kib=$BUDGET_KIB bind=$BIND"

if [[ "$rss_kib" -gt "$BUDGET_KIB" ]]; then
  echo "FAIL: hard RSS budget exceeded ($rss_kib KiB > $BUDGET_KIB KiB)" >&2
  echo "Increase budget in docs/ops/memory.md + scripts/ops/rss_gate.sh if regression is intentional." >&2
  exit 1
fi

echo "PASS: idle RSS within budget ($rss_kib KiB <= $BUDGET_KIB KiB)"
