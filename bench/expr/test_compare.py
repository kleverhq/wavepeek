import importlib.util
import io
import json
import pathlib
import tempfile
import unittest
from typing import Mapping
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
    def _write_summary(
        run_dir: pathlib.Path,
        scenarios: dict[str, tuple[float, float]],
        *,
        bench_target: str = "expr_c1",
        scenario_set_id: str = "c1_parser",
        extra_metadata: Mapping[str, object] | None = None,
    ) -> None:
        run_dir.mkdir(parents=True, exist_ok=True)
        payload = {
            "bench_target": bench_target,
            "scenario_set_id": scenario_set_id,
            "scenario_set_path": f"bench/expr/scenarios/{scenario_set_id}.json",
            "scenarios": [
                {
                    "scenario": name,
                    "mean_ns_per_iter": mean,
                    "median_ns_per_iter": median,
                }
                for name, (mean, median) in sorted(scenarios.items())
            ]
        }
        if extra_metadata:
            payload.update(extra_metadata)
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

    def test_main_fails_when_bench_target_mismatches(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            scenarios = {
                "tokenize_union_iff": (100.0, 100.0),
                "parse_event_union_iff": (200.0, 200.0),
                "parse_event_malformed": (300.0, 300.0),
            }
            self._write_summary(revised, scenarios, bench_target="expr_c2")
            self._write_summary(golden, scenarios, bench_target="expr_c1")

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

        self.assertIn("summary identity mismatch", str(error.exception))

    def test_main_fails_when_scenario_set_id_mismatches(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            scenarios = {
                "tokenize_union_iff": (100.0, 100.0),
                "parse_event_union_iff": (200.0, 200.0),
                "parse_event_malformed": (300.0, 300.0),
            }
            self._write_summary(revised, scenarios, scenario_set_id="c2_event_runtime")
            self._write_summary(golden, scenarios, scenario_set_id="c1_parser")

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

        self.assertIn("summary identity mismatch", str(error.exception))

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

    def test_main_rejects_non_finite_threshold(self) -> None:
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

            with self.assertRaises(SystemExit) as error:
                compare.main(
                    [
                        "--revised",
                        str(revised),
                        "--golden",
                        str(golden),
                        "--max-negative-delta-pct",
                        "nan",
                    ]
                )

        self.assertIn("finite non-negative", str(error.exception))

    def test_main_supports_required_matching_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            scenarios = {
                "tokenize_union_iff": (100.0, 100.0),
                "parse_event_union_iff": (200.0, 200.0),
                "parse_event_malformed": (300.0, 300.0),
            }
            extra = {
                "source_commit": "abc123",
                "worktree_state": "clean",
            }
            self._write_summary(revised, scenarios, extra_metadata=extra)
            self._write_summary(golden, scenarios, extra_metadata=extra)

            exit_code = compare.main(
                [
                    "--revised",
                    str(revised),
                    "--golden",
                    str(golden),
                    "--max-negative-delta-pct",
                    "5",
                    "--require-matching-metadata",
                    "source_commit",
                    "worktree_state",
                ]
            )

        self.assertEqual(exit_code, 0)

    def test_main_fails_when_required_metadata_mismatches(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            scenarios = {
                "tokenize_union_iff": (100.0, 100.0),
                "parse_event_union_iff": (200.0, 200.0),
                "parse_event_malformed": (300.0, 300.0),
            }
            self._write_summary(
                revised,
                scenarios,
                extra_metadata={"source_commit": "abc123"},
            )
            self._write_summary(
                golden,
                scenarios,
                extra_metadata={"source_commit": "def456"},
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
                        "--require-matching-metadata",
                        "source_commit",
                    ]
                )

        self.assertIn("required metadata mismatch", str(error.exception))


if __name__ == "__main__":
    unittest.main()
