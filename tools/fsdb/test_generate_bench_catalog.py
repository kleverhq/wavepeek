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
                        "/opt/rtl-artifacts/sample.fst",
                        "--signals",
                        "top.fsdbfile,top.trace_file"
                      ],
                      "meta": {
                        "waves": "/opt/rtl-artifacts/sample.fst",
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
            self.assertIn("/opt/rtl-artifacts/sample.fsdb", generated)
            self.assertIn("rewrite sample.fsdb text consistently", generated)
            self.assertIn("top.fsdbfile,top.trace_file", generated)

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


if __name__ == "__main__":
    unittest.main()
