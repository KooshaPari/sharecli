#!/usr/bin/env bash
# Soft live pool probe — C08. Starts serve, hits health endpoints.
set -euo pipefail
BIN="${SHARECLI_BIN:-./target/release/sharecli}"
BIND="${SHARECLI_PROBE_BIND:-127.0.0.1:9022}"
"$BIN" serve --bind "$BIND" &
pid=$!
trap 'kill $pid 2>/dev/null || true' EXIT
for _ in $(seq 1 30); do
  curl -sf -o /dev/null "http://${BIND}/healthz" && break
  sleep 1
done
curl -sf "http://${BIND}/healthz" | tee /tmp/healthz.json
curl -sf "http://${BIND}/health/processes" | tee /tmp/procs.json
python3 - <<'PY'
import json
h=json.load(open("/tmp/healthz.json"))
p=json.load(open("/tmp/procs.json"))
assert "status" in h or isinstance(h, dict)
assert isinstance(p, (dict, list))
print("live_pool_probe ok")
PY
