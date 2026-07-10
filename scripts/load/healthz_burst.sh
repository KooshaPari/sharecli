#!/usr/bin/env bash
# Macro / load tier: burst GET /healthz against a running sharecli serve.
# See docs/eval/REPRO.md for pins and recording instructions.
set -euo pipefail

URL="${SHARECLI_LOAD_URL:-http://127.0.0.1:7700/healthz}"
N="${SHARECLI_LOAD_N:-200}"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl required" >&2
  exit 1
fi

ok=0
fail=0
start_ns=$(date +%s%N 2>/dev/null || echo 0)

for _ in $(seq 1 "$N"); do
  if curl -fsS -o /dev/null --max-time 2 "$URL"; then
    ok=$((ok + 1))
  else
    fail=$((fail + 1))
  fi
done

end_ns=$(date +%s%N 2>/dev/null || echo 0)
elapsed_ms=0
if [[ "$start_ns" != "0" && "$end_ns" != "0" ]]; then
  elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
fi

total=$((ok + fail))
rate=0
if [[ "$total" -gt 0 ]]; then
  rate=$(( (ok * 100) / total ))
fi

echo "sharecli load: url=$URL n=$N ok=$ok fail=$fail success_pct=$rate elapsed_ms=$elapsed_ms"
if [[ "$fail" -gt 0 ]]; then
  exit 2
fi
