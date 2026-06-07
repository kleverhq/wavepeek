import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / ".devcontainer" / "initialize.sh"


class DevcontainerInitializeTests(unittest.TestCase):
    def run_script(
        self,
        home: Path,
        verdi_home: Path | None = None,
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["HOME"] = str(home)
        if verdi_home is None:
            env.pop("VERDI_HOME", None)
        else:
            env["VERDI_HOME"] = str(verdi_home)
        return subprocess.run(
            ["bash", str(SCRIPT)],
            cwd=REPO_ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def test_clean_home_uses_only_wavepeek_dev_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            result = self.run_script(home)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse((home / ".claude").exists())
            self.assertFalse((home / ".claude.json").exists())
            self.assertFalse((home / ".codex").exists())
            self.assertFalse((home / ".pi").exists())
            self.assertFalse((home / ".cache" / "wavepeek").exists())

            config_dir = home / ".config" / "wavepeek-dev"
            self.assertTrue((config_dir / "claude").is_dir())
            self.assertEqual((config_dir / "claude.json").read_text(), "{}\n")
            self.assertTrue((config_dir / "codex").is_dir())
            self.assertTrue((config_dir / "pi").is_dir())
            self.assertTrue((config_dir / "verdi").is_dir())
            self.assertEqual((config_dir / "github.empty.env").read_text(), "")
            self.assertTrue((config_dir / "github.env").is_symlink())
            self.assertEqual(os.readlink(config_dir / "github.env"), "github.empty.env")

    def test_existing_legacy_agent_state_is_linked_from_wavepeek_dev(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            (home / ".claude").mkdir()
            (home / ".claude.json").write_text('{"theme":"grim"}\n')
            (home / ".codex").mkdir()
            (home / ".pi").mkdir()

            result = self.run_script(home)

            self.assertEqual(result.returncode, 0, result.stderr)
            config_dir = home / ".config" / "wavepeek-dev"
            for name, legacy in (
                ("claude", ".claude"),
                ("claude.json", ".claude.json"),
                ("codex", ".codex"),
                ("pi", ".pi"),
            ):
                managed_path = config_dir / name
                self.assertTrue(managed_path.is_symlink(), name)
                self.assertEqual(os.readlink(managed_path), str(home / legacy))

    def test_wrong_type_legacy_state_falls_back_to_managed_placeholders(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            (home / ".codex").write_text("not a directory\n")
            (home / ".claude.json").mkdir()

            result = self.run_script(home)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("ignoring", result.stderr)
            config_dir = home / ".config" / "wavepeek-dev"
            self.assertTrue((config_dir / "codex").is_dir())
            self.assertFalse((config_dir / "codex").is_symlink())
            self.assertEqual((config_dir / "claude.json").read_text(), "{}\n")
            self.assertFalse((config_dir / "claude.json").is_symlink())

    def test_empty_file_managed_agent_dir_fails_without_deleting_data(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            managed_codex = home / ".config" / "wavepeek-dev" / "codex"
            managed_codex.parent.mkdir(parents=True)
            managed_codex.write_text("")

            result = self.run_script(home)

            self.assertEqual(result.returncode, 1)
            self.assertIn("exists but is not a dir", result.stderr)
            self.assertEqual(managed_codex.read_text(), "")

    def test_empty_dir_managed_agent_file_fails_without_deleting_data(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            managed_claude_json = home / ".config" / "wavepeek-dev" / "claude.json"
            managed_claude_json.mkdir(parents=True)

            result = self.run_script(home)

            self.assertEqual(result.returncode, 1)
            self.assertIn("exists but is not a file", result.stderr)
            self.assertTrue(managed_claude_json.is_dir())

    def test_invalid_managed_agent_symlink_fails_without_replacing_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            config_dir = home / ".config" / "wavepeek-dev"
            target_file = home / "codex-target-file"
            managed_codex = config_dir / "codex"
            config_dir.mkdir(parents=True)
            target_file.write_text("not a directory\n")
            managed_codex.symlink_to(target_file)

            result = self.run_script(home)

            self.assertEqual(result.returncode, 1)
            self.assertIn("invalid symlink", result.stderr)
            self.assertTrue(managed_codex.is_symlink())
            self.assertEqual(os.readlink(managed_codex), str(target_file))
            self.assertEqual(target_file.read_text(), "not a directory\n")

    def test_rejects_symlinked_wavepeek_dev_config_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp) / "home"
            target = Path(tmp) / "target"
            (home / ".config").mkdir(parents=True)
            target.mkdir()
            (home / ".config" / "wavepeek-dev").symlink_to(target)

            result = self.run_script(home)

            self.assertEqual(result.returncode, 1)
            self.assertIn("must be a real directory, not a symlink", result.stderr)
            self.assertFalse((target / "github.empty.env").exists())

    def test_rejects_symlinked_github_empty_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            config_dir = home / ".config" / "wavepeek-dev"
            outside = home / "outside.env"
            config_dir.mkdir(parents=True)
            outside.write_text("GH_TOKEN=KEEP\n")
            (config_dir / "github.empty.env").symlink_to(outside)

            result = self.run_script(home)

            self.assertEqual(result.returncode, 1)
            self.assertIn("must be a regular empty file, not a symlink", result.stderr)
            self.assertEqual(outside.read_text(), "GH_TOKEN=KEEP\n")

    def test_verdi_home_is_linked_and_stale_symlink_becomes_empty_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp) / "home"
            verdi_home = Path(tmp) / "verdi"
            home.mkdir()
            verdi_home.mkdir()

            result = self.run_script(home, verdi_home=verdi_home)
            self.assertEqual(result.returncode, 0, result.stderr)
            verdi_source = home / ".config" / "wavepeek-dev" / "verdi"
            self.assertTrue(verdi_source.is_symlink())
            self.assertEqual(os.readlink(verdi_source), str(verdi_home))

            result = self.run_script(home)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(verdi_source.is_symlink())
            self.assertTrue(verdi_source.is_dir())

    def test_verdi_home_same_as_managed_symlink_does_not_create_self_link(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp) / "home"
            real_verdi = Path(tmp) / "real-verdi"
            home.mkdir()
            real_verdi.mkdir()
            verdi_source = home / ".config" / "wavepeek-dev" / "verdi"
            verdi_source.parent.mkdir(parents=True)
            verdi_source.symlink_to(real_verdi)

            result = self.run_script(home, verdi_home=verdi_source)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(verdi_source.is_symlink())
            self.assertEqual(os.readlink(verdi_source), str(real_verdi))

    def test_non_empty_managed_verdi_dir_is_not_deleted(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp) / "home"
            host_verdi = Path(tmp) / "host-verdi"
            home.mkdir()
            host_verdi.mkdir()
            managed_verdi = home / ".config" / "wavepeek-dev" / "verdi"
            managed_verdi.mkdir(parents=True)
            marker = managed_verdi / "keep"
            marker.write_text("do not eat configs\n")

            result = self.run_script(home, verdi_home=host_verdi)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("non-empty", result.stderr)
            self.assertFalse(managed_verdi.is_symlink())
            self.assertEqual(marker.read_text(), "do not eat configs\n")

    def test_wrong_type_managed_verdi_source_fails_without_deleting_data(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            managed_verdi = home / ".config" / "wavepeek-dev" / "verdi"
            managed_verdi.parent.mkdir(parents=True)
            managed_verdi.write_text("not a directory\n")

            result = self.run_script(home)

            self.assertEqual(result.returncode, 1)
            self.assertIn("exists but is not a directory", result.stderr)
            self.assertEqual(managed_verdi.read_text(), "not a directory\n")


if __name__ == "__main__":
    unittest.main()
