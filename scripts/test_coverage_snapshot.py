#!/usr/bin/env python3
"""Tests for the llvm-cov snapshot generator."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts.coverage_snapshot import METRICS, build_snapshot, main


def report_with(*, count: int = 10, covered: int = 9, percent: float = 90.0):
    metric = {"count": count, "covered": covered, "percent": percent}
    return {
        "type": "llvm.coverage.json.export",
        "data": [{"totals": {name: metric.copy() for name in METRICS}}],
    }


class CoverageSnapshotTests(unittest.TestCase):
    def test_build_snapshot_preserves_reported_zero_count_percent(self) -> None:
        snapshot = build_snapshot(
            report_with(count=0, covered=0, percent=0.0),
            git_sha="abc123",
            run_id="42",
            threshold=85.0,
        )

        self.assertEqual(snapshot["coverage"]["branches"]["percent"], 0.0)
        self.assertFalse(snapshot["meets_lines_target"])

    def test_build_snapshot_rejects_non_object_root(self) -> None:
        with self.assertRaisesRegex(ValueError, "JSON object"):
            build_snapshot([], git_sha="abc123", run_id="42", threshold=85.0)

    def test_build_snapshot_rejects_non_object_data_entry(self) -> None:
        with self.assertRaisesRegex(ValueError, "data entry as a JSON object"):
            build_snapshot(
                {"data": ["invalid"]},
                git_sha="abc123",
                run_id="42",
                threshold=85.0,
            )

    def test_main_creates_output_parent_directories(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "coverage.json"
            output = root / "nested" / "coverage-snapshot.json"
            source.write_text(json.dumps(report_with()), encoding="utf-8")

            with mock.patch(
                "sys.argv",
                [
                    "coverage_snapshot.py",
                    str(source),
                    str(output),
                    "--git-sha",
                    "abc123",
                ],
            ):
                main()

            self.assertTrue(output.is_file())


if __name__ == "__main__":
    unittest.main()
