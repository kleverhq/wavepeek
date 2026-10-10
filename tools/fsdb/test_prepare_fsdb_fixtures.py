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
            ondas_fixtures = repo / "ondas-fixtures"
            bin_dir = sandbox / "bin"
            root_log = repo / "vcd2fsdbLog"
            sentinel = root_log / "sentinel.txt"

            script.parent.mkdir(parents=True)
            script.write_text(SCRIPT_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            os.chmod(script, 0o755)
            (repo / ".devcontainer").mkdir()
            (repo / ".devcontainer" / "env_contract.sh").write_text(
                f'ONDAS_FIXTURES_DIR="{ondas_fixtures}"\nWAVEPEEK_ONDAS_FIXTURES=""\n',
                encoding="utf-8",
            )
            hand_fixtures.mkdir(parents=True)
            ondas_fixtures.mkdir()
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

    def test_rtl_reuses_persistent_vcd_when_available(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo = pathlib.Path(temp_dir) / "repo"
            script = repo / "tools/fsdb/prepare_fsdb_fixtures.sh"
            script.parent.mkdir(parents=True)
            script.write_text(SCRIPT_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            (repo / ".devcontainer").mkdir()
            artifacts = repo / "artifacts"
            fixture = artifacts / "fst/fst0000-sample"
            fixture.mkdir(parents=True)
            (repo / ".devcontainer/env_contract.sh").write_text(
                f'ONDAS_FIXTURES_DIR="{artifacts}"\n'
                'WAVEPEEK_ONDAS_FIXTURES="fst/fst0000-sample"\n',
                encoding="utf-8",
            )
            source = fixture / "waveform.fst"
            source.write_text("fst", encoding="utf-8")
            vcd = fixture / "waveform.vcd"
            vcd.write_text("persistent vcd", encoding="utf-8")
            os.utime(vcd, (source.stat().st_mtime + 2,) * 2)
            bin_dir = repo / "bin"
            bin_dir.mkdir()
            converter = bin_dir / "vcd2fsdb"
            converter.write_text(
                '#!/bin/sh\n[ "$1" = "' + str(vcd) + '" ] || exit 1\n'
                'printf "fsdb" > "$3"\n',
                encoding="utf-8",
            )
            converter.chmod(0o755)
            result = subprocess.run(
                ["bash", str(script), "--rtl-only"],
                cwd=repo,
                capture_output=True,
                text=True,
                env={**os.environ, "PATH": f"{bin_dir}:{os.environ['PATH']}"},
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((fixture / "waveform.fsdb").read_text(), "fsdb")
            self.assertEqual(vcd.read_text(), "persistent vcd")

    def test_rtl_filter_limits_converted_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            sandbox = pathlib.Path(temp_dir)
            repo = sandbox / "repo"
            script = repo / "tools" / "fsdb" / "prepare_fsdb_fixtures.sh"
            ondas_fixtures = repo / "ondas-fixtures"
            bin_dir = sandbox / "bin"

            script.parent.mkdir(parents=True)
            script.write_text(SCRIPT_PATH.read_text(encoding="utf-8"), encoding="utf-8")
            os.chmod(script, 0o755)
            (repo / ".devcontainer").mkdir()
            (repo / ".devcontainer" / "env_contract.sh").write_text(
                f'ONDAS_FIXTURES_DIR="{ondas_fixtures}"\n'
                'WAVEPEEK_ONDAS_FIXTURES="fst/fst0000-needed fst/fst0001-ignored"\n',
                encoding="utf-8",
            )
            needed = ondas_fixtures / "fst/fst0000-needed"
            ignored = ondas_fixtures / "fst/fst0001-ignored"
            needed.mkdir(parents=True)
            ignored.mkdir()
            bin_dir.mkdir()
            (needed / "waveform.fst").write_text("needed\n", encoding="utf-8")
            (ignored / "waveform.fst").write_text("ignored\n", encoding="utf-8")
            (bin_dir / "fst2vcd").write_text(
                "#!/usr/bin/env sh\nset -eu\nprintf '%s\\n' vcd\n",
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
                    "^fst0000-needed$",
                ],
                check=False,
                capture_output=True,
                text=True,
                cwd=repo,
                env=env,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((needed / "waveform.fsdb").is_file())
            self.assertFalse((ignored / "waveform.fsdb").exists())


if __name__ == "__main__":
    unittest.main()
