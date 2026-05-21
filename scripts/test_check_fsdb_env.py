#!/usr/bin/env python3

from __future__ import annotations

import os
import pathlib
import subprocess
import tempfile
import unittest

SCRIPT_PATH = pathlib.Path(__file__).with_name("check_fsdb_env.py")


class CheckFsdbEnvTest(unittest.TestCase):
    def run_script(self, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
        script_env = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("WAVEPEEK_FSDB_")
            and key not in {"VERDI_HOME", "FSDB_RTL_ARTIFACTS_DIR"}
        }
        script_env.update(env)
        return subprocess.run(
            ["python3", str(SCRIPT_PATH)],
            check=False,
            capture_output=True,
            text=True,
            env=script_env,
        )

    def test_empty_verdi_home_skips_without_default_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            result = self.run_script({"VERDI_HOME": temp_dir})

        self.assertEqual(result.returncode, 77, result.stderr)
        self.assertIn(
            "skip: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run FSDB build checks",
            result.stdout,
        )
        self.assertEqual(result.stderr, "")

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
        self.assertIn("optional artifact directory", result.stdout)


if __name__ == "__main__":
    unittest.main()
