#!/usr/bin/env python3
# comment_flake_tracker.py — post a GitHub PR comment summarizing the
# flake-tracker's report. Invoked by .github/workflows/flake-tracker.yml.

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys


def render_body(report: dict) -> str:
    diff = report.get("baseline_diff") or {}
    flaky = len(report.get("flaky_cases") or [])
    regressions = len(report.get("regression_cases") or [])
    introduced = diff.get("introduced_count", 0)
    resolved = diff.get("resolved_count", 0)
    persistent = diff.get("persistent_count", 0)
    lines = [
        "<!-- flake-tracker -->",
        "### flake-tracker (C07 / L68)",
        "",
        f"- **flake_rate**: {report['flake_rate']:.4%}",
        f"- **flaky**: {flaky}",
        f"- **regression**: {regressions}",
        f"- **introduced**: {introduced}",
        f"- **resolved**: {resolved}",
        f"- **persistent**: {persistent}",
        "",
        "See `audit/.flake-tracker/flake-report.json` artifact for full details.",
    ]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", required=True, type=str)
    parser.add_argument("--pr-number", required=True, type=str)
    args = parser.parse_args()

    with open(args.report, "r", encoding="utf-8") as f:
        report = json.load(f)
    body = render_body(report)

    cmd = [
        "gh",
        "pr",
        "comment",
        args.pr_number,
        "--body",
        body,
    ]
    env = os.environ.copy()
    if "GH_TOKEN" not in env:
        env["GH_TOKEN"] = os.environ.get("GITHUB_TOKEN", "")
    try:
        subprocess.run(cmd, check=True, env=env)
    except subprocess.CalledProcessError as e:
        print(f"comment failed: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
