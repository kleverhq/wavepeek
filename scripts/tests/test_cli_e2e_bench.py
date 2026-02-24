from __future__ import annotations

import argparse
import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "cli_e2e_bench.py"


def load_module():
    spec = importlib.util.spec_from_file_location("cli_e2e_bench", SCRIPT_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load module at {SCRIPT_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class CatalogTests(unittest.TestCase):
    def test_catalog_has_expected_matrix_size(self) -> None:
        module = load_module()

        tests = module.build_test_catalog()
        by_command = {"info": 0, "at": 0, "change": 0}
        for test in tests:
            by_command[test.command] += 1

        self.assertEqual(282, len(tests))
        self.assertEqual(3, by_command["info"])
        self.assertEqual(36, by_command["at"])
        self.assertEqual(243, by_command["change"])

    def test_catalog_filter_uses_regex(self) -> None:
        module = load_module()
        tests = module.build_test_catalog()

        selected = module.select_tests(tests, r"^at_")
        self.assertEqual(36, len(selected))


class ReportTests(unittest.TestCase):
    def test_render_report_includes_delta_annotations(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as revised_tmp, tempfile.TemporaryDirectory() as golden_tmp:
            revised_dir = pathlib.Path(revised_tmp)
            golden_dir = pathlib.Path(golden_tmp)
            self._write_hyperfine(revised_dir, "info_size_small", mean=0.80, stddev=0.05, median=0.79, min_v=0.75, max_v=0.90)
            self._write_hyperfine(golden_dir, "info_size_small", mean=1.00, stddev=0.05, median=1.00, min_v=0.98, max_v=1.10)
            self._write_wavepeek(revised_dir, "info_size_small", data={"k": "v"}, warning=None)

            markdown = module.render_report(revised_dir, golden_dir)

            self.assertIn("| test | command | dimensions | mean_s | stddev_s | median_s | min_s | max_s |", markdown)
            self.assertIn("0.800000 (+20.00%)", markdown)
            self.assertIn("0.790000 (+21.00%)", markdown)

    @staticmethod
    def _write_hyperfine(
        run_dir: pathlib.Path,
        test_name: str,
        *,
        mean: float,
        stddev: float,
        median: float,
        min_v: float,
        max_v: float,
    ) -> None:
        payload = {
            "results": [
                {
                    "command": "echo dummy args",
                    "mean": mean,
                    "stddev": stddev,
                    "median": median,
                    "min": min_v,
                    "max": max_v,
                }
            ]
        }
        (run_dir / f"{test_name}.hyperfine.json").write_text(json.dumps(payload), encoding="utf-8")

    @staticmethod
    def _write_wavepeek(run_dir: pathlib.Path, test_name: str, *, data: object, warning: object) -> None:
        payload = {
            "test_name": test_name,
            "command": "info",
            "dims": {"size": "small"},
            "exit_code": 0,
            "stdout": "dummy args",
            "stderr": "",
            "data": data,
            "warning": warning,
        }
        (run_dir / f"{test_name}.wavepeek.json").write_text(json.dumps(payload), encoding="utf-8")


class CompareTests(unittest.TestCase):
    def test_compare_runs_reports_slowdown_and_field_mismatch(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as revised_tmp, tempfile.TemporaryDirectory() as golden_tmp:
            revised_dir = pathlib.Path(revised_tmp)
            golden_dir = pathlib.Path(golden_tmp)

            self._write_hyperfine(revised_dir, "at_size_small_signals_1_time_5", median=1.20)
            self._write_hyperfine(golden_dir, "at_size_small_signals_1_time_5", median=1.00)

            self._write_wavepeek(revised_dir, "at_size_small_signals_1_time_5", data={"v": 2}, warning="warn")
            self._write_wavepeek(golden_dir, "at_size_small_signals_1_time_5", data={"v": 1}, warning="warn")

            issues = module.compare_runs(
                revised_dir=revised_dir,
                golden_dir=golden_dir,
                max_negative_delta_pct=5.0,
                metric="median",
                required_equal_fields=("data",),
            )

            self.assertTrue(any("-20.00%" in issue for issue in issues))
            self.assertTrue(any("field 'data' mismatch" in issue for issue in issues))

    def test_compare_runs_reports_missing_revised_hyperfine(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as revised_tmp, tempfile.TemporaryDirectory() as golden_tmp:
            revised_dir = pathlib.Path(revised_tmp)
            golden_dir = pathlib.Path(golden_tmp)
            self._write_wavepeek(revised_dir, "info_size_small", data={"k": "v"}, warning=None)

            issues = module.compare_runs(
                revised_dir=revised_dir,
                golden_dir=golden_dir,
                max_negative_delta_pct=5.0,
                metric="median",
                required_equal_fields=(),
            )

            self.assertTrue(any("missing revised hyperfine artifact" in issue for issue in issues))

    def test_cmd_compare_rejects_negative_threshold(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as revised_tmp, tempfile.TemporaryDirectory() as golden_tmp:
            revised_dir = pathlib.Path(revised_tmp)
            golden_dir = pathlib.Path(golden_tmp)
            self._write_hyperfine(revised_dir, "info_size_small", median=1.0)
            self._write_hyperfine(golden_dir, "info_size_small", median=1.0)

            args = argparse.Namespace(
                revised=str(revised_dir),
                golden=str(golden_dir),
                max_negative_delta_pct=-1.0,
                metric="median",
                require_equal_field=[],
            )

            with self.assertRaises(SystemExit):
                module.cmd_compare(args)

    def test_compare_allows_revised_only_tests_with_required_fields(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as revised_tmp, tempfile.TemporaryDirectory() as golden_tmp:
            revised_dir = pathlib.Path(revised_tmp)
            golden_dir = pathlib.Path(golden_tmp)
            self._write_hyperfine(revised_dir, "change_size_small_signals_1_window_pos_5_window_size_2000_trigger_any", median=1.0)
            self._write_wavepeek(
                revised_dir,
                "change_size_small_signals_1_window_pos_5_window_size_2000_trigger_any",
                data={"x": 1},
                warning=None,
            )

            issues = module.compare_runs(
                revised_dir=revised_dir,
                golden_dir=golden_dir,
                max_negative_delta_pct=5.0,
                metric="median",
                required_equal_fields=("data", "warning"),
            )

            self.assertEqual([], issues)

    @staticmethod
    def _write_hyperfine(run_dir: pathlib.Path, test_name: str, *, median: float) -> None:
        payload = {
            "results": [
                {
                    "command": "echo dummy args",
                    "mean": median,
                    "stddev": 0.0,
                    "median": median,
                    "min": median,
                    "max": median,
                }
            ]
        }
        (run_dir / f"{test_name}.hyperfine.json").write_text(json.dumps(payload), encoding="utf-8")

    @staticmethod
    def _write_wavepeek(run_dir: pathlib.Path, test_name: str, *, data: object, warning: object) -> None:
        payload = {
            "test_name": test_name,
            "command": "at",
            "dims": {"size": "small", "signals": "1", "time": "5"},
            "exit_code": 0,
            "stdout": "dummy args",
            "stderr": "",
            "data": data,
            "warning": warning,
        }
        (run_dir / f"{test_name}.wavepeek.json").write_text(json.dumps(payload), encoding="utf-8")


class CommandValidationTests(unittest.TestCase):
    def test_cmd_report_rejects_missing_compare_directory(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as run_tmp:
            run_dir = pathlib.Path(run_tmp)
            missing_compare = run_dir / "missing-compare"
            with self.assertRaises(SystemExit):
                module.cmd_report(str(run_dir), str(missing_compare))

    def test_resolve_run_directory_creates_unique_paths(self) -> None:
        module = load_module()
        with tempfile.TemporaryDirectory() as out_tmp:
            out_dir = pathlib.Path(out_tmp)
            first = module.resolve_run_directory(None, out_dir)
            second = module.resolve_run_directory(None, out_dir)
            self.assertNotEqual(first, second)

    def test_cmd_run_rejects_zero_runs(self) -> None:
        module = load_module()
        args = argparse.Namespace(
            runs=0,
            warmup=0,
            filter=r"^info_",
            wavepeek_bin="target/release/wavepeek",
            run_dir=None,
            out_dir="bench-runs",
            compare=None,
        )
        with self.assertRaises(SystemExit):
            module.cmd_run(args)

    def test_cmd_run_rejects_negative_warmup(self) -> None:
        module = load_module()
        args = argparse.Namespace(
            runs=1,
            warmup=-1,
            filter=r"^info_",
            wavepeek_bin="target/release/wavepeek",
            run_dir=None,
            out_dir="bench-runs",
            compare=None,
        )
        with self.assertRaises(SystemExit):
            module.cmd_run(args)


if __name__ == "__main__":
    unittest.main()
