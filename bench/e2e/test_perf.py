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
            ["warnings"],
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


if __name__ == "__main__":
    unittest.main()
