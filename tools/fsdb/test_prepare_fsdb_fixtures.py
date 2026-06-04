#!/usr/bin/env python3

from __future__ import annotations

import os
import pathlib
import subprocess
import tempfile
import textwrap
import unittest

SCRIPT_PATH = pathlib.Path(__file__).with_name("prepare_fsdb_fixtures.sh")


class PrepareFsdbFixturesTest(unittest.TestCase):
    def test_preserves_pre_existing_repo_root_vcd2fsdb_log(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            sandbox = pathlib.Path(temp_dir)
            repo = sandbox / "repo"
            script = repo / "tools" / "fsdb" / "prepare_fsdb_fixtures.sh"
            hand_fixtures = repo / "tests" / "fixtures" / "hand"
            rtl_artifacts = repo / "rtl-artifacts"
            bin_dir = sandbox / "bin"
            root_log = repo / "vcd2fsdbLog"
            sentinel = root_log / "sentinel.txt"

            script.parent.mkdir(parents=True)
            script.write_text(SCRIPT_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            os.chmod(script, 0o755)
            (repo / ".devcontainer").mkdir()
            (repo / ".devcontainer" / "env_contract.sh").write_text(
                f'RTL_ARTIFACTS_DIR="{rtl_artifacts}"\n', encoding="utf-8"
            )
            hand_fixtures.mkdir(parents=True)
            rtl_artifacts.mkdir()
            bin_dir.mkdir()
            (hand_fixtures / "tiny.vcd").write_text(
                "$date today $end\n$enddefinitions $end\n",
                encoding="utf-8",
            )
            root_log.mkdir()
            sentinel.write_text("user-owned data\n", encoding="utf-8")
            (bin_dir / "vcd2fsdb").write_text(
                textwrap.dedent(
                    """\
                    #!/usr/bin/env sh
                    set -eu
                    output=""
                    while [ "$#" -gt 0 ]; do
                        if [ "$1" = "-o" ]; then
                            shift
                            output="$1"
                        fi
                        shift || true
                    done
                    if [ -z "$output" ]; then
                        printf '%s\n' 'missing -o' >&2
                        exit 2
                    fi
                    mkdir -p "$(dirname "$output")" vcd2fsdbLog
                    printf '%s\n' fsdb > "$output"
                    printf '%s\n' stub-log > vcd2fsdbLog/created-by-stub.txt
                    """
                ),
                encoding="utf-8",
            )
            os.chmod(bin_dir / "vcd2fsdb", 0o755)

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}{os.pathsep}{env['PATH']}"
            result = subprocess.run(
                ["bash", str(script)],
                check=False,
                capture_output=True,
                text=True,
                cwd=repo,
                env=env,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(sentinel.read_text(encoding="utf-8"), "user-owned data\n")
            self.assertFalse((root_log / "created-by-stub.txt").exists())
            generated_fixture = repo / "tests" / "fixtures" / "fsdb" / "tiny.fsdb"
            self.assertTrue(generated_fixture.is_file())

    def test_rtl_filter_limits_converted_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            sandbox = pathlib.Path(temp_dir)
            repo = sandbox / "repo"
            script = repo / "tools" / "fsdb" / "prepare_fsdb_fixtures.sh"
            rtl_artifacts = repo / "rtl-artifacts"
            bin_dir = sandbox / "bin"

            script.parent.mkdir(parents=True)
            script.write_text(SCRIPT_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            os.chmod(script, 0o755)
            (repo / ".devcontainer").mkdir()
            (repo / ".devcontainer" / "env_contract.sh").write_text(
                f'RTL_ARTIFACTS_DIR="{rtl_artifacts}"\n', encoding="utf-8"
            )
            rtl_artifacts.mkdir()
            bin_dir.mkdir()
            (rtl_artifacts / "needed.fst").write_text("needed\n", encoding="utf-8")
            (rtl_artifacts / "ignored.fst").write_text("ignored\n", encoding="utf-8")
            (bin_dir / "fst2vcd").write_text(
                textwrap.dedent(
                    """\
                    #!/usr/bin/env sh
                    set -eu
                    output=""
                    while [ "$#" -gt 0 ]; do
                        if [ "$1" = "-o" ]; then
                            shift
                            output="$1"
                        fi
                        shift || true
                    done
                    if [ -z "$output" ]; then
                        printf '%s\n' 'missing -o' >&2
                        exit 2
                    fi
                    mkdir -p "$(dirname "$output")"
                    printf '%s\n' vcd > "$output"
                    """
                ),
                encoding="utf-8",
            )
            (bin_dir / "vcd2fsdb").write_text(
                textwrap.dedent(
                    """\
                    #!/usr/bin/env sh
                    set -eu
                    output=""
                    while [ "$#" -gt 0 ]; do
                        if [ "$1" = "-o" ]; then
                            shift
                            output="$1"
                        fi
                        shift || true
                    done
                    if [ -z "$output" ]; then
                        printf '%s\n' 'missing -o' >&2
                        exit 2
                    fi
                    mkdir -p "$(dirname "$output")"
                    printf '%s\n' fsdb > "$output"
                    """
                ),
                encoding="utf-8",
            )
            os.chmod(bin_dir / "fst2vcd", 0o755)
            os.chmod(bin_dir / "vcd2fsdb", 0o755)

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}{os.pathsep}{env['PATH']}"
            result = subprocess.run(
                [
                    "bash",
                    str(script),
                    "--rtl-only",
                    "--rtl-filter",
                    "^needed[.]fst$",
                ],
                check=False,
                capture_output=True,
                text=True,
                cwd=repo,
                env=env,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((rtl_artifacts / "needed.fsdb").is_file())
            self.assertFalse((rtl_artifacts / "ignored.fsdb").exists())


if __name__ == "__main__":
    unittest.main()
