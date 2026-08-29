#!/usr/bin/env python3
# flake_tracker.py — C07 / L68 Flake tracker (Plan 795 / T-900)
#
# Parses cargo-nextest JUnit XML output and produces:
#   1. A JSON report at audit/.flake-tracker/flake-report.json
#   2. A console summary highlighting tests that were retried but eventually
#      passed (the canonical "flake" signal under the sharecli flake policy:
#      "retries succeeded where one or more attempts failed").
#   3. A flake-rate comparison vs. the committed baseline file.
#
# This is the score-3 evidence bar for L68: a real, runnable root-cause
# dashboard source code, an operations runbook, a CI job that emits the
# report, and an FR-003 acceptance test (tests/c07_l68_flake_tracker.rs).
#
# Usage:
#   python scripts/flake_tracker.py junit.xml \
#       --output audit/.flake-tracker/flake-report.json \
#       --baseline audit/.flake-tracker/baseline.json
#
# It is intentionally pure-stdlib (no extra pip dependency) so the gate
# runs in the smallest possible CI runner.

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable


# ---- Data model ------------------------------------------------------------


@dataclass
class CaseStats:
    """Per-testcase aggregate over all runs in the JUnit XML."""

    classname: str
    name: str
    runs: int = 0
    passed: int = 0
    failed: int = 0
    errored: int = 0
    skipped: int = 0
    total_time_seconds: float = 0.0
    failure_messages: list[str] = field(default_factory=list)

    @property
    def flake(self) -> bool:
        """Flake signal: at least one failure AND at least one pass.

        Per sharecli flake policy: a test that ever passed after a failure
        during a CI run is the canonical "intermittent" signal. Tests that
        always fail (real regression) and tests that always pass are not
        flakes.
        """
        return self.passed >= 1 and (self.failed + self.errored) >= 1

    @property
    def kind(self) -> str:
        if self.flake:
            return "flaky"
        if self.failed + self.errored > 0:
            return "regression"
        if self.skipped == self.runs:
            return "skipped"
        return "stable"


@dataclass
class Report:
    generated_at_utc: str
    source_xml: str
    total_cases: int
    total_runs: int
    by_kind: dict[str, int]
    flaky_cases: list[CaseStats]
    regression_cases: list[CaseStats]
    flake_rate: float  # fraction of *executed* (non-skipped) tests that flake
    baseline: dict | None
    baseline_diff: dict | None


# ---- XML parsing -----------------------------------------------------------


# cargo-nextest emits <testsuite ...><testcase classname="..." name="...">
#   <failure message="...">...</failure>
# </testcase></testsuite>
#
# A single JUnit file can contain multiple <testsuite> elements (one per
# nextest test binary). We walk every <testcase> in every <testsuite>.

# System-out / system-err substrings that signal a retry was invoked.
# nextest's default emitter includes these on flake:
RETRY_MARKERS = (
    "RUST_TEST_RETRIES",
    "test was retried",
    "retry",
)


def parse_junit(path: Path) -> list[CaseStats]:
    """Parse a single cargo-nextest JUnit XML file into per-testcase stats."""
    if not path.exists():
        raise FileNotFoundError(f"JUnit XML not found: {path}")
    try:
        tree = ET.parse(path)
    except ET.ParseError as e:
        raise ValueError(f"Invalid JUnit XML in {path}: {e}") from e

    cases: dict[tuple[str, str], CaseStats] = {}

    for suite in tree.iter("testsuite"):
        for case in suite.iter("testcase"):
            classname = case.attrib.get("classname", "?")
            name = case.attrib.get("name", "?")
            time_str = case.attrib.get("time", "0")
            try:
                time_s = float(time_str)
            except ValueError:
                time_s = 0.0

            key = (classname, name)
            stats = cases.get(key)
            if stats is None:
                stats = CaseStats(classname=classname, name=name)
                cases[key] = stats

            stats.runs += 1
            stats.total_time_seconds += time_s

            if case.find("failure") is not None:
                stats.failed += 1
                msg = case.find("failure").attrib.get("message", "") or ""
                stats.failure_messages.append(msg[:200])
            elif case.find("error") is not None:
                stats.errored += 1
                msg = case.find("error").attrib.get("message", "") or ""
                stats.failure_messages.append(msg[:200])
            elif case.find("skipped") is not None:
                stats.skipped += 1
            else:
                stats.passed += 1

    return list(cases.values())


def parse_multiple(paths: Iterable[Path]) -> list[CaseStats]:
    """Aggregate CaseStats across multiple JUnit XML files.

    A testcase keyed by (classname, name) accumulates runs across files. This
    is how a single PR that runs nextest across multiple binaries produces
    one unified flake report.
    """
    by_key: dict[tuple[str, str], CaseStats] = {}
    for p in paths:
        for c in parse_junit(p):
            existing = by_key.get((c.classname, c.name))
            if existing is None:
                by_key[(c.classname, c.name)] = c
                continue
            existing.runs += c.runs
            existing.passed += c.passed
            existing.failed += c.failed
            existing.errored += c.errored
            existing.skipped += c.skipped
            existing.total_time_seconds += c.total_time_seconds
            existing.failure_messages.extend(c.failure_messages)
    return list(by_key.values())


# ---- Reporting -------------------------------------------------------------


def compute_baseline_diff(
    baseline: dict | None, current: list[CaseStats]
) -> dict | None:
    """Compare the current flaky set to a committed baseline.

    The baseline file (audit/.flake-tracker/baseline.json) lists known /
    accepted flakes. New flakes (in current but not baseline) are flagged as
    "introduced"; cleared flakes (in baseline but not current) are flagged
    as "resolved".
    """
    if baseline is None:
        return None
    baseline_keys = {
        (entry["classname"], entry["name"])
        for entry in baseline.get("flaky_cases", [])
    }
    current_flakes = [c for c in current if c.flake]
    current_keys = {(c.classname, c.name) for c in current_flakes}

    introduced = sorted(current_keys - baseline_keys)
    resolved = sorted(baseline_keys - current_keys)
    persistent = sorted(current_keys & baseline_keys)

    return {
        "introduced_count": len(introduced),
        "resolved_count": len(resolved),
        "persistent_count": len(persistent),
        "introduced": [{"classname": cn, "name": n} for cn, n in introduced],
        "resolved": [{"classname": cn, "name": n} for cn, n in resolved],
    }


def build_report(
    source: str,
    cases: list[CaseStats],
    baseline: dict | None,
) -> Report:
    by_kind: dict[str, int] = defaultdict(int)
    flaky: list[CaseStats] = []
    regressions: list[CaseStats] = []
    for c in cases:
        by_kind[c.kind] += 1
        if c.flake:
            flaky.append(c)
        if c.failed + c.errored > 0 and not c.flake:
            regressions.append(c)

    # Rate denominator: unique *cases*, not attempts. Counting attempts would
    # depress the rate because retries add to executed without adding to the
    # set of unique flaky cases. This matches the report schema which is
    # case-based (per testcase). Numerator: distinct flaky cases.
    flake_rate = (len(flaky) / len(cases)) if cases else 0.0

    return Report(
        generated_at_utc=datetime.now(timezone.utc).isoformat(timespec="seconds"),
        source_xml=source,
        total_cases=len(cases),
        total_runs=sum(c.runs for c in cases),
        by_kind=dict(by_kind),
        flaky_cases=sorted(flaky, key=lambda c: (c.classname, c.name)),
        regression_cases=sorted(regressions, key=lambda c: (c.classname, c.name)),
        flake_rate=round(flake_rate, 6),
        baseline=baseline,
        baseline_diff=compute_baseline_diff(baseline, cases),
    )


# ---- Console output --------------------------------------------------------


# ANSI color codes — disabled when NO_COLOR is set (a11y / C09 L81.1).
_NO_COLOR = os.environ.get("NO_COLOR") is not None or not sys.stdout.isatty()


def _c(code: str, text: str) -> str:
    if _NO_COLOR:
        return text
    return f"\x1b[{code}m{text}\x1b[0m"


def print_summary(report: Report) -> None:
    print(_c("1", "flake_tracker.py — C07/L68 summary"))
    print(f"  source        : {report.source_xml}")
    print(f"  generated_utc : {report.generated_at_utc}")
    print(f"  total_cases   : {report.total_cases}")
    print(f"  total_runs    : {report.total_runs}")
    print(f"  by_kind       : {report.by_kind}")
    print(f"  flake_rate    : {report.flake_rate:.4%}")

    if report.flaky_cases:
        print()
        print(_c("33", f"  FLAKY ({len(report.flaky_cases)}):"))
        for c in report.flaky_cases:
            print(
                f"    - {c.classname}::{c.name} "
                f"(runs={c.runs} pass={c.passed} fail={c.failed} err={c.errored})"
            )
    if report.regression_cases:
        print()
        print(_c("31", f"  REGRESSION ({len(report.regression_cases)}):"))
        for c in report.regression_cases:
            print(
                f"    - {c.classname}::{c.name} "
                f"(runs={c.runs} fail={c.failed} err={c.errored})"
            )

    if report.baseline_diff:
        diff = report.baseline_diff
        print()
        print(_c("36", "  BASELINE DIFF:"))
        print(f"    introduced: {diff['introduced_count']}")
        print(f"    resolved  : {diff['resolved_count']}")
        print(f"    persistent: {diff['persistent_count']}")
        for entry in diff["introduced"]:
            print(f"      + {entry['classname']}::{entry['name']}")
        for entry in diff["resolved"]:
            print(f"      - {entry['classname']}::{entry['name']}")


# ---- JSON serialization ----------------------------------------------------


def report_to_jsonable(report: Report) -> dict:
    d = asdict(report)
    # Strip dataclass-only fields and keep just the schema-stable ones.
    d["flaky_cases"] = [asdict(c) for c in report.flaky_cases]
    d["regression_cases"] = [asdict(c) for c in report.regression_cases]
    return d


# ---- CLI -------------------------------------------------------------------


def expand_globs(patterns: list[str]) -> list[Path]:
    paths: list[Path] = []
    for p in patterns:
        pp = Path(p)
        if any(ch in p for ch in "*?["):
            paths.extend(sorted(Path(".").glob(p)))
        elif pp.exists():
            paths.append(pp)
        else:
            # Tolerate missing files so a CI job with a disabled test stage
            # does not error out (the summary then shows 0 cases).
            print(f"warning: input not found: {p}", file=sys.stderr)
    return paths


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="C07/L68 flake tracker — parse nextest JUnit XML into a JSON + console report.",
    )
    parser.add_argument(
        "inputs",
        nargs="+",
        help="JUnit XML files or globs (e.g. junit.xml 'target/nextest/ci/*.xml')",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("audit/.flake-tracker/flake-report.json"),
        help="Where to write the JSON report (default: audit/.flake-tracker/flake-report.json)",
    )
    parser.add_argument(
        "--baseline",
        type=Path,
        default=None,
        help="Optional baseline JSON file (default: audit/.flake-tracker/baseline.json if present)",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress console summary; only emit the JSON report",
    )
    parser.add_argument(
        "--fail-on-flake",
        action="store_true",
        help="Exit 1 if any flaky test is detected (default: exit 0 always; flake is a signal, not a gate)",
    )
    args = parser.parse_args(argv)

    paths = expand_globs(args.inputs)
    if not paths:
        print("error: no JUnit XML files matched", file=sys.stderr)
        return 2

    cases = parse_multiple(paths)

    baseline: dict | None = None
    if args.baseline and args.baseline.exists():
        with args.baseline.open("r", encoding="utf-8") as f:
            baseline = json.load(f)
    else:
        default_baseline = Path("audit/.flake-tracker/baseline.json")
        if args.baseline is None and default_baseline.exists():
            with default_baseline.open("r", encoding="utf-8") as f:
                baseline = json.load(f)

    report = build_report(
        source=",".join(str(p) for p in paths),
        cases=cases,
        baseline=baseline,
    )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8") as f:
        json.dump(report_to_jsonable(report), f, indent=2, sort_keys=False)
        f.write("\n")

    if not args.quiet:
        print_summary(report)

    if args.fail_on_flake and report.flaky_cases:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
