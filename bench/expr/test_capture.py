import csv
import importlib.util
import io
import json
import pathlib
import tempfile
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location(
    "bench_expr_capture",
    pathlib.Path(__file__).with_name("capture.py"),
)
assert SPEC is not None
assert SPEC.loader is not None
capture = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(capture)


class CaptureHelpersTest(unittest.TestCase):
    @staticmethod
    def _write_raw_csv(path: pathlib.Path, samples_ns_per_iter: list[float]) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(
                handle,
                fieldnames=[
                    "sample_measured_value",
                    "iteration_count",
                ],
            )
            writer.writeheader()
            for sample in samples_ns_per_iter:
                writer.writerow(
                    {
                        "sample_measured_value": f"{sample * 10.0}",
                        "iteration_count": "10",
                    }
                )

    def test_collect_raw_csv_paths_fails_when_baseline_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            self._write_raw_csv(root / "tokenize_union_iff" / "other" / "raw.csv", [1.0])

            with self.assertRaises(SystemExit) as error:
                capture.collect_raw_csv_paths(root, "requested")

        self.assertIn("requested baseline 'requested' not found", str(error.exception))

    def test_collect_raw_csv_paths_requires_exact_scenario_set(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            self._write_raw_csv(root / "tokenize_union_iff" / "c1" / "raw.csv", [1.0])
            self._write_raw_csv(root / "parse_event_union_iff" / "c1" / "raw.csv", [2.0])

            with self.assertRaises(SystemExit) as error:
                capture.collect_raw_csv_paths(root, "c1")

        self.assertIn("missing scenarios", str(error.exception))

    def test_collect_raw_csv_paths_rejects_duplicate_scenario_exports(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            self._write_raw_csv(root / "a" / "tokenize_union_iff" / "c1" / "raw.csv", [1.0])
            self._write_raw_csv(root / "b" / "tokenize_union_iff" / "c1" / "raw.csv", [2.0])
            self._write_raw_csv(root / "parse_event_union_iff" / "c1" / "raw.csv", [3.0])
            self._write_raw_csv(root / "parse_event_malformed" / "c1" / "raw.csv", [4.0])

            with self.assertRaises(SystemExit) as error:
                capture.collect_raw_csv_paths(root, "c1")

        self.assertIn("duplicate raw.csv for scenario 'tokenize_union_iff'", str(error.exception))

    def test_main_exports_requested_baseline_with_multiple_saved_baselines(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            criterion_root = root / "criterion"
            output = root / "run"

            for scenario in capture.REQUIRED_SCENARIOS:
                self._write_raw_csv(
                    criterion_root / scenario / "wanted" / "raw.csv",
                    [10.0, 20.0],
                )
                self._write_raw_csv(
                    criterion_root / scenario / "other" / "raw.csv",
                    [1000.0, 2000.0],
                )

            with (
                mock.patch.object(capture, "tool_version", side_effect=["cargo 1.0", "rustc 1.0"]),
                mock.patch.object(capture, "cargo_lock_criterion_version", return_value="0.8.0"),
                mock.patch("sys.stdout", new_callable=io.StringIO),
            ):
                exit_code = capture.main(
                    [
                        "--criterion-root",
                        str(criterion_root),
                        "--baseline-name",
                        "wanted",
                        "--output",
                        str(output),
                        "--source-commit",
                        "abc123",
                        "--worktree-state",
                        "clean",
                        "--environment-note",
                        "test-env",
                    ]
                )

            self.assertEqual(exit_code, 0)
            summary = json.loads((output / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["baseline_name"], "wanted")

            scenarios = {row["scenario"]: row for row in summary["scenarios"]}
            self.assertEqual(set(scenarios), set(capture.REQUIRED_SCENARIOS))
            for row in scenarios.values():
                self.assertAlmostEqual(float(row["mean_ns_per_iter"]), 15.0)
                self.assertAlmostEqual(float(row["median_ns_per_iter"]), 15.0)
                raw_path = output / str(row["raw_csv"])
                self.assertTrue(raw_path.is_file())

    def test_parse_raw_csv_rejects_non_finite_values(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = pathlib.Path(temp_dir) / "raw.csv"
            with path.open("w", encoding="utf-8", newline="") as handle:
                writer = csv.DictWriter(
                    handle,
                    fieldnames=["sample_measured_value", "iteration_count"],
                )
                writer.writeheader()
                writer.writerow(
                    {
                        "sample_measured_value": "nan",
                        "iteration_count": "10",
                    }
                )

            with self.assertRaises(SystemExit) as error:
                capture.parse_raw_csv(path)

        self.assertIn("non-finite sample_measured_value", str(error.exception))


if __name__ == "__main__":
    unittest.main()
