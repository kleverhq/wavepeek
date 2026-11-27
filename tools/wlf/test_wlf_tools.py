import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

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


if __name__ == "__main__":
    unittest.main()
