# Release Candidate — audit-v38 grade B (~80%)

**Status:** SOFT RC (evidence stack; not a product GA claim)  
**Pin commit:** `9e805b6` (`main` after #324)  
**Scorecard:** `audit/SCORECARD-v38.md` — weighted **80% B**, unweighted **~79%**

## RC scope (what shipped Jul 14–17)

| Wave | PR range | Theme |
|------|----------|-------|
| Hotfix | #290 | `tracing-subscriber` json feature |
| Reconcile | #279, #320 | SCORECARD table sync |
| Soft docs + CI | #292–#319 | C01–C11 runbooks, soft workflows |
| Code + scripts | #323–#324 | spawn audit JSONL, chaos restart |

## Cluster snapshot

| Cluster | Pct | Grade | RC note |
|---------|:---:|:-----:|---------|
| C03 | 92% | A | Agent readiness — hold |
| C02 | 87% | B | spawn audit partial (#323) |
| C10 | 86% | B | golden plan doc only |
| C01/C04/C06 | 80% | B | hard-fail plans seeded |
| C05 | 77% | B | L47 soak CI merged |
| C07/C09/C11 | 76–78% | B | soft plans + partial CI |
| C00/C08 | 70–73% | C | next lift targets |

## RC blockers (hard — out of RC scope)

- **C11 L112** codesign/notarize org secrets
- **C06** SLSA L3 network-block hard gate
- **C07 L65** mutants required check (7-day soak)
- **C04 L34** GitHub ruleset apply (org admin)

## RC verification checklist

- [x] `cargo build` matrix green on `main`
- [x] FR PR bodies contain `FR-NNN` (pr-lint)
- [x] SCORECARD remediation log matches merged PRs (post #320)
- [ ] Error envelope unified on serve (T-400)
- [ ] Harbor Phase 3 soak started (T-440)
- [ ] PNG visual baselines committed (T-430)

## Supersedes

Informal "64% C" references in stale WBS/GAP headers — use this RC + live SCORECARD instead.
