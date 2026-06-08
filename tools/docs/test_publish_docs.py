from __future__ import annotations

import importlib.util
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import contextmanager

TOOLS_DIR = pathlib.Path(__file__).parent
sys.path.insert(0, str(TOOLS_DIR))
MODULE_PATH = TOOLS_DIR / "publish_docs.py"
SPEC = importlib.util.spec_from_file_location("publish_docs", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
publish_docs = importlib.util.module_from_spec(SPEC)
sys.modules["publish_docs"] = publish_docs
SPEC.loader.exec_module(publish_docs)


@contextmanager
def chdir(path: pathlib.Path):
    old = pathlib.Path.cwd()
    os.chdir(path)
    try:
        yield
    finally:
        os.chdir(old)


def git(repo: pathlib.Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout.strip()


class RecordingRunner:
    def __init__(self) -> None:
        self.commands: list[list[str]] = []

    def run(
        self,
        args,
        *,
        cwd=None,
        env=None,
        check=True,
        capture=False,
    ):
        command = [str(arg) for arg in args]
        self.commands.append(command)
        return subprocess.CompletedProcess(command, 0, "", "")


class PublishDocsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temp.name)
        self.work = self.root / "work"
        self.paths = publish_docs.paths(self.work)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_source_ref_must_match_version_tag(self) -> None:
        publish_docs.require_ref_matches_version("v0.5.0", "0.5.0")

        with self.assertRaisesRegex(publish_docs.PublishError, "does not match"):
            publish_docs.require_ref_matches_version("v0.5.1", "0.5.0")
        with self.assertRaisesRegex(publish_docs.PublishError, "release tag"):
            publish_docs.require_ref_matches_version("main", "0.5.0")

    def test_collect_root_artifacts_copies_versioned_schema(self) -> None:
        source = self.root / "source"
        (source / "docs" / "skills").mkdir(parents=True)
        (source / "schema").mkdir()
        (source / "docs" / "skills" / "wavepeek.md").write_text("skill", encoding="utf-8")
        (source / "schema" / "wavepeek_v0.json").write_text("{}", encoding="utf-8")

        copied = publish_docs.collect_root_artifacts(source, self.paths, "0.5.0")

        self.assertEqual(
            sorted(path.name for path in copied), ["skill.md", "wavepeek_v0.json"]
        )
        self.assertEqual((self.paths.root_artifacts / "skill.md").read_text(), "skill")

    def test_collect_root_artifacts_maps_legacy_major_zero_schema(self) -> None:
        source = self.root / "legacy-source"
        (source / "docs" / "skills").mkdir(parents=True)
        (source / "schema").mkdir()
        (source / "docs" / "skills" / "wavepeek.md").write_text("skill", encoding="utf-8")
        (source / "schema" / "wavepeek.json").write_text("legacy", encoding="utf-8")

        publish_docs.collect_root_artifacts(source, self.paths, "0.5.0")

        self.assertEqual(
            (self.paths.root_artifacts / "wavepeek_v0.json").read_text(), "legacy"
        )

    def test_read_stage_metadata_requires_matching_flags(self) -> None:
        self.paths.work_dir.mkdir(parents=True)
        self.paths.bundle.write_text("bundle", encoding="utf-8")
        self.paths.metadata.write_text(
            json.dumps(
                {
                    "version": "0.5.0",
                    "branch": "gh-pages",
                    "bundle": "gh-pages.bundle",
                    "final_commit": "abc",
                    "repair_existing_version": False,
                    "allowed_path_patterns": publish_docs.allowed_path_patterns("0.5.0"),
                }
            ),
            encoding="utf-8",
        )

        metadata = publish_docs.read_stage_metadata(self.paths, "0.5.0", False)
        self.assertEqual(metadata["version"], "0.5.0")

        with self.assertRaisesRegex(publish_docs.PublishError, "repair flag"):
            publish_docs.read_stage_metadata(self.paths, "0.5.0", True)

        data = json.loads(self.paths.metadata.read_text(encoding="utf-8"))
        data["allowed_path_patterns"] = ["**"]
        self.paths.metadata.write_text(json.dumps(data), encoding="utf-8")
        with self.assertRaisesRegex(publish_docs.PublishError, "allowed_path_patterns"):
            publish_docs.read_stage_metadata(self.paths, "0.5.0", False)

    def test_release_source_ref_must_resolve_to_tag(self) -> None:
        repo = self.root / "repo-tag"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "file.txt").write_text("content", encoding="utf-8")
        git(repo, "add", "file.txt")
        git(repo, "commit", "-q", "-m", "base")
        git(repo, "tag", "v0.5.0")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            self.assertEqual(
                publish_docs.resolve_release_tag("v0.5.0", runner),
                "refs/tags/v0.5.0",
            )
            git(repo, "tag", "-d", "v0.5.0")
            git(repo, "branch", "v0.5.0")
            with self.assertRaisesRegex(publish_docs.PublishError, "Git tag"):
                publish_docs.resolve_release_tag("v0.5.0", runner)

    def test_mike_deploy_uses_copy_aliases_for_verifiable_latest_path(self) -> None:
        runner = RecordingRunner()

        publish_docs.run_mike_deploy("0.5.0", self.paths, runner)

        deploy = runner.commands[0]
        self.assertIn("--alias-type", deploy)
        self.assertEqual(deploy[deploy.index("--alias-type") + 1], "copy")

    def test_allowed_path_patterns_limit_gh_pages_diff(self) -> None:
        publish_docs.verify_allowed_paths(
            [
                "0.5.0/index.html",
                "latest/index.html",
                ".nojekyll",
                "versions.json",
                "skill.md",
            ],
            publish_docs.allowed_path_patterns("0.5.0"),
        )

        with self.assertRaisesRegex(publish_docs.PublishError, "disallowed"):
            publish_docs.verify_allowed_paths(
                ["0.4.0/index.html"], publish_docs.allowed_path_patterns("0.5.0")
            )

    def test_changed_paths_reports_old_path_for_renames(self) -> None:
        repo = self.root / "repo-rename"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "0.4.0").mkdir()
        (repo / "0.4.0" / "index.html").write_text("old", encoding="utf-8")
        git(repo, "add", "0.4.0/index.html")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        shutil.move(repo / "0.4.0", repo / "0.5.0")
        git(repo, "add", "-A")
        git(repo, "commit", "-q", "-m", "rename")

        with chdir(repo):
            changed = publish_docs.changed_paths(base, "HEAD", publish_docs.CommandRunner())

        self.assertIn("0.4.0/index.html", changed)
        self.assertIn("0.5.0/index.html", changed)

    def test_verify_root_artifacts_requires_skill_and_major_schema(self) -> None:
        repo = self.root / "repo-root-artifacts"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "skill.md").write_text("skill", encoding="utf-8")
        (repo / "wavepeek_v0.json").write_text("{}", encoding="utf-8")
        git(repo, "add", "skill.md", "wavepeek_v0.json")
        git(repo, "commit", "-q", "-m", "artifacts")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            publish_docs.verify_root_artifacts("HEAD", "0.5.0", runner)
            git(repo, "rm", "-q", "skill.md")
            git(repo, "commit", "-q", "-m", "remove skill")
            with self.assertRaisesRegex(publish_docs.PublishError, "skill.md"):
                publish_docs.verify_root_artifacts("HEAD", "0.5.0", runner)

    def test_versions_semantics_allow_new_version_and_latest_move(self) -> None:
        repo = self.root / "repo"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "versions.json").write_text(
            json.dumps(
                [
                    {"version": "0.4.0", "title": "0.4.0", "aliases": ["latest"]}
                ]
            ),
            encoding="utf-8",
        )
        git(repo, "add", "versions.json")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        (repo / "versions.json").write_text(
            json.dumps(
                [
                    {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
                ]
            ),
            encoding="utf-8",
        )
        git(repo, "commit", "-q", "-am", "stage")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            publish_docs.verify_versions_semantics(
                remote_base=base,
                staged_branch="HEAD",
                version="0.5.0",
                repair_existing_version=False,
                runner=runner,
            )

    def test_versions_semantics_reject_unrelated_version_changes(self) -> None:
        repo = self.root / "repo-reject"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "versions.json").write_text(
            json.dumps([{"version": "0.4.0", "title": "0.4.0", "aliases": []}]),
            encoding="utf-8",
        )
        git(repo, "add", "versions.json")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        (repo / "versions.json").write_text(
            json.dumps(
                [
                    {"version": "0.4.0", "title": "changed", "aliases": []},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
                ]
            ),
            encoding="utf-8",
        )
        git(repo, "commit", "-q", "-am", "stage")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            with self.assertRaisesRegex(publish_docs.PublishError, "unrelated version"):
                publish_docs.verify_versions_semantics(
                    remote_base=base,
                    staged_branch="HEAD",
                    version="0.5.0",
                    repair_existing_version=False,
                    runner=runner,
                )


if __name__ == "__main__":
    unittest.main()
