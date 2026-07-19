# Mutation testing hard gate (live)

Audit-v38 **C07 L65** — `cargo-mutants` is a **required merge gate** (phase 4 live).
Threshold contract: [`mutants-threshold.md`](mutants-threshold.md).

Related: [`config-proptest.md`](config-proptest.md) (L66 property tests) ·
[`fuzz-soft.yml`](../../.github/workflows/fuzz-soft.yml) (L67 nightly fuzz) ·
[`ruleset-checklist.md`](ruleset-checklist.md) (branch protection runbook).

## Current stance (hard)

| Control | Workflow / config | Gate strength | Notes |
|---------|-------------------|---------------|-------|
| `cargo mutants` examine set | `.github/workflows/mutants.yml` + `ci.yml` `mutants` | **Hard** | No `continue-on-error`; survivors fail the job |
| Scope | `mutants.toml` `examine_globs` | Fixed | `crates/sharecli-thermal-tui/src/lib.rs` pure helpers |
| Survivor threshold | step exit code | **Enforced** | Zero survivors required |
| `ci-success` | `.github/workflows/ci.yml` | Aggregator | `needs:` includes `mutants` |
| Triggers | All PRs via `ci.yml`; cron/dispatch + path-filtered `main` push via `mutants.yml` | Full | Aggregator always reports on PRs |
| Local DX | `just mutants` | Maintainer | Matches CI `--file` + `--locked` |
| Artifact | `mutants-hard-<sha>.json` / `mutants-ci-<sha>.json` | Triage | 14-day retention |

**Net:** a red mutants job **blocks** `CI Success` and must be required (or covered by
the aggregator) on `main` PRs.

## Target stance (achieved)

| Control | Target |
|---------|--------|
| Survivor threshold | **Zero** surviving mutants in `examine_globs` set |
| Timeout | 60s per mutant (`mutants.toml` / CI parity) |
| Job mode | No `continue-on-error` on mutants jobs |
| `ci-success` | `mutants` in `.github/workflows/ci.yml` `needs:` |
| Branch protection | Prefer `CI Success` (includes mutants); optional named `cargo-mutants (required)` from cron workflow |
| Triggers | All PRs via `ci.yml`; cron remains for drift detection |
| Score | L65 **3** — per-PR + threshold gate |

## Survivor threshold (contract)

Inherited from [`mutants-threshold.md`](mutants-threshold.md) — do not relax:

| Metric | Hard (live) |
|--------|-------------|
| Examine scope | `examine_globs` thermal-tui `lib.rs` |
| Pass condition | 0 survivors |
| Fail condition | any survivor → step/job exit 1 |
| Timeout | 60s / mutant |
| Parallelism | `--jobs 2` |
| Lockfile | `--locked -p sharecli-thermal-tui` |

Widening `examine_globs` is a **separate** PR after the hard gate stays green on the
current examine set for one week on `main`.

## Phases (complete through 4)

| Phase | Deliverable | Hard gate? |
|-------|-------------|------------|
| **0** | `mutants.yml` soft + `mutants-threshold.md` + `just mutants` | No |
| **1** | Soft plan doc + scorecard/worklog (FR-003) | No |
| **2** | Examine set green soak on `main` | No |
| **3** | Optional widen PR triggers while soft | No |
| **4** | Remove `continue-on-error`; wire `ci-success`; rename required job | **Yes — live (T-640)** |

### Phase 2 soak checklist (historical)

- [x] Soft fail-on-survivors + JSON artifact on examine set
- [x] `just mutants` matches CI `--file` + `--locked`
- [x] Maintainer acknowledges `cargo install cargo-mutants` cold-start on `ubuntu-24.04`
- [ ] Optional: seven consecutive post-hard `main` cron greens (ops follow-up)

## Hard-gate wiring (phase 4 — done)

1. **Workflow** — `.github/workflows/mutants.yml`:
   - Removed `continue-on-error: true`.
   - Job renamed to `cargo-mutants (required)`.
   - Kept `--json-outfile` artifact for post-mortems.
2. **`ci-success`** — `mutants` job added to `.github/workflows/ci.yml` `needs:`.
3. **Branch protection** — GitHub → Settings → Rules → `main`:
   - Prefer requiring `CI Success` (aggregator includes mutants).
   - Optionally also require `cargo-mutants (required)` (exact name from workflow).
   - Cross-ref [`ruleset-checklist.md`](ruleset-checklist.md).
4. **`mutants-threshold.md`** — Current gate table marks hard mode live.
5. **`mutants.toml` header** — points at this hard-gate doc.

**Do not** add `|| true`, survivor caps, or `continue-on-error` on the hard step.

## Triage (survivors)

When `mutants-hard-*.json` / `mutants-ci-*.json` shows survivors:

| Survivor class | Action |
|----------------|--------|
| Missing test for pure helper | Add unit test in `sharecli-thermal-tui` |
| Equivalent mutant (no observable change) | Add `mutants.toml` `exclude_re` with comment |
| Render/event-loop noise | Already excluded — do not widen examine set to fix |
| Timeout flake | Increase local timeout only after profiling; keep CI at 60s |

## Commands (local dry-run)

```bash
# Matches CI examine set
just mutants

# Explicit parity
cargo mutants --timeout 60 --jobs 2 \
  -p sharecli-thermal-tui \
  --json-outfile mutants-local.json \
  -- --locked

# Inspect survivors
cargo mutants --list -p sharecli-thermal-tui
```

Expect non-zero exit when survivors remain — that is the intended hard-gate signal.

## Audit evidence (C07 L65)

| Line | Evidence | Score |
|------|----------|-------|
| **L65** Mutation testing | `mutants.toml`, `mutants.yml`, `ci.yml` `mutants` + `ci-success`, `just mutants`, [`mutants-threshold.md`](mutants-threshold.md), this doc | **3** — per-PR + threshold gate |

**Follow-up**

| Item | Status |
|------|--------|
| Mutants hard-gate promotion | Done (T-640) |
| Remove `continue-on-error` + `ci-success` wiring | Done |
| Branch protection confirm `CI Success` / named check | Maintainer ops |
| Expand `examine_globs` beyond thermal-tui | Deferred (post-hard) |

**Status:** hard gate live (Phase 4) · **FR:** FR-003 · **Last sync:** 2026-07-18 · **T-640**
