#!/usr/bin/env bash
# Hyperfine load/latency harness for GET /healthz (C08 L72 toolbelt).
# Requires: hyperfine, a running `sharecli serve` (default http://127.0.0.1:9000).
set -euo pipefail

URL="${SHARECLI_HEALTHZ_URL:-http://127.0.0.1:9000/healthz}"
RUNS="${SHARECLI_HYPERFINE_RUNS:-50}"
WARMUP="${SHARECLI_HYPERFINE_WARMUP:-10}"

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "hyperfine not found; install from https://github.com/sharkdp/hyperfine" >&2
  exit 127
fi

echo "hyperfine → ${URL} (runs=${RUNS} warmup=${WARMUP})"
hyperfine \
  --warmup "${WARMUP}" \
  --runs "${RUNS}" \
  --export-json "${SHARECLI_HYPERFINE_OUT:-docs/eval/baselines/hyperfine-healthz.json}" \
  "curl -sf -o /dev/null '${URL}'"
