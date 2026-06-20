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
    env = os.environ.copy()
    for name in (
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_COMMON_DIR",
        "GIT_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_PREFIX",
        "GIT_WORK_TREE",
    ):
        env.pop(name, None)
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        env=env,
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
        (source / "schema").mkdir(parents=True)
        (source / "schema" / "wavepeek_v0.json").write_text("{}", encoding="utf-8")
        (source / "schema" / "wavepeek-stream-v0.json").write_text("{}", encoding="utf-8")

        copied = publish_docs.collect_root_artifacts(source, self.paths, "0.5.0")

        self.assertEqual(sorted(path.name for path in copied), ["wavepeek-stream-v0.json", "wavepeek_v0.json"])
        self.assertFalse((self.paths.root_artifacts / "skill.md").exists())

    def test_collect_root_artifacts_maps_legacy_major_zero_schema(self) -> None:
        source = self.root / "legacy-source"
        (source / "schema").mkdir(parents=True)
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
                    "allowed_path_patterns": publish_docs.allowed_path_patterns("0.5.0", promote_latest=True),
                    "promote_latest": True,
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

        publish_docs.run_mike_deploy("0.5.0", self.paths, runner, promote_latest=True)

        deploy = runner.commands[0]
        self.assertIn("--alias-type", deploy)
        self.assertEqual(deploy[deploy.index("--alias-type") + 1], "copy")

    def test_mike_deploy_can_refresh_version_without_latest_alias(self) -> None:
        runner = RecordingRunner()

        publish_docs.run_mike_deploy("0.4.0", self.paths, runner, promote_latest=False)

        deploy = runner.commands[0]
        self.assertNotIn("latest", deploy)
        self.assertEqual(len(runner.commands), 1)

    def test_copy_installer_entrypoints_writes_versioned_and_root_aliases(self) -> None:
        self.paths.release_assets.mkdir(parents=True)
        (self.paths.release_assets / "wavepeek-installer.sh").write_text("shell\n", encoding="utf-8")
        (self.paths.release_assets / "wavepeek-installer.ps1").write_text("powershell\n", encoding="utf-8")
        self.paths.gh_pages_worktree.mkdir(parents=True)

        copied = publish_docs.copy_installer_entrypoints("0.5.0", self.paths, promote_latest=True)

        self.assertEqual(sorted(copied), ["0.5.0/install.ps1", "0.5.0/install.sh", "install.ps1", "install.sh"])
        self.assertEqual((self.paths.gh_pages_worktree / "0.5.0" / "install.sh").read_text(), "shell\n")
        self.assertEqual((self.paths.gh_pages_worktree / "install.ps1").read_text(), "powershell\n")

    def test_copy_installer_entrypoints_requires_release_assets(self) -> None:
        self.paths.gh_pages_worktree.mkdir(parents=True)

        with self.assertRaisesRegex(publish_docs.PublishError, "missing release installer"):
            publish_docs.copy_installer_entrypoints("0.5.0", self.paths, promote_latest=True)

    def test_stage_publication_artifacts_stages_envelope_and_stream_schema(self) -> None:
        repo = self.root / "repo-stage-artifacts"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "README.md").write_text("base", encoding="utf-8")
        git(repo, "add", "README.md")
        git(repo, "commit", "-q", "-m", "base")
        git(repo, "branch", "gh-pages")

        self.paths.root_artifacts.mkdir(parents=True)
        (self.paths.root_artifacts / "wavepeek_v0.json").write_text("envelope", encoding="utf-8")
        (self.paths.root_artifacts / "wavepeek-stream-v0.json").write_text("stream", encoding="utf-8")
        self.paths.release_assets.mkdir(parents=True)
        (self.paths.release_assets / "wavepeek-installer.sh").write_text("shell", encoding="utf-8")
        (self.paths.release_assets / "wavepeek-installer.ps1").write_text("powershell", encoding="utf-8")

        with chdir(repo):
            publish_docs.stage_publication_artifacts(
                "0.5.0",
                self.paths,
                publish_docs.CommandRunner(),
                promote_latest=True,
            )
            self.assertEqual(
                git(repo, "show", "gh-pages:wavepeek-stream-v0.json"),
                "stream",
            )
            self.assertEqual(git(repo, "show", "gh-pages:wavepeek_v0.json"), "envelope")

    def test_allowed_path_patterns_limit_gh_pages_diff(self) -> None:
        publish_docs.verify_allowed_paths(
            [
                "0.5.0/index.html",
                "latest/index.html",
                ".nojekyll",
                "versions.json",
                "install.sh",
                "install.ps1",
                "wavepeek-stream-v0.json",
            ],
            publish_docs.allowed_path_patterns("0.5.0", promote_latest=True),
        )

        with self.assertRaisesRegex(publish_docs.PublishError, "disallowed"):
            publish_docs.verify_allowed_paths(
                ["0.4.0/index.html"], publish_docs.allowed_path_patterns("0.5.0", promote_latest=True)
            )
        with self.assertRaisesRegex(publish_docs.PublishError, "disallowed"):
            publish_docs.verify_allowed_paths(
                ["wavepeek_v0.json/extra.json"],
                publish_docs.allowed_path_patterns("0.5.0", promote_latest=True),
            )
        with self.assertRaisesRegex(publish_docs.PublishError, "disallowed"):
            publish_docs.verify_allowed_paths(
                ["wavepeek-stream-v0.json/extra.json"],
                publish_docs.allowed_path_patterns("0.5.0", promote_latest=True),
            )
        with self.assertRaisesRegex(publish_docs.PublishError, "disallowed"):
            publish_docs.verify_allowed_paths(
                ["skill.md"], publish_docs.allowed_path_patterns("0.5.0", promote_latest=True)
            )
        with self.assertRaisesRegex(publish_docs.PublishError, "disallowed"):
            publish_docs.verify_allowed_paths(
                ["latest/index.html", "install.sh", "wavepeek_v0.json", "wavepeek-stream-v0.json"],
                publish_docs.allowed_path_patterns("0.4.0", promote_latest=False),
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

    def test_export_pages_artifact_archives_staged_tree(self) -> None:
        repo = self.root / "repo-pages-artifact"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "index.html").write_text("redirect", encoding="utf-8")
        (repo / "versions.json").write_text("[]", encoding="utf-8")
        (repo / "wavepeek_v0.json").write_text("{}", encoding="utf-8")
        (repo / "wavepeek-stream-v0.json").write_text("{}", encoding="utf-8")
        (repo / ".gitattributes").write_text("0.5.0/index.html export-ignore\n", encoding="utf-8")
        (repo / "0.5.0").mkdir()
        (repo / "0.5.0" / "index.html").write_text("docs", encoding="utf-8")
        git(repo, "add", ".")
        git(repo, "commit", "-q", "-m", "pages")

        with chdir(repo):
            publish_docs.export_pages_artifact("HEAD", "0.5.0", self.paths, publish_docs.CommandRunner())

        self.assertEqual((self.paths.pages_artifact / "index.html").read_text(), "redirect")
        self.assertEqual((self.paths.pages_artifact / "0.5.0" / "index.html").read_text(), "docs")
        self.assertTrue((self.paths.pages_artifact / "wavepeek_v0.json").is_file())
        self.assertTrue((self.paths.pages_artifact / "wavepeek-stream-v0.json").is_file())
        self.assertFalse((self.paths.pages_artifact / ".git").exists())
        self.assertFalse(self.paths.pages_worktree.exists())

    def test_export_pages_artifact_requires_pages_root_files(self) -> None:
        repo = self.root / "repo-pages-artifact-missing"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "versions.json").write_text("[]", encoding="utf-8")
        (repo / "wavepeek_v0.json").write_text("{}", encoding="utf-8")
        (repo / "wavepeek-stream-v0.json").write_text("{}", encoding="utf-8")
        git(repo, "add", ".")
        git(repo, "commit", "-q", "-m", "pages")

        with chdir(repo):
            with self.assertRaisesRegex(publish_docs.PublishError, "index.html"):
                publish_docs.export_pages_artifact("HEAD", "0.5.0", self.paths, publish_docs.CommandRunner())

    def test_verify_root_artifacts_requires_major_schema(self) -> None:
        repo = self.root / "repo-root-artifacts"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "wavepeek_v0.json").write_text("{}", encoding="utf-8")
        (repo / "wavepeek-stream-v0.json").write_text("{}", encoding="utf-8")
        git(repo, "add", "wavepeek_v0.json", "wavepeek-stream-v0.json")
        git(repo, "commit", "-q", "-m", "artifacts")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            publish_docs.verify_root_artifacts("HEAD", "0.5.0", runner)
            git(repo, "rm", "-q", "wavepeek-stream-v0.json")
            git(repo, "commit", "-q", "-m", "remove stream schema")
            with self.assertRaisesRegex(publish_docs.PublishError, "wavepeek-stream-v0.json"):
                publish_docs.verify_root_artifacts("HEAD", "0.5.0", runner)

        repo_tree = self.root / "repo-root-artifacts-tree"
        repo_tree.mkdir()
        git(repo_tree, "init", "-q")
        git(repo_tree, "config", "user.email", "docs@example.invalid")
        git(repo_tree, "config", "user.name", "Docs Bot")
        (repo_tree / "wavepeek_v0.json").mkdir()
        (repo_tree / "wavepeek_v0.json" / "extra.json").write_text("{}", encoding="utf-8")
        (repo_tree / "wavepeek-stream-v0.json").write_text("{}", encoding="utf-8")
        git(repo_tree, "add", "wavepeek_v0.json/extra.json", "wavepeek-stream-v0.json")
        git(repo_tree, "commit", "-q", "-m", "tree artifact")
        with chdir(repo_tree):
            with self.assertRaisesRegex(publish_docs.PublishError, "wavepeek_v0.json"):
                publish_docs.verify_root_artifacts("HEAD", "0.5.0", runner)

    def test_versions_json_rejects_duplicate_versions_and_bad_aliases(self) -> None:
        with self.assertRaisesRegex(publish_docs.PublishError, "duplicate version"):
            publish_docs.version_entries_by_name(
                [
                    {"version": "0.5.0", "aliases": []},
                    {"version": "0.5.0", "aliases": ["latest"]},
                ]
            )
        with self.assertRaisesRegex(publish_docs.PublishError, "only strings"):
            publish_docs.aliases({"version": "0.5.0", "aliases": ["latest", 1]})

    def write_versions(self, repo: pathlib.Path, versions: list[dict[str, object]]) -> None:
        (repo / "versions.json").write_text(json.dumps(versions), encoding="utf-8")

    def test_should_promote_latest_only_for_newer_or_current_latest(self) -> None:
        repo = self.root / "repo-promote"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        self.write_versions(
            repo,
            [
                {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
            ],
        )
        git(repo, "add", "versions.json")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            self.assertFalse(
                publish_docs.should_promote_latest(
                    base,
                    version="0.4.0",
                    repair_existing_version=False,
                    runner=runner,
                )
            )
            self.assertTrue(
                publish_docs.should_promote_latest(
                    base,
                    version="0.6.0",
                    repair_existing_version=False,
                    runner=runner,
                )
            )
            self.assertTrue(
                publish_docs.should_promote_latest(
                    base,
                    version="0.5.0",
                    repair_existing_version=True,
                    runner=runner,
                )
            )
            self.assertFalse(
                publish_docs.should_promote_latest(
                    base,
                    version="0.4.0",
                    repair_existing_version=True,
                    runner=runner,
                )
            )

    def test_should_promote_latest_uses_highest_version_when_alias_is_missing(self) -> None:
        repo = self.root / "repo-promote-no-latest"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        self.write_versions(
            repo,
            [
                {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                {"version": "0.5.0", "title": "0.5.0", "aliases": []},
            ],
        )
        git(repo, "add", "versions.json")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            self.assertFalse(
                publish_docs.should_promote_latest(
                    base,
                    version="0.4.0",
                    repair_existing_version=False,
                    runner=runner,
                )
            )
            self.assertTrue(
                publish_docs.should_promote_latest(
                    base,
                    version="0.6.0",
                    repair_existing_version=False,
                    runner=runner,
                )
            )
            self.assertFalse(
                publish_docs.should_promote_latest(
                    base,
                    version="0.5.0",
                    repair_existing_version=True,
                    runner=runner,
                )
            )

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
                promote_latest=True,
                runner=runner,
            )

    def test_versions_semantics_preserve_latest_during_non_latest_repair(self) -> None:
        repo = self.root / "repo-repair"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "versions.json").write_text(
            json.dumps(
                [
                    {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
                ]
            ),
            encoding="utf-8",
        )
        git(repo, "add", "versions.json")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            publish_docs.verify_versions_semantics(
                remote_base=base,
                staged_branch=base,
                version="0.4.0",
                repair_existing_version=True,
                promote_latest=False,
                runner=runner,
            )

    def test_versions_semantics_reject_latest_rollback_during_non_latest_repair(self) -> None:
        repo = self.root / "repo-repair-reject"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "versions.json").write_text(
            json.dumps(
                [
                    {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
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
                    {"version": "0.4.0", "title": "0.4.0", "aliases": ["latest"]},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": []},
                ]
            ),
            encoding="utf-8",
        )
        git(repo, "commit", "-q", "-am", "rollback latest")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            with self.assertRaisesRegex(publish_docs.PublishError, "preserve the existing latest"):
                publish_docs.verify_versions_semantics(
                    remote_base=base,
                    staged_branch="HEAD",
                    version="0.4.0",
                    repair_existing_version=True,
                    promote_latest=False,
                    runner=runner,
                )

    def test_latest_tree_preserved_during_non_latest_repair(self) -> None:
        repo = self.root / "repo-latest-tree-repair"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "latest").mkdir()
        (repo / "latest" / "index.html").write_text("current latest\n", encoding="utf-8")
        (repo / "versions.json").write_text(
            json.dumps(
                [
                    {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
                ]
            ),
            encoding="utf-8",
        )
        git(repo, "add", ".")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        (repo / "0.4.0").mkdir()
        (repo / "0.4.0" / "index.html").write_text("repaired old\n", encoding="utf-8")
        git(repo, "add", "0.4.0/index.html")
        git(repo, "commit", "-q", "-m", "repair old")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            publish_docs.verify_latest_tree_semantics(
                remote_base=base,
                staged_branch="HEAD",
                promote_latest=False,
                runner=runner,
            )
            (repo / "latest" / "index.html").write_text("rolled back\n", encoding="utf-8")
            git(repo, "add", "latest/index.html")
            git(repo, "commit", "-q", "-m", "rollback latest")
            with self.assertRaisesRegex(publish_docs.PublishError, "changes latest docs"):
                publish_docs.verify_latest_tree_semantics(
                    remote_base=base,
                    staged_branch="HEAD",
                    promote_latest=False,
                    runner=runner,
                )

    def test_installer_entrypoints_preserve_root_alias_during_non_latest_repair(self) -> None:
        repo = self.root / "repo-installer-repair"
        repo.mkdir()
        git(repo, "init", "-q")
        git(repo, "config", "user.email", "docs@example.invalid")
        git(repo, "config", "user.name", "Docs Bot")
        (repo / "versions.json").write_text(
            json.dumps(
                [
                    {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
                ]
            ),
            encoding="utf-8",
        )
        (repo / "install.sh").write_text("latest shell\n", encoding="utf-8")
        (repo / "install.ps1").write_text("latest ps1\n", encoding="utf-8")
        git(repo, "add", ".")
        git(repo, "commit", "-q", "-m", "base")
        base = git(repo, "rev-parse", "HEAD")
        (repo / "0.4.0").mkdir()
        (repo / "0.4.0" / "install.sh").write_text("old shell\n", encoding="utf-8")
        (repo / "0.4.0" / "install.ps1").write_text("old ps1\n", encoding="utf-8")
        git(repo, "add", ".")
        git(repo, "commit", "-q", "-m", "repair old installers")
        runner = publish_docs.CommandRunner()

        with chdir(repo):
            publish_docs.verify_installer_entrypoints(
                remote_base=base,
                staged_branch="HEAD",
                version="0.4.0",
                promote_latest=False,
                runner=runner,
            )
            (repo / "install.sh").write_text("rolled back\n", encoding="utf-8")
            git(repo, "add", "install.sh")
            git(repo, "commit", "-q", "-m", "rollback root")
            with self.assertRaisesRegex(publish_docs.PublishError, "changes root installer"):
                publish_docs.verify_installer_entrypoints(
                    remote_base=base,
                    staged_branch="HEAD",
                    version="0.4.0",
                    promote_latest=False,
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
                    promote_latest=True,
                    runner=runner,
                )


if __name__ == "__main__":
    unittest.main()
