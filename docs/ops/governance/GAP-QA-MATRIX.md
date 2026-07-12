# GAP-QA-MATRIX — sharecli

**Status:** ACTIVE  
**Companion:** [`WBS-PHASED.md`](./WBS-PHASED.md) · [`WORK_DAG.md`](../../../WORK_DAG.md) · [`TEST_COVERAGE_MATRIX.md`](../../../TEST_COVERAGE_MATRIX.md)  
**Spine:** phenotype-org-audits audit-v38 · `audit/SCORECARD-v38.md`  
**Machine tokens:** `Status: Covered` | `Gap` | `Closed` | `Blocked` | `READY` | `IN_PROGRESS` | `DONE`  
**Last sync:** 2026-07-12 (FR-002 Covered; L39/L118/L119/L32 Closed; brew PLACEHOLDER Blocked)

> Agents: update `Status:` + Evidence path only; keep Cluster/Pillar/FR-WBS keys stable for greps.

## FR acceptance (C03 / Wave3)

| Cluster | Pillar | Gap | Severity | FR/WBS link | Status | Evidence path | Owner(machine) |
|---------|--------|-----|----------|-------------|--------|---------------|----------------|
| C03 | L30.3 | FR-001 lifecycle suite | Med | FR-001 | Status: Covered | `tests/fr001_*.rs` | agent-c03 |
| C03 | L30.3 | FR-002 config suite | High | FR-002 · T-200 · W3.1 | Status: Covered | `tests/fr002_*.rs` · `docs/specs/TRACEABILITY.md` | agent-c03 |
| C03 | L30.3 | FR-003 registry suite | High | FR-003 · T-210 · W3.2 | Status: Gap | target `tests/fr003_*.rs` | agent-c03 |
| C03 | L30.3 | FR-004 health/status suite | High | FR-004 · T-220 · W3.3 | Status: Gap | target `tests/fr004_*.rs` | agent-c03 |
| C03 | L30.3 | FR-005 limits suite | High | FR-005 · T-230 · W3.4 | Status: Gap | target `tests/fr005_*.rs` | agent-c03 |
| C03 | L30.6 | Outside-in journey e2e | Med | T-240 · W3.5 | Status: Gap | `docs/journeys/` | agent-c03 |
| C03 | L30.7 | Golden CLI/TUI fixtures | Med | T-250 · W3.5 | Status: Gap | `tests/golden/` | agent-c03 |
| C03 | audit | C03 re-score after FR suites | Low | T-310 · W3.6 | Status: Blocked | `audit/.lane-c03/C03.md` | agent-c03 |

## Security / packaging closures

| Cluster | Pillar | Gap | Severity | FR/WBS link | Status | Evidence path | Owner(machine) |
|---------|--------|-----|----------|-------------|--------|---------------|----------------|
| C04 | L39 | Standalone STRIDE threat model | High | W4/W5 · L39 | Status: Closed | `THREAT_MODEL.md` | agent-c04 |
| C04 | L32 | SBOM in release tarball | Med | W4.1 · L32 | Status: Closed | `release.yml` package embeds `sharecli.cdx.json` | agent-c04 |
| C02 | L20 | STRIDE attack surface | Med | L20 | Status: Closed | `THREAT_MODEL.md` · `SECURITY.md` | agent-c04 |
| C11 | L118 | GH Release asset attach | High | W4.1 · L118 | Status: Closed | `release.yml` `github-release` job | agent-c11 |
| C11 | L119 | MSRV rust-version | Med | W4.4 · L119 | Status: Closed | `Cargo.toml` `rust-version = "1.89"` | agent-c11 |
| C11 | L108/L120 | Homebrew bottle sha PLACEHOLDER | High | W4.2 | Status: Blocked | `Formula/sharecli.rb` | agent-c11 |
| C11 | L112 | Codesign / notarize | High | W4.3 | Status: Blocked | Apple secrets | maintainer |

## Cluster residual gaps

| Cluster | Pillar | Gap | Severity | FR/WBS link | Status | Evidence path | Owner(machine) |
|---------|--------|-----|----------|-------------|--------|---------------|----------------|
| C05 | L45+ | Pyroscope push / multi-hop / live PD | Med | residual | Status: Gap | `audit/.lane-c05/C05.md` | agent-c05 |
| C02 | L21 | Federated IdP (beyond Bearer) | High | W5.1 | Status: Gap | `docs/ops/AUTH.md` | maintainer |
| C08 | L74 | Tighter bench thresholds | Low | Wave2 | Status: READY | `docs/eval/TRENDS.md` | agent-c08 |
| C06 | L51–L60 | Provenance / deny lock gaps | Med | backlog | Status: Gap | SCORECARD C06 | agent-c06 |

## Update recipe

```text
# After landing a row:
# 1) set Status: <token>
# 2) set Evidence path to file:line or glob
# 3) mirror WBS Status: in WBS-PHASED.md + WORK_DAG.md
# 4) if cluster pct changes → SCORECARD + .lane-cxx + audit_scorecard.json
```
