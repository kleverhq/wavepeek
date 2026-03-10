import importlib.util
import io
import json
import pathlib
import tempfile
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location(
    "bench_expr_compare",
    pathlib.Path(__file__).with_name("compare.py"),
)
assert SPEC is not None
assert SPEC.loader is not None
compare = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(compare)


class CompareHelpersTest(unittest.TestCase):
    @staticmethod
    def _write_summary(run_dir: pathlib.Path, scenarios: dict[str, tuple[float, float]]) -> None:
        run_dir.mkdir(parents=True, exist_ok=True)
        payload = {
            "scenarios": [
                {
                    "scenario": name,
                    "mean_ns_per_iter": mean,
                    "median_ns_per_iter": median,
                }
                for name, (mean, median) in sorted(scenarios.items())
            ]
        }
        (run_dir / "summary.json").write_text(
            json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def test_main_succeeds_when_all_scenarios_within_threshold(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            scenarios = {
                "tokenize_union_iff": (100.0, 100.0),
                "parse_event_union_iff": (200.0, 200.0),
                "parse_event_malformed": (300.0, 300.0),
            }
            self._write_summary(revised, scenarios)
            self._write_summary(golden, scenarios)

            stdout = io.StringIO()
            with mock.patch("sys.stdout", stdout):
                exit_code = compare.main(
                    [
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "5",
                    ]
                )

        self.assertEqual(exit_code, 0)
        self.assertIn(
            "ok: no matched scenario exceeded 5.00% negative delta in mean or median",
            stdout.getvalue(),
        )

    def test_main_fails_when_scenario_set_differs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            self._write_summary(
                revised,
                {
                    "tokenize_union_iff": (100.0, 100.0),
                },
            )
            self._write_summary(
                golden,
                {
                    "tokenize_union_iff": (100.0, 100.0),
                    "parse_event_union_iff": (200.0, 200.0),
                },
            )

            with self.assertRaises(SystemExit) as error:
                compare.main(
                    [
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "5",
                    ]
                )

        self.assertIn("scenario set mismatch", str(error.exception))

    def test_main_fails_when_negative_delta_exceeds_threshold(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            self._write_summary(
                revised,
                {
                    "tokenize_union_iff": (120.0, 100.0),
                    "parse_event_union_iff": (200.0, 200.0),
                    "parse_event_malformed": (300.0, 300.0),
                },
            )
            self._write_summary(
                golden,
                {
                    "tokenize_union_iff": (100.0, 100.0),
                    "parse_event_union_iff": (200.0, 200.0),
                    "parse_event_malformed": (300.0, 300.0),
                },
            )

            with self.assertRaises(SystemExit) as error:
                compare.main(
                    [
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "5",
                    ]
                )

        self.assertIn("exceeded allowed negative delta", str(error.exception))

    def test_load_summary_rejects_duplicate_scenarios(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir)
            payload = {
                "scenarios": [
                    {
                        "scenario": "tokenize_union_iff",
                        "mean_ns_per_iter": 100.0,
                        "median_ns_per_iter": 100.0,
                    },
                    {
                        "scenario": "tokenize_union_iff",
                        "mean_ns_per_iter": 101.0,
                        "median_ns_per_iter": 101.0,
                    },
                ]
            }
            (run_dir / "summary.json").write_text(
                json.dumps(payload),
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit) as error:
                compare.load_summary(run_dir)

        self.assertIn("duplicate scenario", str(error.exception))

    def test_load_summary_rejects_non_finite_metrics(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir)
            payload = {
                "scenarios": [
                    {
                        "scenario": "tokenize_union_iff",
                        "mean_ns_per_iter": "nan",
                        "median_ns_per_iter": 100.0,
                    }
                ]
            }
            (run_dir / "summary.json").write_text(
                json.dumps(payload),
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit) as error:
                compare.load_summary(run_dir)

        self.assertIn("non-finite", str(error.exception))


if __name__ == "__main__":
    unittest.main()
