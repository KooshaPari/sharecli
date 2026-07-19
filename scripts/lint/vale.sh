#!/usr/bin/env bash
# Vale inclusive-language lint for user-facing docs (C09 L81.10).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if ! command -v vale >/dev/null 2>&1; then
  echo "vale: not installed; brew install vale or see https://vale.sh" >&2
  exit 1
fi

vale sync --config .vale.ini 2>/dev/null || true

PATHS=(
  README.md
  CONTRIBUTING.md
  docs/a11y
  docs/journeys
)

vale --config .vale.ini --output=line "${PATHS[@]}"
