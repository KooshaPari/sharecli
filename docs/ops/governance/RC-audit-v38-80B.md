# Release Candidate — audit-v38 grade B (~82%)

**Status:** SOFT RC (evidence stack; not a product GA claim)  
**Pin commit:** `6466b4e` (`main` after #335)  
**Scorecard:** `audit/SCORECARD-v38.md` — weighted **~82% B**, unweighted **~81%**

## RC scope (what shipped Jul 14–18)

| Wave | PR range | Theme |
|------|----------|-------|
| Hotfix | #290 | `tracing-subscriber` json feature |
| Reconcile | #279, #320 | SCORECARD table sync |
| Soft docs + CI | #292–#319 | C01–C11 runbooks, soft workflows |
| Code + scripts | #323–#324 | spawn audit JSONL, chaos restart |
| Wave12 lifts | #326–#330 | error envelope, proptest, trace inject, PNG scaffold, Harbor Phase 3 plan |
| Wave13 lifts | #332–#335 | OpenAPI component, soak scaffold, IPC/tray trace, PNG bytes + soft diff |

## Cluster snapshot

| Cluster | Pct | Grade | RC note |
|---------|:---:|:-----:|---------|
| C03 | 92% | A | Agent readiness — hold |
| C10 | 89% | B | PNG bytes + deterministic hard diff (T-600; score unchanged) |
| C02 | 87% | B | spawn audit partial (#323) |
| C00/C01/C04/C05/C06/C08 | 77–80% | B/C | OpenAPI component (#332); IPC/tray trace (#334); soak scaffold (#333) |
| C07/C09/C11 | 76–78% | B | soft plans + partial CI |

## RC blockers (hard — out of RC scope)

- **C11 L112** codesign/notarize org secrets
- **C06** SLSA L3 network-block hard gate
- **C07 L65** mutants required check (7-day soak)
- **C04 L34** GitHub ruleset apply (org admin)

## RC verification checklist

- [x] `cargo build` matrix green on `main`
- [x] FR PR bodies contain `FR-NNN` (pr-lint)
- [x] SCORECARD remediation log matches merged PRs (post #335)
- [x] Error envelope unified on serve (T-400 · #330)
- [x] OpenAPI `ErrorEnvelope` component (T-500 · #332)
- [x] PNG bytes committed + soft diff gate (T-510 · #335)
- [x] Harbor Phase 3 soak execution scaffold (T-520 · #333)
- [x] IPC/tray traceparent inject (T-530 · #334)
- [ ] Seven-day Harbor soak log completion (Wave14)
- [x] Visual-soft hard promote (Wave14 · T-600)

## Supersedes

Informal "64% C" / "~80% B pre-Wave12" / "~81% B pre-Wave13" references in stale headers — use this RC + live SCORECARD instead.
