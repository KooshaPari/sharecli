#!/usr/bin/env bash
# Soft C08 Harbor eval stub (ADR 0005 Phase 2).
# Validates supervisor corpus fixtures, then prints stub pass — no Harbor/portage env.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

echo "== Harbor eval stub (ADR 0005 Phase 2) =="
echo "Policy: docs/adr/0005-agent-eval-supersede.md (ADR 0002 remains authoritative)"
echo "Ops:   docs/ops/harbor-eval-stub.md"
echo ""

echo ">> Preflight: supervisor corpus fixtures"
bash "${ROOT}/scripts/eval/run-corpus.sh"

echo ""
echo "STUB PASS: corpus valid; Harbor task runner not wired (Phase 2 soft)"
echo "Harbor/portage env provisioning deferred — see ADR 0005 Phase 3 soak"
