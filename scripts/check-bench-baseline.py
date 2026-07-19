#!/usr/bin/env python3
"""Compare Criterion estimates.json means against a committed baseline.

Fails (exit 1) when any bench mean exceeds baseline_mean * (1 + threshold).
Default threshold is baseline ``default_max_regression`` (0.25 / 25% as of
2026-07-18) — tightened from 50% using committed trend peak-to-peak evidence
(see docs/eval/TRENDS.md). Fallback if the key is absent remains 0.5.

Criterion's --save-baseline / --baseline is useful locally for HTML reports,
but does not exit non-zero on regression; this script is the merge gate.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_json(path: Path) -> dict:
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def _mean_from_estimates(path: Path) -> float:
    data = load_json(path)
    mean = data.get("mean") or {}
    pe = mean.get("point_estimate")
    if pe is None:
        raise SystemExit(f"missing mean.point_estimate in {path}")
    return float(pe)


def criterion_mean_ns(criterion_root: Path, bench_name: str) -> float:
    """Read mean point estimate (ns) from Criterion estimates.json.

    Prefers ``new/``, then ``base/``, then any named baseline dir
    (e.g. ``ci-gate/`` from ``--save-baseline``).
    """
    bench_dir = criterion_root / bench_name
    preferred = [
        bench_dir / "new" / "estimates.json",
        bench_dir / "base" / "estimates.json",
    ]
    for path in preferred:
        if path.is_file():
            return _mean_from_estimates(path)

    if bench_dir.is_dir():
        found = sorted(bench_dir.glob("*/estimates.json"))
        if found:
            return _mean_from_estimates(found[0])

    raise SystemExit(
        f"no Criterion estimates for {bench_name!r} under {criterion_root}"
    )


def fmt_ns(ns: float) -> str:
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.3f} ms"
    if ns >= 1_000:
        return f"{ns / 1_000:.3f} µs"
    return f"{ns:.1f} ns"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--baseline",
        type=Path,
        default=Path("docs/eval/baselines/criterion-baseline.json"),
        help="committed baseline JSON",
    )
    parser.add_argument(
        "--criterion-dir",
        type=Path,
        default=Path("target/criterion"),
        help="Criterion output root after cargo bench",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=None,
        help="max allowed relative regression (default: baseline default_max_regression or 0.25)",
    )
    parser.add_argument(
        "--strict-missing",
        action="store_true",
        help="fail if baseline lists a bench with no Criterion output (default: yes)",
    )
    args = parser.parse_args()

    baseline = load_json(args.baseline)
    benches = baseline.get("benches") or {}
    if not benches:
        print(f"error: no benches in {args.baseline}", file=sys.stderr)
        return 2

    threshold = (
        args.threshold
        if args.threshold is not None
        else float(baseline.get("default_max_regression", 0.5))
    )
    if threshold < 0:
        print("error: threshold must be >= 0", file=sys.stderr)
        return 2

    print(
        f"bench-gate: baseline={args.baseline} threshold={threshold:.0%} "
        f"criterion_dir={args.criterion_dir}"
    )

    failures: list[str] = []
    for name, meta in sorted(benches.items()):
        base_ns = float(meta["mean_ns"])
        limit_ns = base_ns * (1.0 + threshold)
        try:
            measured_ns = criterion_mean_ns(args.criterion_dir, name)
        except SystemExit as exc:
            msg = str(exc)
            print(f"FAIL  {name}: {msg}")
            failures.append(name)
            continue

        ratio = (measured_ns / base_ns) if base_ns > 0 else float("inf")
        status = "ok" if measured_ns <= limit_ns else "REGRESSED"
        line = (
            f"{status:9} {name}: measured={fmt_ns(measured_ns)} "
            f"baseline={fmt_ns(base_ns)} limit={fmt_ns(limit_ns)} "
            f"ratio={ratio:.2f}x"
        )
        print(line)
        if measured_ns > limit_ns:
            failures.append(name)

    if failures:
        print(
            f"\nbench-gate FAILED ({len(failures)}): {', '.join(failures)}",
            file=sys.stderr,
        )
        return 1

    print(f"\nbench-gate PASSED ({len(benches)} benches)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
