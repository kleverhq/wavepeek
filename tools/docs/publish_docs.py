#!/usr/bin/env python3

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from typing import Any, Sequence

import prepare_mkdocs

VERSION_RE = re.compile(r"^(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)\.(?P<patch>0|[1-9][0-9]*)$")
TAG_RE = re.compile(r"^v(?P<version>(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*))$")
DEFAULT_WORK_DIR = pathlib.Path("tmp/docs-site")
GH_PAGES_BRANCH = "gh-pages"
STAGED_BRANCH = "staged-gh-pages"
METADATA_NAME = "staged-deploy.json"
BUNDLE_NAME = "gh-pages.bundle"
PUSH_TOKEN_ENV = {
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GITHUB_PAT",
    "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
}


class PublishError(Exception):
    pass


@dataclass(frozen=True)
class Paths:
    work_dir: pathlib.Path
    export_dir: pathlib.Path
    mkdocs_src: pathlib.Path
    mkdocs_config: pathlib.Path
    root_artifacts: pathlib.Path
    source_worktree: pathlib.Path
    gh_pages_worktree: pathlib.Path
    metadata: pathlib.Path
    bundle: pathlib.Path


@dataclass(frozen=True)
class CommandRunner:
    dry_run: bool = False

    def run(
        self,
        args: Sequence[str | pathlib.Path],
        *,
        cwd: pathlib.Path | None = None,
        env: dict[str, str] | None = None,
        check: bool = True,
        capture: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        rendered = [str(arg) for arg in args]
        if self.dry_run:
            print("+ " + " ".join(rendered))
            return subprocess.CompletedProcess(rendered, 0, "", "")
        return subprocess.run(
            rendered,
            cwd=cwd,
            env=env,
            check=check,
            text=True,
            stdout=subprocess.PIPE if capture else None,
            stderr=subprocess.PIPE if capture else None,
        )


@dataclass(frozen=True)
class CheckResult:
    source_root: pathlib.Path
    cli_version: str
    root_artifacts: list[pathlib.Path]


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build and stage wavepeek GitHub Pages docs.")
    subcommands = parser.add_subparsers(dest="command", required=True)

    def add_common(subparser: argparse.ArgumentParser) -> None:
        subparser.add_argument("--version", required=True)
        subparser.add_argument("--work-dir", type=pathlib.Path, default=DEFAULT_WORK_DIR)
        subparser.add_argument("--repair-existing-version", action="store_true")

    check = subcommands.add_parser("check", help="build the site and root artifacts without gh-pages")
    add_common(check)
    check.add_argument("--source-root", type=pathlib.Path)
    check.add_argument("--source-ref")

    stage = subcommands.add_parser("stage-deploy", help="stage a local gh-pages update and bundle")
    add_common(stage)
    stage.add_argument("--source-ref", required=True)

    push = subcommands.add_parser("push-staged", help="verify and push a staged gh-pages bundle")
    add_common(push)

    return parser.parse_args(argv)


def fail(message: str) -> None:
    raise PublishError(message)


def paths(work_dir: pathlib.Path) -> Paths:
    work_dir = work_dir.resolve()
    return Paths(
        work_dir=work_dir,
        export_dir=work_dir / "export",
        mkdocs_src=work_dir / "mkdocs-src",
        mkdocs_config=work_dir / "mkdocs.yml",
        root_artifacts=work_dir / "root-artifacts",
        source_worktree=work_dir / "source",
        gh_pages_worktree=work_dir / "gh-pages",
        metadata=work_dir / METADATA_NAME,
        bundle=work_dir / BUNDLE_NAME,
    )


def validate_version(version: str) -> str:
    if VERSION_RE.fullmatch(version) is None:
        fail(f"version must be SemVer X.Y.Z, got {version!r}")
    return version


def version_from_source_ref(source_ref: str) -> str:
    match = TAG_RE.fullmatch(source_ref)
    if match is None:
        fail(f"source ref must be a SemVer release tag vX.Y.Z, got {source_ref!r}")
    return match.group("version")


def require_ref_matches_version(source_ref: str, version: str) -> None:
    ref_version = version_from_source_ref(source_ref)
    if ref_version != version:
        fail(f"source ref {source_ref!r} does not match version {version!r}")


def resolve_release_tag(source_ref: str, runner: CommandRunner) -> str:
    full_ref = f"refs/tags/{source_ref}"
    result = runner.run(
        ["git", "rev-parse", "--verify", f"{full_ref}^{{commit}}"],
        check=False,
        capture=True,
    )
    if result.returncode != 0:
        fail(f"source ref {source_ref!r} must resolve to an existing Git tag")
    return full_ref


def package_version(source_root: pathlib.Path) -> str:
    cargo_toml = source_root / "Cargo.toml"
    if not cargo_toml.is_file():
        fail(f"missing Cargo.toml at {cargo_toml}")
    data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    version = data.get("package", {}).get("version")
    if not isinstance(version, str):
        fail(f"Cargo.toml at {cargo_toml} is missing package.version")
    return version


def major_version(version: str) -> int:
    return int(validate_version(version).split(".", maxsplit=1)[0])


def clean_owned_path(path: pathlib.Path) -> None:
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()


def remove_git_worktree(path: pathlib.Path, runner: CommandRunner) -> None:
    if not path.exists():
        return
    result = runner.run(
        ["git", "worktree", "remove", "--force", path], check=False, capture=True
    )
    if result.returncode != 0 and path.exists():
        clean_owned_path(path)


def add_source_worktree(source_ref: str, run_paths: Paths, runner: CommandRunner) -> pathlib.Path:
    full_ref = resolve_release_tag(source_ref, runner)
    remove_git_worktree(run_paths.source_worktree, runner)
    run_paths.source_worktree.parent.mkdir(parents=True, exist_ok=True)
    runner.run(["git", "worktree", "add", "--detach", run_paths.source_worktree, full_ref])
    return run_paths.source_worktree


def child_env_without_push_tokens() -> dict[str, str]:
    env = os.environ.copy()
    for name in PUSH_TOKEN_ENV:
        env.pop(name, None)
    return env


def export_docs(source_root: pathlib.Path, run_paths: Paths, runner: CommandRunner) -> None:
    clean_owned_path(run_paths.export_dir)
    run_paths.export_dir.parent.mkdir(parents=True, exist_ok=True)
    runner.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            source_root / "Cargo.toml",
            "--",
            "docs",
            "export",
            run_paths.export_dir,
            "--force",
        ],
        env=child_env_without_push_tokens(),
    )


def build_mkdocs(run_paths: Paths, version: str, runner: CommandRunner) -> str:
    prepare_mkdocs.prepare_tree(
        run_paths.export_dir,
        run_paths.mkdocs_src,
        run_paths.mkdocs_config,
        version,
        force=True,
    )
    runner.run(
        ["mkdocs", "build", "--strict", "--config-file", run_paths.mkdocs_config],
        env=child_env_without_push_tokens(),
    )
    manifest = json.loads((run_paths.export_dir / "manifest.json").read_text(encoding="utf-8"))
    cli_version = manifest.get("cli_version")
    if not isinstance(cli_version, str):
        fail("export manifest cli_version is missing after prepare")
    return cli_version


def collect_root_artifacts(source_root: pathlib.Path, run_paths: Paths, version: str) -> list[pathlib.Path]:
    clean_owned_path(run_paths.root_artifacts)
    run_paths.root_artifacts.mkdir(parents=True, exist_ok=True)

    skill_source = source_root / "docs" / "skills" / "wavepeek.md"
    if not skill_source.is_file():
        fail(f"missing packaged skill at {skill_source}")
    skill_target = run_paths.root_artifacts / "skill.md"
    shutil.copyfile(skill_source, skill_target)

    schema_dir = source_root / "schema"
    schemas = sorted(schema_dir.glob("wavepeek_v*.json")) if schema_dir.is_dir() else []
    copied: list[pathlib.Path] = [skill_target]
    if schemas:
        for schema in schemas:
            target = run_paths.root_artifacts / schema.name
            shutil.copyfile(schema, target)
            copied.append(target)
    else:
        legacy_schema = schema_dir / "wavepeek.json"
        if major_version(version) == 0 and legacy_schema.is_file():
            target = run_paths.root_artifacts / "wavepeek_v0.json"
            shutil.copyfile(legacy_schema, target)
            copied.append(target)
        else:
            fail(f"missing versioned schema artifacts under {schema_dir}")

    schema_targets = [path for path in copied if path.name.startswith("wavepeek_v")]
    if not schema_targets:
        fail(f"missing versioned schema artifacts under {schema_dir}")
    return copied


def resolve_source_root(
    *,
    source_root: pathlib.Path | None,
    source_ref: str | None,
    version: str,
    run_paths: Paths,
    runner: CommandRunner,
    for_stage: bool,
) -> pathlib.Path:
    validate_version(version)
    if source_ref:
        require_ref_matches_version(source_ref, version)
        return add_source_worktree(source_ref, run_paths, runner).resolve()
    if for_stage:
        fail("stage-deploy requires --source-ref")
    return (source_root or pathlib.Path(".")).resolve()


def perform_check(
    *,
    version: str,
    run_paths: Paths,
    runner: CommandRunner,
    source_root: pathlib.Path | None = None,
    source_ref: str | None = None,
    for_stage: bool = False,
) -> CheckResult:
    actual_source_root = resolve_source_root(
        source_root=source_root,
        source_ref=source_ref,
        version=version,
        run_paths=run_paths,
        runner=runner,
        for_stage=for_stage,
    )
    source_version = package_version(actual_source_root)
    if source_version != version:
        fail(f"Cargo.toml version {source_version!r} does not match {version!r}")

    worktree_source = source_ref is not None
    try:
        export_docs(actual_source_root, run_paths, runner)
        cli_version = build_mkdocs(run_paths, version, runner)
        artifacts = collect_root_artifacts(actual_source_root, run_paths, version)
    finally:
        if worktree_source:
            remove_git_worktree(run_paths.source_worktree, runner)
    return CheckResult(actual_source_root, cli_version, artifacts)


def git_capture(args: Sequence[str | pathlib.Path], runner: CommandRunner) -> str:
    result = runner.run(["git", *args], check=True, capture=True)
    return (result.stdout or "").strip()


def git_ref_exists(ref: str, runner: CommandRunner) -> bool:
    result = runner.run(["git", "rev-parse", "--verify", ref], check=False, capture=True)
    return result.returncode == 0


def remote_branch_exists(runner: CommandRunner) -> bool:
    result = runner.run(
        ["git", "ls-remote", "--exit-code", "--heads", "origin", GH_PAGES_BRANCH],
        check=False,
        capture=True,
    )
    return result.returncode == 0


def fetch_publication_branch(runner: CommandRunner) -> str | None:
    if remote_branch_exists(runner):
        runner.run(["git", "fetch", "origin", GH_PAGES_BRANCH])
        base = git_capture(["rev-parse", "origin/gh-pages"], runner)
        if git_ref_exists(GH_PAGES_BRANCH, runner):
            runner.run(["git", "branch", "-f", GH_PAGES_BRANCH, "origin/gh-pages"])
        else:
            runner.run(["git", "branch", GH_PAGES_BRANCH, "origin/gh-pages"])
        return base
    if git_ref_exists(GH_PAGES_BRANCH, runner):
        runner.run(["git", "branch", "-D", GH_PAGES_BRANCH])
    return None


def git_show_json(ref: str, path: str, runner: CommandRunner) -> Any | None:
    result = runner.run(["git", "show", f"{ref}:{path}"], check=False, capture=True)
    if result.returncode != 0:
        return None
    try:
        return json.loads(result.stdout or "")
    except json.JSONDecodeError as error:
        fail(f"{ref}:{path} is invalid JSON: {error}")


def version_entries_by_name(versions: Any) -> dict[str, dict[str, Any]]:
    if versions is None:
        return {}
    if not isinstance(versions, list):
        fail("versions.json must contain a list")
    entries: dict[str, dict[str, Any]] = {}
    for entry in versions:
        if not isinstance(entry, dict) or not isinstance(entry.get("version"), str):
            fail("versions.json entries must be objects with a string version")
        version = entry["version"]
        if version in entries:
            fail(f"versions.json contains duplicate version {version!r}")
        entries[version] = entry
    return entries


def requested_version_exists(ref: str, version: str, runner: CommandRunner) -> bool:
    versions = version_entries_by_name(git_show_json(ref, "versions.json", runner))
    return version in versions


def run_mike_deploy(version: str, run_paths: Paths, runner: CommandRunner) -> None:
    config = run_paths.mkdocs_config
    runner.run(
        [
            "mike",
            "deploy",
            "--config-file",
            config,
            "--branch",
            GH_PAGES_BRANCH,
            "--remote",
            "origin",
            "--update-aliases",
            "--ignore-remote-status",
            "--alias-type",
            "copy",
            version,
            "latest",
        ],
        env=child_env_without_push_tokens(),
    )
    runner.run(
        [
            "mike",
            "set-default",
            "--config-file",
            config,
            "--branch",
            GH_PAGES_BRANCH,
            "--remote",
            "origin",
            "--ignore-remote-status",
            "latest",
        ],
        env=child_env_without_push_tokens(),
    )


def stage_root_artifacts(version: str, run_paths: Paths, runner: CommandRunner) -> None:
    remove_git_worktree(run_paths.gh_pages_worktree, runner)
    runner.run(["git", "worktree", "add", run_paths.gh_pages_worktree, GH_PAGES_BRANCH])
    for artifact in sorted(run_paths.root_artifacts.iterdir()):
        if artifact.is_file():
            shutil.copyfile(artifact, run_paths.gh_pages_worktree / artifact.name)
    runner.run(["git", "add", "skill.md", "wavepeek_v*.json"], cwd=run_paths.gh_pages_worktree)
    diff = runner.run(
        ["git", "diff", "--cached", "--quiet"],
        cwd=run_paths.gh_pages_worktree,
        check=False,
        capture=True,
    )
    if diff.returncode != 0:
        runner.run(
            ["git", "commit", "-m", f"docs: publish root artifacts for {version}"],
            cwd=run_paths.gh_pages_worktree,
        )


def allowed_path_patterns(version: str) -> list[str]:
    return [
        f"{version}/**",
        "latest/**",
        ".nojekyll",
        "versions.json",
        "index.html",
        "404.html",
        "sitemap.xml",
        "sitemap.xml.gz",
        "skill.md",
        "wavepeek_v*.json",
    ]


def write_stage_metadata(
    *,
    version: str,
    source_ref: str,
    repair_existing_version: bool,
    remote_base: str | None,
    run_paths: Paths,
    runner: CommandRunner,
) -> dict[str, Any]:
    final_commit = git_capture(["rev-parse", GH_PAGES_BRANCH], runner)
    metadata = {
        "version": version,
        "source_ref": source_ref,
        "branch": GH_PAGES_BRANCH,
        "remote_base": remote_base,
        "first_publish": remote_base is None,
        "final_commit": final_commit,
        "bundle": BUNDLE_NAME,
        "repair_existing_version": repair_existing_version,
        "allowed_path_patterns": allowed_path_patterns(version),
    }
    run_paths.metadata.write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return metadata


def create_bundle(run_paths: Paths, runner: CommandRunner) -> None:
    if run_paths.bundle.exists():
        run_paths.bundle.unlink()
    runner.run(["git", "bundle", "create", run_paths.bundle, GH_PAGES_BRANCH])


def stage_deploy(
    *,
    version: str,
    source_ref: str,
    repair_existing_version: bool,
    run_paths: Paths,
    runner: CommandRunner,
) -> dict[str, Any]:
    validate_version(version)
    require_ref_matches_version(source_ref, version)
    perform_check(
        version=version,
        run_paths=run_paths,
        runner=runner,
        source_ref=source_ref,
        for_stage=True,
    )
    remote_base = fetch_publication_branch(runner)
    if remote_base is not None and requested_version_exists("gh-pages", version, runner):
        if not repair_existing_version:
            fail(
                f"version {version} already exists on gh-pages; rerun with --repair-existing-version"
            )
    run_mike_deploy(version, run_paths, runner)
    stage_root_artifacts(version, run_paths, runner)
    metadata = write_stage_metadata(
        version=version,
        source_ref=source_ref,
        repair_existing_version=repair_existing_version,
        remote_base=remote_base,
        run_paths=run_paths,
        runner=runner,
    )
    create_bundle(run_paths, runner)
    return metadata


def read_stage_metadata(run_paths: Paths, version: str, repair_existing_version: bool) -> dict[str, Any]:
    if not run_paths.metadata.is_file():
        fail(f"missing staged deploy metadata at {run_paths.metadata}")
    try:
        metadata = json.loads(run_paths.metadata.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"staged deploy metadata is invalid JSON: {error}")
    if metadata.get("version") != version:
        fail("staged deploy metadata version mismatch")
    if metadata.get("branch") != GH_PAGES_BRANCH:
        fail("staged deploy metadata branch mismatch")
    if bool(metadata.get("repair_existing_version")) != repair_existing_version:
        fail("staged deploy metadata repair flag mismatch")
    if metadata.get("bundle") != BUNDLE_NAME:
        fail("staged deploy metadata bundle name mismatch")
    if not isinstance(metadata.get("final_commit"), str) or not metadata["final_commit"]:
        fail("staged deploy metadata final_commit is missing")
    if not isinstance(metadata.get("allowed_path_patterns"), list):
        fail("staged deploy metadata allowed_path_patterns is missing")
    if [str(pattern) for pattern in metadata["allowed_path_patterns"]] != allowed_path_patterns(version):
        fail("staged deploy metadata allowed_path_patterns mismatch")
    if not run_paths.bundle.is_file():
        fail(f"missing staged gh-pages bundle at {run_paths.bundle}")
    return metadata


def current_remote_base(runner: CommandRunner) -> str | None:
    if not remote_branch_exists(runner):
        return None
    runner.run(["git", "fetch", "origin", GH_PAGES_BRANCH])
    return git_capture(["rev-parse", "origin/gh-pages"], runner)


def import_bundle(run_paths: Paths, metadata: dict[str, Any], runner: CommandRunner) -> str:
    if git_ref_exists(STAGED_BRANCH, runner):
        runner.run(["git", "branch", "-D", STAGED_BRANCH])
    runner.run(
        [
            "git",
            "fetch",
            run_paths.bundle,
            f"refs/heads/{GH_PAGES_BRANCH}:refs/heads/{STAGED_BRANCH}",
        ]
    )
    staged_commit = git_capture(["rev-parse", STAGED_BRANCH], runner)
    if staged_commit != metadata["final_commit"]:
        fail("staged bundle final commit does not match metadata")
    return staged_commit


def verify_fast_forward(remote_base: str | None, staged_branch: str, runner: CommandRunner) -> None:
    if remote_base is None:
        return
    result = runner.run(
        ["git", "merge-base", "--is-ancestor", remote_base, staged_branch],
        check=False,
        capture=True,
    )
    if result.returncode != 0:
        fail("staged gh-pages update is not a fast-forward from the remote base")


def changed_paths(remote_base: str | None, staged_branch: str, runner: CommandRunner) -> list[str]:
    if remote_base is None:
        output = git_capture(["ls-tree", "-r", "--name-only", staged_branch], runner)
    else:
        output = git_capture(
            ["diff", "--name-only", "--no-renames", remote_base, staged_branch], runner
        )
    return [line for line in output.splitlines() if line]


def path_allowed(path: str, patterns: list[str]) -> bool:
    for pattern in patterns:
        if pattern == "wavepeek_v*.json":
            if re.fullmatch(r"wavepeek_v[0-9]+[.]json", path):
                return True
        elif pattern.endswith("/**"):
            prefix = pattern[:-3] + "/"
            if path.startswith(prefix):
                return True
        elif "*" in pattern:
            if fnmatch.fnmatch(path, pattern):
                return True
        elif path == pattern:
            return True
    return False


def verify_allowed_paths(paths_changed: list[str], patterns: list[str]) -> None:
    bad = [path for path in paths_changed if not path_allowed(path, patterns)]
    if bad:
        fail("staged gh-pages bundle changes disallowed path(s): " + ", ".join(bad))


def aliases(entry: dict[str, Any] | None) -> set[str]:
    if entry is None:
        return set()
    raw = entry.get("aliases", [])
    if not isinstance(raw, list):
        fail("versions.json aliases must be arrays")
    if any(not isinstance(item, str) for item in raw):
        fail("versions.json aliases must contain only strings")
    return set(raw)


def comparable_entry(entry: dict[str, Any]) -> dict[str, Any]:
    clone = dict(entry)
    clone["aliases"] = sorted(alias for alias in aliases(entry) if alias != "latest")
    return clone


def verify_root_artifacts(staged_branch: str, version: str, runner: CommandRunner) -> None:
    expected = ["skill.md", f"wavepeek_v{major_version(version)}.json"]
    missing: list[str] = []
    for artifact in expected:
        result = runner.run(
            ["git", "cat-file", "-t", f"{staged_branch}:{artifact}"],
            check=False,
            capture=True,
        )
        if result.returncode != 0 or (result.stdout or "").strip() != "blob":
            missing.append(artifact)
    if missing:
        fail("staged gh-pages bundle is missing root artifact(s): " + ", ".join(missing))


def verify_versions_semantics(
    *,
    remote_base: str | None,
    staged_branch: str,
    version: str,
    repair_existing_version: bool,
    runner: CommandRunner,
) -> None:
    base_entries = version_entries_by_name(
        git_show_json(remote_base, "versions.json", runner) if remote_base else None
    )
    staged_entries = version_entries_by_name(git_show_json(staged_branch, "versions.json", runner))

    if version not in staged_entries:
        fail(f"staged versions.json does not contain requested version {version}")
    if version in base_entries and not repair_existing_version:
        fail(f"version {version} already exists and repair mode is not enabled")

    base_unrelated = set(base_entries) - {version}
    staged_unrelated = set(staged_entries) - {version}
    if base_unrelated != staged_unrelated:
        fail("staged versions.json changes unrelated version entries")

    for name in base_unrelated:
        if comparable_entry(base_entries[name]) != comparable_entry(staged_entries[name]):
            fail(f"staged versions.json changes unrelated version {name}")

    latest_holders = [name for name, entry in staged_entries.items() if "latest" in aliases(entry)]
    if latest_holders != [version]:
        fail("staged versions.json must assign the latest alias only to the requested version")


def push_staged(
    *,
    version: str,
    repair_existing_version: bool,
    run_paths: Paths,
    runner: CommandRunner,
) -> None:
    validate_version(version)
    metadata = read_stage_metadata(run_paths, version, repair_existing_version)
    remote_base = current_remote_base(runner)
    if remote_base != metadata.get("remote_base"):
        fail("remote gh-pages base changed after staging")
    import_bundle(run_paths, metadata, runner)
    verify_fast_forward(remote_base, STAGED_BRANCH, runner)
    changed = changed_paths(remote_base, STAGED_BRANCH, runner)
    verify_allowed_paths(changed, allowed_path_patterns(version))
    verify_root_artifacts(STAGED_BRANCH, version, runner)
    verify_versions_semantics(
        remote_base=remote_base,
        staged_branch=STAGED_BRANCH,
        version=version,
        repair_existing_version=repair_existing_version,
        runner=runner,
    )
    runner.run(["git", "push", "origin", f"{STAGED_BRANCH}:{GH_PAGES_BRANCH}"])


def run_check_command(args: argparse.Namespace, runner: CommandRunner) -> None:
    run_paths = paths(args.work_dir)
    if args.source_root is not None and args.source_ref is not None:
        fail("check accepts either --source-root or --source-ref, not both")
    result = perform_check(
        version=validate_version(args.version),
        run_paths=run_paths,
        runner=runner,
        source_root=args.source_root,
        source_ref=args.source_ref,
        for_stage=False,
    )
    print(
        f"checked docs for wavepeek {result.cli_version}; "
        f"prepared {len(result.root_artifacts)} root artifact(s) under {run_paths.root_artifacts}"
    )


def main(argv: list[str] | None = None, runner: CommandRunner | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    runner = runner or CommandRunner()
    try:
        if args.command == "check":
            run_check_command(args, runner)
        elif args.command == "stage-deploy":
            metadata = stage_deploy(
                version=validate_version(args.version),
                source_ref=args.source_ref,
                repair_existing_version=args.repair_existing_version,
                run_paths=paths(args.work_dir),
                runner=runner,
            )
            print(
                f"staged gh-pages update {metadata['final_commit']} for {metadata['source_ref']}"
            )
        elif args.command == "push-staged":
            push_staged(
                version=validate_version(args.version),
                repair_existing_version=args.repair_existing_version,
                run_paths=paths(args.work_dir),
                runner=runner,
            )
            print(f"pushed gh-pages update for {args.version}")
        else:
            fail(f"unknown command {args.command!r}")
    except (PublishError, prepare_mkdocs.PrepareError) as error:
        print(f"error: docs-publish: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
