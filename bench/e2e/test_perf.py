import argparse
from collections import Counter
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
    def _write_hyperfine_artifact(
        path: pathlib.Path,
        mean: float,
        median: float | None = None,
        times: list[float] | None = None,
    ) -> None:
        metric_median = mean if median is None else median
        sample_times = [metric_median] if times is None else times
        path.write_text(
            json.dumps(
                {
                    "results": [
                        {
                            "command": "wavepeek change --json",
                            "mean": mean,
                            "stddev": 0.0,
                            "median": metric_median,
                            "min": min(sample_times),
                            "max": max(sample_times),
                            "times": sample_times,
                        }
                    ]
                }
            ),
            encoding="utf-8",
        )

    @staticmethod
    def _diag(
        message: str = "careful", code: str = "WPK-W0001"
    ) -> dict[str, str]:
        return {"kind": "warning", "code": code, "message": message}

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

    @staticmethod
    def _write_wavepeek_artifact(
        path: pathlib.Path,
        data: object | None = None,
        diagnostics: object | None = None,
    ) -> None:
        payload = {
            "data": [{"id": 1}] if data is None else data,
            "diagnostics": [] if diagnostics is None else diagnostics,
        }
        path.write_text(json.dumps(payload), encoding="utf-8")

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
                {"name": "failed"},
                {"name": "pending"},
            ]
            run_dir.joinpath("done.hyperfine.json").write_text("{}", encoding="utf-8")
            run_dir.joinpath("done.wavepeek.json").write_text("{}", encoding="utf-8")
            perf.write_failure_artifact(
                run_dir,
                "failed",
                perf.make_failure_artifact(
                    test_name="failed",
                    phase="preflight",
                    command=["wavepeek", "info"],
                    exit_code=2,
                ),
            )
            runnable, skipped = perf.partition_missing_only_tests(selected, run_dir)

        self.assertEqual([str(test["name"]) for test in runnable], ["pending"])
        self.assertEqual(skipped, ["done", "failed"])

    def test_run_parser_missing_only_and_fail_fast_flags(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["run"])
        self.assertFalse(default_args.missing_only)
        self.assertFalse(default_args.fail_fast)
        flag_args = parser.parse_args(["run", "--missing-only", "--fail-fast"])
        self.assertTrue(flag_args.missing_only)
        self.assertTrue(flag_args.fail_fast)

    def test_write_and_load_failure_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            run_dir = pathlib.Path(temp_dir)
            payload = perf.make_failure_artifact(
                test_name="sample",
                phase="preflight",
                command=["wavepeek", "change", "--sample-mode", "native"],
                exit_code=2,
                stderr="unknown option: --sample-mode",
                binary_label="baseline",
            )
            perf.write_failure_artifact(run_dir, "sample", payload)
            loaded = perf.load_failure_results(run_dir)

        self.assertEqual(set(loaded), {"sample"})
        self.assertEqual(loaded["sample"]["kind"], perf.FAILURE_KIND)
        self.assertEqual(loaded["sample"]["phase"], "preflight")
        self.assertEqual(loaded["sample"]["binary_label"], "baseline")
        self.assertIn("--sample-mode", loaded["sample"]["command"])

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
        self.assertEqual(default_args.schedule, "round-robin")

        override_args = parser.parse_args(["run", "--tests", "bench/e2e/tests_commit.json"])
        self.assertEqual(override_args.tests, "bench/e2e/tests_commit.json")

    def test_parse_binary_specs_requires_explicit_labeled_binaries(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            binary = pathlib.Path(temp_dir) / "wavepeek"
            binary.write_text("#!/bin/sh\n", encoding="utf-8")

            specs = perf.parse_binary_specs([f"base={binary}"])

        self.assertEqual(specs, [perf.BinarySpec("base", str(binary.resolve()))])
        with self.assertRaises(SystemExit):
            perf.parse_binary_specs(None)
        with self.assertRaises(SystemExit):
            perf.parse_binary_specs(["missing-label"])
        with self.assertRaises(SystemExit):
            perf.parse_binary_specs([f"README.md={binary}"])

    def test_report_parser_tests_flag_defaults_and_override(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["report", "--run-dir", "bench/e2e/runs/sample"])
        self.assertEqual(default_args.tests, str(perf.TESTS_PATH))

        override_args = parser.parse_args(
            [
                "report",
                "--run-dir",
                "bench/e2e/runs/sample",
                "--tests",
                "bench/e2e/tests_fsdb.json",
            ]
        )
        self.assertEqual(override_args.tests, "bench/e2e/tests_fsdb.json")

    def test_compare_parser_functional_only_flags(self) -> None:
        parser = perf.build_parser()
        args = parser.parse_args(
            [
                "compare",
                "--revised",
                "revised",
                "--golden",
                "golden",
                "--functional-only",
                "--allow-golden-extra",
            ]
        )
        self.assertTrue(args.functional_only)
        self.assertTrue(args.allow_golden_extra)
        self.assertIsNone(args.max_negative_delta_pct)
        self.assertEqual(args.max_negative_delta_seconds, 0.0)
        self.assertIsNone(args.ignore_functional_test)

        ignored_args = parser.parse_args(
            [
                "compare",
                "--revised",
                "revised",
                "--golden",
                "golden",
                "--functional-only",
                "--ignore-functional-test",
                "scope_case=metadata differs across formats",
            ]
        )
        self.assertEqual(
            ignored_args.ignore_functional_test,
            ["scope_case=metadata differs across formats"],
        )

    def test_run_parser_verbose_flag(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(["run"])
        self.assertFalse(default_args.verbose)

        short_args = parser.parse_args(["run", "-v"])
        self.assertTrue(short_args.verbose)

        long_args = parser.parse_args(["run", "--verbose"])
        self.assertTrue(long_args.verbose)

    def test_confirm_parser_requires_explicit_tests_and_thresholds(self) -> None:
        parser = perf.build_parser()
        args = parser.parse_args(
            [
                "confirm",
                "--revised",
                "revised",
                "--golden",
                "golden",
                "--test",
                "a",
                "--test",
                "b",
                "--max-negative-delta-pct",
                "5",
                "--max-negative-delta-seconds",
                "0.005",
            ]
        )
        self.assertEqual(args.test, ["a", "b"])
        self.assertEqual(args.max_negative_delta_pct, 5.0)
        self.assertEqual(args.max_negative_delta_seconds, 0.005)

    def test_compare_parser_verbose_flag(self) -> None:
        parser = perf.build_parser()
        default_args = parser.parse_args(
            [
                "compare",
                "--revised",
                "bench/e2e/runs/revised",
                "--golden",
                "bench/e2e/runs/golden",
                "--max-negative-delta-pct",
                "5",
            ]
        )
        self.assertFalse(default_args.verbose)
        self.assertEqual(default_args.max_negative_delta_seconds, 0.0)
        self.assertIsNone(default_args.result_json)

        short_args = parser.parse_args(
            [
                "compare",
                "--revised",
                "bench/e2e/runs/revised",
                "--golden",
                "bench/e2e/runs/golden",
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
                "bench/e2e/runs/revised",
                "--golden",
                "bench/e2e/runs/golden",
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

    def test_tests_commit_catalog_exact_subset_and_distribution(self) -> None:
        payload = json.loads(
            (perf.SCRIPT_DIR / "tests_commit.json").read_text(encoding="utf-8")
        )
        tests = payload["tests"]
        names = [test["name"] for test in tests]

        expected_names = {
            "change_scr1_coremark_imem_axi_1sig_to_1000ps",
            "change_scr1_coremark_imem_axi_araddr_to_1000ps_trigger_posedge_clk_sample_native",
            "change_scr1_coremark_imem_axi_araddr_to_1000ps_trigger_posedge_clk_sample_pre_edge",
            "change_scr1_signals_1_window_2ns_trigger_any",
            "info_picorv32_ez",
            "info_scr1_isr_sample",
            "property_scr1_coremark_imem_axi_araddr_to_200ps_match_posedge_clk_sample_native",
            "property_scr1_coremark_imem_axi_araddr_to_200ps_match_posedge_clk_sample_pre_edge",
            "scope_clustered_all_depth13_json",
            "scope_scr1_all_depth7_json",
            "signal_scr1_top_recursive_depth2_json",
            "signal_scr1_top_recursive_filter_valid_json",
            "value_scr1_signals_1",
            "value_scr1_signals_10",
        }
        self.assertEqual(set(names), expected_names)
        self.assertEqual(len(names), 14)

        category_counts = Counter(str(test["category"]) for test in tests)
        self.assertEqual(
            category_counts,
            Counter(
                {
                    "change": 4,
                    "info": 2,
                    "property": 2,
                    "scope": 2,
                    "signal": 2,
                    "value": 2,
                }
            ),
        )

        for test in tests:
            if "sampling_mode" in test.get("meta", {}):
                self.assertEqual(test["runs"], 5)
                self.assertEqual(test["warmup"], 1)
            else:
                self.assertEqual(test["runs"], 1)
                self.assertEqual(test["warmup"], 0)

    def test_release_catalogs_use_gate_sample_minimums(self) -> None:
        for catalog in ("tests.json", "tests_fsdb.json"):
            payload = json.loads((perf.SCRIPT_DIR / catalog).read_text(encoding="utf-8"))
            for test in payload["tests"]:
                self.assertGreaterEqual(test["runs"], 10, f"{catalog}:{test['name']}")
                self.assertGreaterEqual(test["warmup"], 5, f"{catalog}:{test['name']}")

    def test_change_property_catalogs_pin_required_trigger_sampling(self) -> None:
        def is_edge_only(trigger: str) -> bool:
            if trigger == "*" or " or " in trigger or "," in trigger:
                return False
            parts = trigger.split()
            return bool(parts) and parts[0] in {"posedge", "negedge", "edge"}

        for catalog in ("tests.json", "tests_commit.json", "tests_fsdb.json"):
            payload = json.loads((perf.SCRIPT_DIR / catalog).read_text(encoding="utf-8"))
            for test in payload["tests"]:
                command = test["command"]
                if len(command) < 2 or command[1] not in {"change", "property"}:
                    continue

                with self.subTest(catalog=catalog, name=test["name"]):
                    self.assertIn("--on", command)
                    trigger = command[command.index("--on") + 1]
                    mode = (
                        command[command.index("--sample-mode") + 1]
                        if "--sample-mode" in command
                        else "pre-edge"
                    )
                    if not is_edge_only(trigger):
                        self.assertEqual(mode, "native")

    def test_tests_json_contains_expected_scope_benchmarks(self) -> None:
        payload = json.loads((perf.SCRIPT_DIR / "tests.json").read_text(encoding="utf-8"))
        scope_tests = {
            test["name"]: test for test in payload["tests"] if test["category"] == "scope"
        }

        self.assertEqual(
            scope_tests,
            {
                "scope_clustered_all_depth13_json": {
                    "name": "scope_clustered_all_depth13_json",
                    "category": "scope",
                    "runs": 10,
                    "warmup": 5,
                    "command": [
                        "{wavepeek_bin}",
                        "scope",
                        "--waves",
                        "/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst",
                        "--max",
                        "200000",
                        "--max-depth",
                        "13",
                        "--json",
                    ],
                    "meta": {
                        "waves": "/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst",
                        "size": "411M",
                        "filter": ".*",
                        "max_depth": 13,
                        "scope_count": 4625,
                    },
                },
                "scope_dualrocket_filter_frontend_depth12_json": {
                    "name": "scope_dualrocket_filter_frontend_depth12_json",
                    "category": "scope",
                    "runs": 10,
                    "warmup": 5,
                    "command": [
                        "{wavepeek_bin}",
                        "scope",
                        "--waves",
                        "/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst",
                        "--filter",
                        ".*frontend.*",
                        "--max",
                        "200000",
                        "--max-depth",
                        "12",
                        "--json",
                    ],
                    "meta": {
                        "waves": "/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst",
                        "size": "76M",
                        "filter": ".*frontend.*",
                        "max_depth": 12,
                        "scope_count": 118,
                    },
                },
                "scope_scr1_all_depth7_json": {
                    "name": "scope_scr1_all_depth7_json",
                    "category": "scope",
                    "runs": 15,
                    "warmup": 5,
                    "command": [
                        "{wavepeek_bin}",
                        "scope",
                        "--waves",
                        "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
                        "--max",
                        "200000",
                        "--max-depth",
                        "7",
                        "--json",
                    ],
                    "meta": {
                        "waves": "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
                        "size": "4M",
                        "filter": ".*",
                        "max_depth": 7,
                        "scope_count": 136,
                    },
                },
            },
        )

    def test_tests_json_contains_property_benchmarks(self) -> None:
        payload = json.loads((perf.SCRIPT_DIR / "tests.json").read_text(encoding="utf-8"))
        property_tests = {
            test["name"]: test for test in payload["tests"] if test["category"] == "property"
        }

        self.assertEqual(
            set(property_tests),
            {
                "property_chipyard_clusteredrocketconfig_dhrystone_window_2us_match_posedge_clk",
                "property_chipyard_clusteredrocketconfig_dhrystone_window_2us_match_posedge_clk_sample_native",
                "property_chipyard_clusteredrocketconfig_dhrystone_window_2us_match_posedge_clk_sample_pre_edge",
                "property_chipyard_clusteredrocketconfig_dhrystone_window_2us_switch_wildcard",
                "property_scr1_coremark_imem_axi_araddr_to_200ps_match_posedge_clk_sample_native",
                "property_scr1_coremark_imem_axi_araddr_to_200ps_match_posedge_clk_sample_pre_edge",
            },
        )

    def test_tests_commit_catalog_commands_match_tests_json(self) -> None:
        full_payload = json.loads((perf.SCRIPT_DIR / "tests.json").read_text(encoding="utf-8"))
        commit_payload = json.loads(
            (perf.SCRIPT_DIR / "tests_commit.json").read_text(encoding="utf-8")
        )

        full_by_name = {
            str(test["name"]): test
            for test in full_payload["tests"]
        }

        for test in commit_payload["tests"]:
            name = str(test["name"])
            self.assertIn(name, full_by_name)
            self.assertEqual(test["command"], full_by_name[name]["command"])

    def test_tests_fsdb_catalog_matches_fst_catalog_except_extension(self) -> None:
        fst_payload = json.loads((perf.SCRIPT_DIR / "tests.json").read_text(encoding="utf-8"))
        fsdb_payload = json.loads(
            (perf.SCRIPT_DIR / "tests_fsdb.json").read_text(encoding="utf-8")
        )

        artifact_prefix = (
            os.environ.get("RTL_ARTIFACTS_DIR", "/opt/rtl-artifacts").rstrip("/") + "/"
        )

        def normalize(value: object) -> object:
            if isinstance(value, str):
                if value.startswith(artifact_prefix) and value.endswith(".fsdb"):
                    return value[: -len(".fsdb")] + ".fst"
                return value
            if isinstance(value, list):
                return [normalize(item) for item in value]
            if isinstance(value, dict):
                return {key: normalize(item) for key, item in value.items()}
            return value

        self.assertEqual(normalize(fsdb_payload), normalize(fst_payload))

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
                binary=["subject=/bin/wavepeek"],
                schedule="round-robin",
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
                    mock.patch.object(perf, "parse_binary_specs", return_value=[perf.BinarySpec("subject", "/bin/wavepeek")]),
                    mock.patch.object(perf, "run_test"),
                    mock.patch.object(
                        perf,
                        "run_functional_capture",
                        return_value={"data": [], "diagnostics": []},
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

    def test_cmd_report_regenerates_labeled_run_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            (run_dir / "base").mkdir(parents=True)
            (run_dir / "rev").mkdir()
            (run_dir / "manifest.json").write_text(
                json.dumps(
                    {
                        "kind": "wavepeek-e2e-bench-run",
                        "binaries": [
                            {"label": "base", "path": "/bin/base"},
                            {"label": "rev", "path": "/bin/rev"},
                        ],
                    }
                ),
                encoding="utf-8",
            )
            args = argparse.Namespace(
                run_dir=str(run_dir),
                compare=None,
                tests=str(perf.TESTS_PATH),
            )
            report_calls: list[pathlib.Path] = []

            def fake_write_report(path, tests_by_name, compare_dir):
                report_calls.append(path)
                return path / "README.md"

            with (
                mock.patch.object(perf, "load_tests", return_value=[self._sample_test("sample")]),
                mock.patch.object(perf, "write_report", side_effect=fake_write_report),
            ):
                exit_code = perf.cmd_report(args)

        self.assertEqual(exit_code, 0)
        self.assertEqual(report_calls, [run_dir / "base", run_dir / "rev"])

    def test_cmd_report_resolves_relative_tests_path_from_cwd(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_from = root / "nested"
            run_dir = root / "run"
            run_from.mkdir()
            run_dir.mkdir()
            args = argparse.Namespace(
                run_dir=str(run_dir),
                compare=None,
                tests="fixtures/tests_fsdb.json",
            )
            load_tests_mock = mock.Mock(return_value=[self._sample_test("sample")])

            previous_cwd = pathlib.Path.cwd()
            try:
                os.chdir(run_from)
                stdout = io.StringIO()
                with (
                    mock.patch.object(perf, "load_tests", load_tests_mock),
                    mock.patch.object(perf, "write_report", return_value=run_dir / "README.md"),
                    mock.patch("sys.stdout", stdout),
                ):
                    exit_code = perf.cmd_report(args)
            finally:
                os.chdir(previous_cwd)

        self.assertEqual(exit_code, 0)
        load_tests_mock.assert_called_once_with((run_from / "fixtures/tests_fsdb.json").resolve())

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
                binary=["subject=/bin/wavepeek"],
                schedule="round-robin",
            )
            tests = [self._sample_test("sample")]
            stdout = io.StringIO()
            with (
                mock.patch.object(perf, "load_tests", return_value=tests),
                mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                mock.patch.object(perf, "ensure_hyperfine"),
                mock.patch.object(perf, "parse_binary_specs", return_value=[perf.BinarySpec("subject", "/bin/wavepeek")]),
                mock.patch.object(perf, "run_test"),
                mock.patch.object(
                    perf,
                    "run_functional_capture",
                    return_value={"data": [], "diagnostics": []},
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
                binary=["subject=/bin/wavepeek"],
                schedule="round-robin",
            )
            tests = [self._sample_test("sample")]
            stdout = io.StringIO()
            with (
                mock.patch.object(perf, "load_tests", return_value=tests),
                mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                mock.patch.object(perf, "ensure_hyperfine"),
                mock.patch.object(perf, "parse_binary_specs", return_value=[perf.BinarySpec("subject", "/bin/wavepeek")]),
                mock.patch.object(perf, "run_test"),
                mock.patch.object(
                    perf,
                    "run_functional_capture",
                    return_value={"data": [], "diagnostics": []},
                ),
                mock.patch.object(perf, "write_wavepeek_artifact"),
                mock.patch.object(perf, "write_report", return_value=run_dir / "README.md"),
                mock.patch("sys.stdout", stdout),
            ):
                exit_code = perf.cmd_run(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("[1/1] subject/sample", stdout.getvalue())
        self.assertIn("info: run directory:", stdout.getvalue())

    def test_cmd_run_round_robin_orders_tests_across_binaries(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            run_dir.mkdir()
            args = argparse.Namespace(
                filter=None,
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=False,
                wavepeek_timeout_seconds=300,
                tests=str(perf.TESTS_PATH),
                verbose=False,
                binary=["base=/bin/base", "rev=/bin/rev"],
                schedule="round-robin",
            )
            tests = [self._sample_test("a"), self._sample_test("b")]
            order: list[tuple[str, str]] = []

            def fake_run_test(test, label_dir, wavepeek_bin, timeout_seconds, verbose):
                order.append((str(test["name"]), pathlib.Path(label_dir).name))

            with (
                mock.patch.object(perf, "load_tests", return_value=tests),
                mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                mock.patch.object(perf, "parse_binary_specs", return_value=[
                    perf.BinarySpec("base", "/bin/base"),
                    perf.BinarySpec("rev", "/bin/rev"),
                ]),
                mock.patch.object(perf, "ensure_hyperfine"),
                mock.patch.object(perf, "run_test", side_effect=fake_run_test),
                mock.patch.object(
                    perf,
                    "run_functional_capture",
                    return_value={"data": [], "diagnostics": []},
                ),
                mock.patch.object(perf, "write_wavepeek_artifact"),
                mock.patch.object(perf, "write_report", return_value=run_dir / "README.md"),
            ):
                exit_code = perf.cmd_run(args)
            base_dir_exists = (run_dir / "base").is_dir()
            rev_dir_exists = (run_dir / "rev").is_dir()

        self.assertEqual(exit_code, 0)
        self.assertEqual(order, [("a", "base"), ("a", "rev"), ("b", "base"), ("b", "rev")])
        self.assertTrue(base_dir_exists)
        self.assertTrue(rev_dir_exists)

    def test_cmd_run_continues_after_preflight_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            run_dir.mkdir()
            tests = [self._sample_test("bad"), self._sample_test("good")]
            args = argparse.Namespace(
                filter=None,
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=False,
                fail_fast=False,
                wavepeek_timeout_seconds=300,
                tests=str(perf.TESTS_PATH),
                verbose=False,
                binary=["subject=/bin/wavepeek"],
                schedule="round-robin",
            )
            preflight_failure = perf.make_failure_artifact(
                test_name="bad",
                phase="preflight",
                command=["/bin/wavepeek", "info", "--sample-mode", "native"],
                exit_code=2,
                stderr="unknown option: --sample-mode",
            )
            run_test_mock = mock.Mock(return_value=None)
            with (
                mock.patch.object(perf, "load_tests", return_value=tests),
                mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                mock.patch.object(perf, "ensure_hyperfine"),
                mock.patch.object(perf, "parse_binary_specs", return_value=[perf.BinarySpec("subject", "/bin/wavepeek")]),
                mock.patch.object(
                    perf,
                    "run_functional_capture",
                    side_effect=[
                        (None, preflight_failure),
                        ({"data": [], "diagnostics": []}, None),
                    ],
                ),
                mock.patch.object(perf, "run_test", run_test_mock),
                mock.patch.object(perf, "write_report", return_value=run_dir / "subject" / "README.md"),
            ):
                exit_code = perf.cmd_run(args)
            bad_failure_exists = (run_dir / "subject" / "bad.failure.json").is_file()
            bad_wavepeek_exists = (run_dir / "subject" / "bad.wavepeek.json").exists()
            good_wavepeek_exists = (run_dir / "subject" / "good.wavepeek.json").is_file()
            run_test_count = run_test_mock.call_count
            run_test_name = run_test_mock.call_args.args[0]["name"]

        self.assertEqual(exit_code, 0)
        self.assertTrue(bad_failure_exists)
        self.assertFalse(bad_wavepeek_exists)
        self.assertTrue(good_wavepeek_exists)
        self.assertEqual(run_test_count, 1)
        self.assertEqual(run_test_name, "good")

    def test_cmd_run_fail_fast_stops_after_preflight_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            run_dir = root / "run"
            run_dir.mkdir()
            tests = [self._sample_test("bad"), self._sample_test("good")]
            args = argparse.Namespace(
                filter=None,
                run_dir=str(run_dir),
                out_dir=str(root),
                compare=None,
                missing_only=False,
                fail_fast=True,
                wavepeek_timeout_seconds=300,
                tests=str(perf.TESTS_PATH),
                verbose=False,
                binary=["subject=/bin/wavepeek"],
                schedule="round-robin",
            )
            preflight_failure = perf.make_failure_artifact(
                test_name="bad",
                phase="preflight",
                command=["/bin/wavepeek", "info"],
                exit_code=2,
            )
            run_test_mock = mock.Mock(return_value=None)
            with (
                mock.patch.object(perf, "load_tests", return_value=tests),
                mock.patch.object(perf, "resolve_run_dir", return_value=run_dir),
                mock.patch.object(perf, "ensure_hyperfine"),
                mock.patch.object(perf, "parse_binary_specs", return_value=[perf.BinarySpec("subject", "/bin/wavepeek")]),
                mock.patch.object(
                    perf,
                    "run_functional_capture",
                    return_value=(None, preflight_failure),
                ),
                mock.patch.object(perf, "run_test", run_test_mock),
            ):
                with self.assertRaises(SystemExit):
                    perf.cmd_run(args)
            bad_failure_exists = (run_dir / "subject" / "bad.failure.json").is_file()
            run_test_count = run_test_mock.call_count

        self.assertTrue(bad_failure_exists)
        self.assertEqual(run_test_count, 0)

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
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
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
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
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
        self.assertIn("sample: median revised=2.000000s", stderr.getvalue())

    def test_cmd_compare_fails_on_median_regression(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(
                revised / "sample.hyperfine.json", mean=1.0, median=2.0
            )
            self._write_hyperfine_artifact(
                golden / "sample.hyperfine.json", mean=1.0, median=1.0
            )
            (revised / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
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
        self.assertIn("sample: median revised=2.000000s", stderr.getvalue())

    def test_cmd_compare_writes_result_json_for_timing_failures(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(
                revised / "sample.hyperfine.json", mean=1.0, median=2.0
            )
            self._write_hyperfine_artifact(
                golden / "sample.hyperfine.json", mean=1.0, median=1.0
            )
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                functional_only=False,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["status"], "failed")
        self.assertEqual(payload["timing_failures"][0]["test_name"], "sample")
        self.assertEqual(payload["timing_failures"][0]["metric"], "median")
        self.assertEqual(payload["functional_mismatches"], [])

    def test_cmd_compare_skips_golden_failure_revised_success(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()
            for run_dir in (revised, golden):
                self._write_hyperfine_artifact(run_dir / "ok.hyperfine.json", 1.0)
                self._write_wavepeek_artifact(run_dir / "ok.wavepeek.json")
            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 1.0)
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            perf.write_failure_artifact(
                golden,
                "sample",
                perf.make_failure_artifact(
                    test_name="sample",
                    phase="preflight",
                    command=["old-wavepeek", "change", "--sample-mode", "native"],
                    exit_code=2,
                    stderr="unknown option: --sample-mode",
                ),
            )
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                functional_only=False,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 0)
        self.assertEqual(payload["comparable_count"], 1)
        self.assertEqual(payload["skipped_uncomparable_count"], 1)
        self.assertEqual(payload["baseline_unsupported"][0]["test_name"], "sample")
        self.assertEqual(payload["timing_failures"], [])

    def test_cmd_compare_fails_when_no_tests_are_comparable(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()
            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 1.0)
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            perf.write_failure_artifact(
                golden,
                "sample",
                perf.make_failure_artifact(
                    test_name="sample",
                    phase="preflight",
                    command=["old-wavepeek", "change", "--sample-mode", "native"],
                    exit_code=2,
                ),
            )
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                functional_only=False,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["comparable_count"], 0)
        self.assertEqual(payload["skipped_uncomparable_count"], 1)
        self.assertIn(
            "no comparable tests between revised and golden",
            payload["functional_artifact_errors"],
        )

    def test_cmd_compare_fails_on_revised_failure_golden_success(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 1.0)
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")
            perf.write_failure_artifact(
                revised,
                "sample",
                perf.make_failure_artifact(
                    test_name="sample",
                    phase="preflight",
                    command=["new-wavepeek", "change"],
                    exit_code=1,
                ),
            )
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                functional_only=False,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["failed_uncomparable_count"], 1)
        self.assertEqual(payload["revised_failures"][0]["test_name"], "sample")

    def test_cmd_compare_skips_both_side_failures(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()
            for run_dir in (revised, golden):
                self._write_hyperfine_artifact(run_dir / "ok.hyperfine.json", 1.0)
                self._write_wavepeek_artifact(run_dir / "ok.wavepeek.json")
            for run_dir, command in ((revised, "new-wavepeek"), (golden, "old-wavepeek")):
                perf.write_failure_artifact(
                    run_dir,
                    "sample",
                    perf.make_failure_artifact(
                        test_name="sample",
                        phase="preflight",
                        command=[command, "change"],
                        exit_code=2,
                    ),
                )
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                functional_only=False,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 0)
        self.assertEqual(payload["comparable_count"], 1)
        self.assertEqual(payload["skipped_uncomparable_count"], 1)
        self.assertEqual(payload["both_side_failures"][0]["test_name"], "sample")

    def test_cmd_compare_reports_invalid_uncomparable_artifact_without_parsing_it(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()
            for run_dir in (revised, golden):
                self._write_hyperfine_artifact(run_dir / "ok.hyperfine.json", 1.0)
                self._write_wavepeek_artifact(run_dir / "ok.wavepeek.json")
            perf.write_failure_artifact(
                revised,
                "bad",
                perf.make_failure_artifact(
                    test_name="bad",
                    phase="benchmark",
                    command=["hyperfine"],
                    exit_code=1,
                ),
            )
            (revised / "bad.hyperfine.json").write_text("not json", encoding="utf-8")
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                functional_only=False,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["comparable_count"], 1)
        self.assertIn(
            "revised: bad: has normal artifacts and a failure artifact",
            payload["integrity_errors"],
        )

    def test_cmd_compare_fails_on_missing_outcome(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()
            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 1.0)
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_hyperfine_artifact(revised / "other.hyperfine.json", 1.0)
            self._write_wavepeek_artifact(revised / "other.wavepeek.json")
            self._write_hyperfine_artifact(golden / "other.hyperfine.json", 1.0)
            self._write_wavepeek_artifact(golden / "other.wavepeek.json")
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                functional_only=False,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["comparable_count"], 1)
        self.assertIn("sample: missing outcome in golden run", payload["integrity_errors"])

    def test_cmd_confirm_passes_using_best_samples(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "confirm-result.json"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(
                golden / "sample.hyperfine.json",
                mean=1.0,
                median=1.0,
                times=[1.00, 1.50, 1.60],
            )
            self._write_hyperfine_artifact(
                revised / "sample.hyperfine.json",
                mean=1.0,
                median=1.4,
                times=[1.02, 1.40, 1.45],
            )
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                test=["sample"],
                max_negative_delta_pct=5.0,
                max_negative_delta_seconds=0.005,
                result_json=str(result_json),
                verbose=True,
            )
            stdout = io.StringIO()
            with mock.patch("sys.stdout", stdout):
                exit_code = perf.cmd_confirm(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 0)
        self.assertIn("best-sample timing confirmation passed", stdout.getvalue())
        self.assertEqual(payload["status"], "passed")
        self.assertEqual(payload["confirmed"][0]["metric"], "best")
        self.assertAlmostEqual(payload["confirmed"][0]["golden_seconds"], 1.0)
        self.assertAlmostEqual(payload["confirmed"][0]["revised_seconds"], 1.02)

    def test_cmd_confirm_fails_when_best_sample_exceeds_threshold(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(
                golden / "sample.hyperfine.json",
                mean=1.0,
                median=1.0,
                times=[1.00, 1.10],
            )
            self._write_hyperfine_artifact(
                revised / "sample.hyperfine.json",
                mean=1.0,
                median=1.2,
                times=[1.20, 1.30],
            )
            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                test=["sample"],
                max_negative_delta_pct=5.0,
                max_negative_delta_seconds=0.005,
                result_json=None,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_confirm(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("sample: best revised=1.200000s", stderr.getvalue())

    def test_cmd_compare_ignores_mean_regression_when_median_is_stable(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(
                revised / "sample.hyperfine.json", mean=2.0, median=1.0
            )
            self._write_hyperfine_artifact(
                golden / "sample.hyperfine.json", mean=1.0, median=1.0
            )
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.0,
                verbose=True,
            )
            exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 0)

    def test_cmd_compare_applies_absolute_slowdown_floor(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 0.104)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 0.100)
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.005,
                verbose=True,
            )
            exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 0)

    def test_cmd_compare_fails_when_absolute_slowdown_floor_is_exceeded(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 0.106)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 0.100)
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=0.0,
                max_negative_delta_seconds=0.005,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("allowed=0.005000s", stderr.getvalue())

    def test_cmd_compare_non_verbose_success_has_concise_ok_line(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            self._write_hyperfine_artifact(revised / "sample.hyperfine.json", 1.0)
            self._write_hyperfine_artifact(golden / "sample.hyperfine.json", 1.0)
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

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

    def test_cmd_compare_functional_only_success_without_hyperfine(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                verbose=False,
            )
            stdout = io.StringIO()
            with mock.patch("sys.stdout", stdout):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("functional checks passed", stdout.getvalue())

    def test_cmd_compare_functional_only_fails_when_no_tests_are_comparable(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "compare-result.json"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            perf.write_failure_artifact(
                golden,
                "sample",
                perf.make_failure_artifact(
                    test_name="sample",
                    phase="preflight",
                    command=["old-wavepeek", "change", "--sample-mode", "native"],
                    exit_code=2,
                ),
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                result_json=str(result_json),
                verbose=True,
            )
            with mock.patch("sys.stderr", io.StringIO()):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["comparable_count"], 0)
        self.assertEqual(payload["skipped_uncomparable_count"], 1)
        self.assertIn(
            "no comparable tests between revised and golden",
            payload["functional_artifact_errors"],
        )

    def test_cmd_compare_functional_only_fails_on_revised_extra(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(revised / "extra.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=True,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("tests only in revised run: extra", stderr.getvalue())

    def test_cmd_compare_functional_only_allows_golden_extra_when_requested(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "extra.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=True,
                verbose=False,
            )
            exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 0)

    def test_cmd_compare_functional_only_fails_on_golden_extra_by_default(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "extra.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("tests only in golden run: extra", stderr.getvalue())

    def test_cmd_compare_functional_only_fails_on_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json", data=[{"id": 1}])
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json", data=[{"id": 2}])

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("mismatched fields data", stderr.getvalue())

    def test_cmd_compare_functional_only_ignores_named_mismatch_with_reason(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            result_json = root / "result.json"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json", data=[{"id": 1}])
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json", data=[{"id": 2}])

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                ignore_functional_test=["sample=metadata differs across formats"],
                result_json=str(result_json),
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)
            payload = json.loads(result_json.read_text(encoding="utf-8"))

        self.assertEqual(exit_code, 0)
        self.assertIn("ignored functional tests", stderr.getvalue())
        self.assertEqual(payload["ignored_functional_tests"][0]["test_name"], "sample")
        self.assertEqual(
            payload["ignored_functional_tests"][0]["reason"],
            "metadata differs across formats",
        )
        self.assertEqual(payload["functional_mismatches"], [])

    def test_cmd_compare_functional_ignore_requires_presence_on_both_sides(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "ignored.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=True,
                ignore_functional_test=["ignored=metadata differs across formats"],
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn(
            "ignored functional test `ignored` missing from revised",
            stderr.getvalue(),
        )

    def test_cmd_compare_functional_ignore_does_not_hide_diagnostics_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(
                revised / "sample.wavepeek.json",
                data=[{"id": 1}],
                diagnostics=[
                    {"kind": "warning", "code": "WPK-W0001", "message": "revised"}
                ],
            )
            self._write_wavepeek_artifact(
                golden / "sample.wavepeek.json",
                data=[{"id": 2}],
                diagnostics=[
                    {"kind": "warning", "code": "WPK-W0001", "message": "golden"}
                ],
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                ignore_functional_test=["sample=metadata differs across formats"],
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("mismatched fields diagnostics", stderr.getvalue())
        self.assertNotIn("mismatched fields data", stderr.getvalue())

    def test_cmd_compare_functional_ignore_does_not_hide_revised_only_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            self._write_wavepeek_artifact(revised / "sample.wavepeek.json")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")
            self._write_wavepeek_artifact(revised / "extra.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=True,
                ignore_functional_test=["extra=metadata differs across formats"],
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("tests only in revised run: extra", stderr.getvalue())

    def test_cmd_compare_functional_ignore_does_not_hide_timeout_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            (revised / "sample.wavepeek.json").write_text("{}\n", encoding="utf-8")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                ignore_functional_test=["sample=metadata differs across formats"],
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("timeout artifact", stderr.getvalue())

    def test_cmd_compare_rejects_functional_ignore_outside_functional_only(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=5.0,
                functional_only=False,
                allow_golden_extra=False,
                ignore_functional_test=["sample=metadata differs"],
                verbose=False,
            )
            with self.assertRaises(SystemExit):
                perf.cmd_compare(args)

    def test_cmd_compare_functional_only_fails_on_timeout_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()
            (revised / "sample.wavepeek.json").write_text("{}\n", encoding="utf-8")
            self._write_wavepeek_artifact(golden / "sample.wavepeek.json")

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=True,
                allow_golden_extra=False,
                verbose=True,
            )
            stderr = io.StringIO()
            with mock.patch("sys.stderr", stderr):
                exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 1)
        self.assertIn("timeout artifact", stderr.getvalue())

    def test_cmd_compare_requires_threshold_outside_functional_only(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            revised = root / "revised"
            golden = root / "golden"
            revised.mkdir()
            golden.mkdir()

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=None,
                functional_only=False,
                allow_golden_extra=False,
                verbose=False,
            )
            with self.assertRaises(SystemExit) as error:
                perf.cmd_compare(args)

        self.assertIn("--max-negative-delta-pct is required", str(error.exception))

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
                stdout=json.dumps({"data": [], "diagnostics": []}),
                stderr="",
            )
            perf.run_functional_capture(test, "wavepeek", "run", 123)

        self.assertEqual(run_mock.call_count, 1)
        self.assertEqual(run_mock.call_args.kwargs["timeout"], 123)

    def test_functional_diff_fields(self) -> None:
        baseline = {"data": [1], "diagnostics": ["w1"]}
        self.assertEqual(perf.functional_diff_fields(baseline, baseline), [])
        self.assertEqual(
            perf.functional_diff_fields(
                {"data": [2], "diagnostics": ["w1"]},
                baseline,
            ),
            ["data"],
        )
        self.assertEqual(
            perf.functional_diff_fields(
                {"data": [1], "diagnostics": ["w2"]},
                baseline,
            ),
            ["diagnostics"],
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
        self.assertIn("missing key `diagnostics`", error)

    def test_load_wavepeek_artifact_for_compare_accepts_object_data(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": {"time_unit": "ps"}, "diagnostics": []}),
                encoding="utf-8",
            )
            payload, error = perf.load_wavepeek_artifact_for_compare(
                temp_path,
                "sample",
                "revised",
            )

        self.assertIsNone(error)
        self.assertEqual(payload, {"data": {"time_unit": "ps"}, "diagnostics": []})

    def test_load_wavepeek_artifact_for_compare_rejects_scalar_data(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": "oops", "diagnostics": []}),
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

    def test_load_wavepeek_artifact_for_compare_invalid_diagnostics_type(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": [], "diagnostics": "not-a-list"}),
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
        self.assertIn("field `diagnostics` must be list", error)

    def test_load_wavepeek_artifact_for_compare_valid_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps(
                    {
                        "data": [{"id": 1}],
                        "diagnostics": [self._diag()],
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
        self.assertEqual(payload, {"data": [{"id": 1}], "diagnostics": [self._diag()]})

    def test_load_wavepeek_artifact_for_compare_rejects_legacy_warnings_key(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": [], "diagnostics": [], "warnings": []}),
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
        self.assertIn("must not contain legacy key `warnings`", error)

    def test_load_wavepeek_artifact_for_compare_rejects_wrong_kind_code_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": [], "diagnostics": [self._diag(code="WPK-E0001")]}),
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
        self.assertIn("must use WPK-W for warning", error)

    def test_load_wavepeek_artifact_for_compare_rejects_untyped_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            artifact = temp_path / "sample.wavepeek.json"
            artifact.write_text(
                json.dumps({"data": [], "diagnostics": ["legacy warning"]}),
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
        self.assertIn("field `diagnostics[0]` must be object", error)

    def test_report_functional_status_missing_counterpart(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [1], "diagnostics": []}},
                {},
            ),
            perf.FUNCTIONAL_MISSING_MARKER,
        )

    def test_report_functional_status_data_mismatch(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [1], "diagnostics": ["a"]}},
                {"t": {"data": [2], "diagnostics": ["b"]}},
            ),
            f"{perf.FUNCTIONAL_MISMATCH_MARKER}D",
        )

    def test_report_functional_status_diagnostic_mismatch(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [1], "diagnostics": ["a"]}},
                {"t": {"data": [1], "diagnostics": ["b"]}},
            ),
            f"{perf.FUNCTIONAL_MISMATCH_MARKER}X",
        )

    def test_report_functional_status_empty_data_match(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [], "diagnostics": ["a"]}},
                {"t": {"data": [], "diagnostics": ["a"]}},
            ),
            f"{perf.FUNCTIONAL_MATCH_MARKER}E",
        )

    def test_report_functional_status_empty_object_data_match(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": {}, "diagnostics": ["a"]}},
                {"t": {"data": {}, "diagnostics": ["a"]}},
            ),
            f"{perf.FUNCTIONAL_MATCH_MARKER}E",
        )

    def test_report_functional_status_nonempty_data_match(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {"data": [{"id": 1}], "diagnostics": ["a"]}},
                {"t": {"data": [{"id": 1}], "diagnostics": ["a"]}},
            ),
            perf.FUNCTIONAL_MATCH_MARKER,
        )

    def test_report_functional_status_timeout_artifact(self) -> None:
        self.assertEqual(
            perf.report_functional_status(
                "t",
                {"t": {}},
                {"t": {"data": [{"id": 1}], "diagnostics": []}},
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
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
                encoding="utf-8",
            )
            (golden / "sample.wavepeek.json").write_text(
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
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
                json.dumps({"data": [{"id": 1}], "diagnostics": []}),
                encoding="utf-8",
            )

            args = argparse.Namespace(
                revised=str(revised),
                golden=str(golden),
                max_negative_delta_pct=5.0,
            )
            exit_code = perf.cmd_compare(args)

        self.assertEqual(exit_code, 0)


class JustfileBenchmarkRecipeTest(unittest.TestCase):
    JUSTFILE_PATH = pathlib.Path(__file__).resolve().parents[2] / "justfile"

    @classmethod
    def _recipe_body(cls, recipe_name: str) -> str:
        lines = cls.JUSTFILE_PATH.read_text(encoding="utf-8").splitlines()
        for line_index, line in enumerate(lines):
            if not (line.startswith(f"{recipe_name}:") or line.startswith(f"{recipe_name} ")):
                continue
            body: list[str] = []
            for body_line in lines[line_index + 1 :]:
                if not body_line.startswith((" ", "\t")):
                    break
                body.append(body_line.strip())
            return "\n".join(body)
        raise AssertionError(f"recipe not found: {recipe_name}")

    def test_public_benchmark_recipes_use_manual_gate_helpers(self) -> None:
        self.assertIn("tools/bench/gate.py", self._recipe_body("bench-gate"))
        self.assertIn("tools/bench/capture.py", self._recipe_body("bench-capture"))
        self.assertIn("tools/bench/compare.py", self._recipe_body("bench-compare"))

    def test_baseline_update_recipes_removed(self) -> None:
        justfile = self.JUSTFILE_PATH.read_text(encoding="utf-8")
        self.assertNotIn("bench-e2e-update-baseline:", justfile)
        self.assertNotIn("bench-e2e-fsdb-update-baseline:", justfile)

    def test_pre_commit_smoke_no_longer_compares_against_baseline(self) -> None:
        for recipe_name in ("bench-e2e-smoke-commit", "bench-e2e-fsdb-smoke-commit"):
            body = self._recipe_body(recipe_name)
            self.assertIn("bench/e2e/perf.py run", body)
            self.assertNotIn("bench/e2e/perf.py compare", body)
            self.assertNotIn("baseline", body)


if __name__ == "__main__":
    unittest.main()
