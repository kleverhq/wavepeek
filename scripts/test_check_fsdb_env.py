#!/usr/bin/env python3

from __future__ import annotations

import os
import pathlib
import subprocess
import tempfile
import unittest

SCRIPT_PATH = pathlib.Path(__file__).with_name("check_fsdb_env.py")


class CheckFsdbEnvTest(unittest.TestCase):
    def run_script(
        self, env: dict[str, str], args: list[str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        script_env = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("WAVEPEEK_FSDB_") and key != "VERDI_HOME"
        }
        script_env.update(env)
        return subprocess.run(
            ["python3", str(SCRIPT_PATH), *(args or [])],
            check=False,
            capture_output=True,
            text=True,
            env=script_env,
        )

    def test_unset_verdi_home_skips_without_default_fallback(self) -> None:
        result = self.run_script({})

        self.assertEqual(result.returncode, 77, result.stderr)
        self.assertIn(
            "skip: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run FSDB build checks",
            result.stdout,
        )
        self.assertEqual(result.stderr, "")

    def test_empty_verdi_home_skips_without_default_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            result = self.run_script({"VERDI_HOME": temp_dir})

        self.assertEqual(result.returncode, 77, result.stderr)
        self.assertIn(
            "skip: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run FSDB build checks",
            result.stdout,
        )
        self.assertEqual(result.stderr, "")

    def test_require_mode_fails_instead_of_skipping(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            result = self.run_script({"VERDI_HOME": temp_dir}, args=["--require"])

        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout, "")
        self.assertIn("error: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME", result.stderr)
        self.assertNotIn(temp_dir, result.stderr)

    def test_explicit_reader_override_without_verdi_home_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            result = self.run_script({"WAVEPEEK_FSDB_READER_LIBDIR": temp_dir})

        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout, "")
        self.assertIn("error: fsdb: VERDI_HOME is required", result.stderr)
        self.assertNotIn(temp_dir, result.stderr)

    def test_explicit_bad_library_directory_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            reader_root = root / "share" / "FsdbReader"
            reader_root.mkdir(parents=True)
            for header in ("ffrAPI.h", "ffrKit.h", "fsdbShr.h"):
                (reader_root / header).write_text("", encoding="utf-8")
            bad_libdir = root / "bad-libdir"
            bad_libdir.mkdir()

            result = self.run_script(
                {
                    "VERDI_HOME": str(root),
                    "WAVEPEEK_FSDB_READER_LIBDIR": str(bad_libdir),
                }
            )

        self.assertEqual(result.returncode, 1)
        self.assertIn("error: fsdb: selected FSDB Reader library directory is incomplete", result.stderr)
        self.assertIn("libnffr.so", result.stderr)
        self.assertNotIn(str(root), result.stderr)

    def test_minimal_file_complete_sdk_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            reader_root = root / "share" / "FsdbReader"
            libdir = reader_root / "linux64"
            libdir.mkdir(parents=True)
            for header in ("ffrAPI.h", "ffrKit.h", "fsdbShr.h"):
                (reader_root / header).write_text("", encoding="utf-8")
            for library in ("libnffr.so", "libnsys.so"):
                (libdir / library).write_text("", encoding="utf-8")

            result = self.run_script({"VERDI_HOME": str(root)})

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("ok: fsdb: Verdi FSDB Reader SDK found", result.stdout)
        self.assertEqual(result.stdout.count("\n"), 1)
        self.assertNotIn(str(root), result.stdout)


if __name__ == "__main__":
    unittest.main()
