from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from threading import Event
import unittest
from unittest import mock

import prepare_wlf_fixtures as prepare


class WlfToolsTest(unittest.TestCase):
    def test_environment_distinguishes_absent_and_incomplete_reader(self) -> None:
        script = Path(__file__).with_name("check_wlf_env.py")
        env = dict(os.environ)
        env.pop("QUESTA_HOME", None)
        for args, expected in (([], 77), (["--require"], 1)):
            self.assertEqual(subprocess.run(
                [sys.executable, str(script), *args], env=env, capture_output=True,
            ).returncode, expected)
        with tempfile.TemporaryDirectory() as directory:
            env["QUESTA_HOME"] = directory
            result = subprocess.run([sys.executable, str(script)], env=env, capture_output=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn(b"missing reader file", result.stderr)

    def test_unsupported_declaration_kinds_keep_identifiers_and_values(self) -> None:
        original = (
            b"$var logic 12 ! a [11:0] $end\n"
            b"$var longint 64 # b [63:0] $end\n"
            b"$var real_parameter 1 $ c $end\n"
            b"$var wire 1 % logic $end\n"
            b"#7\nb100 !\nb01 #\nr1.0 $\n0%\n"
        )
        expected = original.replace(b"$var logic ", b"$var reg ").replace(
            b"$var longint ", b"$var integer ",
        ).replace(b"$var real_parameter ", b"$var real ")
        output = subprocess.check_output(["sed", "-E", prepare.VCD_DECLARATIONS], input=original)
        self.assertEqual(output, expected)

    def test_generated_benchmark_and_regression_share_input_and_output(self) -> None:
        vcd = prepare.GENERATED / "extract_global_include.vcd"
        fst = vcd.with_suffix(".fst")
        self.assertEqual(prepare.fixture_paths(vcd), prepare.fixture_paths(fst))
        self.assertEqual(prepare.fixture_paths(fst), (vcd, prepare.WLF_FIXTURES / "extract_global_include.wlf"))

    def test_generated_input_identity_survives_identical_regeneration(self) -> None:
        with tempfile.TemporaryDirectory() as directory, mock.patch.object(
            prepare, "GENERATED", Path(directory),
        ):
            source = Path(directory) / "input.vcd"
            source.write_bytes(b"input VCD")
            first = prepare.input_identity(source)
            os.utime(source, ns=(0, 0))
            self.assertEqual(prepare.input_identity(source), first)
            source.write_bytes(b"new data")
            self.assertNotEqual(prepare.input_identity(source), first)

    def test_concurrent_preparation_reuses_the_completed_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, converter, output = (root / name for name in ("input.vcd", "converter", "input.wlf"))
            source.write_bytes(b"input")
            converter.write_bytes(b"converter")
            first_conversion = Event()
            finish_conversion = Event()

            def convert(command, **kwargs):
                first_conversion.set()
                self.assertTrue(finish_conversion.wait(timeout=5))
                Path(command[-1]).write_bytes(b"output")

            with (
                mock.patch.object(prepare.subprocess, "run", side_effect=convert) as conversion,
                ThreadPoolExecutor(max_workers=2) as pool,
            ):
                first = pool.submit(prepare.prepare, source, output, converter)
                self.assertTrue(first_conversion.wait(timeout=5))
                try:
                    with output.with_suffix(".wlf.lock").open("a") as lock:
                        with self.assertRaises(BlockingIOError):
                            prepare.fcntl.flock(lock, prepare.fcntl.LOCK_SH | prepare.fcntl.LOCK_NB)
                    second = pool.submit(prepare.prepare, source, output, converter)
                finally:
                    finish_conversion.set()
                first.result(timeout=5)
                second.result(timeout=5)
                self.assertEqual(conversion.call_count, 1)
            self.assertEqual(output.read_bytes(), b"output")
            stamp = json.loads(output.with_suffix(".wlf.source.json").read_text())
            self.assertEqual(stamp["input"], prepare.input_identity(source))


if __name__ == "__main__":
    unittest.main()
