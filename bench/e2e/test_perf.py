import importlib.util
import json
import pathlib
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "bench_e2e_perf",
    pathlib.Path(__file__).with_name("perf.py"),
)
assert SPEC is not None
assert SPEC.loader is not None
perf = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(perf)


class PerfHelpersTest(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
