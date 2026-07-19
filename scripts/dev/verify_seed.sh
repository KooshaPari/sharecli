#!/usr/bin/env bash
# C07 L70 — verify committed dev seed fixture parses and passes validate_config.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

echo ">> dev seed verify (fixture + validate_config gate)"
cargo test --locked c07_l70_dev_seed_fixture_valid -- --exact --nocapture
