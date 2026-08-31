# Wave 20 Spec — ShareCLI Post-1.0.0 Audit Gaps

**Date:** 2026-08-30
**Author:** Forge (automated)
**Status:** READY
**Trigger:** v1.0.0 release complete, all Wave18+19 gaps resolved

---

## 1. Context

ShareCLI v1.0.0 shipped with:
- 150-pillar audit scorecard at 99.1% A+
- 108 new tests across Wave18+Wave19
- 7 new CI workflows (codesign, soak, packaging, dco, gpg, hermetic, visual)
- 4 ADRs (coverage ratchet, code signing, harbor soak, packaging)
- 570-line soak harness replacing the stub
- 48 hard CI gates
- SonarCloud Quality Gate passing (all 6 conditions green)
- Dockerfile hardened (multi-stage, non-root, .dockerignore)

The release workflow (run 33377205218) is building binaries for Linux x86_64, Windows x86_64, macOS aarch64, plus tray apps for Linux, Windows, and macOS.

## 2. Remaining Gaps

### 2.1 External Infrastructure (Manual Setup Required)

| Gap | Pillar | Blocker | Resolution |
|-----|--------|---------|------------|
| Apple code signing secrets | C11 L112 | Apple Dev account + manual secret config | `docs/ops/governance/APPLE_SECRETS_SETUP.md` |
| Azure Key Vault (Windows signing) | C11 L112 | Azure subscription (~$0.36/yr) | Create Standard Key Vault + 4 secrets |
| Homebrew bottle SHA | C11 L111 | v* tag + `brew bottle` on macOS | After release tag + macOS build |
| Harbor 7-day soak | C08 L75 | Needs CI nightly soak run | `soak.yml` cron handles this |

### 2.2 Code Quality (Post-Release)

| Gap | Pillar | Effort | Priority |
|-----|--------|--------|----------|
| WinFSP driver install docs for Windows users | C07 DevEx | S | HIGH |
| Dashboard functional verification (HTML + WebSocket) | C09 UX | L | HIGH |
| Live OTel collector (needs `podman-compose up`) | C05 Obs | M | MEDIUM |
| Dockerfile production readiness | Container | M | MEDIUM |
| Version bump from 0.1.0 in docs/examples | C10 Visual | S | LOW |

### 2.3 Process & Governance

| Gap | Pillar | Effort | Priority |
|-----|--------|--------|----------|
| Wave 20 scorecard refresh | Governance | S | HIGH |
| WORK_DAG sync with merged PRs | Governance | S | HIGH |
| Update COMPREHENSIVE_AUDIT_SCORECARD.md post-merge | Governance | S | MEDIUM |
| ADR-011: Wave 20 gap remediation decisions | Governance | M | MEDIUM |

## 3. Implementation Plan

### Phase 1: Documentation & Scorecard (1 day)

1. **T-1200**: Update COMPREHENSIVE_AUDIT_SCORECARD.md with v1.0.0 state
2. **T-1210**: Update WORK_DAG.md — mark Wave18/19 tasks DONE, add Wave20 tasks
3. **T-1220**: Create ADR-011: Post-1.0.0 gap remediation decisions
4. **T-1230**: Write WinFSP installation guide for Windows users
5. **T-1240**: Update audit_scorecard.json to v39 schema

### Phase 2: Dashboard Verification (2 days)

6. **T-1250**: Verify dashboard HTML loads correctly in browser
7. **T-1260**: Verify WebSocket connection to sharecli daemon
8. **T-1270**: Add dashboard smoke test to CI (screenshot comparison)
9. **T-1280**: Fix any dashboard rendering issues

### Phase 3: OTel & Observability (1 day)

10. **T-1290**: Start OTel collector locally via podman-compose
11. **T-1300**: Verify trace context propagation across IPC → tray → dashboard
12. **T-1310**: Add OTel integration test that validates trace spans

### Phase 4: Container & Packaging (1 day)

13. **T-1320**: Build production Dockerfile (remove `|| true` hack)
14. **T-1330**: Test Docker build with Zig pre-installed in builder stage
15. **T-1340**: Verify DEB/RPM packages install and run correctly

### Phase 5: Release Validation (1 day)

16. **T-1350**: Download and test release binaries on each platform
17. **T-1360**: Verify attestation signatures on release artifacts
18. **T-1370**: Update Homebrew formula with real SHA (after `brew bottle`)
19. **T-1380**: Create v1.0.0 GitHub Release with changelog

## 4. Success Criteria

| Metric | Target | Current |
|--------|--------|---------|
| Scorecard | 99.5% A+ | 99.1% A+ |
| Hard CI gates | 50+ | 48 |
| Test count | 280+ | 277+ |
| Dashboard | Functional | Unknown |
| OTel | Trace propagation verified | Stub only |
| Dockerfile | Production-ready | Smoke image |
| Release binaries | Downloaded + tested | Building |

## 5. Dependencies

- **External**: Apple signing secrets, Azure Key Vault, Docker Hub access
- **Internal**: None — all Wave20 tasks are self-contained
- **Blocked by**: v1.0.0 release completion (DONE)

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Zig mirror failures | HIGH | MEDIUM | Already soft-gated with `continue-on-error` |
| WinFSP kernel driver install | MEDIUM | LOW | Documented in guide, non-blocking |
| Dashboard WebSocket auth | LOW | HIGH | Add integration test |
| OTel collector not running | MEDIUM | LOW | Graceful degradation in sharecli |

## 7. ADR Decisions

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-011a | Dashboard as soft gate | Requires browser automation, expensive CI |
| ADR-011b | OTel as soft gate | Requires podman-compose, optional dependency |
| ADR-011c | Dockerfile as hard gate after Zig fix | Production image must build cleanly |
| ADR-011d | Release binaries as hard gate | Must pass smoke test on each platform |

---

*This spec is a living document. Update as Wave20 tasks are claimed and completed.*
