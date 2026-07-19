#!/usr/bin/env bash
# C04 L38 — OSV / GHSA lockfile scan (HIGH + CRITICAL hard gate).
# See docs/ops/osv-hard-fail.md.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

if ! command -v osv-scanner >/dev/null 2>&1; then
  echo "osv-scanner not found; install: go install github.com/google/osv-scanner/cmd/osv-scanner@latest" >&2
  exit 1
fi

echo ">> osv-scanner scan -L Cargo.lock (fail on HIGH/CRITICAL summary counts)"
set +e
output="$(osv-scanner scan -L Cargo.lock 2>&1 | tee /dev/stderr)"
set -e

# Parity with CI `osv-scanner-action` `--severity=HIGH,CRITICAL`.
if echo "${output}" | grep -Eq '\([1-9][0-9]* Critical,|[0-9]+ Critical, [1-9][0-9]* High'; then
  echo "OSV HIGH/CRITICAL vulnerabilities present in Cargo.lock" >&2
  exit 1
fi

echo ">> osv HIGH/CRITICAL gate green"
