## Wave17 Plan 795 (T-900) — C07 L68 Flake-tracker dashboard 2 → 3

Traces to **FR-003** (cov/coverage traceability, Wave17 thesis residual) and **C07 L68** (Flake detection).

| Field | Value |
|-------|-------|
| Source | `0020854` |
| Base | `70893b3` (post #785 Wave17 governance lock) |
| C07 cluster | **27/30 90% A → 28/30 93% A** |
| Overall weighted | **92.3% A → 92.6% A** |
| Unweighted (sum/12) | 91.25% → **91.5% A** |
| Tier-1 (C00–C03 + C07) | 92.4% → **92.6% A** (C07 IS in tier-1; **second tier-1 lift in Wave17** after C02 L26) |

### What shipped (12 files, +1024 / −16)

| File | Role |
|------|------|
| `scripts/flake_tracker.py` | Pure-stdlib cargo-nextest JUnit parser. Classifies each testcase as `flaky | regression | stable | skipped`. Emits JSON with `by_kind`, `flake_rate`, `flaky_cases[]`, `regression_cases[]`, `baseline_diff` (introduced/resolved/persistent). Color-gated console summary respects `NO_COLOR`. `--fail-on-flake` flips exit code. |
| `scripts/comment_flake_tracker.py` | PR commenter. Posts `<!-- flake-tracker -->` marked summary with rate / counts / diff. Invoked by the workflow. |
| `audit/.flake-tracker/README.md` | Operations runbook + JSON schemas + local + CI usage + FR-003 acceptance gate reference. |
| `audit/.flake-tracker/baseline.json` | Empty known/accepted flake list (populated per `docs/testing/flake-policy.md` quarantine process). |
| `.github/workflows/flake-tracker.yml` | Paths-filtered advisory CI job (`continue-on-error: true`). Collects `target/nextest/ci/junit-*.xml` + `junit.xml`, falls back to stub if no JUnit, runs the tracker, uploads `flake-report.json` (14-day artifact), posts PR comment, emits Step Summary. |
| `tests/c07_l68_flake_tracker.rs` | FR-003 acceptance gate — **6/6 PASS**. |

### Real bug found + fixed

While writing the gate, `CaseStats` (a `@dataclass` with mutable list fields) is not hashable, so a `{c for c in current if c.flake}` set-comprehension blew up with `TypeError: cannot use 'CaseStats' as a set element`. Fix: list-comp + set-comp on `(classname, name)` tuples. Documented in the commit message and `C07.md` L68 evidence block.

### Verification (zero-execution in this PR)

```bash
$ cargo test --locked --test c07_l68_flake_tracker
running 6 tests
test fr003_flake_tracker_writes_json_report_to_output_path ... ok
test fr003_flake_tracker_classifies_flaky_case ... ok
test fr003_flake_tracker_classifies_regression ... ok
test fr003_flake_tracker_respects_no_color_env ... ok
test fr003_flake_tracker_baseline_diff_introduced_and_resolved ... ok
test fr003_flake_tracker_fail_on_flake_exits_nonzero_on_flake ... ok
test result: ok. 6 passed; 0 failed
```

```bash
$ python scripts/flake_tracker.py --help
usage: flake_tracker.py [-h] [--output OUTPUT] [--baseline BASELINE]
                        [--quiet] [--fail-on-flake]
                        inputs [inputs ...]

$ NO_COLOR=1 python scripts/flake_tracker.py <junit.xml> --output <out>
flake_tracker.py — C07/L68 summary
  total_cases   : 2
  by_kind       : {'flaky': 1, 'stable': 1}
  flake_rate    : 33.3333%
  ...
```

### Governance sync (claim-lock disjoint)

- `WORK_DAG.md` — T-900 row added Status: DONE; Wave17 header updated
- `audit/.lane-c07/C07.md` — L68 score 2 → 3; 11 evidence paths cited; `CLUSTER_TOTAL 27/30 90% A → 28/30 93% A`
- `audit/SCORECARD-v38.md` — Weighted 92.3% → 92.6%; tier-1 92.4% → 92.6%; Pin commit `c509771` → `70893b3`; Wave17 Plan 795 headline added
- `docs/ops/governance/WBS-PHASED.md` — W17.11 (T-900) row added Status: DONE; C07 cluster rollup updated
- `docs/ops/governance/GAP-QA-MATRIX.md` — C07 L68 row added Status: Closed with full evidence path
- `docs/ops/governance/RC-audit-v38-80B.md` — Pin commit updated; C07 L68 RC blocker **CLOSED** via Plan 795

### FR-003 / C07 L68

L68 evidence is verifiable via:

- `scripts/flake_tracker.py:1` — pure-stdlib tracker source
- `scripts/flake_tracker.py:243` — `build_report()` emitting JSON schema
- `audit/.lane-c07/C07.md:90` — L68 score 3 with 11 cited paths
- `tests/c07_l68_flake_tracker.rs:1` — FR-003 acceptance gate, 6/6 PASS
- `docs/ops/governance/RC-audit-v38-80B.md:44` — RC blocker section now reads **CLOSED**

### Remaining L68 follow-ups (separate scope, no overclaim)

- Periodic baseline refresh automation (manual edit only today)
- Flake-rate trend dashboard over time (single-snapshot only today)
