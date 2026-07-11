#!/usr/bin/env python3
"""Export Criterion means into a nightly trend JSON row.

Reads target/criterion/*/new/estimates.json (or baseline dir) and writes a
single JSON object suitable for appending to docs/eval/trends/ or CI artifacts.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path


def mean_from_estimates(path: Path) -> float | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    mean = data.get("mean", {})
    if isinstance(mean, dict) and "point_estimate" in mean:
        return float(mean["point_estimate"])
    return None


def collect(criterion_dir: Path) -> dict[str, float]:
    out: dict[str, float] = {}
    if not criterion_dir.is_dir():
        return out
    for estimates in criterion_dir.glob("**/new/estimates.json"):
        # path: .../<group>/<bench>/new/estimates.json
        bench = estimates.parent.parent.name
        group = estimates.parent.parent.parent.name
        key = f"{group}/{bench}" if group != criterion_dir.name else bench
        val = mean_from_estimates(estimates)
        if val is not None:
            out[key] = val
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--criterion-dir", type=Path, default=Path("target/criterion"))
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--sha", default=os.environ.get("GITHUB_SHA", ""))
    ap.add_argument("--seed", default=os.environ.get("SHARECLI_BENCH_SEED", "42"))
    args = ap.parse_args()

    means = collect(args.criterion_dir)
    row = {
        "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "sha": args.sha,
        "seed": args.seed,
        "runner": os.environ.get("RUNNER_OS", ""),
        "means_ns": means,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(row, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {args.out} ({len(means)} means)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
