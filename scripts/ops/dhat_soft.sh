#!/usr/bin/env bash
# Soft heap profile sample for sharecli (C00 L8 / dhat-heap feature).
# Non-blocking in CI (continue-on-error). See docs/ops/alloc-profiling.md.
set -euo pipefail

BUDGET_BYTES="${SHARECLI_DHAT_BUDGET_BYTES:-67108864}" # 64 MiB soft ceiling
# Avoid inheriting another worktree's CARGO_TARGET_DIR (binary would land elsewhere).
unset CARGO_TARGET_DIR

echo "dhat-soft: building sharecli with --features dhat-heap (release)..."
cargo build --locked --release -p sharecli --features dhat-heap

BIN="./target/release/sharecli"
if [[ ! -x "$BIN" ]]; then
  echo "dhat-soft: missing binary at $BIN" >&2
  exit 2
fi
rm -f dhat-heap.json

# clap --help/--version call process::exit and skip Drop, so Profiler never
# flushes dhat-heap.json. Use a subcommand that returns from main() normally.
stderr=$(SHARECLI_DHAT_PROFILE=1 "$BIN" completions bash 2>&1 >/dev/null || true)

if [[ ! -f dhat-heap.json ]]; then
  echo "dhat-soft: missing dhat-heap.json artifact (need normal main return; not --help)" >&2
  echo "$stderr" | tail -5 >&2
  exit 2
fi

total=$(python3 - <<'PY'
import json
from pathlib import Path
data = json.loads(Path("dhat-heap.json").read_text())
# Legacy flat keys (if ever present)
if "total_bytes" in data:
    print(int(data["total_bytes"]))
elif "total" in data:
    print(int(data["total"]))
else:
    # dhatFileVersion 2 heap profile: sum program-point totals (`tb`)
    print(sum(int(pp.get("tb", 0)) for pp in data.get("pps", [])))
PY
)
if [[ -z "$total" || "$total" -eq 0 ]]; then
  # Fallback: parse dhat stderr summary line
  total=$(echo "$stderr" | sed -n 's/^dhat: Total:[[:space:]]*\([0-9,]*\) bytes.*/\1/p' | tr -d ',')
fi
if [[ -z "$total" ]]; then
  echo "dhat-soft: could not parse heap total from JSON or dhat stderr" >&2
  exit 2
fi

echo "dhat-soft: total_bytes=$total budget_bytes=$BUDGET_BYTES"
rm -f dhat-heap.json

if [[ "$total" -gt "$BUDGET_BYTES" ]]; then
  echo "dhat-soft: soft heap budget exceeded (non-blocking)" >&2
  exit 2
fi
