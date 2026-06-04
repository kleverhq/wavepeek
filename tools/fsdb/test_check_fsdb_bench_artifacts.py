#!/usr/bin/env python3

from __future__ import annotations

import json
import os
import pathlib
import subprocess
import tempfile
import unittest

SCRIPT_PATH = pathlib.Path(__file__).with_name("check_fsdb_bench_artifacts.py").resolve()


class CheckFsdbBenchArtifactsTest(unittest.TestCase):
    def run_script(
        self, args: list[str], rtl_dir: pathlib.Path
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["RTL_ARTIFACTS_DIR"] = str(rtl_dir)
        return subprocess.run(
            ["python3", str(SCRIPT_PATH), *args],
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def write_catalog(self, root: pathlib.Path, rtl_dir: pathlib.Path) -> pathlib.Path:
        catalog = root / "tests_fsdb.json"
        catalog.write_text(
            json.dumps(
                {
                    "tests": [
                        {
                            "name": "smoke_a",
                            "command": [
                                "wavepeek",
                                "info",
                                "--waves",
                                str(rtl_dir / "a.fsdb"),
                            ],
                        },
                        {
                            "name": "full_b",
                            "command": [
                                "wavepeek",
                                "info",
                                "--waves",
                                str(rtl_dir / "b.fsdb"),
                            ],
                        },
                    ]
                }
            ),
            encoding="utf-8",
        )
        return catalog

    def test_filter_verifies_only_matching_benchmark_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            rtl_dir = root / "rtl"
            rtl_dir.mkdir()
            (rtl_dir / "a.fsdb").write_text("a\n", encoding="utf-8")
            catalog = self.write_catalog(root, rtl_dir)

            result = self.run_script([str(catalog), "--filter", "^smoke_"], rtl_dir)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("verified 1 RTL benchmark FSDB artifacts", result.stdout)

    def test_unfiltered_check_reports_missing_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            rtl_dir = root / "rtl"
            rtl_dir.mkdir()
            (rtl_dir / "a.fsdb").write_text("a\n", encoding="utf-8")
            catalog = self.write_catalog(root, rtl_dir)

            result = self.run_script([str(catalog)], rtl_dir)

            self.assertEqual(result.returncode, 1)
            self.assertIn(str(rtl_dir / "b.fsdb"), result.stderr)

    def test_filter_rejects_empty_matches(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            rtl_dir = root / "rtl"
            rtl_dir.mkdir()
            catalog = self.write_catalog(root, rtl_dir)

            result = self.run_script([str(catalog), "--filter", "^missing$"], rtl_dir)

            self.assertEqual(result.returncode, 1)
            self.assertIn("no FSDB benchmark tests matched", result.stderr)


if __name__ == "__main__":
    unittest.main()
