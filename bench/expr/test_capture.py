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
    SCENARIOS = (
        "tokenize_union_iff",
        "parse_event_union_iff",
        "parse_event_malformed",
    )

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
                capture.collect_raw_csv_paths(root, "requested", self.SCENARIOS)

        self.assertIn("requested baseline 'requested' not found", str(error.exception))

    def test_collect_raw_csv_paths_requires_exact_scenario_set(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            self._write_raw_csv(root / "tokenize_union_iff" / "c1" / "raw.csv", [1.0])
            self._write_raw_csv(root / "parse_event_union_iff" / "c1" / "raw.csv", [2.0])

            with self.assertRaises(SystemExit) as error:
                capture.collect_raw_csv_paths(root, "c1", self.SCENARIOS)

        self.assertIn("missing scenarios", str(error.exception))

    def test_collect_raw_csv_paths_rejects_duplicate_scenario_exports(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            self._write_raw_csv(root / "a" / "tokenize_union_iff" / "c1" / "raw.csv", [1.0])
            self._write_raw_csv(root / "b" / "tokenize_union_iff" / "c1" / "raw.csv", [2.0])
            self._write_raw_csv(root / "parse_event_union_iff" / "c1" / "raw.csv", [3.0])
            self._write_raw_csv(root / "parse_event_malformed" / "c1" / "raw.csv", [4.0])

            with self.assertRaises(SystemExit) as error:
                capture.collect_raw_csv_paths(root, "c1", self.SCENARIOS)

        self.assertIn("duplicate raw.csv for scenario 'tokenize_union_iff'", str(error.exception))

    def test_main_exports_requested_baseline_with_multiple_saved_baselines(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            criterion_root = root / "criterion"
            scenario_set = root / "scenario_set.json"
            output = root / "run"

            scenario_set.write_text(
                json.dumps(
                    {
                        "id": "c1_parser",
                        "bench_target": "expr_c1",
                        "scenarios": list(self.SCENARIOS),
                    },
                    ensure_ascii=True,
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )

            for scenario in self.SCENARIOS:
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
                        "--bench-target",
                        "expr_c1",
                        "--scenario-set",
                        str(scenario_set),
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
            self.assertEqual(summary["bench_target"], "expr_c1")
            self.assertEqual(summary["scenario_set_id"], "c1_parser")

            scenarios = {row["scenario"]: row for row in summary["scenarios"]}
            self.assertEqual(set(scenarios), set(self.SCENARIOS))
            for row in scenarios.values():
                self.assertAlmostEqual(float(row["mean_ns_per_iter"]), 15.0)
                self.assertAlmostEqual(float(row["median_ns_per_iter"]), 15.0)
                raw_path = output / str(row["raw_csv"])
                self.assertTrue(raw_path.is_file())

    def test_main_rejects_bench_target_mismatch_with_scenario_set(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            criterion_root = root / "criterion"
            scenario_set = root / "scenario_set.json"

            scenario_set.write_text(
                json.dumps(
                    {
                        "id": "c2_event_runtime",
                        "bench_target": "expr_c2",
                        "scenarios": list(self.SCENARIOS),
                    },
                    ensure_ascii=True,
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )

            for scenario in self.SCENARIOS:
                self._write_raw_csv(
                    criterion_root / scenario / "wanted" / "raw.csv",
                    [10.0, 20.0],
                )

            with self.assertRaises(SystemExit) as error:
                capture.main(
                    [
                        "--criterion-root",
                        str(criterion_root),
                        "--baseline-name",
                        "wanted",
                        "--bench-target",
                        "expr_c1",
                        "--scenario-set",
                        str(scenario_set),
                        "--output",
                        str(root / "run"),
                        "--source-commit",
                        "abc123",
                        "--worktree-state",
                        "clean",
                        "--environment-note",
                        "test-env",
                    ]
                )

        self.assertIn("bench target mismatch", str(error.exception))

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
