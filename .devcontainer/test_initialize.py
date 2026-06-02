#!/usr/bin/env python3

from __future__ import annotations

import json
import os
import pathlib
import shlex
import subprocess
import tempfile
import textwrap
import unittest

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT_PATH = pathlib.Path(__file__).with_name("initialize.sh")
DEVCONTAINER_PATH = pathlib.Path(__file__).with_name("devcontainer.json")


class DevcontainerInitializeTest(unittest.TestCase):
    def run_initialize(
        self,
        home: pathlib.Path,
        *,
        env_overrides: dict[str, str] | None = None,
        args: list[str] | None = None,
    ) -> subprocess.CompletedProcess[str]:
        env = {
            key: value
            for key, value in os.environ.items()
            if key not in {"HOME", "HOST_VERDI_HOME", "VERDI_HOME", "SHELL"}
        }
        env["HOME"] = str(home)
        env.update(env_overrides or {})
        return subprocess.run(
            ["bash", str(SCRIPT_PATH), *(args or [])],
            cwd=REPO_ROOT,
            env=env,
            check=False,
            capture_output=True,
            text=True,
        )

    def test_uses_host_verdi_home_env(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            home = root / "home"
            verdi_home = root / "verdi sdk"
            verdi_home.mkdir(parents=True)

            result = self.run_initialize(home, env_overrides={"VERDI_HOME": str(verdi_home)})

            self.assertEqual(result.returncode, 0, result.stderr)
            mount_source = home / ".cache" / "wavepeek" / "verdi"
            self.assertTrue(mount_source.is_symlink())
            self.assertEqual(mount_source.resolve(), verdi_home.resolve())

    def test_does_not_probe_login_shell_for_verdi_home(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = pathlib.Path(temp_dir)
            home = root / "home"
            marker = root / "shell-was-called"
            fake_shell = root / "fake-shell"
            fake_verdi_home = root / "verdi-from-shell"
            fake_verdi_home.mkdir()
            fake_shell.write_text(
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env bash
                    printf shell-called > {shlex.quote(str(marker))}
                    printf '%s' {shlex.quote(str(fake_verdi_home))}
                    """
                ),
                encoding="utf-8",
            )
            fake_shell.chmod(0o755)

            result = self.run_initialize(home, env_overrides={"SHELL": str(fake_shell)})

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(marker.exists(), "initialize.sh must not execute the host login shell")
            mount_source = home / ".cache" / "wavepeek" / "verdi"
            self.assertTrue(mount_source.is_dir())
            self.assertFalse(mount_source.is_symlink())

    def test_devcontainer_runs_initialize_without_arguments(self) -> None:
        devcontainer = json.loads(DEVCONTAINER_PATH.read_text(encoding="utf-8"))

        self.assertEqual(
            devcontainer["initializeCommand"],
            [
                "bash",
                "${localWorkspaceFolder}/.devcontainer/initialize.sh",
            ],
        )


if __name__ == "__main__":
    unittest.main()
