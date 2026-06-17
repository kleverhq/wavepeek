#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import pathlib
import re
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request
from typing import Any, Mapping, Sequence

CRATE_NAME = "wavepeek"
VERSION_RE = re.compile(r"^(0|[1-9][0-9]*)[.](0|[1-9][0-9]*)[.](0|[1-9][0-9]*)$")
SOURCE_REF_RE = re.compile(r"^v(?P<version>(0|[1-9][0-9]*)[.](0|[1-9][0-9]*)[.](0|[1-9][0-9]*))$")
GIT_LOCAL_ENV = (
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_WORK_TREE",
)


class ReleaseError(RuntimeError):
    pass


def fail(message: str) -> None:
    print(f"error: publish-crate: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_stable_version(version: str) -> str:
    if VERSION_RE.fullmatch(version) is None:
        raise ReleaseError(f"version must be stable SemVer X.Y.Z, got {version!r}")
    return version


def validate_source_ref(version: str, source_ref: str) -> str:
    match = SOURCE_REF_RE.fullmatch(source_ref)
    if match is None:
        raise ReleaseError(f"source_ref must be a stable SemVer tag vX.Y.Z, got {source_ref!r}")
    ref_version = match.group("version")
    if ref_version != version:
        raise ReleaseError(f"source_ref {source_ref!r} does not match version {version!r}")
    return source_ref


def load_json(path: pathlib.Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise ReleaseError(f"failed to read {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise ReleaseError(f"{path} is not valid JSON: {error}") from error
    if not isinstance(data, dict):
        raise ReleaseError(f"{path} must contain a JSON object")
    return data


def fetch_json(url: str, token: str | None = None) -> dict[str, Any] | None:
    headers = {
        "Accept": "application/vnd.github+json, application/json",
        "User-Agent": "wavepeek-release-helper",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            data = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise ReleaseError(f"GET {url} failed with HTTP {error.code}") from error
    except (OSError, json.JSONDecodeError) as error:
        raise ReleaseError(f"GET {url} failed: {error}") from error
    if not isinstance(data, dict):
        raise ReleaseError(f"GET {url} returned non-object JSON")
    return data


def validate_release_object(release: Mapping[str, Any], version: str, source_ref: str) -> None:
    tag_name = release.get("tag_name")
    if tag_name != source_ref:
        raise ReleaseError(f"GitHub Release tag_name {tag_name!r} does not match {source_ref!r}")
    if release.get("draft") is True:
        raise ReleaseError(f"GitHub Release {source_ref} is a draft")
    if release.get("prerelease") is True:
        raise ReleaseError(f"GitHub Release {source_ref} is marked as a prerelease")
    validate_source_ref(version, source_ref)


def load_release(
    *,
    repository: str,
    version: str,
    source_ref: str,
    release_json: pathlib.Path | None,
    token: str | None,
) -> dict[str, Any]:
    release = load_json(release_json) if release_json else fetch_json(
        f"https://api.github.com/repos/{repository}/releases/tags/{source_ref}", token
    )
    if release is None:
        raise ReleaseError(f"GitHub Release {source_ref} does not exist in {repository}")
    validate_release_object(release, version, source_ref)
    return dict(release)


def validate_dispatch_inputs(
    *,
    version: str,
    source_ref: str,
    dispatch_ref: str,
    default_branch: str,
    repository: str,
    release_json: pathlib.Path | None,
    token: str | None,
) -> None:
    validate_stable_version(version)
    validate_source_ref(version, source_ref)
    if dispatch_ref != default_branch:
        raise ReleaseError(
            f"workflow must run from default branch {default_branch!r}, got {dispatch_ref!r}"
        )
    load_release(
        repository=repository,
        version=version,
        source_ref=source_ref,
        release_json=release_json,
        token=token,
    )


def read_package_version(source_root: pathlib.Path) -> str:
    cargo_toml = source_root / "Cargo.toml"
    try:
        data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    except OSError as error:
        raise ReleaseError(f"failed to read {cargo_toml}: {error}") from error
    package = data.get("package")
    if not isinstance(package, dict):
        raise ReleaseError(f"{cargo_toml} is missing [package]")
    name = package.get("name")
    if name != CRATE_NAME:
        raise ReleaseError(f"{cargo_toml} package.name must be {CRATE_NAME!r}, got {name!r}")
    version = package.get("version")
    if not isinstance(version, str):
        raise ReleaseError(f"{cargo_toml} package.version must be a string")
    return version


def child_env_without_git_local(env: Mapping[str, str] | None = None) -> dict[str, str]:
    command_env = os.environ.copy() if env is None else dict(env)
    for name in GIT_LOCAL_ENV:
        command_env.pop(name, None)
    return command_env


def run_capture(command: Sequence[str], *, cwd: pathlib.Path) -> str:
    try:
        result = subprocess.run(
            list(command),
            cwd=cwd,
            check=True,
            capture_output=True,
            text=True,
            env=child_env_without_git_local(),
        )
    except subprocess.CalledProcessError as error:
        stderr = (error.stderr or error.stdout or "").strip()
        raise ReleaseError(f"command {' '.join(command)!r} failed: {stderr}") from error
    return result.stdout.strip()


def verify_source_checkout(source_root: pathlib.Path, source_ref: str) -> None:
    if not source_root.is_dir():
        raise ReleaseError(f"source root {source_root} does not exist")
    head = run_capture(["git", "rev-parse", "--verify", "HEAD"], cwd=source_root)
    tag_commit = run_capture(
        ["git", "rev-parse", "--verify", f"refs/tags/{source_ref}^{{commit}}"],
        cwd=source_root,
    )
    if head != tag_commit:
        raise ReleaseError(
            f"source checkout HEAD {head} does not match refs/tags/{source_ref} {tag_commit}"
        )


def crate_json_says_published(data: Mapping[str, Any], version: str) -> bool:
    explicit = data.get("published")
    if isinstance(explicit, bool):
        return explicit
    version_obj = data.get("version")
    if isinstance(version_obj, Mapping) and version_obj.get("num") == version:
        return True
    versions = data.get("versions")
    if isinstance(versions, list):
        return any(isinstance(item, Mapping) and item.get("num") == version for item in versions)
    return False


def crate_version_published(version: str, crate_json: pathlib.Path | None) -> bool:
    if crate_json:
        return crate_json_says_published(load_json(crate_json), version)
    response = fetch_json(f"https://crates.io/api/v1/crates/{CRATE_NAME}/{version}")
    return response is not None


def registry_token(env: Mapping[str, str]) -> str | None:
    return env.get("CARGO_REGISTRY_TOKEN") or env.get("CRATES_IO_TOKEN")


def publish_crate(
    *,
    version: str,
    source_ref: str,
    source_root: pathlib.Path,
    repository: str,
    release_json: pathlib.Path | None,
    crate_json: pathlib.Path | None,
    cargo_bin: str,
    env: Mapping[str, str],
) -> None:
    validate_stable_version(version)
    validate_source_ref(version, source_ref)
    load_release(
        repository=repository,
        version=version,
        source_ref=source_ref,
        release_json=release_json,
        token=env.get("GH_TOKEN") or env.get("GITHUB_TOKEN"),
    )
    verify_source_checkout(source_root, source_ref)
    package_version = read_package_version(source_root)
    if package_version != version:
        raise ReleaseError(
            f"release source Cargo.toml version {package_version!r} does not match {version!r}"
        )
    if crate_version_published(version, crate_json):
        print(f"crate {CRATE_NAME} {version} is already published; no-op")
        return
    token = registry_token(env)
    if not token:
        raise ReleaseError(
            "CARGO_REGISTRY_TOKEN or CRATES_IO_TOKEN is required because the crate version is not published"
        )
    cargo_env = dict(env)
    cargo_env["CARGO_REGISTRY_TOKEN"] = token
    subprocess.run(
        [cargo_bin, "publish", "--locked"],
        cwd=source_root,
        check=True,
        env=child_env_without_git_local(cargo_env),
    )


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate and publish wavepeek to crates.io.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser("validate-dispatch")
    validate_parser.add_argument("--version", default=os.environ.get("VERSION"))
    validate_parser.add_argument("--source-ref", default=os.environ.get("SOURCE_REF"))
    validate_parser.add_argument("--dispatch-ref", default=os.environ.get("DISPATCH_REF"))
    validate_parser.add_argument("--default-branch", default=os.environ.get("DEFAULT_BRANCH"))
    validate_parser.add_argument("--repository", default=os.environ.get("GITHUB_REPOSITORY"))
    validate_parser.add_argument("--release-json", type=pathlib.Path)

    publish_parser = subparsers.add_parser("publish")
    publish_parser.add_argument("--version", default=os.environ.get("VERSION"))
    publish_parser.add_argument("--source-ref", default=os.environ.get("SOURCE_REF"))
    publish_parser.add_argument("--source-root", type=pathlib.Path, required=True)
    publish_parser.add_argument("--repository", default=os.environ.get("GITHUB_REPOSITORY"))
    publish_parser.add_argument("--release-json", type=pathlib.Path)
    publish_parser.add_argument("--crate-json", type=pathlib.Path)
    publish_parser.add_argument("--cargo-bin", default="cargo")
    return parser.parse_args(list(argv))


def required_arg(name: str, value: str | None) -> str:
    if not value:
        raise ReleaseError(f"missing required argument {name}")
    return value


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        if args.command == "validate-dispatch":
            validate_dispatch_inputs(
                version=required_arg("--version", args.version),
                source_ref=required_arg("--source-ref", args.source_ref),
                dispatch_ref=required_arg("--dispatch-ref", args.dispatch_ref),
                default_branch=required_arg("--default-branch", args.default_branch),
                repository=required_arg("--repository", args.repository),
                release_json=args.release_json,
                token=os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN"),
            )
            print("crate publication dispatch inputs are valid")
        elif args.command == "publish":
            publish_crate(
                version=required_arg("--version", args.version),
                source_ref=required_arg("--source-ref", args.source_ref),
                source_root=args.source_root,
                repository=required_arg("--repository", args.repository),
                release_json=args.release_json,
                crate_json=args.crate_json,
                cargo_bin=args.cargo_bin,
                env=os.environ,
            )
        else:
            raise ReleaseError(f"unknown command {args.command!r}")
    except ReleaseError as error:
        fail(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
