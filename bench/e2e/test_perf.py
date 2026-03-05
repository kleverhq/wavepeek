import argparse
import importlib.util
import io
import json
import os
import pathlib
import tempfile
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location(
    "bench_e2e_perf",
    pathlib.Path(__file__).with_name("perf.py"),
)
assert SPEC is not None
assert SPEC.loader is not None
perf = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(perf)


class PerfHelpersTest(unittest.TestCase):
    @staticmethod
    def _write_hyperfine_artifact(path: pathlib.Path, mean: float) -> None:
        path.write_text(
            json.dumps(
                {
                    "results": [
                        {
                            "command": "wavepeek change --json",
                            "mean": mean,
                            "stddev": 0.0,
                            "median": mean,
                            "min": mean,
                            "max": mean,
                        }
                    ]
                }
            ),
            encoding="utf-8",
        )

    @staticmethod
    def _sample_test(name: str = "sample") -> dict[str, object]:
        return {
            "name": name,
            "category": "info",
            "runs": 1,
            "warmup": 0,
            "command": ["{wavepeek_bin}", "info", "--waves", "/tmp/sample.fst", "--json"],
            "meta": {},
        }

    def test_test_has_complete_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir)
            test_name = "sample"
            run_dir.joinpath(f"{test_name}.hyperfine.json").write_text("{}", encoding="utf-8")
            self.assertFalse(perf.test_has_complete_artifacts(run_dir, test_name))

            run_dir.joinpath(f"{test_name}.wavepeek.json").write_text("{}", encoding="utf-8")
            self.assertTrue(perf.test_has_complete_artifacts(run_dir, test_name))

    def test_partition_missing_only_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir)
            selected = [
                {"name": "done"},
                {"name": "pending"},
            ]
            run_dir.joinpath("done.hyperfine.json").write_text("{}", encoding="utf-8")
            run_dir.joinpath("done.wavepeek.json").write_text("{}", encoding="utf-8")
            runnable, skipped = perf.partition_missing_only_tests(selected, run_dir)

        self.assertEqual([str(test["name"]) for test in runnable], ["pending"])
        self.assertEqual(skipped, ["done"])

    def test_run_parser_missing_only_flag(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["run"])
        self.assertFalse(default_args.missing_only)
        missing_only_args = parser.parse_args(["run", "--missing-only"])
        self.assertTrue(missing_only_args.missing_only)

    def test_list_parser_tests_flag_defaults_and_override(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["list"])
        self.assertEqual(default_args.tests, str(perf.TESTS_PATH))

        override_args = parser.parse_args(["list", "--tests", "bench/e2e/tests_commit.json"])
        self.assertEqual(override_args.tests, "bench/e2e/tests_commit.json")

    def test_run_parser_tests_flag_defaults_and_override(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["run"])
        self.assertEqual(default_args.tests, str(perf.TESTS_PATH))

        override_args = parser.parse_args(["run", "--tests", "bench/e2e/tests_commit.json"])
        self.assertEqual(override_args.tests, "bench/e2e/tests_commit.json")

    def test_run_parser_verbose_flag(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["run"])
        self.assertFalse(default_args.verbose)

        short_args = parser.parse_args(["run", "-v"])
        self.assertTrue(short_args.verbose)

        long_args = parser.parse_args(["run", "--verbose"])
        self.assertTrue(long_args.verbose)

    def test_compare_parser_verbose_flag(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(
            [
                "compare",
                "--revised",
                "bench/e2e/runs/baseline",
                "--golden",
                "bench/e2e/runs/baseline",
                "--max-negative-delta-pct",
                "5",
            ]
        )
        self.assertFalse(default_args.verbose)

        short_args = parser.parse_args(
            [
                "compare",
                "--revised",
                "bench/e2e/runs/baseline",
                "--golden",
                "bench/e2e/runs/baseline",
                "--max-negative-delta-pct",
                "5",
                "-v",
            ]
        )
        self.assertTrue(short_args.verbose)

        long_args = parser.parse_args(
            [
                "compare",
                "--revised",
                "bench/e2e/runs/baseline",
                "--golden",
                "bench/e2e/runs/baseline",
                "--max-negative-delta-pct",
                "5",
                "--verbose",
            ]
        )
        self.assertTrue(long_args.verbose)

    def test_run_parser_wavepeek_timeout_seconds_flag(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["run"])
        self.assertEqual(default_args.wavepeek_timeout_seconds, 300)

        override_args = parser.parse_args(
            ["run", "--wavepeek-timeout-seconds", "42"]
        )
        self.assertEqual(override_args.wavepeek_timeout_seconds, 42)

    def test_load_tests_fails_for_missing_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            missing = pathlib.Path(temp_dir) / "missing.json"
            with self.assertRaises(SystemExit) as error:
                perf.load_tests(missing)
        self.assertIn("error: tests:", str(error.exception))
        self.assertIn("missing file", str(error.exception))

    def test_load_tests_fails_for_invalid_json(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            tests_path = pathlib.Path(temp_dir) / "tests.json"
            tests_path.write_text("{", encoding="utf-8")
            with self.assertRaises(SystemExit) as error:
                perf.load_tests(tests_path)
        self.assertIn("error: tests: invalid JSON", str(error.exception))

    def test_cmd_list_resolves_relative_tests_path_from_cwd(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_from = root / "nested"
            run_from.mkdir()
            args = argparse.Namespace(filter=None, tests="fixtures/tests.json")
            load_tests_mock = mock.Mock(return_value=[])

            previous_cwd = pathlib.Path.cwd()
            try:
                os.chdir(run_from)
                stdout = io.StringIO()
                with mock.patch.object(perf, "load_tests", load_tests_mock), mock.patch(
                    "sys.stdout",
                    stdout,
                ):
                    exit_code = perf.cmd_list(args)
            finally:
                os.chdir(previous_cwd)

        self.assertEqual(exit_code, 0)
        load_tests_mock.assert_called_once_with((run_from / "fixtures/tests.json").resolve())

    def test_cmd_run_resolves_relative_tests_path_from_cwd(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_from = root / "nested"
            run_dir = root / "run"
            run_from.mkdir()
            run_dir.mkdir()

            test_case = self._sample_test("sample")
            args = argparse.Namespace(
                filter=None,
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=False,
                wavepeek_timeout_seconds=300,
                tests="fixtures/tests.json",
                verbose=False,
            )
            load_tests_mock = mock.Mock(return_value=[test_case])

            previous_cwd = pathlib.Path.cwd()
            try:
                os.chdir(run_from)
                stdout = io.StringIO()
                with (
                    mock.patch.object(perf, "load_tests", load_tests_mock),
                    mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                    mock.patch.object(perf, "ensure_hyperfine"),
                    mock.patch.object(perf, "resolve_wavepeek_bin", return_value="wavepeek"),
                    mock.patch.object(perf, "run_test"),
                    mock.patch.object(
                        perf,
                        "run_functional_capture",
                        return_value={"data": [], "warnings": []},
                    ),
                    mock.patch.object(perf, "write_wavepeek_artifact"),
                    mock.patch.object(perf, "write_report", return_value=run_dir / "README.md"),
                    mock.patch("sys.stdout", stdout),
                ):
                    exit_code = perf.cmd_run(args)
            finally:
                os.chdir(previous_cwd)

        self.assertEqual(exit_code, 0)
        load_tests_mock.assert_called_once_with((run_from / "fixtures/tests.json").resolve())

    def test_cmd_list_outputs_names_only(self) -> None:
        args = argparse.Namespace(filter=None, tests=str(perf.TESTS_PATH))
        tests = [
            self._sample_test("alpha"),
            self._sample_test("beta"),
        ]
        stdout = io.StringIO()
        with mock.patch.object(perf, "load_tests", return_value=tests), mock.patch(
            "sys.stdout",
            stdout,
        ):
            exit_code = perf.cmd_list(args)

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue(), "alpha\nbeta\n")

    def test_render_report_omits_run_directory_line(self) -> None:
        report = perf.render_report(
            pathlib.Path("/tmp/example-run"),
            {},
            {},
            {},
            None,
            {},
            {},
        )
        self.assertNotIn("- Run directory:", report)

    def test_cmd_run_non_verbose_is_quiet_with_concise_outcome(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir) / "run"
            run_dir.mkdir()
            args = argparse.Namespace(
                filter=None,
                run_dir=str(run_dir),
                out_dir=str(run_dir.parent),
                compare=None,
                missing_only=False,
                wavepeek_timeout_seconds=300,
                tests=str(perf.TESTS_PATH),
                verbose=False,
            )
            tests = [self._sample_test("sample")]
            stdout = io.StringIO()
            with (
                mock.patch.object(perf, "load_tests", return_value=tests),
                mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                mock.patch.object(perf, "ensure_hyperfine"),
                mock.patch.object(perf, "resolve_wavepeek_bin", return_value="wavepeek"),
                mock.patch.object(perf, "run_test"),
                mock.patch.object(
                    perf,
                    "run_functional_capture",
                    return_value={"data": [], "warnings": []},
                ),
                mock.patch.object(perf, "write_wavepeek_artifact"),
                mock.patch.object(perf, "write_report", return_value=run_dir / "README.md"),
                mock.patch("sys.stdout", stdout),
            ):
                exit_code = perf.cmd_run(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("use --verbose for detailed logs", stdout.getvalue())
        self.assertNotIn("[1/1]", stdout.getvalue())
        self.assertNotIn("info: run directory:", stdout.getvalue())

    def test_cmd_run_verbose_includes_progress_logs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir) / "run"
            run_dir.mkdir()
            args = argparse.Namespace(
                filter=None,
                run_dir=str(run_dir),
                out_dir=str(run_dir.parent),
                compare=None,
                missing_only=False,
                wavepeek_timeout_seconds=300,
                tests=str(perf.TESTS_PATH),
                verbose=True,
            )
            tests = [self._sample_test("sample")]
            stdout = io.StringIO()
            with (
                mock.patch.object(perf, "load_tests", return_value=tests),
                mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                mock.patch.object(perf, "ensure_hyperfine"),
                mock.patch.object(perf, "resolve_wavepeek_bin", return_value="wavepeek"),
                mock.patch.object(perf, "run_test"),
                mock.patch.object(
                    perf,
                    "run_functional_capture",
                    return_value={"data": [], "warnings": []},
                ),
                mock.patch.object(perf, "write_wavepeek_artifact"),
                mock.patch.object(perf, "write_report", return_value=run_dir / "README.md"),
                mock.patch("sys.stdout", stdout),
            ):
                exit_code = perf.cmd_run(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("[1/1] sample", stdout.getvalue())
        self.assertIn("info: run directory:", stdout.getvalue())

    def test_cmd_compare_non_verbose_emits_concise_failure_only(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 2.0)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 1.0)
            (revised / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                verbose=False,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("use --verbose for detailed logs", stderr.getvalue())
        self.assertNotIn("sample: mean revised=", stderr.getvalue())

    def test_cmd_compare_verbose_includes_detailed_failures(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 2.0)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 1.0)
            (revised / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("sample: mean revised=2.000000s", stderr.getvalue())

    def test_cmd_compare_non_verbose_success_has_concise_ok_line(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 1.0)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 1.0)
            (revised / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=5.0,
                verbose=False,
            )
            stdout = io.StringIO()
            with mock.patch("sys.stdout", stdout):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("use --verbose for detailed logs", stdout.getvalue())

    def test_build_functional_command_appends_json_once(self) -> None:
        command = ["wavepeek", "info", "--waves", "/tmp/a.fst"]
        self.assertEqual(
            perf.build_functional_command(command),
            ["wavepeek", "info", "--waves", "/tmp/a.fst", "--json"],
        )
        self.assertEqual(
            perf.build_functional_command(["wavepeek", "info", "--json"]),
            ["wavepeek", "info", "--json"],
        )

    def test_run_functional_capture_passes_timeout_to_subprocess(self) -> None:
        test = {
            "name": "sample",
            "command": ["{wavepeek_bin}", "change", "--waves", "/tmp/a.fst"],
        }
        with mock.patch.object(perf.subprocess, "run") as run_mock:
            run_mock.return_value = mock.Mock(
                returncode=0,
                stdout=json.dumps({"data": [], "warnings": []}),
                stderr="",
            )
            perf.run_functional_capture(test, "wavepeek", "run", 123)

        self.assertEqual(run_mock.call_count, 1)
        self.assertEqual(run_mock.call_args.kwargs["timeout"], 123)

    def test_functional_diff_fields(self) -> None:
        baseline = {"data": [1], "warnings": ["w1"]}
        self.assertEqual(perf.functional_diff_fields(baseline, baseline), [])
        self.assertEqual(
            perf.functional_diff_fields(
                {"data": [2], "warnings": ["w1"]},
                baseline,
            ),
            ["data"],
        )
        self.assertEqual(
            perf.functional_diff_fields(
                {"data": [1], "warnings": ["w2"]},
                baseline,
            ),
            [],
        )
        self.assertEqual(perf.functional_diff_fields({}, baseline), [])
        self.assertEqual(perf.functional_diff_fields(baseline, {}), [])

    def test_load_wavepeek_artifact_for_compare_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            payload, error = perf.load_wavepeek_artifact_for_compare(
                pathlib.Path(temp_dir),
                "missing",
                "revised",
            )
        self.assertIsNone(payload)
        self.assertIsNotNone(error)
        assert error is not None
        self.assertIn("revised: missing artifact", error)

    def test_load_wavepeek_artifact_for_compare_invalid_json(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text("not-json", encoding="utf-8")
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "golden",
            )

        self.assertIsNone(payload)
        self.assertIsNotNone(error)
        assert error is not None
        self.assertIn("golden: invalid JSON", error)

    def test_parse_wavepeek_result_file_accepts_timeout_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text("{}\n", encoding="utf-8")
            payload = perf.parse_wavepeek_result_file(artifact)
        self.assertEqual(payload, {})

    def test_load_wavepeek_artifact_for_compare_accepts_timeout_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text("{}\n", encoding="utf-8")
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "revised",
            )
        self.assertEqual(payload, {})
        self.assertIsNone(error)

    def test_load_wavepeek_artifact_for_compare_missing_keys(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(json.dumps({"data": []}), encoding="utf-8")
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "golden",
            )

        self.assertIsNone(payload)
        self.assertIsNotNone(error)
        assert error is not None
        self.assertIn("missing key `warnings`", error)

    def test_load_wavepeek_artifact_for_compare_accepts_object_data(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": {"time_unit": "ps"}, "warnings": []}),
                encoding="utf-8",
            )
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "revised",
            )

        self.assertIsNone(error)
        self.assertEqual(payload, {"data": {"time_unit": "ps"}, "warnings": []})

    def test_load_wavepeek_artifact_for_compare_rejects_scalar_data(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": "oops", "warnings": []}),
                encoding="utf-8",
            )
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "revised",
            )

        self.assertIsNone(payload)
        self.assertIsNotNone(error)
        assert error is not None
        self.assertIn("field `data` must be object or list", error)

    def test_load_wavepeek_artifact_for_compare_invalid_warnings_type(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": [], "warnings": "not-a-list"}),
                encoding="utf-8",
            )
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "golden",
            )

        self.assertIsNone(payload)
        self.assertIsNotNone(error)
        assert error is not None
        self.assertIn("field `warnings` must be list", error)

    def test_load_wavepeek_artifact_for_compare_valid_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps(
                    {
                        "data": [{"id": 1}],
                        "warnings": ["w"],
                        "schema": "ignored",
                    }
                ),
                encoding="utf-8",
            )
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "revised",
            )

        self.assertIsNone(error)
        self.assertEqual(payload, {"data": [{"id": 1}], "warnings": ["w"]})

    def test_report_functional_status_missing_counterpart(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [1], "warnings": []}},
                {},
            ),
            perf.FUNCTIONAL_MISSING_MARKER,
        )

    def test_report_functional_status_data_mismatch(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [1], "warnings": ["a"]}},
                {"t": {"data": [2], "warnings": ["b"]}},
            ),
            f"{perf.FUNCTIONAL_MISMATCH_MARKER}D",
        )

    def test_report_functional_status_empty_data_match(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [], "warnings": ["a"]}},
                {"t": {"data": [], "warnings": ["b"]}},
            ),
            f"{perf.FUNCTIONAL_MATCH_MARKER}E",
        )

    def test_report_functional_status_empty_object_data_match(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": {}, "warnings": ["a"]}},
                {"t": {"data": {}, "warnings": ["b"]}},
            ),
            f"{perf.FUNCTIONAL_MATCH_MARKER}E",
        )

    def test_report_functional_status_nonempty_data_match(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [{"id": 1}], "warnings": ["a"]}},
                {"t": {"data": [{"id": 1}], "warnings": ["b"]}},
            ),
            perf.FUNCTIONAL_MATCH_MARKER,
        )

    def test_report_functional_status_timeout_artifact(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {}},
                {"t": {"data": [{"id": 1}], "warnings": []}},
            ),
            perf.FUNCTIONAL_TIMEOUT_MARKER,
        )

    def test_format_metric_includes_speed_factor_when_faster(self) -> None:
        rendered = perf.format_metric(1.0, {"mean": 2.0}, "mean")
        self.assertEqual(rendered, "1.000000 (+50.00%, 2.00x faster) 🟢")

    def test_format_metric_includes_speed_factor_when_slower(self) -> None:
        rendered = perf.format_metric(3.0, {"mean": 2.0}, "mean")
        self.assertEqual(rendered, "3.000000 (-50.00%, 1.50x slower) 🔴")

    def test_format_metric_includes_speed_factor_when_equal(self) -> None:
        rendered = perf.format_metric(2.0, {"mean": 2.0}, "mean")
        self.assertEqual(rendered, "2.000000 (+0.00%, 1.00x)")

    def test_cmd_compare_reports_speed_factor_on_regression(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 2.0)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 1.0)
            (revised / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("speed=2.00x slower", stderr.getvalue())

    def test_cmd_compare_timeout_artifact_is_non_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 1.0)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 1.0)

            (revised / "sample.wavepeek.json").write_text("{}\n", encoding="utf-8")
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "warnings": []}),
                encoding="utf-8",
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=5.0,
            )
            exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 0)


if __name__ == "__main__":
    unittest.main()
