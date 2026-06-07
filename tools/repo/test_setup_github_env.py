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

    def assert_maintainer_env(self, home: Path) -> None:
        config_dir = home / ".config" / "wavepeek-dev"
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

    def test_creates_active_maintainer_env_for_clean_home(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            result = self.run_script(home)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn("TEST_TOKEN", result.stdout)
            self.assertNotIn("TEST_TOKEN", result.stderr)
            self.assert_maintainer_env(home)

    def test_allows_unrelated_managed_state_and_default_empty_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            config_dir = home / ".config" / "wavepeek-dev"
            config_dir.mkdir(parents=True)
            (config_dir / "codex").mkdir()
            (config_dir / "verdi").mkdir()
            (config_dir / "github.empty.env").write_text("")
            (config_dir / "github.env").symlink_to("github.empty.env")

            result = self.run_script(home)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((config_dir / "codex").is_dir())
            self.assertTrue((config_dir / "verdi").is_dir())
            self.assert_maintainer_env(home)

    def test_rejects_symlinked_config_dir_before_reading_token(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp) / "home"
            target = Path(tmp) / "target"
            (home / ".config").mkdir(parents=True)
            target.mkdir()
            (home / ".config" / "wavepeek-dev").symlink_to(target)

            result = self.run_script(home, input_text="")

            self.assertEqual(result.returncode, 1)
            self.assertIn("must be a real directory, not a symlink", result.stderr)
            self.assertFalse((target / "github.maintainer.env").exists())

    def test_rejects_existing_maintainer_env_before_reading_token(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            config_dir = home / ".config" / "wavepeek-dev"
            config_dir.mkdir(parents=True)
            maintainer_env = config_dir / "github.maintainer.env"
            maintainer_env.write_text("GH_TOKEN=KEEP\n")

            result = self.run_script(home, input_text="")

            self.assertEqual(result.returncode, 1)
            self.assertIn("already exists", result.stderr)
            self.assertIn("edit the GitHub auth env files manually", result.stderr)
            self.assertEqual(maintainer_env.read_text(), "GH_TOKEN=KEEP\n")

    def test_rejects_non_default_active_env_before_reading_token(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            config_dir = home / ".config" / "wavepeek-dev"
            config_dir.mkdir(parents=True)
            active_env = config_dir / "github.env"
            active_env.write_text("GH_TOKEN=KEEP\n")

            result = self.run_script(home, input_text="")

            self.assertEqual(result.returncode, 1)
            self.assertIn("is not the default github.empty.env symlink", result.stderr)
            self.assertEqual(active_env.read_text(), "GH_TOKEN=KEEP\n")
            self.assertFalse((config_dir / "github.maintainer.env").exists())

    def test_rejects_positional_token_argument(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            result = self.run_script(home, input_text="", extra_args=("TEST_TOKEN",))

            self.assertEqual(result.returncode, 2)
            self.assertIn("usage:", result.stderr)
            self.assertFalse((home / ".config" / "wavepeek-dev").exists())

    def test_rejects_empty_token(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            result = self.run_script(home, input_text="\n")

            self.assertEqual(result.returncode, 1)
            self.assertIn("must not be empty", result.stderr)
            self.assertFalse((home / ".config" / "wavepeek-dev").exists())


if __name__ == "__main__":
    unittest.main()
