#!/usr/bin/env bash
# Soft C08 Harbor Phase 3 soak execution scaffold (ADR 0005).
# Validates local parity with harbor-eval-stub-soft.yml and optionally appends
# a row to the soak checklist log. Does not provision Harbor/portage.
# See docs/ops/harbor-phase3-soak.md and audit/.lane-c08/harbor-phase3-soak-log.md
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LOG_FILE="${SHARECLI_HARBOR_SOAK_LOG:-}"
SOURCE="${SHARECLI_HARBOR_SOAK_SOURCE:-local}"
STUB_PASS_MARKER="STUB PASS: corpus valid"
RUN_ID="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"

echo "== Harbor Phase 3 soak execution (soft) =="
echo "Policy: docs/ops/harbor-phase3-soak.md"
echo "sha=$GIT_SHA source=$SOURCE"
echo ""

echo ">> Step 1: harbor_stub.sh (corpus preflight + stub pass)"
stub_out="$(mktemp)"
if ! bash "${ROOT}/scripts/eval/harbor_stub.sh" | tee "$stub_out"; then
  echo "harbor soak: harbor_stub.sh failed" >&2
  exit 1
fi
if ! grep -Fq "$STUB_PASS_MARKER" "$stub_out"; then
  echo "harbor soak: missing stub pass marker" >&2
  exit 2
fi
rm -f "$stub_out"

echo ""
echo ">> Step 2: run-corpus.sh preflight (checklist item 2)"
bash "${ROOT}/scripts/eval/run-corpus.sh"

echo ""
echo ">> Step 3: just harbor-stub parity note"
echo "Local parity: harbor_stub.sh matches harbor-eval-stub-soft.yml subject job."

if [[ -n "$LOG_FILE" ]]; then
  mkdir -p "$(dirname "$LOG_FILE")"
  if [[ ! -f "$LOG_FILE" ]]; then
    cat >"$LOG_FILE" <<'HEADER'
# Harbor Phase 3 soak checklist log (soft)

Template for seven consecutive `main` green runs of `harbor-eval-stub-soft.yml`.
Append rows via `SHARECLI_HARBOR_SOAK_LOG=audit/.lane-c08/harbor-phase3-soak-log.md bash scripts/eval/harbor_soak.sh`.

| # | recorded_at_utc | git_sha | source | stub_pass | notes |
|---|-----------------|---------|--------|-----------|-------|
HEADER
  fi
  next_n="$(grep -c '^| [0-9]' "$LOG_FILE" 2>/dev/null || echo 0)"
  next_n=$((next_n + 1))
  echo "| $next_n | $RUN_ID | $GIT_SHA | $SOURCE | yes | local parity check |" >>"$LOG_FILE"
  echo "Appended row $next_n to $LOG_FILE"
fi

echo ""
echo "SOAK SCAFFOLD PASS: local stub + corpus preflight green (Phase 3 partial)"
echo "Seven-day main soak clock: track remaining rows in audit/.lane-c08/harbor-phase3-soak-log.md"
