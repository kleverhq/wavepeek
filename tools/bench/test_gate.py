from __future__ import annotations

import argparse
import pathlib
import sys
import tempfile
import types
import unittest
from unittest import mock

BENCH_DIR = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(BENCH_DIR))

import capture  # noqa: E402
import common  # noqa: E402
import compare  # noqa: E402
import gate  # noqa: E402


class BenchGateHelperTest(unittest.TestCase):
    def test_parser_defaults(self) -> None:
        capture_args = capture.build_parser().parse_args([])
        self.assertEqual(capture_args.ref, "HEAD")
        self.assertEqual(capture_args.fsdb, "auto")
        self.assertFalse(hasattr(capture_args, "allow_dirty_source"))

        compare_args = compare.build_parser().parse_args(
            ["--golden", "golden", "--revised", "revised"]
        )
        self.assertEqual(compare_args.max_negative_delta_pct, common.DEFAULT_TIMING_THRESHOLD_PCT)
        self.assertEqual(compare_args.max_negative_delta_seconds, common.DEFAULT_TIMING_THRESHOLD_SECONDS)

        gate_args = gate.build_parser().parse_args(["--baseline-ref", "v0.1.0"])
        self.assertEqual(gate_args.revised_ref, "HEAD")
        self.assertEqual(gate_args.fsdb, "auto")
        self.assertEqual(gate_args.max_negative_delta_pct, common.DEFAULT_TIMING_THRESHOLD_PCT)
        self.assertEqual(gate_args.max_negative_delta_seconds, common.DEFAULT_TIMING_THRESHOLD_SECONDS)
        self.assertFalse(hasattr(gate_args, "allow_dirty_source"))

    def test_ensure_empty_dir_rejects_non_empty_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = pathlib.Path(temp_dir) / "out"
            path.mkdir()
            (path / "keep.txt").write_text("data", encoding="utf-8")

            with self.assertRaises(common.BenchGateError):
                common.ensure_empty_dir(path)

    def test_ensure_empty_dir_rejects_file_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = pathlib.Path(temp_dir) / "out"
            path.write_text("data", encoding="utf-8")

            with self.assertRaises(common.BenchGateError):
                common.ensure_empty_dir(path)

    def test_dirty_head_refs_fail_without_override(self) -> None:
        source = pathlib.Path("/repo")
        with mock.patch.object(common, "worktree_status", return_value=" M src/main.rs"):
            with self.assertRaises(common.BenchGateError):
                common.enforce_clean_source_for_head_refs(source, ["HEAD"])

        with mock.patch.object(common, "worktree_status") as status:
            common.enforce_clean_source_for_head_refs(source, ["v1.0.0"])
        status.assert_not_called()

    def test_clean_worktree_required_for_current_tooling(self) -> None:
        source = pathlib.Path("/repo")
        with mock.patch.object(common, "worktree_status", return_value=" M bench/e2e/perf.py"):
            with self.assertRaisesRegex(common.BenchGateError, "current benchmark tooling"):
                common.enforce_clean_worktree(
                    source,
                    reason="current benchmark tooling must be committed before running the gate",
                )

    def test_assess_fsdb_auto_skips_when_support_files_are_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            checkout = pathlib.Path(temp_dir) / "checkout"
            tooling = pathlib.Path(temp_dir) / "tooling"
            checkout.mkdir()
            tooling.mkdir()
            (checkout / "Cargo.toml").write_text('[features]\nfsdb = []\n', encoding="utf-8")
            plan = capture.assess_fsdb(
                checkout,
                tooling_root=tooling,
                log_path=tooling / "fsdb.log",
                mode="auto",
            )

        self.assertFalse(plan.capture)
        self.assertEqual(plan.status, "unsupported")
        self.assertIn("FSDB support files missing", plan.reason or "")

    def test_gate_fsdb_plan_fails_asymmetric_support(self) -> None:
        baseline = common.FsdbPlan(capture=True, status="available")
        revised = common.FsdbPlan(
            capture=False,
            status="unsupported",
            reason="missing tests_fsdb.json",
        )
        with self.assertRaises(common.BenchGateError):
            capture.resolve_gate_fsdb_plan(baseline, revised, mode="auto")

    def test_fsdb_catalog_filter_skips_vcd_style_scalar_elements(self) -> None:
        supported = {"command": ["wavepeek", "value", "--signals", "top.bus"]}
        unsupported = {"command": ["wavepeek", "value", "--signals", "top.mem.[0]"]}

        self.assertTrue(capture.fsdb_catalog_test_is_supported(supported))
        self.assertFalse(capture.fsdb_catalog_test_is_supported(unsupported))

    def test_write_runnable_fsdb_catalog_records_skipped_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            checkout = root / "checkout"
            catalog_dir = checkout / "bench/e2e"
            catalog_dir.mkdir(parents=True)
            (catalog_dir / "tests_fsdb.json").write_text(
                '{"tests":[{"name":"ok","command":["wavepeek","info"]},'
                '{"name":"skip","command":["wavepeek","value","top.mem.[0]"]}]}\n',
                encoding="utf-8",
            )

            output, skipped = capture.write_runnable_fsdb_catalog(
                tooling_root=checkout,
                capture_dir=root / "capture",
            )
            payload = common.read_json(output)

        self.assertEqual(skipped, ["skip"])
        self.assertEqual([test["name"] for test in payload["tests"]], ["ok"])

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

            with self.assertRaises(common.BenchGateError):
                compare.assert_matching_e2e_artifacts(golden, revised)

    def test_compare_command_requires_clean_current_tooling(self) -> None:
        args = argparse.Namespace(
            golden=pathlib.Path("/golden"),
            revised=pathlib.Path("/revised"),
            out_dir=None,
            source_root=pathlib.Path("/repo"),
            max_negative_delta_pct=5.0,
            max_negative_delta_seconds=0.005,
        )
        with mock.patch.object(
            compare,
            "enforce_clean_worktree",
            side_effect=common.BenchGateError("dirty tooling"),
        ):
            with self.assertRaisesRegex(common.BenchGateError, "dirty tooling"):
                compare.compare_command(args)

    def test_compare_captures_fails_when_required_suites_are_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            golden.mkdir()
            revised.mkdir()

            result = compare.compare_captures(
                golden_dir=golden,
                revised_dir=revised,
                compare_dir=root / "compare",
                timing_threshold_pct=5.0,
            )

        self.assertEqual(result.exit_code, 1)
        self.assertEqual(result.manifest["suites"]["e2e-fst"]["status"], "failed")
        self.assertNotIn("expr", result.manifest["suites"])
        self.assertEqual(result.manifest["suites"]["e2e-fsdb"]["status"], "skipped")

    def test_compare_captures_runs_same_format_timing_and_cross_functional(self) -> None:
        calls: list[tuple[str, list[str]]] = []

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> common.CommandResult:
            calls.append((name, list(args)))
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            return common.CommandResult(
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
                for suite in ("e2e-fst", "e2e-fsdb"):
                    (base / suite).mkdir(parents=True)
                for suite in ("e2e-fst", "e2e-fsdb"):
                    for suffix in ("hyperfine", "wavepeek"):
                        (base / suite / f"case.{suffix}.json").write_text(
                            "{}", encoding="utf-8"
                        )

            with mock.patch.object(compare, "run_command", side_effect=fake_run_command):
                result = compare.compare_captures(
                    golden_dir=golden,
                    revised_dir=revised,
                    compare_dir=root / "compare",
                    timing_threshold_pct=5.0,
                )

        self.assertEqual(result.exit_code, 0)
        names = [name for name, _ in calls]
        self.assertEqual(
            names,
            [
                "compare-e2e-fst",
                "compare-e2e-fsdb",
                "compare-cross-golden-fst-fsdb",
                "compare-cross-revised-fst-fsdb",
            ],
        )
        same_format = {name: args for name, args in calls[:2]}
        self.assertIn("--max-negative-delta-pct", same_format["compare-e2e-fst"])
        self.assertIn("--max-negative-delta-seconds", same_format["compare-e2e-fst"])
        self.assertIn("--max-negative-delta-pct", same_format["compare-e2e-fsdb"])
        self.assertIn("--max-negative-delta-seconds", same_format["compare-e2e-fsdb"])
        for name, args in calls[2:]:
            self.assertIn("--functional-only", args, name)
            self.assertIn("--allow-golden-extra", args, name)
            self.assertIn("--ignore-functional-test", args, name)
            self.assertNotIn("--max-negative-delta-pct", args, name)
        self.assertFalse(result.manifest["suites"]["e2e-fst"]["functional_only"])
        self.assertFalse(result.manifest["suites"]["e2e-fsdb"]["functional_only"])
        cross_suite = result.manifest["suites"]["cross-golden-fst-fsdb"]
        self.assertTrue(cross_suite["functional_only"])
        self.assertEqual(len(cross_suite["ignored_functional_tests"]), 5)
        self.assertEqual(
            cross_suite["ignored_functional_tests"][0]["test_name"],
            "scope_clustered_all_depth13_json",
        )

    def test_compare_captures_confirms_timing_only_failures_with_best_samples(self) -> None:
        calls: list[str] = []

        def arg_value(args: list[str], flag: str) -> pathlib.Path:
            return pathlib.Path(args[args.index(flag) + 1])

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> common.CommandResult:
            calls.append(name)
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            if name == "compare-e2e-fst":
                common.write_json(
                    arg_value(args, "--result-json"),
                    {
                        "status": "failed",
                        "timing_failures": [{"test_name": "case"}],
                        "functional_mismatches": [],
                        "functional_artifact_errors": [],
                    },
                )
                return common.CommandResult(name=name, args=list(args), cwd=str(cwd), returncode=1, log_path=str(log_path))
            if name == "confirm-e2e-fst-best":
                common.write_json(
                    arg_value(args, "--result-json"),
                    {"status": "passed", "failures": []},
                )
                return common.CommandResult(name=name, args=list(args), cwd=str(cwd), returncode=0, log_path=str(log_path))
            return common.CommandResult(name=name, args=list(args), cwd=str(cwd), returncode=0, log_path=str(log_path))

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            for base in (golden, revised):
                (base / "e2e-fst").mkdir(parents=True)
                for suffix in ("hyperfine", "wavepeek"):
                    (base / "e2e-fst" / f"case.{suffix}.json").write_text("{}", encoding="utf-8")

            with mock.patch.object(compare, "run_command", side_effect=fake_run_command):
                result = compare.compare_captures(
                    golden_dir=golden,
                    revised_dir=revised,
                    compare_dir=root / "compare",
                    timing_threshold_pct=5.0,
                )

        self.assertEqual(result.exit_code, 0)
        self.assertEqual(calls, ["compare-e2e-fst", "confirm-e2e-fst-best"])
        suite = result.manifest["suites"]["e2e-fst"]
        self.assertEqual(suite["status"], "passed")
        self.assertEqual(suite["median_compare_status"], "failed")
        self.assertEqual(suite["timing_confirm"]["status"], "passed")

    def test_compare_captures_does_not_confirm_functional_failures(self) -> None:
        calls: list[str] = []

        def arg_value(args: list[str], flag: str) -> pathlib.Path:
            return pathlib.Path(args[args.index(flag) + 1])

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> common.CommandResult:
            calls.append(name)
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            common.write_json(
                arg_value(args, "--result-json"),
                {
                    "status": "failed",
                    "timing_failures": [{"test_name": "case"}],
                    "functional_mismatches": ["case: mismatched fields data"],
                    "functional_artifact_errors": [],
                },
            )
            return common.CommandResult(name=name, args=list(args), cwd=str(cwd), returncode=1, log_path=str(log_path))

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            for base in (golden, revised):
                (base / "e2e-fst").mkdir(parents=True)
                for suffix in ("hyperfine", "wavepeek"):
                    (base / "e2e-fst" / f"case.{suffix}.json").write_text("{}", encoding="utf-8")

            with mock.patch.object(compare, "run_command", side_effect=fake_run_command):
                result = compare.compare_captures(
                    golden_dir=golden,
                    revised_dir=revised,
                    compare_dir=root / "compare",
                    timing_threshold_pct=5.0,
                )

        self.assertEqual(result.exit_code, 1)
        self.assertEqual(calls, ["compare-e2e-fst"])
        self.assertNotIn("timing_confirm", result.manifest["suites"]["e2e-fst"])

    def test_compare_captures_does_not_confirm_functional_timeout_warnings(self) -> None:
        calls: list[str] = []

        def arg_value(args: list[str], flag: str) -> pathlib.Path:
            return pathlib.Path(args[args.index(flag) + 1])

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> common.CommandResult:
            calls.append(name)
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            common.write_json(
                arg_value(args, "--result-json"),
                {
                    "status": "failed",
                    "timing_failures": [{"test_name": "case"}],
                    "functional_timeout_warnings": ["case: timeout artifact on revised"],
                    "functional_mismatches": [],
                    "functional_artifact_errors": [],
                },
            )
            return common.CommandResult(name=name, args=list(args), cwd=str(cwd), returncode=1, log_path=str(log_path))

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            for base in (golden, revised):
                (base / "e2e-fst").mkdir(parents=True)
                for suffix in ("hyperfine", "wavepeek"):
                    (base / "e2e-fst" / f"case.{suffix}.json").write_text("{}", encoding="utf-8")

            with mock.patch.object(compare, "run_command", side_effect=fake_run_command):
                result = compare.compare_captures(
                    golden_dir=golden,
                    revised_dir=revised,
                    compare_dir=root / "compare",
                    timing_threshold_pct=5.0,
                )

        self.assertEqual(result.exit_code, 1)
        self.assertEqual(calls, ["compare-e2e-fst"])
        self.assertNotIn("timing_confirm", result.manifest["suites"]["e2e-fst"])

    def test_compare_captures_uses_manifest_suite_paths(self) -> None:
        calls: list[tuple[pathlib.Path, pathlib.Path]] = []

        def fake_run_e2e_compare(**kwargs: object) -> dict[str, object]:
            calls.append((pathlib.Path(str(kwargs["golden"])), pathlib.Path(str(kwargs["revised"]))))
            return {"status": "passed", "functional_only": False}

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            (root / "shared/fst/base").mkdir(parents=True)
            (root / "shared/fst/rev").mkdir(parents=True)
            golden.mkdir()
            revised.mkdir()
            common.write_json(
                golden / "manifest.json",
                {"suites": {"e2e-fst": {"path": "../shared/fst/base"}}},
            )
            common.write_json(
                revised / "manifest.json",
                {"suites": {"e2e-fst": {"path": "../shared/fst/rev"}}},
            )

            with mock.patch.object(compare, "run_e2e_compare", side_effect=fake_run_e2e_compare):
                result = compare.compare_captures(
                    golden_dir=golden,
                    revised_dir=revised,
                    compare_dir=root / "compare",
                    timing_threshold_pct=5.0,
                )

        self.assertEqual(result.exit_code, 0)
        self.assertEqual(calls, [(root / "shared/fst/base", root / "shared/fst/rev")])

    def test_compare_captures_skips_optional_fsdb_and_cross_checks_when_missing(self) -> None:
        calls: list[str] = []

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> common.CommandResult:
            calls.append(name)
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            return common.CommandResult(name=name, args=list(args), cwd=str(cwd), returncode=0, log_path=str(log_path))

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            golden = root / "golden"
            revised = root / "revised"
            for base in (golden, revised):
                (base / "e2e-fst").mkdir(parents=True)
                for suffix in ("hyperfine", "wavepeek"):
                    (base / "e2e-fst" / f"case.{suffix}.json").write_text(
                        "{}", encoding="utf-8"
                    )

            with mock.patch.object(compare, "run_command", side_effect=fake_run_command):
                result = compare.compare_captures(
                    golden_dir=golden,
                    revised_dir=revised,
                    compare_dir=root / "compare",
                    timing_threshold_pct=5.0,
                )

        self.assertEqual(result.exit_code, 0)
        self.assertEqual(calls, ["compare-e2e-fst"])
        self.assertEqual(result.manifest["suites"]["e2e-fsdb"]["status"], "skipped")
        self.assertEqual(result.manifest["suites"]["cross-golden-fst-fsdb"]["status"], "skipped")

    def test_capture_checkout_uses_current_tooling_with_selected_binary(self) -> None:
        calls: list[tuple[str, list[str], pathlib.Path, dict[str, str] | None]] = []

        def fake_run_command(
            name: str,
            args: list[str],
            *,
            cwd: pathlib.Path,
            log_path: pathlib.Path,
            env: dict[str, str] | None = None,
            check: bool = True,
        ) -> common.CommandResult:
            calls.append((name, list(args), cwd, dict(env) if env else None))
            log_path.parent.mkdir(parents=True, exist_ok=True)
            log_path.write_text("ok\n", encoding="utf-8")
            return common.CommandResult(
                name=name,
                args=list(args),
                cwd=str(cwd),
                returncode=0,
                log_path=str(log_path),
            )

        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            checkout = root / "checkout"
            tooling = root / "tooling"
            checkout.mkdir()
            tooling.mkdir()
            with (
                mock.patch.object(capture, "run_command", side_effect=fake_run_command),
                mock.patch.object(capture, "tool_version", return_value="tool 1.0"),
            ):
                result = capture.capture_checkout(
                    tooling_root=tooling,
                    tooling_sha="toolsha",
                    binary_checkout=checkout,
                    capture_dir=root / "capture",
                    source_ref="HEAD",
                    source_sha="abc123",
                    fsdb_mode="auto",
                    fsdb_plan=common.FsdbPlan(
                        capture=False,
                        status="skipped",
                        reason="test skip",
                    ),
                    environment_note="test env",
                )

        self.assertEqual(result.manifest["suites"]["e2e-fst"]["status"], "passed")
        self.assertEqual(result.manifest["binary_sha"], "abc123")
        self.assertEqual(result.manifest["tooling_sha"], "toolsha")
        self.assertEqual([call[2] for call in calls], [checkout, tooling])
        e2e_call = next(call for call in calls if call[0] == "bench-e2e-fst")
        self.assertIn("bench/e2e/perf.py", e2e_call[1])
        self.assertEqual(e2e_call[2], tooling)
        self.assertIsNone(e2e_call[3])
        self.assertIn("--binary", e2e_call[1])
        self.assertIn(f"subject={checkout / 'target/release/wavepeek'}", e2e_call[1])

    def test_gate_runs_preparation_before_paired_measurement(self) -> None:
        order: list[str] = []

        def fake_init_capture_session(**kwargs: object) -> types.SimpleNamespace:
            label = pathlib.Path(str(kwargs["capture_dir"])).name
            order.append(f"init-{label}")
            return types.SimpleNamespace(label=label)

        def record(name: str):
            def inner(session: types.SimpleNamespace) -> None:
                order.append(f"{name}-{session.label}")
            return inner

        def fake_compare_captures(**kwargs: object) -> common.CompareResult:
            order.append("compare")
            compare_dir = pathlib.Path(str(kwargs["compare_dir"]))
            compare_dir.mkdir(parents=True)
            return common.CompareResult(compare_dir=compare_dir, manifest={"status": "passed"}, exit_code=0)

        with tempfile.TemporaryDirectory() as temp_dir:
            out_dir = pathlib.Path(temp_dir) / "gate"
            args = argparse.Namespace(
                source_root=pathlib.Path("/repo"),
                baseline_ref="v1.0.0",
                revised_ref="v1.0.1",
                out_dir=out_dir,
                fsdb="auto",
                max_negative_delta_pct=5.0,
                max_negative_delta_seconds=0.005,
                environment_note="test env",
            )
            with (
                mock.patch.object(gate, "resolve_ref", side_effect=["aaa", "bbb", "toolsha"]),
                mock.patch.object(gate, "enforce_clean_worktree"),
                mock.patch.object(gate, "clone_checkout", side_effect=lambda *a, **k: order.append("clone")),
                mock.patch.object(gate, "assess_fsdb", return_value=common.FsdbPlan(capture=True, status="available")),
                mock.patch.object(gate, "resolve_gate_fsdb_plan", return_value=common.FsdbPlan(capture=True, status="available")),
                mock.patch.object(gate, "init_capture_session", side_effect=fake_init_capture_session),
                mock.patch.object(gate, "build_release", side_effect=record("build")),
                mock.patch.object(gate, "build_release_fsdb", side_effect=record("build-fsdb")),
                mock.patch.object(gate, "prepare_fsdb", side_effect=record("prepare-fsdb")),
                mock.patch.object(gate, "write_fsdb_capture_catalog", side_effect=record("fsdb-catalog")),
                mock.patch.object(gate, "run_e2e_fst_many", side_effect=lambda *a, **k: order.append("e2e-fst-many")),
                mock.patch.object(gate, "run_e2e_fsdb_many", side_effect=lambda *a, **k: order.append("e2e-fsdb-many")),
                mock.patch.object(gate, "finalize_capture", side_effect=record("finalize")),
                mock.patch.object(gate, "compare_captures", side_effect=fake_compare_captures),
            ):
                exit_code = gate.gate_command(args)

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            order,
            [
                "clone",
                "clone",
                "init-baseline",
                "init-revised",
                "build-baseline",
                "build-revised",
                "build-fsdb-baseline",
                "build-fsdb-revised",
                "prepare-fsdb-baseline",
                "fsdb-catalog-revised",
                "e2e-fst-many",
                "e2e-fsdb-many",
                "finalize-baseline",
                "finalize-revised",
                "compare",
            ],
        )


if __name__ == "__main__":
    unittest.main()
