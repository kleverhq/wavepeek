#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import pathlib
import subprocess
import sys
from collections.abc import Mapping, Sequence
from typing import Any

import publish_docs

DEFAULT_WORK_DIR = "tmp/docs-site"
ACTIONS_BOT_EMAIL = "41898282+github-actions[bot]@users.noreply.github.com"
ACTIONS_BOT_NAME = "github-actions[bot]"


class WorkflowError(Exception):
    pass


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="GitHub Actions wrappers for wavepeek docs publication."
    )
    subcommands = parser.add_subparsers(dest="command", required=True)

    validate = subcommands.add_parser(
        "validate-dispatch", help="validate docs workflow dispatch inputs"
    )
    validate.add_argument("--version")
    validate.add_argument("--source-ref")
    validate.add_argument("--dispatch-ref")
    validate.add_argument("--default-branch")
    validate.add_argument("--repository")
    validate.add_argument("--release-json", type=pathlib.Path)

    subcommands.add_parser("stage-deploy", help="run docs stage-deploy from workflow env")
    subcommands.add_parser("push-staged", help="run docs push-staged from workflow env")
    return parser.parse_args(list(argv))


def env_value(
    env: Mapping[str, str],
    name: str,
    override: str | None = None,
) -> str:
    value = override if override is not None else env.get(name)
    if not value:
        raise WorkflowError(f"missing required {name}")
    return value


def repair_enabled(env: Mapping[str, str]) -> bool:
    return env.get("REPAIR_EXISTING_VERSION", "false").lower() == "true"


def load_release_from_github(repository: str, source_ref: str) -> dict[str, Any]:
    result = subprocess.run(
        ["gh", "api", f"repos/{repository}/releases/tags/{source_ref}"],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        message = (result.stderr or result.stdout).strip()
        raise WorkflowError(
            f"failed to load GitHub Release for {source_ref}: {message}"
        )
    try:
        release = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise WorkflowError(f"GitHub Release response is not valid JSON: {error}") from error
    if not isinstance(release, dict):
        raise WorkflowError("GitHub Release response must be a JSON object")
    return release


def load_release(path: pathlib.Path | None, repository: str, source_ref: str) -> dict[str, Any]:
    if path is None:
        return load_release_from_github(repository, source_ref)
    try:
        release = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise WorkflowError(f"release JSON is not valid JSON: {error}") from error
    if not isinstance(release, dict):
        raise WorkflowError("release JSON must be an object")
    return release


def validate_release_is_stable(release: Mapping[str, Any]) -> None:
    if release.get("draft") or release.get("prerelease"):
        raise WorkflowError("docs publication requires a published stable GitHub Release")


def validate_dispatch(args: argparse.Namespace, env: Mapping[str, str]) -> None:
    version = env_value(env, "VERSION", args.version)
    source_ref = env_value(env, "SOURCE_REF", args.source_ref)
    dispatch_ref = env_value(env, "DISPATCH_REF", args.dispatch_ref)
    default_branch = env_value(env, "DEFAULT_BRANCH", args.default_branch)
    repository = env_value(env, "GITHUB_REPOSITORY", args.repository)

    if dispatch_ref != default_branch:
        raise WorkflowError(f"docs publication must be dispatched from {default_branch}")
    publish_docs.validate_version(version)
    publish_docs.require_ref_matches_version(source_ref, version)
    release = load_release(args.release_json, repository, source_ref)
    validate_release_is_stable(release)


def workflow_publish_args(command: str, env: Mapping[str, str]) -> list[str]:
    version = env_value(env, "VERSION")
    args = [command, "--version", version, "--work-dir", DEFAULT_WORK_DIR]
    if command == "stage-deploy":
        args.extend(["--source-ref", env_value(env, "SOURCE_REF")])
    elif command != "push-staged":
        raise WorkflowError(f"unsupported workflow publish command {command!r}")
    if repair_enabled(env):
        args.append("--repair-existing-version")
    return args


def configure_actions_git() -> None:
    subprocess.run(
        ["git", "config", "--global", "user.email", ACTIONS_BOT_EMAIL], check=True
    )
    subprocess.run(["git", "config", "--global", "user.name", ACTIONS_BOT_NAME], check=True)


def run_publish(command: str, env: Mapping[str, str]) -> int:
    configure_actions_git()
    return publish_docs.main(workflow_publish_args(command, env))


def main(argv: Sequence[str] | None = None, env: Mapping[str, str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    env = os.environ if env is None else env
    try:
        if args.command == "validate-dispatch":
            validate_dispatch(args, env)
            print("validated docs workflow dispatch")
            return 0
        if args.command in {"stage-deploy", "push-staged"}:
            return run_publish(args.command, env)
        raise WorkflowError(f"unknown command {args.command!r}")
    except (WorkflowError, publish_docs.PublishError) as error:
        print(f"error: docs-workflow: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
