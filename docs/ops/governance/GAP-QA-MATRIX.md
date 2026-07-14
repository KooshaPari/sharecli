# GAP-QA-MATRIX — sharecli

**Status:** ACTIVE  
**Companion:** [`WBS-PHASED.md`](./WBS-PHASED.md) · [`WORK_DAG.md`](../../../WORK_DAG.md) · [`TEST_COVERAGE_MATRIX.md`](../../../TEST_COVERAGE_MATRIX.md)  
**Spine:** phenotype-org-audits audit-v38 · `audit/SCORECARD-v38.md`  
**Machine tokens:** `Status: Covered` | `Gap` | `Closed` | `Blocked` | `READY` | `IN_PROGRESS` | `DONE`  
**Last sync:** 2026-07-13 (C06 67% C L52+L55; C09 67% C a11y W6.1–W6.4 Closed; overall ~67% C)

> Agents: update `Status:` + Evidence path only; keep Cluster/Pillar/FR-WBS keys stable for greps.

## FR acceptance (C03 / Wave3)

| Cluster | Pillar | Gap | Severity | FR/WBS link | Status | Evidence path | Owner(machine) |
|---------|--------|-----|----------|-------------|--------|---------------|----------------|
| C03 | L30.3 | FR-001 lifecycle suite | Med | FR-001 | Status: Covered | `tests/fr001_*.rs` | agent-c03 |
| C03 | L30.3 | FR-002 config suite | High | FR-002 · T-200 · W3.1 | Status: Covered | `tests/fr002_*.rs` · `docs/specs/TRACEABILITY.md` | agent-c03 |
| C03 | L30.3 | FR-003 registry suite | High | FR-003 · T-210 · W3.2 | Status: Covered | `tests/fr003_project_registry.rs` · `tests/fr003_project_discover.rs` | agent-c03 |
| C03 | L30.3 | FR-004 health/status suite | High | FR-004 · T-220 · W3.3 | Status: Covered | `tests/fr004_status_health.rs` · `tests/fr004_pool_status.rs` | agent-c03 |
| C03 | L30.3 | FR-005 limits suite | High | FR-005 · T-230 · W3.4 | Status: Covered | `tests/fr005_project_limits.rs` · `tests/fr005_resource_check.rs` | agent-c03 |
| C03 | L30.6 | Outside-in journey e2e | Med | T-240 · W3.5 | Status: Covered | `tests/quick_start_journey.rs` · `docs/journeys/quick-start.md` | agent-c03 |
| C03 | L30.7 | Golden CLI/TUI fixtures | Med | T-250 · W3.5 | Status: Covered | `tests/golden/` · `tests/golden_snapshots.rs` | agent-c03 |
| C03 | L30.12 | Unhappy-path friction suite | Med | T-300 · W3.5 | Status: Covered | `tests/fr_invalid_missing_friction.rs` | agent-c03 |
| C03 | L30.9 | Multi-agent claim-lock | Low | T-260 | Status: Covered | `AGENTS.md` Claim-lock protocol | agent-c03 |
| C03 | L30.10 | Local loop timing budgets | Low | T-270 | Status: Covered | `docs/ops/LOCAL_LOOP_BUDGETS.md` | agent-c03 |
| C03 | audit | C03 re-score after FR suites | Low | T-310 · W3.6 | Status: Covered | `audit/.lane-c03/C03.md` · SCORECARD 92% A | agent-c03 |

## Security / packaging closures

| Cluster | Pillar | Gap | Severity | FR/WBS link | Status | Evidence path | Owner(machine) |
|---------|--------|-----|----------|-------------|--------|---------------|----------------|
| C04 | L39 | Standalone STRIDE threat model | High | W4/W5 · L39 | Status: Closed | `THREAT_MODEL.md` | agent-c04 |
| C04 | L32 | SBOM in release tarball | Med | W4.1 · L32 | Status: Closed | `release.yml` package embeds `sharecli.cdx.json` | agent-c04 |
| C02 | L20 | STRIDE attack surface | Med | L20 | Status: Closed | `THREAT_MODEL.md` · `SECURITY.md` | agent-c04 |
| C11 | L118 | GH Release asset attach | High | W4.1 · L118 | Status: Closed | `release.yml` `github-release` job | agent-c11 |
| C11 | L119 | MSRV rust-version | Med | W4.4 · L119 | Status: Closed | `Cargo.toml` `rust-version = "1.89"` | agent-c11 |
| C11 | L108/L120 | Homebrew bottle sha PLACEHOLDER | High | W4.2 | Status: Closed | `Formula/sharecli.rb` sha256 from v0.3.0 darwin | agent-c11 |
| C11 | L112 | Codesign / notarize | High | W4.3 | Status: Blocked | Apple secrets | maintainer |

## Cluster residual gaps

| Cluster | Pillar | Gap | Severity | FR/WBS link | Status | Evidence path | Owner(machine) |
|---------|--------|-----|----------|-------------|--------|---------------|----------------|
| C05 | L45+ | Pyroscope push / multi-hop / live PD | Med | residual | Status: Gap | `audit/.lane-c05/C05.md` | agent-c05 |
| C02 | L21 | Federated IdP (beyond Bearer) | High | W5.1 | Status: Closed | `src/serve_auth.rs` + `docs/ops/AUTH.md` + `tests/fr012_serve_jwt_auth.rs` | maintainer |
| C02 | L23 | Audit retention + rotation | Med | W5.2 | Status: Closed | `src/audit_log.rs` + `docs/ops/AUTH.md` | maintainer |
| C02 | L27 | AuthN/HTTP burn alerts | Med | W5.2 | Status: Closed | `docs/ops/alertmanager/sharecli.yml` + `src/http_red.rs` | maintainer |
| C08 | L74 | Tighter bench thresholds | Low | Wave2 | Status: READY | `docs/eval/TRENDS.md` | agent-c08 |
| C06 | L52 | Bit-identical repro-check CI | Med | FR-002 · W6.1 | Status: Closed | `scripts/repro-check.sh` · `repro-check.yml` | agent-c06 |
| C06 | L55 | Dependency confusion / deny sources | Med | W6.2 | Status: Closed | `deny.toml` · `deny.yml` | agent-c06 |
| C06 | L56 | Container cosign publish | Low | W6.3 | Status: Gap | `docs/slsa.md` cosign roadmap | agent-c06 |
| C06 | L53–L54 | SLSA L3 / hermetic builds | Med | backlog | Status: Gap | `audit/.lane-c06/C06.md` | agent-c06 |

## C09 accessibility (Wave6)

| Cluster | Pillar | Gap | Severity | FR/WBS link | Status | Evidence path | Owner(machine) |
|---------|--------|-----|----------|-------------|--------|---------------|----------------|
| C09 | L81.1 | WCAG Level A baseline + landmark tests | Med | W6.1 · FR-004 NFR | Status: Closed | `docs/a11y/README.md` · `tests/a11y/dashboard_landmarks.rs` | agent-c09 |
| C09 | L81.2 | Contrast ratios for theme tokens | Med | W6.2 | Status: Closed | `docs/a11y/contrast.md` · `assets/tokens.css` | agent-c09 |
| C09 | L81.3 | TUI keyboard quit-key matrix + unit tests | Med | W6.3 | Status: Closed | `docs/a11y/keyboard.md` · `crates/sharecli-thermal-tui` `is_quit_key` | agent-c09 |
| C09 | L81.5 | Dashboard landmarks (`nav`/`main`) | Med | W6.1 | Status: Closed | `src/dashboard.html` · `tests/a11y/` | agent-c09 |
| C09 | L81.6/L81.7 | Status visibility + degraded-mode recovery docs | Med | W6.4 · FR-004 | Status: Closed | `docs/a11y/status-and-recovery.md` · `src/main.rs` after_long_help | agent-c09 |
| C09 | L81.1 | axe-core CI for dashboard | Low | backlog | Status: Gap | `audit/.lane-c09/C09.md` | agent-c09 |
| C09 | L81.11 | Responsive TUI + viewport e2e | Low | backlog | Status: Gap | `audit/.lane-c09/C09.md` | agent-c09 |

## Update recipe

```text
# After landing a row:
# 1) set Status: <token>
# 2) set Evidence path to file:line or glob
# 3) mirror WBS Status: in WBS-PHASED.md + WORK_DAG.md
# 4) if cluster pct changes → SCORECARD + .lane-cxx + audit_scorecard.json
```
