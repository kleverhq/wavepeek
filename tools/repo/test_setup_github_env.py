import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "tools" / "repo" / "setup_github_env.sh"


class SetupGithubEnvTests(unittest.TestCase):
    def run_script(
        self,
        home: Path,
        input_text: str = "TEST_TOKEN\n",
        extra_args: tuple[str, ...] = (),
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["HOME"] = str(home)
        env.pop("WAVEPEEK_GITHUB_CONFIG_DIR", None)
        return subprocess.run(
            ["bash", str(SCRIPT), *extra_args],
            cwd=REPO_ROOT,
            env=env,
            input=input_text,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def test_creates_active_maintainer_env_for_clean_home(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            result = self.run_script(home)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn("TEST_TOKEN", result.stdout)
            self.assertNotIn("TEST_TOKEN", result.stderr)

            config_dir = home / ".config" / "wavepeek"
            empty_env = config_dir / "github.empty.env"
            maintainer_env = config_dir / "github.maintainer.env"
            active_env = config_dir / "github.env"

            self.assertEqual(empty_env.read_text(), "")
            self.assertEqual(
                maintainer_env.read_text(),
                "GH_TOKEN=TEST_TOKEN\n"
                "GITHUB_TOKEN=TEST_TOKEN\n"
                "WAVEPEEK_GITHUB_ROLE=maintainer\n",
            )
            self.assertTrue(active_env.is_symlink())
            self.assertEqual(os.readlink(active_env), "github.maintainer.env")
            self.assertEqual(stat.S_IMODE(config_dir.stat().st_mode), 0o700)
            self.assertEqual(stat.S_IMODE(maintainer_env.stat().st_mode), 0o600)

    def test_rejects_non_empty_config_dir_before_reading_token(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            config_dir = home / ".config" / "wavepeek"
            config_dir.mkdir(parents=True)
            marker = config_dir / "existing"
            marker.write_text("keep\n")

            result = self.run_script(home, input_text="")

            self.assertEqual(result.returncode, 1)
            self.assertIn("exists and is not empty", result.stderr)
            self.assertIn("edit the GitHub auth env files manually", result.stderr)
            self.assertEqual(marker.read_text(), "keep\n")
            self.assertFalse((config_dir / "github.maintainer.env").exists())

    def test_rejects_positional_token_argument(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            result = self.run_script(home, input_text="", extra_args=("TEST_TOKEN",))

            self.assertEqual(result.returncode, 2)
            self.assertIn("usage:", result.stderr)
            self.assertFalse((home / ".config" / "wavepeek").exists())

    def test_rejects_empty_token(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            result = self.run_script(home, input_text="\n")

            self.assertEqual(result.returncode, 1)
            self.assertIn("must not be empty", result.stderr)
            self.assertFalse((home / ".config" / "wavepeek").exists())


if __name__ == "__main__":
    unittest.main()
