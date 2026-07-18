# Release Candidate — audit-v38 grade B (~81%)

**Status:** SOFT RC (evidence stack; not a product GA claim)  
**Pin commit:** `03391c4` (`main` after #330)  
**Scorecard:** `audit/SCORECARD-v38.md` — weighted **~81% B**, unweighted **~80%**

## RC scope (what shipped Jul 14–18)

| Wave | PR range | Theme |
|------|----------|-------|
| Hotfix | #290 | `tracing-subscriber` json feature |
| Reconcile | #279, #320 | SCORECARD table sync |
| Soft docs + CI | #292–#319 | C01–C11 runbooks, soft workflows |
| Code + scripts | #323–#324 | spawn audit JSONL, chaos restart |
| Wave12 lifts | #326–#330 | error envelope, proptest, trace inject, PNG scaffold, Harbor Phase 3 plan |

## Cluster snapshot

| Cluster | Pct | Grade | RC note |
|---------|:---:|:-----:|---------|
| C03 | 92% | A | Agent readiness — hold |
| C10 | 89% | B | PNG scaffold + golden plan (#327) |
| C02 | 87% | B | spawn audit partial (#323) |
| C01/C04/C06 | 80% | B | hard-fail plans seeded |
| C05 | 80% | B | CLI traceparent inject (#328) |
| C07/C09/C11 | 76–78% | B | soft plans + partial CI |
| C00/C08 | 73–77% | C/B | error envelope (#330); Harbor soak plan (#326) |

## RC blockers (hard — out of RC scope)

- **C11 L112** codesign/notarize org secrets
- **C06** SLSA L3 network-block hard gate
- **C07 L65** mutants required check (7-day soak)
- **C04 L34** GitHub ruleset apply (org admin)

## RC verification checklist

- [x] `cargo build` matrix green on `main`
- [x] FR PR bodies contain `FR-NNN` (pr-lint)
- [x] SCORECARD remediation log matches merged PRs (post #330)
- [x] Error envelope unified on serve (T-400 · #330)
- [x] Harbor Phase 3 soak plan landed (T-440 · #326)
- [x] PNG visual baseline scaffold committed (T-430 · #327)
- [ ] OpenAPI `ErrorEnvelope` component (Wave13 W13.1)
- [ ] PNG bytes committed + soft diff gate (Wave13 W13.2)
- [ ] Harbor seven-day soak execution (Wave13 W13.3)

## Supersedes

Informal "64% C" / "~80% B pre-Wave12" references in stale headers — use this RC + live SCORECARD instead.
