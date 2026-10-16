from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).parent
spec = importlib.util.spec_from_file_location("prepare_shm", DIRECTORY / "prepare_shm_fixtures.py")
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)


class ShmEnvironmentTest(unittest.TestCase):
    def probe(self, home: str, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", "-B", str(DIRECTORY / "check_shm_env.py"), *args],
            env={**os.environ, "XCELIUM_HOME": home}, capture_output=True, text=True,
        )

    def test_absent_sdk_skips_but_required_or_configured_broken_sdk_fails(self) -> None:
        self.assertEqual(self.probe("").returncode, 77)
        self.assertEqual(self.probe("", "--require").returncode, 1)
        with tempfile.TemporaryDirectory() as temporary:
            self.assertEqual(self.probe(temporary).returncode, 1)
            library = Path(temporary) / "tools/lib/64bit/libsst2.so"
            library.parent.mkdir(parents=True)
            library.touch()
            result = self.probe(temporary, "--print-library")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), str(library.resolve()))


class ShmPreparationTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / "source.vcd"
        self.source.write_text("source")
        self.converter = self.root / "simvisdbutil"
        self.converter.touch()
        self.output = self.root / "fixture.shm"
        self.root_patch = patch.object(prepare, "ROOT", self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)
        self.env_patch = patch.dict(os.environ, {"XCELIUM_HOME": str(self.root)})
        self.env_patch.start()
        self.addCleanup(self.env_patch.stop)

    def convert(self, command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
        output = Path(command[command.index("-output") + 1])
        output.mkdir()
        (output / "fixture.dsn").write_bytes(b"design")
        (output / "fixture.trn").write_bytes(b"transitions")
        return subprocess.CompletedProcess(command, 0, "", "")

    def test_cache_reuses_complete_output_and_invalidates_source_and_reader_changes(self) -> None:
        with patch.object(prepare.subprocess, "run", side_effect=self.convert) as run:
            prepare.prepare(self.source, self.output, self.converter)
            prepare.prepare(self.source, self.output, self.converter)
            self.assertEqual(run.call_count, 1)
            self.source.write_text("changed source")
            prepare.prepare(self.source, self.output, self.converter)
            self.converter.write_text("new converter")
            prepare.prepare(self.source, self.output, self.converter)
            self.assertEqual(run.call_count, 3)
            (self.output / "fixture.trn").unlink()
            prepare.prepare(self.source, self.output, self.converter)
            self.assertEqual(run.call_count, 4)

    def test_failed_conversion_preserves_existing_database(self) -> None:
        with patch.object(prepare.subprocess, "run", side_effect=self.convert):
            prepare.prepare(self.source, self.output, self.converter)
        original = (self.output / prepare.STAMP).read_text()
        self.source.write_text("new input")
        failure = subprocess.CompletedProcess([], 1, "", "converter failure")
        with patch.object(prepare.subprocess, "run", return_value=failure):
            with self.assertRaisesRegex(RuntimeError, "converter failure"):
                prepare.prepare(self.source, self.output, self.converter)
        self.assertEqual((self.output / prepare.STAMP).read_text(), original)
        self.assertTrue(prepare.complete_database(self.output))

    def test_rejects_unowned_database_and_incomplete_converter_output(self) -> None:
        self.output.mkdir()
        with self.assertRaisesRegex(RuntimeError, "unowned database"):
            prepare.prepare(self.source, self.output, self.converter)
        (self.output / prepare.STAMP).write_text(json.dumps({}))
        with patch.object(prepare.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "", "")):
            with self.assertRaisesRegex(RuntimeError, "conversion failed"):
                prepare.prepare(self.source, self.output, self.converter)


if __name__ == "__main__":
    unittest.main()
