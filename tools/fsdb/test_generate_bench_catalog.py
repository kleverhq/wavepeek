#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import subprocess
import tempfile
import textwrap
import unittest


SCRIPT_PATH = pathlib.Path(__file__).with_name("generate_bench_catalog.py").resolve()


class GenerateBenchCatalogCliTest(unittest.TestCase):
    def run_script(
        self, args: list[str], cwd: pathlib.Path
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(SCRIPT_PATH), *args],
            cwd=cwd,
            check=False,
            capture_output=True,
            text=True,
        )

    def write_source_catalog(self, root: pathlib.Path) -> tuple[pathlib.Path, pathlib.Path]:
        source = root / "tests.json"
        output = root / "tests_fsdb.json"
        source.write_text(
            textwrap.dedent(
                """\
                {
                  "tests": [
                    {
                      "name": "sample",
                      "category": "value",
                      "runs": 1,
                      "warmup": 0,
                      "command": [
                        "{wavepeek_bin}",
                        "value",
                        "--waves",
                        "/opt/ondas-fixtures/fst/fst0000-sample/waveform.fst",
                        "--signals",
                        "top.fsdbfile,top.trace_file"
                      ],
                      "meta": {
                        "waves": "/opt/ondas-fixtures/fst/fst0000-sample/waveform.fst",
                        "note": "rewrite sample.fst text consistently"
                      }
                    }
                  ]
                }
                """
            ),
            encoding="utf-8",
        )
        return source, output

    def test_generates_fsdb_catalog_by_replacing_fst_suffixes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source, output = self.write_source_catalog(root)

            result = self.run_script(
                ["--source", str(source), "--output", str(output)], root
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            expected = source.read_text(encoding="utf-8").replace(".fst", ".fsdb")
            generated = output.read_text(encoding="utf-8")
            self.assertEqual(generated, expected)
            self.assertIn("/opt/ondas-fixtures/fst/fst0000-sample/waveform.fsdb", generated)
            self.assertIn("rewrite sample.fsdb text consistently", generated)
            self.assertIn("top.fsdbfile,top.trace_file", generated)

    def test_generates_vcd_catalog_and_checks_freshness(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source, _ = self.write_source_catalog(root)
            output = root / "tests_vcd.json"
            args = ["--source", str(source), "--output", str(output), "--target", "vcd"]

            generated = self.run_script(args, root)
            checked = self.run_script([*args, "--check"], root)

            self.assertEqual(generated.returncode, 0, generated.stderr)
            self.assertEqual(checked.returncode, 0, checked.stderr)
            self.assertEqual(
                output.read_text(encoding="utf-8"),
                source.read_text(encoding="utf-8").replace(".fst", ".vcd"),
            )

    def test_artifact_dir_option_is_accepted_for_compatibility(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source, output = self.write_source_catalog(root)

            result = self.run_script(
                [
                    "--source",
                    str(source),
                    "--output",
                    str(output),
                    "--artifact-dir",
                    "/other/artifacts",
                ],
                root,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            expected = source.read_text(encoding="utf-8").replace(".fst", ".fsdb")
            self.assertEqual(output.read_text(encoding="utf-8"), expected)

    def test_check_passes_for_fresh_generated_catalog(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source, output = self.write_source_catalog(root)
            update = self.run_script(["--source", str(source), "--output", str(output)], root)
            self.assertEqual(update.returncode, 0, update.stderr)

            result = self.run_script(
                ["--source", str(source), "--output", str(output), "--check"], root
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("ok: fsdb catalog:", result.stdout)

    def test_check_fails_for_stale_catalog(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source, output = self.write_source_catalog(root)
            output.write_text('{"tests": []}\n', encoding="utf-8")

            result = self.run_script(
                ["--source", str(source), "--output", str(output), "--check"], root
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("is stale", result.stderr)
            self.assertIn("just update-bench-e2e-fsdb-catalog", result.stderr)

    def test_fails_when_source_is_invalid_json(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source = root / "tests.json"
            output = root / "tests_fsdb.json"
            source.write_text('{"tests": ["sample.fst"]\n', encoding="utf-8")

            result = self.run_script(["--source", str(source), "--output", str(output)], root)

            self.assertEqual(result.returncode, 1)
            self.assertIn("invalid JSON", result.stderr)
            self.assertFalse(output.exists())

    def test_fails_when_source_has_no_fst_suffixes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source = root / "tests.json"
            output = root / "tests_fsdb.json"
            source.write_text(
                textwrap.dedent(
                    """\
                    {
                      "tests": [
                        {"name": "sample", "command": ["sample.vcd"]}
                      ]
                    }
                    """
                ),
                encoding="utf-8",
            )

            result = self.run_script(["--source", str(source), "--output", str(output)], root)

            self.assertEqual(result.returncode, 1)
            self.assertIn("no .fst suffixes found", result.stderr)
            self.assertFalse(output.exists())

    def test_vcd_errors_name_selected_target(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source = root / "tests.json"
            output = root / "tests_vcd.json"
            args = ["--source", str(source), "--output", str(output), "--target", "vcd"]
            for contents in (None, "{invalid", '{"tests": []}'):
                with self.subTest(contents=contents):
                    if contents is not None:
                        source.write_text(contents, encoding="utf-8")
                    result = self.run_script(args, root)
                    self.assertEqual(result.returncode, 1)
                    self.assertTrue(result.stderr.startswith("error: vcd catalog:"))
                    self.assertFalse(output.exists())
            source.write_text('{"tests": ["sample.fst"]}', encoding="utf-8")
            result = self.run_script([*args, "--check"], root)
            self.assertEqual(result.returncode, 1)
            self.assertTrue(result.stderr.startswith("error: vcd catalog:"))

    def test_generated_waveforms_use_existing_fsdb_fixture_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            source = root / "tests.json"
            output = root / "tests_fsdb.json"
            source.write_text(
                '{"tests": ["tests/fixtures/generated/extract_global_include.fst"]}',
                encoding="utf-8",
            )
            result = self.run_script(["--source", str(source), "--output", str(output)], root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                output.read_text(encoding="utf-8"),
                '{"tests": ["tests/fixtures/fsdb/extract_global_include.fsdb"]}',
            )


if __name__ == "__main__":
    unittest.main()
