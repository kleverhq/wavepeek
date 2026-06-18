#!/usr/bin/env python3

from __future__ import annotations

import json
import os
import pathlib
import subprocess
import tempfile
import textwrap
import unittest

SCRIPT_PATH = pathlib.Path(__file__).with_name("publish_crate.py")


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


class PublishCrateCliTest(unittest.TestCase):
    def release_json(self, path: pathlib.Path, *, draft: bool = False, prerelease: bool = False) -> pathlib.Path:
        path.write_text(
            json.dumps({"tag_name": "v1.2.3", "draft": draft, "prerelease": prerelease}),
            encoding="utf-8",
        )
        return path

    def crate_json(self, path: pathlib.Path, *, published: bool) -> pathlib.Path:
        path.write_text(json.dumps({"published": published}), encoding="utf-8")
        return path

    def base_env(self) -> dict[str, str]:
        env = env_without_git_local()
        env.pop("CARGO_REGISTRY_TOKEN", None)
        env.pop("CRATES_IO_TOKEN", None)
        env.update(
            {
                "VERSION": "1.2.3",
                "SOURCE_REF": "v1.2.3",
                "DISPATCH_REF": "main",
                "DEFAULT_BRANCH": "main",
                "GITHUB_REPOSITORY": "kleverhq/wavepeek",
            }
        )
        return env

    def run_script(
        self,
        args: list[str],
        *,
        env: dict[str, str] | None = None,
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(SCRIPT_PATH), *args],
            check=False,
            capture_output=True,
            text=True,
            env=self.base_env() if env is None else env,
        )

    def init_release_source(self, path: pathlib.Path, *, extra_commit: bool = False) -> pathlib.Path:
        path.mkdir()
        (path / "Cargo.toml").write_text(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3"
                edition = "2024"
                """
            ),
            encoding="utf-8",
        )
        git_env = env_without_git_local()
        subprocess.run(["git", "init", "-q"], cwd=path, check=True, env=git_env)
        subprocess.run(["git", "add", "Cargo.toml"], cwd=path, check=True, env=git_env)
        subprocess.run(
            [
                "git",
                "-c",
                "user.email=test@example.com",
                "-c",
                "user.name=Test User",
                "commit",
                "-q",
                "-m",
                "initial",
            ],
            cwd=path,
            check=True,
            env=git_env,
        )
        subprocess.run(["git", "tag", "v1.2.3"], cwd=path, check=True, env=git_env)
        if extra_commit:
            (path / "README.md").write_text("later\n", encoding="utf-8")
            subprocess.run(["git", "add", "README.md"], cwd=path, check=True, env=git_env)
            subprocess.run(
                [
                    "git",
                    "-c",
                    "user.email=test@example.com",
                    "-c",
                    "user.name=Test User",
                    "commit",
                    "-q",
                    "-m",
                    "later",
                ],
                cwd=path,
                check=True,
                env=git_env,
            )
        return path

    def test_validate_dispatch_accepts_default_branch_release(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            release_path = self.release_json(temp_path / "release.json")
            result = self.run_script(["validate-dispatch", "--release-json", str(release_path)])

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("crate publication dispatch inputs are valid", result.stdout)

    def test_validate_dispatch_rejects_non_default_branch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            release_path = self.release_json(temp_path / "release.json")
            env = self.base_env()
            env["DISPATCH_REF"] = "feature"
            result = self.run_script(["validate-dispatch", "--release-json", str(release_path)], env=env)

        self.assertEqual(result.returncode, 1)
        self.assertIn("workflow must run from default branch", result.stderr)

    def test_validate_dispatch_rejects_draft_release(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            release_path = self.release_json(temp_path / "release.json", draft=True)
            result = self.run_script(["validate-dispatch", "--release-json", str(release_path)])

        self.assertEqual(result.returncode, 1)
        self.assertIn("is a draft", result.stderr)

    def test_publish_noops_when_crate_version_exists_without_token(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            source_root = self.init_release_source(temp_path / "source")
            release_path = self.release_json(temp_path / "release.json")
            crate_path = self.crate_json(temp_path / "crate.json", published=True)
            failing_cargo = temp_path / "cargo-fails"
            failing_cargo.write_text("#!/usr/bin/env sh\nexit 99\n", encoding="utf-8")
            failing_cargo.chmod(0o755)
            result = self.run_script(
                [
                    "publish",
                    "--source-root",
                    str(source_root),
                    "--release-json",
                    str(release_path),
                    "--crate-json",
                    str(crate_path),
                    "--cargo-bin",
                    str(failing_cargo),
                ]
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("crate wavepeek 1.2.3 is already published; no-op", result.stdout)

    def test_publish_requires_token_when_crate_version_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            source_root = self.init_release_source(temp_path / "source")
            release_path = self.release_json(temp_path / "release.json")
            crate_path = self.crate_json(temp_path / "crate.json", published=False)
            result = self.run_script(
                [
                    "publish",
                    "--source-root",
                    str(source_root),
                    "--release-json",
                    str(release_path),
                    "--crate-json",
                    str(crate_path),
                ]
            )

        self.assertEqual(result.returncode, 1)
        self.assertIn("CARGO_REGISTRY_TOKEN or CRATES_IO_TOKEN is required", result.stderr)

    def test_publish_runs_cargo_publish_from_release_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            source_root = self.init_release_source(temp_path / "source")
            release_path = self.release_json(temp_path / "release.json")
            crate_path = self.crate_json(temp_path / "crate.json", published=False)
            log_path = temp_path / "cargo.log"
            fake_cargo = temp_path / "cargo-records"
            fake_cargo.write_text(
                f"#!/usr/bin/env sh\nprintf '%s\\n' \"$PWD\" \"$*\" > {log_path}\n",
                encoding="utf-8",
            )
            fake_cargo.chmod(0o755)
            env = self.base_env()
            env["CARGO_REGISTRY_TOKEN"] = "sekret"
            result = self.run_script(
                [
                    "publish",
                    "--source-root",
                    str(source_root),
                    "--release-json",
                    str(release_path),
                    "--crate-json",
                    str(crate_path),
                    "--cargo-bin",
                    str(fake_cargo),
                ],
                env=env,
            )

            log = log_path.read_text(encoding="utf-8")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(log, f"{source_root}\npublish --locked\n")

    def test_publish_rejects_checkout_that_is_not_the_release_tag(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            source_root = self.init_release_source(temp_path / "source", extra_commit=True)
            release_path = self.release_json(temp_path / "release.json")
            crate_path = self.crate_json(temp_path / "crate.json", published=True)
            result = self.run_script(
                [
                    "publish",
                    "--source-root",
                    str(source_root),
                    "--release-json",
                    str(release_path),
                    "--crate-json",
                    str(crate_path),
                ]
            )

        self.assertEqual(result.returncode, 1)
        self.assertIn("source checkout HEAD", result.stderr)


if __name__ == "__main__":
    unittest.main()
