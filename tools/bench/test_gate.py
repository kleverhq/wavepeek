from __future__ import annotations

import argparse
import importlib.util
import pathlib
import sys
import tempfile
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location(
    "bench_gate",
    pathlib.Path(__file__).with_name("gate.py"),
)
assert SPEC is not None
assert SPEC.loader is not None
gate = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = gate
SPEC.loader.exec_module(gate)


class GateHelperTest(unittest.TestCase):
    def test_parser_defaults(self) -> None:
        parser = gate.build_parser()
        capture = parser.parse_args(["capture"])
        self.assertEqual(capture.ref, "HEAD")
        self.assertEqual(capture.fsdb, "auto")
        self.assertFalse(capture.allow_dirty_source)

        compare = parser.parse_args(
            ["compare", "--golden", "golden", "--revised", "revised"]
        )
        self.assertEqual(compare.e2e_threshold_pct, gate.DEFAULT_E2E_THRESHOLD_PCT)
        self.assertEqual(compare.fsdb_threshold_pct, gate.DEFAULT_E2E_THRESHOLD_PCT)
        self.assertEqual(compare.expr_threshold_pct, gate.DEFAULT_EXPR_THRESHOLD_PCT)

        full_gate = parser.parse_args(["gate", "--baseline-ref", "v0.1.0"])
        self.assertEqual(full_gate.revised_ref, "HEAD")
        self.assertEqual(full_gate.fsdb, "auto")

    def test_ensure_empty_dir_rejects_non_empty_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = pathlib.Path(temp_dir) / "out"
            path.mkdir()
            (path / "keep.txt").write_text("data", encoding="utf-8")

            with self.assertRaises(gate.BenchGateError):
                gate.ensure_empty_dir(path)

    def test_ensure_empty_dir_rejects_file_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = pathlib.Path(temp_dir) / "out"
            path.write_text("data", encoding="utf-8")

            with self.assertRaises(gate.BenchGateError):
                gate.ensure_empty_dir(path)

    def test_dirty_head_refs_require_override(self) -> None:
        source = pathlib.Path("/repo")
        with (
            mock.patch.object(gate, "current_head", return_value="abc123"),
            mock.patch.object(gate, "worktree_status", return_value=" M src/main.rs"),
        ):
            with self.assertRaises(gate.BenchGateError):
                gate.enforce_clean_source_for_head_refs(
                    source,
                    [("HEAD", "abc123")],
                    allow_dirty_source=False,
                )

            gate.enforce_clean_source_for_head_refs(
                source,
                [("HEAD", "abc123")],
                allow_dirty_source=True,
            )

    def test_assess_fsdb_auto_skips_when_support_files_are_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            checkout = pathlib.Path(temp_dir)
            plan = gate.assess_fsdb(
                checkout,
                log_path=checkout / "fsdb.log",
                mode="auto",
            )

        self.assertFalse(plan.capture)
        self.assertEqual(plan.status, "unsupported")
        self.assertIn("FSDB support files missing", plan.reason or "")

    def test_gate_fsdb_plan_fails_asymmetric_support(self) -> None:
        baseline = gate.FsdbPlan(capture=True, status="available")
        revised = gate.FsdbPlan(
            capture=False,
            status="unsupported",
            reason="missing tests_fsdb.json",
        )
        with self.assertRaises(gate.BenchGateError):
            gate.resolve_gate_fsdb_plan(baseline, revised, mode="auto")

    def test_e2e_artifact_identity_rejects_partial_intersection(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            golden.mkdir()
            revised.mkdir()
            for suffix in ("hyperfine", "wavepeek"):
                (golden / f"a.{suffix}.json").write_text("{}", encoding="utf-8")
                (revised / f"b.{suffix}.json").write_text("{}", encoding="utf-8")

            with self.assertRaises(gate.BenchGateError):
                gate.assert_matching_e2e_artifacts(golden, revised)

    def test_compare_captures_fails_when_required_suites_are_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            golden.mkdir()
            revised.mkdir()

            result = gate.compare_captures(
                golden_dir=golden,
                revised_dir=revised,
                compare_dir=root / "compare",
                e2e_threshold_pct=5.0,
                fsdb_threshold_pct=5.0,
                expr_threshold_pct=15.0,
            )

        self.assertEqual(result.exit_code, 1)
        self.assertEqual(result.manifest["suites"]["e2e-fst"]["status"], "failed")
        self.assertEqual(result.manifest["suites"]["expr"]["status"], "failed")
        self.assertEqual(result.manifest["suites"]["e2e-fsdb"]["status"], "skipped")

    def test_compare_captures_aggregates_suites_and_skips_missing_fsdb(self) -> None:
        calls: list[tuple[str, list[str], pathlib.Path]] = []

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> gate.CommandResult:
            calls.append((name, list(args), cwd))
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            return gate.CommandResult(
                name=name,
                args=list(args),
                cwd=str(cwd),
                returncode=0,
                log_path=str(log_path),
            )

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            for base in (golden, revised):
                (base / "e2e-fst").mkdir(parents=True)
                (base / "expr").mkdir(parents=True)
                for suffix in ("hyperfine", "wavepeek"):
                    (base / "e2e-fst" / f"case.{suffix}.json").write_text(
                        "{}", encoding="utf-8"
                    )

            with mock.patch.object(gate, "run_command", side_effect=fake_run_command):
                result = gate.compare_captures(
                    golden_dir=golden,
                    revised_dir=revised,
                    compare_dir=root / "compare",
                    e2e_threshold_pct=5.0,
                    fsdb_threshold_pct=5.0,
                    expr_threshold_pct=15.0,
                )

        self.assertEqual(result.exit_code, 0)
        self.assertEqual([call[0] for call in calls], ["compare-e2e-fst", "compare-expr"])
        self.assertIn("--verbose", calls[0][1])
        self.assertNotIn("--verbose", calls[1][1])
        self.assertEqual(result.manifest["suites"]["e2e-fsdb"]["status"], "skipped")

    def test_capture_checkout_uses_selected_checkout_harnesses(self) -> None:
        calls: list[tuple[str, list[str], pathlib.Path, dict[str, str] | None]] = []

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> gate.CommandResult:
            calls.append((name, list(args), cwd, dict(env) if env else None))
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            return gate.CommandResult(
                name=name,
                args=list(args),
                cwd=str(cwd),
                returncode=0,
                log_path=str(log_path),
            )

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            checkout = root / "checkout"
            checkout.mkdir()
            with (
                mock.patch.object(gate, "run_command", side_effect=fake_run_command),
                mock.patch.object(gate, "tool_version", return_value="tool 1.0"),
            ):
                result = gate.capture_checkout(
                    checkout=checkout,
                    capture_dir=root / "capture",
                    source_ref="HEAD",
                    source_sha="abc123",
                    fsdb_mode="auto",
                    fsdb_plan=gate.FsdbPlan(
                        capture=False,
                        status="skipped",
                        reason="test skip",
                    ),
                    environment_note="test env",
                )

        self.assertEqual(result.manifest["suites"]["e2e-fst"]["status"], "passed")
        self.assertTrue(all(call[2] == checkout for call in calls))
        e2e_call = next(call for call in calls if call[0] == "bench-e2e-fst")
        self.assertIn("bench/e2e/perf.py", e2e_call[1])
        self.assertEqual(
            e2e_call[3],
            {"WAVEPEEK_BIN": str(checkout / "target/release/wavepeek")},
        )


if __name__ == "__main__":
    unittest.main()
