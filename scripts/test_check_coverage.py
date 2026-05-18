#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import subprocess
import tempfile
import textwrap
import unittest


SCRIPT_PATH = pathlib.Path(__file__).with_name("check_coverage.py")


class CheckCoverageCliTest(unittest.TestCase):
    def run_script(
        self,
        summary_text: str,
        *extra_args: str,
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            summary_path = temp_path / "coverage.txt"
            summary_path.write_text(summary_text, encoding="utf-8")
            return subprocess.run(
                [
                    "python3",
                    str(SCRIPT_PATH),
                    "--summary",
                    str(summary_path),
                    *extra_args,
                ],
                check=False,
                capture_output=True,
                text=True,
            )

    def test_accepts_total_row_above_threshold_and_reports_summary(self) -> None:
        result = self.run_script(sample_summary("95.56%", "96.14%", "96.21%"))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("coverage ok:", result.stdout)
        self.assertIn("regions=95.56%", result.stdout)
        self.assertIn("average=95.97%", result.stdout)

    def test_rejects_metric_below_threshold(self) -> None:
        result = self.run_script(sample_summary("89.99%", "96.14%", "96.21%"))

        self.assertEqual(result.returncode, 1)
        self.assertIn("error: coverage: source coverage gate failed", result.stderr)
        self.assertIn("regions 89.99% < 90.00%", result.stderr)

    def test_fails_when_total_row_is_missing(self) -> None:
        result = self.run_script("Filename\n---\n")

        self.assertEqual(result.returncode, 1)
        self.assertIn("error: coverage: TOTAL row not found", result.stderr)


def sample_summary(regions: str, functions: str, lines: str) -> str:
    return textwrap.dedent(
        f"""\
        Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
        -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
        expr/parser.rs                   2996                93    96.90%         436                 1    99.77%        2728                84    96.92%           0                 0         -
        -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
        TOTAL                           23378              1038    {regions}        1372                53    {functions}       17897               678    {lines}           0                 0         -
        """
    )


if __name__ == "__main__":
    unittest.main()
