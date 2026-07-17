# Mutation testing hard-gate promotion (soft plan)

Audit-v38 **C07 L65** — promote `cargo-mutants` from **soft detection (today)** to a
**required merge gate** without flipping CI yet. Builds on the threshold contract in
[`mutants-threshold.md`](mutants-threshold.md).

Related: [`config-proptest.md`](config-proptest.md) (L66 property tests) ·
[`fuzz-soft.yml`](../../.github/workflows/fuzz-soft.yml) (L67 nightly fuzz) ·
[`ruleset-checklist.md`](ruleset-checklist.md) (branch protection runbook).

## Current stance (soft)

| Control | Workflow / config | Gate strength | Notes |
|---------|-------------------|---------------|-------|
| `cargo mutants` examine set | `.github/workflows/mutants.yml` | **Soft** | `continue-on-error: true`; step fails on survivors |
| Scope | `mutants.toml` `examine_re` | Fixed | `crates/sharecli-thermal-tui/src/lib.rs` pure helpers |
| Survivor threshold | step exit code | **Enforced in step** | Zero survivors required; job still soft-red |
| Triggers | PR path filter + daily cron + dispatch | Partial | Only when thermal-tui / mutants config changes |
| Local DX | `just mutants` | Maintainer | Matches CI `--file` + `--locked` |
| Artifact | `mutants-soft-<sha>.json` | Triage | 14-day retention on soft CI |

**Net:** mutation testing runs and **reports** survivor failures, but a red mutants
job does **not** block merge today.

## Target stance (hard)

| Control | Target |
|---------|--------|
| Survivor threshold | **Zero** surviving mutants in `examine_re` set (unchanged from soft) |
| Timeout | 60s per mutant (`mutants.toml` / CI parity) |
| Job mode | Remove `continue-on-error` from `mutants.yml` |
| `ci-success` | Add `mutants` (or renamed job) to `.github/workflows/ci.yml` `needs:` |
| Branch protection | `cargo-mutants (soft threshold)` → required check on `main` PRs |
| Triggers | Keep PR path filter until scope widens; cron remains for drift detection |
| Score | L65 **2 → 3** per rubric (“per-PR + threshold gate”) |

L65 stays **2** until the hard gate is live; this doc closes the **plan** gap on the
C07 scorecard Top-3 list.

## Survivor threshold (contract)

Inherited from [`mutants-threshold.md`](mutants-threshold.md) — do not relax on promotion:

| Metric | Soft (today) | Hard (target) |
|--------|--------------|---------------|
| Examine scope | `examine_re` thermal-tui `lib.rs` | same |
| Pass condition | 0 survivors | 0 survivors |
| Fail condition | any survivor → step exit 1 | same → **job exit 1** |
| Timeout | 60s / mutant | 60s |
| Parallelism | `--jobs 2` | `--jobs 2` |
| Lockfile | `--locked -p sharecli-thermal-tui` | same |

Widening `examine_re` is a **separate** PR after the hard gate is green on the
current examine set for one week on `main`.

## Soft phases (no hard gate yet)

| Phase | Deliverable | Hard gate? |
|-------|-------------|------------|
| **0 — today** | `mutants.yml` soft + `mutants-threshold.md` + `just mutants` | No |
| **1 — plan** | This doc + scorecard/worklog (FR-003) | No |
| **2 — soak** | Examine set green on `main` for **7 consecutive days** (cron + merged PRs) | No |
| **3 — widen triggers** | Optional: run on every PR (remove `paths:` filter) while still soft | No |
| **4 — hard gate** | Remove `continue-on-error`; wire `ci-success`; branch protection | **Yes — deferred** |

Phase 0–3 are **documentation + soft CI** only. Phase 4 needs maintainer sign-off
after phase-2 soak completes.

### Phase 2 soak checklist

Track in PR comments or `audit/.lane-c07/` until phase 4:

- [ ] Seven consecutive `main` cron runs with zero survivors (check `mutants-soft-*.json` artifacts).
- [ ] No survivor regressions from merged PRs touching `examine_re` files.
- [ ] `just mutants` reproduces CI locally on `main` HEAD.
- [ ] Maintainer acknowledges `cargo install cargo-mutants` cold-start time on `ubuntu-24.04`.

## Hard-gate wiring (phase 4 — planned)

1. **Workflow** — `.github/workflows/mutants.yml`:
   - Remove `continue-on-error: true` from `mutants-soft` job.
   - Rename job to `cargo-mutants (required)` (update branch protection name).
   - Keep `--json-outfile` artifact for post-mortems.
2. **`ci-success`** — add `mutants-soft` to `needs:` in `.github/workflows/ci.yml` **or**
   document mutants as a standalone required check (prefer aggregator for truthful red).
3. **Branch protection** — GitHub → Settings → Rules → `main`:
   - Add required status check: `cargo-mutants (required)` (exact name from workflow).
   - Cross-ref [`ruleset-checklist.md`](ruleset-checklist.md) for ruleset vs classic protection.
4. **`mutants-threshold.md`** — update “Current gate” table to mark hard mode live; link here.
5. **`mutants.toml` header** — point at hard-gate doc once phase 4 lands.

**Do not** add `|| true`, survivor caps, or `continue-on-error` on the hard step.

## Triage (survivors before hard gate)

When `mutants-soft-*.json` shows survivors during soak:

| Survivor class | Action |
|----------------|--------|
| Missing test for pure helper | Add unit test in `sharecli-thermal-tui` |
| Equivalent mutant (no observable change) | Add `mutants.toml` `exclude_re` with comment |
| Render/event-loop noise | Already excluded — do not widen examine set to fix |
| Timeout flake | Increase local timeout only after profiling; keep CI at 60s |

## Commands (local dry-run)

```bash
# Matches CI examine set (soft or hard)
just mutants

# Explicit parity
cargo mutants --timeout 60 --jobs 2 \
  --file 'crates/sharecli-thermal-tui/src/lib.rs' \
  --json-outfile mutants-local.json \
  -- --locked -p sharecli-thermal-tui

# Inspect survivors
cargo mutants --list --file 'crates/sharecli-thermal-tui/src/lib.rs'
```

Expect non-zero exit when survivors remain — that is the intended hard-gate signal.

## Audit evidence (C07 L65)

| Line | Evidence | Score |
|------|----------|-------|
| **L65** Mutation testing | `mutants.toml`, `mutants.yml`, `just mutants`, [`mutants-threshold.md`](mutants-threshold.md), this promotion plan | **2** — unchanged; hard gate deferred |

**Soft follow-up**

| Item | Status |
|------|--------|
| Mutants hard-gate promotion plan | Done (this file) |
| Seven-day green soak on `main` | Open |
| Remove `continue-on-error` + required check | Deferred |
| Expand `examine_re` beyond thermal-tui | Deferred (post-hard) |

**Status:** soft plan (Phase 1) · **FR:** FR-003 traceability · **Last sync:** 2026-07-17
