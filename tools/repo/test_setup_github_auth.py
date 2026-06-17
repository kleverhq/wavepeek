import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / ".devcontainer" / "setup-github-auth.sh"
UPSTREAM_REPO = "kleverhq/wavepeek"
HELPER_PREFIX = "!wavepeek_github_credential_helper"
GIT_LOCAL_ENV = (
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_WORK_TREE",
)


def env_without_git_local() -> dict[str, str]:
    env = os.environ.copy()
    for name in GIT_LOCAL_ENV:
        env.pop(name, None)
    return env


class SetupGithubAuthTests(unittest.TestCase):
    def init_repo(self, tmp: str) -> Path:
        repo = Path(tmp) / "repo"
        repo.mkdir()
        git_env = env_without_git_local()
        subprocess.run(["git", "init", "-q", "-b", "main"], cwd=repo, check=True, env=git_env)
        subprocess.run(
            ["git", "remote", "add", "origin", f"https://github.com/{UPSTREAM_REPO}"],
            cwd=repo,
            check=True,
            env=git_env,
        )
        return repo

    def run_script(self, repo: Path, token: str | None = "TEST_TOKEN") -> subprocess.CompletedProcess[str]:
        fake_bin = repo.parent / "fake-bin"
        fake_bin.mkdir(exist_ok=True)
        fake_gh = fake_bin / "gh"
        fake_gh.write_text("#!/usr/bin/env sh\nexit 1\n")
        fake_gh.chmod(0o755)

        env = env_without_git_local()
        env["PATH"] = f"{fake_bin}{os.pathsep}{env['PATH']}"
        env["WAVEPEEK_UPSTREAM_REPO"] = UPSTREAM_REPO
        env["GIT_TERMINAL_PROMPT"] = "0"
        env["GIT_ASKPASS"] = ""
        if token is None:
            env.pop("GH_TOKEN", None)
            env.pop("GITHUB_TOKEN", None)
        else:
            env["GH_TOKEN"] = token
            env["GITHUB_TOKEN"] = token

        return subprocess.run(
            ["bash", str(SCRIPT)],
            cwd=repo,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def git_config_get_all(self, repo: Path, key: str) -> list[str]:
        result = subprocess.run(
            ["git", "config", "--local", "--get-all", key],
            cwd=repo,
            env=env_without_git_local(),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if result.returncode == 1:
            return []
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout.splitlines()

    def test_token_installs_path_specific_helpers_with_reset_entries(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = self.init_repo(tmp)

            result = self.run_script(repo)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                self.git_config_get_all(
                    repo, f"credential.https://github.com/{UPSTREAM_REPO}.helper"
                )[0],
                "",
            )
            self.assertTrue(
                self.git_config_get_all(
                    repo, f"credential.https://github.com/{UPSTREAM_REPO}.helper"
                )[1].startswith(HELPER_PREFIX)
            )
            self.assertEqual(
                self.git_config_get_all(
                    repo, f"credential.https://github.com/{UPSTREAM_REPO}.git.helper"
                )[0],
                "",
            )
            self.assertTrue(
                self.git_config_get_all(
                    repo, f"credential.https://github.com/{UPSTREAM_REPO}.git.helper"
                )[1].startswith(HELPER_PREFIX)
            )
            self.assertEqual(
                self.git_config_get_all(repo, "credential.https://github.com.helper"), []
            )

    def test_path_specific_reset_preempts_stale_broader_helper(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = self.init_repo(tmp)
            subprocess.run(
                [
                    "git",
                    "config",
                    "credential.helper",
                    "!f(){ echo username=stale; echo password=stale; }; f",
                ],
                cwd=repo,
                check=True,
                env=env_without_git_local(),
            )
            result = self.run_script(repo)
            self.assertEqual(result.returncode, 0, result.stderr)

            env = env_without_git_local()
            env["GH_TOKEN"] = "TEST_TOKEN"
            env["GITHUB_TOKEN"] = "TEST_TOKEN"
            env["WAVEPEEK_UPSTREAM_REPO"] = UPSTREAM_REPO
            env["GIT_TERMINAL_PROMPT"] = "0"
            env["GIT_ASKPASS"] = ""
            fill = subprocess.run(
                ["git", "credential", "fill"],
                cwd=repo,
                env=env,
                input=(
                    "protocol=https\n"
                    "host=github.com\n"
                    f"path={UPSTREAM_REPO}.git\n"
                    "\n"
                ),
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

            self.assertEqual(fill.returncode, 0, fill.stderr)
            self.assertIn("username=x-access-token\n", fill.stdout)
            self.assertIn("password=TEST_TOKEN\n", fill.stdout)
            self.assertNotIn("stale", fill.stdout)

    def test_no_token_removes_previous_installed_helpers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = self.init_repo(tmp)
            subprocess.run(
                [
                    "git",
                    "config",
                    "--add",
                    "credential.https://github.com.helper",
                    "!wavepeek_github_credential_helper old",
                ],
                cwd=repo,
                check=True,
                env=env_without_git_local(),
            )
            subprocess.run(
                [
                    "git",
                    "config",
                    "--add",
                    f"credential.https://github.com/{UPSTREAM_REPO}.helper",
                    "",
                ],
                cwd=repo,
                check=True,
                env=env_without_git_local(),
            )
            subprocess.run(
                [
                    "git",
                    "config",
                    "--add",
                    f"credential.https://github.com/{UPSTREAM_REPO}.helper",
                    "!wavepeek_github_credential_helper old",
                ],
                cwd=repo,
                check=True,
                env=env_without_git_local(),
            )

            result = self.run_script(repo, token=None)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                self.git_config_get_all(repo, "credential.https://github.com.helper"), []
            )
            self.assertEqual(
                self.git_config_get_all(
                    repo, f"credential.https://github.com/{UPSTREAM_REPO}.helper"
                ),
                [],
            )


if __name__ == "__main__":
    unittest.main()
