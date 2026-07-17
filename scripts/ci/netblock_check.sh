#!/usr/bin/env bash
# C06 L54 — soft probe: CARGO_NET_OFFLINE=1 cargo check after locked fetch.
# See docs/ops/network-block-build.md and docs/ops/hermetic-builds.md.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

export CARGO_NET_OFFLINE=1

echo ">> warming registry cache (network allowed for fetch only)"
if ! CARGO_NET_OFFLINE=0 cargo fetch --locked; then
  echo "::warning::cargo fetch --locked failed — network or lockfile issue" >&2
  echo "Fix lockfile or run fetch on a networked host; see docs/ops/network-block-build.md" >&2
  exit 1
fi

echo ">> CARGO_NET_OFFLINE=1 cargo check --locked --offline -p sharecli"
if cargo check --locked --offline -p sharecli; then
  echo ">> netblock check green"
  exit 0
fi

echo "::warning::offline check failed with CARGO_NET_OFFLINE=1" >&2
echo "Remediation:" >&2
echo "  1. CARGO_NET_OFFLINE=0 cargo fetch --locked" >&2
echo "  2. scripts/ci/netblock_check.sh" >&2
echo "  3. docs/ops/network-block-build.md (vendor spike if cache stays cold)" >&2
exit 1
