# Wave 20 Spec — ShareCLI v0.5.0 Roadmap

**Date:** 2026-08-31
**Base:** v0.4.0 (PR #810)
**Scorecard:** A+ (99.1%)

## Executive Summary

Wave 20 focuses on three strategic goals:
1. **External infrastructure integration** (Apple signing, Azure KV, OTel collector)
2. **Production readiness** (Dockerfile, dashboard verification, WinFSP docs)
3. **Dogfood validation** (use ShareCLI to govern its own development)

## Gap Analysis

### G01: Apple Code Signing (C11 L112)
- **Current:** Soft gate (continue-on-error: true)
- **Target:** Hard gate
- **Blocker:** Apple Developer secrets not configured in GH Actions
- **Owner:** Koosh (manual setup)
- **Effort:** M (30 min manual + 1 hour CI verification)
- **Path:** Follow `docs/ops/governance/APPLE_SECRETS_SETUP.md`
  1. Export .p12 from Keychain Access
  2. Base64-encode: `base64 -i cert.p12 | pbcopy`
  3. Add 5 secrets to GitHub Actions
  4. Push to trigger codesign workflow
  5. Verify notarize + staple in CI logs

### G02: Azure Key Vault Windows Signing (C11 L112)
- **Current:** Soft gate (codesign.yml Windows job)
- **Target:** Hard gate
- **Blocker:** Azure subscription needed (~$0.36/yr)
- **Owner:** Koosh (Azure portal setup)
- **Effort:** M (1 hour setup)
- **Path:**
  1. Create Azure Key Vault (Standard tier)
  2. Generate self-signed certificate
  3. Register Azure AD app + service principal
  4. Add 4 secrets: AZURE_KEY_VAULT_URL, CLIENT_ID, CLIENT_SECRET, TENANT_ID
  5. Update codesign.yml Windows job

### G03: Homebrew Bottle SHA (C11 L111)
- **Current:** PLACEHOLDER sha256 in Formula/sharecli.rb
- **Target:** Real SHA256 from `brew bottle`
- **Blocker:** Needs macOS machine + v* tag
- **Owner:** Koosh (macOS, post-v0.4.0-tag)
- **Effort:** S (15 min)
- **Path:**
  1. Merge PR #810 (v0.4.0 version bump)
  2. Create tag: `gh release create v0.4.0 --generate-notes`
  3. Wait for release.yml to produce binaries
  4. On macOS: `brew install --build-from-source Formula/sharecli.rb`
  5. `brew bottle Formula/sharecli.rb`
  6. Replace PLACEHOLDER with real SHA256

### G04: Dockerfile Production Readiness (C06)
- **Current:** Smoke image with `|| true` on cargo build
- **Target:** Production multi-stage Dockerfile
- **Owner:** Forge
- **Effort:** M (2 hours)
- **Path:**
  1. Multi-stage: builder (rust:1.89) + runtime (debian:bookworm-slim)
  2. Install Zig 0.14.1 in builder stage
  3. Full `cargo build --release` (no `|| true`)
  4. Copy only binary + minimal runtime deps
  5. Non-root user, read-only filesystem
  6. Health check: `CMD ["sharecli", "health"]`
  7. CI verification: `docker build` passes clean

### G05: WinFSP Driver Install Documentation (C07 DevEx)
- **Current:** WinFSP DLL extracted from MSI, driver not installed
- **Target:** Complete Windows setup guide
- **Owner:** Forge
- **Effort:** S (30 min)
- **Path:**
  1. Document `winget install WinFsp.WinFsp` or MSI install
  2. Document `winfsp-x64.dll` copy for dev builds
  3. Document `cargo build` without FUSE feature for minimal Windows use
  4. Add to README.md Windows section

### G06: Dashboard Functional Verification (FR-009)
- **Current:** Dashboard HTML + WebSocket exists but unverified functional
- **Target:** Playwright screenshot test or manual verification
- **Owner:** Forge
- **Effort:** L (4 hours)
- **Path:**
  1. Start `sharecli serve` with dashboard
  2. Use Playwright or curl to verify:
     - HTML loads at /dashboard
     - WebSocket connects
     - Theme tokens apply
     - Skeleton/loading/empty/error states work
  3. Add `tests/c09_dashboard_functional_gate.rs`
  4. CI verification: dashboard smoke test

### G07: OTel Collector Local Stack (C05 L57)
- **Current:** `docker-compose.otel.yml` / `podman-compose.otel.yml` exists but not run
- **Target:** Verified local OTel + Jaeger stack
- **Owner:** Forge
- **Effort:** M (2 hours)
- **Path:**
  1. `podman-compose -f podman-compose.otel.yml up -d`
  2. Verify OTLP receiver at localhost:4318
  3. Verify Jaeger UI at localhost:16686
  4. Start `sharecli serve` with `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318`
  5. Verify traces appear in Jaeger
  6. Add `tests/c05_otel_collector_local_gate.rs`

### G08: Multi-Agent Worktree Coordination (C03)
- **Current:** Scale test exists but not run at scale
- **Target:** 10+ concurrent agents verified
- **Owner:** Forge
- **Effort:** L (4 hours)
- **Path:**
  1. Create 10 temporary worktrees
  2. Spawn 10 concurrent `sharecli session claim` operations
  3. Verify no deadlocks, no lost claims
  4. Verify thermal gate ADMIT/DENY under load
  5. Clean up worktrees
  6. Add `tests/c03_multi_agent_stress_gate.rs`

### G09: Soak Harness Real Run (C08 L75)
- **Current:** `sharecli soak run` exists but never run on CI
- **Target:** 7-day nightly soak passing threshold
- **Owner:** Forge (CI automation)
- **Effort:** M (1 hour setup + monitoring)
- **Path:**
  1. Ensure soak.yml nightly cron is active
  2. Wait 7 nights for soak data
  3. Verify soak-report.json shows: error_rate < 5%, uptime >= 95%, p99 <= 2s
  4. Promote soak to hard gate

### G10: Dependabot PR Auto-Merge (C04)
- **Current:** 2 dependabot PRs stuck (behind main, need rebase)
- **Target:** Dependabot PRs auto-merge after CI passes
- **Owner:** Koosh (repo settings)
- **Effort:** S (10 min)
- **Path:**
  1. Enable "Allow auto-merge" in repo settings
  2. Dependabot PRs will auto-rebase and merge after checks pass
  3. Or: manually rebase via `gh pr rebase 808 --admin`

## Task IDs (WORK_DAG Wave20)

| ID | Task | Gap | FR | Effort | Dependencies |
|----|------|-----|-----|--------|-------------|
| T-1100 | Wave20 kickoff | all | - | S | v0.4.0 merged |
| T-1110 | Apple signing secrets setup | G01 | FR-011 | M | Apple Dev acct |
| T-1120 | Azure KV Windows signing | G02 | FR-011 | M | Azure sub |
| T-1130 | Homebrew bottle SHA | G03 | FR-011 | S | v0.4.0 tag |
| T-1140 | Dockerfile production | G04 | FR-006 | M | - |
| T-1150 | WinFSP install docs | G05 | FR-007 | S | - |
| T-1160 | Dashboard functional gate | G06 | FR-009 | L | - |
| T-1170 | OTel collector local stack | G07 | FR-005 | M | Podman |
| T-1180 | Multi-agent stress test | G08 | FR-003 | L | - |
| T-1190 | Soak 7-day CI verification | G09 | FR-008 | M | 7 nights |
| T-1200 | Dependabot auto-merge | G10 | FR-004 | S | Repo settings |
| T-1210 | Wave20 governance closeout | all | - | S | T-1110..T-1200 |

## Projected Scorecard After Wave20

| Metric | Current (v0.4.0) | Projected (v0.5.0) | Change |
|--------|------------------|---------------------|--------|
| Weighted Overall | 99.1% | **99.8%** | +0.7% |
| C05 Observability | 97.2% | 100% | +2.8% |
| C06 Supply Chain | 97.2% | 100% | +2.8% |
| C07 DevEx | 96.7% | 100% | +3.3% |
| C11 Packaging | 97.8% | 100% | +2.2% |
| Grade | A+ | **A+** | maintained |

## External Dependencies (Not Automatable)

| Item | Cost | Owner | Timeline |
|------|------|-------|----------|
| Apple Developer Account | Already owned | Koosh | When ready |
| Azure Key Vault | ~$0.36/yr | Koosh | When ready |
| macOS for `brew bottle` | Already owned | Koosh | After v0.4.0 tag |
| Podman running locally | Free | Koosh | For OTel testing |
| 7 nights for soak data | Free | CI | Automated |

## Success Criteria

Wave 20 is complete when:
1. Apple signing secrets configured and codesign.yml passes as hard gate
2. Azure KV configured and Windows signing works
3. Homebrew bottle SHA replaced with real value
4. Dockerfile builds clean (no `|| true`)
5. WinFSP setup documented in README
6. Dashboard functional test passes
7. OTel collector verified locally with trace output
8. 10+ concurrent agent coordination verified
9. 7-day soak passes threshold
10. Dependabot auto-merge enabled
