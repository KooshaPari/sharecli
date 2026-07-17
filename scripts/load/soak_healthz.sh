#!/usr/bin/env bash
# Soft soak: poll GET /healthz for SHARECLI_SOAK_SEC (default 300 = 5 min).
# Exit 0 iff every response is HTTP 2xx.
# See docs/ops/soak-chaos.md and docs/eval/REPRO.md.
set -euo pipefail

URL="${SHARECLI_LOAD_URL:-http://127.0.0.1:7700/healthz}"
DURATION_SEC="${SHARECLI_SOAK_SEC:-300}"
INTERVAL_SEC="${SHARECLI_SOAK_INTERVAL_SEC:-1}"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl required" >&2
  exit 1
fi

if ! [[ "$DURATION_SEC" =~ ^[0-9]+$ ]] || [[ "$DURATION_SEC" -lt 1 ]]; then
  echo "SHARECLI_SOAK_SEC must be a positive integer" >&2
  exit 1
fi

if ! [[ "$INTERVAL_SEC" =~ ^[0-9]+$ ]] || [[ "$INTERVAL_SEC" -lt 1 ]]; then
  echo "SHARECLI_SOAK_INTERVAL_SEC must be a positive integer" >&2
  exit 1
fi

ok=0
fail=0
start_s=$(date +%s)
deadline_s=$((start_s + DURATION_SEC))

while [[ "$(date +%s)" -lt "$deadline_s" ]]; do
  code=$(curl -sS -o /dev/null -w "%{http_code}" --max-time 2 "$URL" || echo "000")
  if [[ "$code" =~ ^2[0-9][0-9]$ ]]; then
    ok=$((ok + 1))
  else
    fail=$((fail + 1))
    echo "soak non-2xx: url=$URL code=$code" >&2
  fi

  now_s=$(date +%s)
  remaining_s=$((deadline_s - now_s))
  if [[ "$remaining_s" -le 0 ]]; then
    break
  fi
  if [[ "$INTERVAL_SEC" -gt "$remaining_s" ]]; then
    sleep "$remaining_s"
  else
    sleep "$INTERVAL_SEC"
  fi
done

end_s=$(date +%s)
elapsed_s=$((end_s - start_s))
total=$((ok + fail))
rate=0
if [[ "$total" -gt 0 ]]; then
  rate=$(( (ok * 100) / total ))
fi

echo "sharecli soak: url=$URL duration_sec=$DURATION_SEC interval_sec=$INTERVAL_SEC ok=$ok fail=$fail success_pct=$rate elapsed_s=$elapsed_s"
if [[ "$fail" -gt 0 ]]; then
  exit 2
fi
