#!/usr/bin/env python3
"""Create a compact, machine-readable snapshot from llvm-cov export JSON."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


METRICS = ("lines", "functions", "regions", "branches")


def metric_snapshot(totals: dict[str, Any], name: str) -> dict[str, int | float]:
    metric = totals.get(name)
    if not isinstance(metric, dict):
        raise ValueError(f"llvm-cov totals are missing {name!r}")

    count = metric.get("count")
    covered = metric.get("covered")
    percent = metric.get("percent")
    if (
        not isinstance(count, int)
        or isinstance(count, bool)
        or not isinstance(covered, int)
        or isinstance(covered, bool)
    ):
        raise ValueError(f"llvm-cov {name!r} counts must be integers")
    if count < 0 or covered < 0 or covered > count:
        raise ValueError(f"llvm-cov {name!r} counts are invalid")
    if (
        not isinstance(percent, (int, float))
        or isinstance(percent, bool)
        or not 0.0 <= percent <= 100.0
    ):
        raise ValueError(f"llvm-cov {name!r} percent must be between 0 and 100")

    return {"count": count, "covered": covered, "percent": round(float(percent), 2)}


def build_snapshot(
    report: Any, *, git_sha: str, run_id: str, threshold: float
) -> dict[str, Any]:
    if not isinstance(report, dict):
        raise ValueError("llvm-cov report must be a JSON object")
    data = report.get("data")
    if (
        not isinstance(data, list)
        or len(data) != 1
        or not isinstance(data[0], dict)
    ):
        raise ValueError("expected one llvm-cov data entry as a JSON object")
    totals = data[0].get("totals")
    if not isinstance(totals, dict):
        raise ValueError("llvm-cov report is missing totals")

    coverage = {name: metric_snapshot(totals, name) for name in METRICS}
    return {
        "schema_version": 1,
        "source": {
            "tool": "cargo llvm-cov",
            "format": report.get("type", "llvm.coverage.json.export"),
            "git_sha": git_sha,
            "github_run_id": run_id,
        },
        "target": {
            "lines_percent": threshold,
            "enforcement": ".github/workflows/quality-gate.yml",
        },
        "coverage": coverage,
        "meets_lines_target": coverage["lines"]["percent"] >= threshold,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path, help="llvm-cov export JSON")
    parser.add_argument("output", type=Path, help="compact snapshot JSON")
    parser.add_argument("--git-sha", default="unknown")
    parser.add_argument("--run-id", default="local")
    parser.add_argument("--threshold", type=float, default=85.0)
    args = parser.parse_args()

    with args.input.open(encoding="utf-8") as source:
        report = json.load(source)
    snapshot = build_snapshot(
        report,
        git_sha=args.git_sha,
        run_id=args.run_id,
        threshold=args.threshold,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(snapshot, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
