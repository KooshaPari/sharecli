#!/usr/bin/env bash
# Soft C08 corpus harness — validate synthetic scenario JSON fixtures.
# Optional live probe: SHARECLI_CORPUS_LIVE=1 SHARECLI_BASE_URL=http://127.0.0.1:9000
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CORPUS="${ROOT}/docs/eval/corpus/scenarios"
fail=0
count=0
shopt -s nullglob
for f in "${CORPUS}"/*.json; do
  count=$((count + 1))
  if ! python3 - "${f}" <<'PY'
import json, sys
path = sys.argv[1]
with open(path, encoding="utf-8") as fh:
    data = json.load(fh)
for key in ("id", "name", "expect"):
    if key not in data:
        raise SystemExit(f"{path}: missing {key}")
expect = data["expect"]
if not isinstance(expect, dict) or not (("health" in expect) or ("gate" in expect)):
    raise SystemExit(f"{path}: expect.health or expect.gate required")
print(f"OK {data['id']} {data['name']}")
PY
  then
    echo "FAIL ${f}" >&2
    fail=1
  fi
done
if [[ "${count}" -eq 0 ]]; then
  echo "No corpus scenarios under ${CORPUS}" >&2
  exit 1
fi
echo "Validated ${count} corpus scenario(s)"

if [[ "${SHARECLI_CORPUS_LIVE:-}" == "1" ]]; then
  BASE="${SHARECLI_BASE_URL:-http://127.0.0.1:9000}"
  echo "Live corpus: GET ${BASE}/healthz"
  body="$(curl -fsS "${BASE}/healthz")"
  status="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["status"])' <<<"${body}")"
  if [[ "${status}" != "ok" ]]; then
    echo "Live healthz status=${status} (want ok)" >&2
    fail=1
  else
    echo "Live healthz OK"
  fi
fi

exit "${fail}"
