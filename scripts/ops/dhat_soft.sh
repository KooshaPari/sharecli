#!/usr/bin/env bash
# Soft heap profile sample for sharecli (C00 L8 / dhat-heap feature).
# Non-blocking in CI (continue-on-error). See docs/ops/alloc-profiling.md.
set -euo pipefail

BUDGET_BYTES="${SHARECLI_DHAT_BUDGET_BYTES:-67108864}" # 64 MiB soft ceiling

echo "dhat-soft: building sharecli with --features dhat-heap (release)..."
cargo build --locked --release -p sharecli --features dhat-heap

BIN="./target/release/sharecli"
rm -f dhat-heap.json

# Short-lived CLI smoke so Profiler drops cleanly and writes dhat-heap.json.
"$BIN" --help >/dev/null

if [[ ! -f dhat-heap.json ]]; then
  echo "dhat-soft: missing dhat-heap.json artifact" >&2
  exit 2
fi

total=$(python3 - <<'PY'
import json
from pathlib import Path
data = json.loads(Path("dhat-heap.json").read_text())
print(int(data.get("total_bytes", data.get("total", 0))))
PY
)

echo "dhat-soft: total_bytes=$total budget_bytes=$BUDGET_BYTES"
rm -f dhat-heap.json

if [[ "$total" -gt "$BUDGET_BYTES" ]]; then
  echo "dhat-soft: soft heap budget exceeded (non-blocking)" >&2
  exit 2
fi
