#!/usr/bin/env python3

from __future__ import annotations

import json
import pathlib
import subprocess
import tempfile
import unittest


SCRIPT_PATH = pathlib.Path(__file__).with_name("check_coverage.py")


class CheckCoverageCliTest(unittest.TestCase):
    def run_script(
        self,
        payload: dict,
        *extra_args: str,
        repo_root: pathlib.Path | None = None,
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            root = repo_root or temp_path
            summary_path = temp_path / "coverage.json"
            summary_path.write_text(json.dumps(payload), encoding="utf-8")
            return subprocess.run(
                [
                    "python3",
                    str(SCRIPT_PATH),
                    "--summary-json",
                    str(summary_path),
                    "--repo-root",
                    str(root),
                    *extra_args,
                ],
                check=False,
                capture_output=True,
                text=True,
            )

    def test_accepts_scoped_src_totals_and_ignores_non_src_and_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            payload = sample_export(
                [
                    file_entry(root / "src/engine.rs", regions=(10, 9), functions=(10, 9), lines=(10, 9)),
                    file_entry(root / "bench/helper.rs", regions=(100, 0), functions=(100, 0), lines=(100, 0)),
                    file_entry(root / "src/tests/ignored.rs", regions=(100, 0), functions=(100, 0), lines=(100, 0)),
                ]
            )

            result = self.run_script(payload, repo_root=root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("scope=src/**", result.stdout)
        self.assertIn("regions=90.00%", result.stdout)
        self.assertIn("average=90.00%", result.stdout)

    def test_rejects_metric_below_threshold_and_reports_all_failures(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            payload = sample_export(
                [file_entry(root / "src/core.rs", regions=(100, 89), functions=(100, 88), lines=(100, 87))]
            )

            result = self.run_script(payload, repo_root=root)

        self.assertEqual(result.returncode, 1)
        self.assertIn("error: coverage: source coverage gate failed", result.stderr)
        self.assertIn("regions 89.00% < 90.00%", result.stderr)
        self.assertIn("functions 88.00% < 90.00%", result.stderr)
        self.assertIn("lines 87.00% < 90.00%", result.stderr)

    def test_writes_markdown_summary_before_threshold_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            summary_path = root / "coverage.json"
            markdown_path = root / "coverage.md"
            summary_path.write_text(
                json.dumps(sample_export([file_entry(root / "src/core.rs", regions=(10, 8), functions=(10, 8), lines=(10, 8))])),
                encoding="utf-8",
            )

            result = subprocess.run(
                [
                    "python3",
                    str(SCRIPT_PATH),
                    "--summary-json",
                    str(summary_path),
                    "--repo-root",
                    str(root),
                    "--markdown-output",
                    str(markdown_path),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            markdown = markdown_path.read_text(encoding="utf-8")
            self.assertIn("## Source coverage gate", markdown)
            self.assertIn("- Scope: `src/**` excluding `**/tests/**`", markdown)
            self.assertIn("- Regions: `80.00%` (min `90.00%`)", markdown)

    def test_fails_when_summary_shape_is_malformed(self) -> None:
        result = self.run_script({"data": [{"totals": {}}]})

        self.assertEqual(result.returncode, 1)
        self.assertIn("expected each export block to contain a 'files' array", result.stderr)

    def test_fails_when_no_files_match_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            payload = sample_export([file_entry(root / "bench/helper.rs", regions=(10, 10), functions=(10, 10), lines=(10, 10))])

            result = self.run_script(payload, repo_root=root)

        self.assertEqual(result.returncode, 1)
        self.assertIn("no files matched scope 'src/**'", result.stderr)

    def test_fails_when_metric_counts_are_invalid(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            payload = sample_export([file_entry(root / "src/core.rs", regions=(10, 11), functions=(10, 10), lines=(10, 10))])

            result = self.run_script(payload, repo_root=root)

        self.assertEqual(result.returncode, 1)
        self.assertIn("metric 'regions' has invalid counts", result.stderr)


def sample_export(files: list[dict]) -> dict:
    return {
        "data": [{"files": files, "totals": {}}],
        "type": "llvm.coverage.json.export",
        "version": "2.0.1",
        "cargo_llvm_cov": {"version": "0.0.0"},
    }


def file_entry(
    path: pathlib.Path,
    *,
    regions: tuple[int, int],
    functions: tuple[int, int],
    lines: tuple[int, int],
) -> dict:
    return {
        "filename": str(path),
        "summary": {
            "regions": metric(*regions),
            "functions": metric(*functions),
            "lines": metric(*lines),
        },
    }


def metric(count: int, covered: int) -> dict:
    return {
        "count": count,
        "covered": covered,
        "notcovered": count - covered,
        "percent": 0,
    }


if __name__ == "__main__":
    unittest.main()
