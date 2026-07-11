#!/usr/bin/env python3
"""Refresh docs/eval/baselines/criterion-baseline.json from Criterion output.

Run after:
  cargo bench --locked --bench config_parse -- --sample-size 10 --warm-up-time 1 --measurement-time 2
  cargo bench --locked --bench pool_list -- --sample-size 10 --warm-up-time 1 --measurement-time 2
  cargo bench --locked --bench prometheus_render -- --sample-size 10 --warm-up-time 1 --measurement-time 2

Optional local Criterion baseline (HTML compare only; gate still uses JSON):
  cargo bench --locked --bench config_parse -- --save-baseline ci
  cargo bench --locked --bench config_parse -- --baseline ci
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Keep in sync with docs/ops/SLO.md BENCH-1..3
DEFAULT_BENCHES = {
    "config_toml_from_str": "BENCH-1",
    "pool_new_and_list_empty": "BENCH-2",
    "prometheus_render_32": "BENCH-3",
}


def criterion_mean_ns(criterion_root: Path, bench_name: str) -> float:
    for sub in ("new", "base"):
        path = criterion_root / bench_name / sub / "estimates.json"
        if path.is_file():
            data = json.loads(path.read_text(encoding="utf-8"))
            return float(data["mean"]["point_estimate"])
    raise FileNotFoundError(bench_name)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("docs/eval/baselines/criterion-baseline.json"),
    )
    parser.add_argument(
        "--criterion-dir",
        type=Path,
        default=Path("target/criterion"),
    )
    parser.add_argument(
        "--max-regression",
        type=float,
        default=0.5,
        help="default_max_regression written into the baseline file",
    )
    args = parser.parse_args()

    benches: dict[str, dict] = {}
    missing: list[str] = []
    for name, slo_id in DEFAULT_BENCHES.items():
        try:
            mean_ns = criterion_mean_ns(args.criterion_dir, name)
        except FileNotFoundError:
            missing.append(name)
            continue
        benches[name] = {
            "mean_ns": int(round(mean_ns)),
            "slo_id": slo_id,
            "notes": "Seeded from Criterion mean.point_estimate on this host.",
        }

    if missing:
        print(
            f"error: missing Criterion output for: {', '.join(missing)}",
            file=sys.stderr,
        )
        return 1

    payload = {
        "schema_version": 1,
        "description": (
            "Criterion mean baselines for the C08 perf gate. "
            "Regenerate with scripts/seed-bench-baseline.py after cargo bench."
        ),
        "unit": "ns",
        "default_max_regression": args.max_regression,
        "runner": "local-or-ci",
        "sample_args": "--sample-size 10 --warm-up-time 1 --measurement-time 2",
        "benches": benches,
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {args.out}")
    for name, meta in benches.items():
        print(f"  {name}: {meta['mean_ns']} ns")
    return 0


if __name__ == "__main__":
    sys.exit(main())
