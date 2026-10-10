import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRE_COMMIT_SKIPS = (
    "rust-format,rust-lint,rust-check,justfile-format-check,"
    "github-actions-lint,rust-test,aux-test,bench-e2e-smoke-commit"
)


@unittest.skipUnless(
    os.environ.get("WAVEPEEK_RUN_DOCKER_HOOK_SMOKE") == "1"
    and not os.environ.get("WAVEPEEK_IN_CONTAINER"),
    "set WAVEPEEK_RUN_DOCKER_HOOK_SMOKE=1 on the host",
)
class DockerGitHookSmokeTests(unittest.TestCase):
    def test_main_and_linked_worktree_host_commits(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            main = root / "main"
            linked = root / "linked"
            env = {
                name: value
                for name, value in os.environ.items()
                if not name.startswith(("GIT_", "WAVEPEEK_FSDB_"))
                and name not in {"VERDI_HOME", "WAVEPEEK_DEV_CONFIG"}
            }
            containers: list[str] = []
            try:
                branch = subprocess.run(
                    ["git", "branch", "--show-current"],
                    cwd=REPO_ROOT,
                    env=env,
                    check=True,
                    capture_output=True,
                    text=True,
                ).stdout.strip()
                subprocess.run(
                    [
                        "git",
                        "clone",
                        "-q",
                        "--no-local",
                        "--branch",
                        branch,
                        str(REPO_ROOT),
                        str(main),
                    ],
                    env=env,
                    check=True,
                )
                self._git(main, env, "config", "user.name", "Hook Smoke")
                self._git(main, env, "config", "user.email", "hook@example.com")
                self._git(
                    main,
                    env,
                    "worktree",
                    "add",
                    "-qb",
                    "hook-smoke-linked",
                    str(linked),
                )

                for worktree in (main, linked):
                    self._run(worktree, env, "./dev", "true")
                    ids = subprocess.run(
                        [
                            "docker",
                            "ps",
                            "-q",
                            "--filter",
                            f"label=dev.wavepeek.worktree={worktree}",
                        ],
                        env=env,
                        check=True,
                        capture_output=True,
                        text=True,
                    ).stdout.split()
                    self.assertEqual(len(ids), 1)
                    containers.extend(ids)
                self.assertEqual(len(set(containers)), 2)

                for worktree in (main, linked):
                    self._run(worktree, env, "./dev", "--install-hooks")
                main_hooks = self._git(
                    main, env, "config", "--worktree", "--get", "core.hooksPath"
                ).stdout.strip()
                linked_hooks = self._git(
                    linked, env, "config", "--worktree", "--get", "core.hooksPath"
                ).stdout.strip()
                self.assertNotEqual(main_hooks, linked_hooks)
                commit_env = env | {"SKIP": PRE_COMMIT_SKIPS}
                for worktree, name in ((main, "main"), (linked, "linked")):
                    (worktree / f"{name}-hook-smoke").write_text("smoke\n")
                    self._git(worktree, commit_env, "add", f"{name}-hook-smoke")
                    result = self._git(
                        worktree,
                        commit_env,
                        "commit",
                        "-m",
                        f"test: {name} hook smoke",
                    )
                    self.assertIn("Commit style check", result.stdout + result.stderr)

                invalid = subprocess.run(
                    ["git", "commit", "--allow-empty", "-m", "invalid"],
                    cwd=linked,
                    env=commit_env,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=False,
                )
                self.assertNotEqual(invalid.returncode, 0)
                self.assertIn("Commit style check", invalid.stdout + invalid.stderr)
            finally:
                if containers:
                    subprocess.run(
                        ["docker", "rm", "-f", *set(containers)],
                        env=env,
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL,
                        check=False,
                    )

    @staticmethod
    def _run(cwd: Path, env: dict[str, str], *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            list(args),
            cwd=cwd,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )

    @staticmethod
    def _git(
        cwd: Path, env: dict[str, str], *args: str
    ) -> subprocess.CompletedProcess[str]:
        return DockerGitHookSmokeTests._run(cwd, env, "git", *args)


if __name__ == "__main__":
    unittest.main()
