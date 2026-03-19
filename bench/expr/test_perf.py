import argparse
import csv
import importlib.util
import io
import json
import pathlib
import tempfile
import unittest
from typing import Mapping
from unittest import mock


SPEC = importlib.util.spec_from_file_location(
    "bench_expr_perf",
    pathlib.Path(__file__).with_name("perf.py"),
)
assert SPEC is not None
assert SPEC.loader is not None
perf = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(perf)


class PerfHelpersTest(unittest.TestCase):
    @staticmethod
    def _suite(
        suite_id: str = "syntax",
        bench_target: str = "expr_syntax",
        scenarios: list[str] | None = None,
    ) -> dict[str, object]:
        return {
            "id": suite_id,
            "bench_target": bench_target,
            "description": f"suite {suite_id}",
            "scenarios": list(scenarios or ["tokenize_union_iff"]),
        }

    @staticmethod
    def _summary(
        selected_suite_ids: list[str],
        suites: list[dict[str, object]],
        *,
        catalog_fingerprint: str = "fingerprint-a",
        extra_metadata: Mapping[str, object] | None = None,
    ) -> dict[str, object]:
        payload: dict[str, object] = {
            "generated_at_utc": "2026-03-19T22:00:00Z",
            "run_name": "sample-run",
            "catalog_path": "bench/expr/suites.json",
            "catalog_fingerprint": catalog_fingerprint,
            "selected_suite_ids": list(selected_suite_ids),
            "cargo_version": "cargo 1.93.0",
            "rustc_version": "rustc 1.93.0",
            "criterion_version": "0.8.2",
            "source_commit": "abc123",
            "worktree_state": "clean",
            "environment_note": "test-env",
            "suites": suites,
        }
        if extra_metadata:
            payload.update(extra_metadata)
        return payload

    @staticmethod
    def _write_summary(run_dir: pathlib.Path, payload: dict[str, object]) -> None:
        run_dir.mkdir(parents=True, exist_ok=True)
        (run_dir / "summary.json").write_text(
            json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    @staticmethod
    def _write_raw_csv(path: pathlib.Path, samples_ns_per_iter: list[float]) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(
                handle,
                fieldnames=["sample_measured_value", "iteration_count"],
            )
            writer.writeheader()
            for sample in samples_ns_per_iter:
                writer.writerow(
                    {
                        "sample_measured_value": f"{sample * 10.0}",
                        "iteration_count": "10",
                    }
                )

    def test_load_catalog_rejects_duplicate_suite_ids(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            catalog_path = pathlib.Path(temp_dir) / "suites.json"
            catalog_path.write_text(
                json.dumps(
                    {
                        "suites": [
                            self._suite("syntax"),
                            self._suite("syntax", "expr_other", ["parse_event_union_iff"]),
                        ]
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit) as error:
                perf.load_catalog(catalog_path)

        self.assertIn("duplicate suite id", str(error.exception))

    def test_load_catalog_rejects_duplicate_scenarios_within_suite(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            catalog_path = pathlib.Path(temp_dir) / "suites.json"
            catalog_path.write_text(
                json.dumps(
                    {
                        "suites": [
                            self._suite(
                                scenarios=["tokenize_union_iff", "tokenize_union_iff"]
                            )
                        ]
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit) as error:
                perf.load_catalog(catalog_path)

        self.assertIn("duplicate scenario", str(error.exception))

    def test_capture_suite_results_exports_namespaced_raw_csv(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            criterion_root = root / "criterion"
            run_dir = root / "run"
            suite = self._suite(scenarios=["tokenize_union_iff", "parse_event_union_iff"])

            self._write_raw_csv(
                criterion_root / "syntax__tokenize_union_iff" / "wanted" / "raw.csv",
                [10.0, 20.0],
            )
            self._write_raw_csv(
                criterion_root / "syntax__parse_event_union_iff" / "wanted" / "raw.csv",
                [30.0, 40.0],
            )

            suite_result = perf.capture_suite_results(
                criterion_root,
                "wanted",
                run_dir,
                suite,
            )

            scenarios = {row["scenario"]: row for row in suite_result["scenarios"]}
            self.assertEqual(
                scenarios["tokenize_union_iff"]["raw_csv"],
                "syntax__tokenize_union_iff.raw.csv",
            )
            self.assertEqual(
                scenarios["parse_event_union_iff"]["raw_csv"],
                "syntax__parse_event_union_iff.raw.csv",
            )
            self.assertTrue((run_dir / "syntax__tokenize_union_iff.raw.csv").is_file())
            self.assertTrue((run_dir / "syntax__parse_event_union_iff.raw.csv").is_file())

    def test_capture_suite_results_rejects_legacy_unprefixed_ids(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            criterion_root = root / "criterion"
            run_dir = root / "run"
            suite = self._suite(scenarios=["tokenize_union_iff"])

            self._write_raw_csv(
                criterion_root / "tokenize_union_iff" / "wanted" / "raw.csv",
                [10.0, 20.0],
            )

            with self.assertRaises(SystemExit) as error:
                perf.capture_suite_results(criterion_root, "wanted", run_dir, suite)

        self.assertIn("missing scenarios", str(error.exception))

    def test_capture_suite_results_rejects_unexpected_prefixed_scenario(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            criterion_root = root / "criterion"
            run_dir = root / "run"
            suite = {
                "id": "event",
                "bench_target": "expr_event",
                "description": "suite event",
                "scenarios": ["bind_event_union_iff"],
            }

            self._write_raw_csv(
                criterion_root / "event__bind_event_union_iff" / "wanted" / "raw.csv",
                [10.0, 20.0],
            )
            self._write_raw_csv(
                criterion_root / "event__unexpected_case" / "wanted" / "raw.csv",
                [30.0, 40.0],
            )

            with self.assertRaises(SystemExit) as error:
                perf.capture_suite_results(criterion_root, "wanted", run_dir, suite)

        self.assertIn("unexpected scenarios", str(error.exception))

    def test_load_summary_rejects_duplicate_suite_scenario_rows(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir)
            self._write_summary(
                run_dir,
                self._summary(
                    ["syntax"],
                    [
                        {
                            "id": "syntax",
                            "bench_target": "expr_syntax",
                            "description": "suite parser",
                            "scenarios": [
                                {
                                    "scenario": "tokenize_union_iff",
                                    "criterion_benchmark_id": "syntax__tokenize_union_iff",
                                    "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                                    "sample_count": 100,
                                    "mean_ns_per_iter": 10.0,
                                    "median_ns_per_iter": 10.0,
                                },
                                {
                                    "scenario": "tokenize_union_iff",
                                    "criterion_benchmark_id": "syntax__tokenize_union_iff",
                                    "raw_csv": "syntax__tokenize_union_iff.dup.raw.csv",
                                    "sample_count": 100,
                                    "mean_ns_per_iter": 11.0,
                                    "median_ns_per_iter": 11.0,
                                },
                            ],
                        }
                    ],
                ),
            )

            with self.assertRaises(SystemExit) as error:
                perf.load_summary(run_dir)

        self.assertIn("duplicate suite/scenario", str(error.exception))

    def test_cmd_run_rejects_non_empty_run_dir_without_missing_only(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            run_dir.mkdir()
            (run_dir / "stale.txt").write_text("stale\n", encoding="utf-8")
            args = argparse.Namespace(
                catalog=str(root / "suites.json"),
                suite=[],
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=False,
                criterion_root=str(root / "criterion"),
                environment_note="test-env",
            )
            pathlib.Path(args.catalog).write_text(
                json.dumps({"suites": [self._suite()]}) + "\n",
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit) as error:
                perf.cmd_run(args)

        self.assertIn("must be empty", str(error.exception))

    def test_cmd_run_missing_only_rejects_stale_catalog_fingerprint(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            suite = {
                "id": "syntax",
                "bench_target": "expr_syntax",
                "description": "suite parser",
                "scenarios": [],
            }
            self._write_summary(run_dir, self._summary(["syntax"], [suite]))
            catalog_path = root / "suites.json"
            catalog_path.write_text(json.dumps({"suites": [self._suite()]}) + "\n", encoding="utf-8")
            args = argparse.Namespace(
                catalog=str(catalog_path),
                suite=[],
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=True,
                criterion_root=str(root / "criterion"),
                environment_note="test-env",
            )

            with mock.patch.object(perf, "catalog_fingerprint", return_value="different"):
                with self.assertRaises(SystemExit) as error:
                    perf.cmd_run(args)

        self.assertIn("catalog fingerprint mismatch", str(error.exception))

    def test_cmd_run_missing_only_rejects_provenance_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            suite = {
                "id": "syntax",
                "bench_target": "expr_syntax",
                "description": "suite syntax",
                "scenarios": [],
            }
            self._write_summary(
                run_dir,
                self._summary(
                    ["syntax"],
                    [suite],
                    extra_metadata={
                        "cargo_version": "cargo 1.0",
                        "rustc_version": "rustc 1.0",
                        "criterion_version": "0.8.0",
                        "source_commit": "old-commit",
                        "worktree_state": "clean",
                        "environment_note": "test-env",
                    },
                ),
            )
            catalog_path = root / "suites.json"
            catalog_path.write_text(json.dumps({"suites": [self._suite()]}) + "\n", encoding="utf-8")
            args = argparse.Namespace(
                catalog=str(catalog_path),
                suite=[],
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=True,
                criterion_root=str(root / "criterion"),
                environment_note="test-env",
            )

            with (
                mock.patch.object(perf, "tool_version", side_effect=["cargo 1.0", "rustc 1.0"]),
                mock.patch.object(perf, "cargo_lock_criterion_version", return_value="0.8.0"),
                mock.patch.object(perf, "git_source_commit", return_value="new-commit"),
                mock.patch.object(perf, "git_worktree_state", return_value="clean"),
                mock.patch.object(perf, "catalog_fingerprint", return_value="fingerprint-a"),
            ):
                with self.assertRaises(SystemExit) as error:
                    perf.cmd_run(args)

        self.assertIn("source_commit mismatch", str(error.exception))

    def test_cmd_run_missing_only_runs_only_missing_suites(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            syntax_suite = {
                "id": "syntax",
                "bench_target": "expr_syntax",
                "description": "suite syntax",
                "scenarios": [
                    {
                        "scenario": "tokenize_union_iff",
                        "criterion_benchmark_id": "syntax__tokenize_union_iff",
                        "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                        "sample_count": 100,
                        "mean_ns_per_iter": 10.0,
                        "median_ns_per_iter": 10.0,
                    }
                ],
            }
            self._write_summary(
                run_dir,
                self._summary(
                    ["syntax", "event"],
                    [syntax_suite],
                    extra_metadata={
                        "cargo_version": "cargo 1.0",
                        "rustc_version": "rustc 1.0",
                        "criterion_version": "0.8.0",
                        "source_commit": "abc123",
                        "worktree_state": "clean",
                        "environment_note": "test-env",
                    },
                ),
            )
            (run_dir / "syntax__tokenize_union_iff.raw.csv").write_text("csv\n", encoding="utf-8")

            catalog_path = root / "suites.json"
            catalog_path.write_text(
                json.dumps(
                    {
                        "suites": [
                            self._suite("syntax", "expr_syntax", ["tokenize_union_iff"]),
                            self._suite(
                                "event",
                                "expr_event",
                                ["bind_event_union_iff"],
                            ),
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            args = argparse.Namespace(
                catalog=str(catalog_path),
                suite=[],
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=True,
                criterion_root=str(root / "criterion"),
                environment_note="test-env",
            )

            captured_event_suite = {
                "id": "event",
                "bench_target": "expr_event",
                "description": "suite event",
                "scenarios": [
                    {
                        "scenario": "bind_event_union_iff",
                        "criterion_benchmark_id": "event__bind_event_union_iff",
                        "raw_csv": "event__bind_event_union_iff.raw.csv",
                        "sample_count": 100,
                        "mean_ns_per_iter": 20.0,
                        "median_ns_per_iter": 20.0,
                    }
                ],
            }

            with (
                mock.patch.object(perf, "tool_version", side_effect=["cargo 1.0", "rustc 1.0"]),
                mock.patch.object(perf, "cargo_lock_criterion_version", return_value="0.8.0"),
                mock.patch.object(perf, "git_source_commit", return_value="abc123"),
                mock.patch.object(perf, "git_worktree_state", return_value="clean"),
                mock.patch.object(perf, "catalog_fingerprint", return_value="fingerprint-a"),
                mock.patch.object(perf, "run_suite_benchmark") as run_bench,
                mock.patch.object(perf, "capture_suite_results", return_value=captured_event_suite) as capture,
            ):
                exit_code = perf.cmd_run(args)

            self.assertEqual(exit_code, 0)
            run_bench.assert_called_once()
            capture.assert_called_once()
            self.assertEqual(capture.call_args.args[3]["id"], "event")
            summary = json.loads((run_dir / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["selected_suite_ids"], ["syntax", "event"])
            self.assertEqual(
                [suite["id"] for suite in summary["suites"]],
                ["syntax", "event"],
            )

    def test_cmd_run_writes_summary_and_readme(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            catalog_path = root / "suites.json"
            catalog_path.write_text(json.dumps({"suites": [self._suite()]}) + "\n", encoding="utf-8")
            args = argparse.Namespace(
                catalog=str(catalog_path),
                suite=[],
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=False,
                criterion_root=str(root / "criterion"),
                environment_note="test-env",
            )
            captured_suite = {
                "id": "syntax",
                "bench_target": "expr_syntax",
                "description": "suite syntax",
                "scenarios": [
                    {
                        "scenario": "tokenize_union_iff",
                        "criterion_benchmark_id": "syntax__tokenize_union_iff",
                        "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                        "sample_count": 100,
                        "mean_ns_per_iter": 10.0,
                        "median_ns_per_iter": 9.5,
                    }
                ],
            }

            with (
                mock.patch.object(perf, "tool_version", side_effect=["cargo 1.0", "rustc 1.0"]),
                mock.patch.object(perf, "cargo_lock_criterion_version", return_value="0.8.0"),
                mock.patch.object(perf, "git_source_commit", return_value="abc123"),
                mock.patch.object(perf, "git_worktree_state", return_value="clean"),
                mock.patch.object(perf, "catalog_fingerprint", return_value="fingerprint-a"),
                mock.patch.object(perf, "run_suite_benchmark"),
                mock.patch.object(perf, "capture_suite_results", return_value=captured_suite),
            ):
                exit_code = perf.cmd_run(args)

            self.assertEqual(exit_code, 0)
            summary = json.loads((run_dir / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["catalog_fingerprint"], "fingerprint-a")
            self.assertEqual(summary["selected_suite_ids"], ["syntax"])
            readme = (run_dir / "README.md").read_text(encoding="utf-8")
            self.assertIn("# Expression Bench Run: run", readme)
            self.assertIn("## syntax", readme)
            self.assertIn("tokenize_union_iff", readme)
            self.assertNotIn("Compare baseline", readme)

    def test_cmd_report_writes_compare_context(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            suite_rows = [
                {
                    "id": "syntax",
                    "bench_target": "expr_syntax",
                    "description": "suite syntax",
                    "scenarios": [
                        {
                            "scenario": "tokenize_union_iff",
                            "criterion_benchmark_id": "syntax__tokenize_union_iff",
                            "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                            "sample_count": 100,
                            "mean_ns_per_iter": 8.0,
                            "median_ns_per_iter": 8.0,
                        }
                    ],
                }
            ]
            self._write_summary(revised, self._summary(["syntax"], suite_rows))
            golden_rows = json.loads(json.dumps(suite_rows))
            golden_rows[0]["scenarios"][0]["mean_ns_per_iter"] = 10.0
            golden_rows[0]["scenarios"][0]["median_ns_per_iter"] = 10.0
            self._write_summary(golden, self._summary(["syntax"], golden_rows))

            args = argparse.Namespace(run_dir=str(revised), compare=str(golden))
            exit_code = perf.cmd_report(args)

            self.assertEqual(exit_code, 0)
            readme = (revised / "README.md").read_text(encoding="utf-8")
            self.assertIn("Compare baseline", readme)
            self.assertIn("+20.00%", readme)

    def test_cmd_report_uses_summary_timestamp(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir)
            self._write_summary(
                run_dir,
                self._summary(
                    ["syntax"],
                    [
                        {
                            "id": "syntax",
                            "bench_target": "expr_syntax",
                            "description": "suite syntax",
                            "scenarios": [
                                {
                                    "scenario": "tokenize_union_iff",
                                    "criterion_benchmark_id": "syntax__tokenize_union_iff",
                                    "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                                    "sample_count": 100,
                                    "mean_ns_per_iter": 10.0,
                                    "median_ns_per_iter": 10.0,
                                }
                            ],
                        }
                    ],
                    extra_metadata={"generated_at_utc": "2026-03-20T00:00:00Z"},
                ),
            )

            exit_code = perf.cmd_report(argparse.Namespace(run_dir=str(run_dir), compare=None))

            self.assertEqual(exit_code, 0)
            readme = (run_dir / "README.md").read_text(encoding="utf-8")
            self.assertIn("2026-03-20T00:00:00Z", readme)

    def test_cmd_compare_rejects_suite_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            self._write_summary(revised, self._summary(["syntax"], []))
            self._write_summary(golden, self._summary(["event"], []))

            with self.assertRaises(SystemExit) as error:
                perf.main(
                    [
                        "compare",
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "5",
                    ]
                )

        self.assertIn("selected suite mismatch", str(error.exception))

    def test_cmd_compare_rejects_scenario_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised_suites = [
                {
                    "id": "syntax",
                    "bench_target": "expr_syntax",
                    "description": "suite syntax",
                    "scenarios": [
                        {
                            "scenario": "tokenize_union_iff",
                            "criterion_benchmark_id": "syntax__tokenize_union_iff",
                            "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                            "sample_count": 100,
                            "mean_ns_per_iter": 10.0,
                            "median_ns_per_iter": 10.0,
                        }
                    ],
                }
            ]
            golden_suites = [
                {
                    "id": "syntax",
                    "bench_target": "expr_syntax",
                    "description": "suite syntax",
                    "scenarios": [
                        {
                            "scenario": "parse_event_union_iff",
                            "criterion_benchmark_id": "syntax__parse_event_union_iff",
                            "raw_csv": "syntax__parse_event_union_iff.raw.csv",
                            "sample_count": 100,
                            "mean_ns_per_iter": 10.0,
                            "median_ns_per_iter": 10.0,
                        }
                    ],
                }
            ]
            self._write_summary(revised, self._summary(["syntax"], revised_suites))
            self._write_summary(golden, self._summary(["syntax"], golden_suites))

            with self.assertRaises(SystemExit) as error:
                perf.main(
                    [
                        "compare",
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "5",
                    ]
                )

        self.assertIn("scenario mismatch", str(error.exception))

    def test_cmd_compare_supports_required_matching_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            suites = [
                {
                    "id": "syntax",
                    "bench_target": "expr_syntax",
                    "description": "suite syntax",
                    "scenarios": [
                        {
                            "scenario": "tokenize_union_iff",
                            "criterion_benchmark_id": "syntax__tokenize_union_iff",
                            "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                            "sample_count": 100,
                            "mean_ns_per_iter": 10.0,
                            "median_ns_per_iter": 10.0,
                        }
                    ],
                }
            ]
            extra = {"source_commit": "abc123"}
            self._write_summary(revised, self._summary(["syntax"], suites, extra_metadata=extra))
            self._write_summary(golden, self._summary(["syntax"], suites, extra_metadata=extra))

            stdout = io.StringIO()
            with mock.patch("sys.stdout", stdout):
                exit_code = perf.main(
                    [
                        "compare",
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "5",
                        "--require-matching-metadata",
                        "source_commit",
                    ]
                )

        self.assertEqual(exit_code, 0)
        self.assertIn("ok:", stdout.getvalue())

    def test_cmd_compare_fails_required_matching_metadata_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            suites = [
                {
                    "id": "syntax",
                    "bench_target": "expr_syntax",
                    "description": "suite syntax",
                    "scenarios": [
                        {
                            "scenario": "tokenize_union_iff",
                            "criterion_benchmark_id": "syntax__tokenize_union_iff",
                            "raw_csv": "syntax__tokenize_union_iff.raw.csv",
                            "sample_count": 100,
                            "mean_ns_per_iter": 10.0,
                            "median_ns_per_iter": 10.0,
                        }
                    ],
                }
            ]
            self._write_summary(
                revised,
                self._summary(["syntax"], suites, extra_metadata={"source_commit": "abc123"}),
            )
            self._write_summary(
                golden,
                self._summary(["syntax"], suites, extra_metadata={"source_commit": "def456"}),
            )

            with self.assertRaises(SystemExit) as error:
                perf.main(
                    [
                        "compare",
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "5",
                        "--require-matching-metadata",
                        "source_commit",
                    ]
                )

        self.assertIn("required metadata mismatch", str(error.exception))


if __name__ == "__main__":
    unittest.main()
