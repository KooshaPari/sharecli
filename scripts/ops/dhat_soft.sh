#!/usr/bin/env bash
# Soft heap profile sample for sharecli (C00 L8 / dhat-heap feature).
# Non-blocking in CI (continue-on-error). See docs/ops/alloc-profiling.md.
set -euo pipefail

BUDGET_BYTES="${SHARECLI_DHAT_BUDGET_BYTES:-67108864}" # 64 MiB soft ceiling

echo "dhat-soft: building sharecli with --features dhat-heap (release)..."
cargo build --locked --release -p sharecli --features dhat-heap

BIN="./target/release/sharecli"
rm -f dhat-heap.json

# Use a subcommand that returns normally (clap --help exits via process::exit).
stderr=$("$BIN" completions bash 2>&1 >/dev/null || true)

if [[ ! -f dhat-heap.json ]]; then
  echo "dhat-soft: missing dhat-heap.json artifact" >&2
  echo "$stderr" | tail -5 >&2
  exit 2
fi

total=$(echo "$stderr" | sed -n 's/^dhat: Total:[[:space:]]*\([0-9,]*\) bytes.*/\1/p' | tr -d ',')
if [[ -z "$total" ]]; then
  echo "dhat-soft: could not parse dhat Total line" >&2
  exit 2
fi

echo "dhat-soft: total_bytes=$total budget_bytes=$BUDGET_BYTES"
rm -f dhat-heap.json

if [[ "$total" -gt "$BUDGET_BYTES" ]]; then
  echo "dhat-soft: soft heap budget exceeded (non-blocking)" >&2
  exit 2
fi
