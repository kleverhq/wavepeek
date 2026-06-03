#!/usr/bin/env python3

from __future__ import annotations

import argparse
import os
import pathlib
import sys
import tomllib


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def load_crate_version(manifest_path: pathlib.Path) -> str:
    try:
        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    except OSError as error:
        fail(f"error: release-tag: failed to read {manifest_path}: {error}")
    except tomllib.TOMLDecodeError as error:
        fail(f"error: release-tag: invalid TOML in {manifest_path}: {error}")

    try:
        version = manifest["package"]["version"]
    except KeyError:
        fail(f"error: release-tag: missing package.version in {manifest_path}")

    if not isinstance(version, str) or not version:
        fail(f"error: release-tag: package.version in {manifest_path} must be a string")

    return version


def validate_tag_version(tag: str, crate_version: str) -> str:
    if not tag.startswith("v"):
        fail(f"error: release-tag: expected tag that starts with 'v', got {tag!r}")

    tag_version = tag[1:]
    if tag_version != crate_version:
        fail(
            "error: release-tag: tag version "
            f"({tag_version}) does not match Cargo.toml version ({crate_version})"
        )

    return tag_version


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate that the release git tag matches Cargo.toml package.version."
    )
    parser.add_argument(
        "--manifest",
        default="Cargo.toml",
        help="Path to Cargo.toml (default: Cargo.toml).",
    )
    parser.add_argument(
        "--tag",
        default=os.environ.get("GITHUB_REF_NAME"),
        help="Git tag to validate (default: GITHUB_REF_NAME).",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if not args.tag:
        fail("error: release-tag: tag is required; pass --tag or set GITHUB_REF_NAME")

    crate_version = load_crate_version(pathlib.Path(args.manifest))
    tag_version = validate_tag_version(args.tag, crate_version)
    print(f"tag_version={tag_version}")


if __name__ == "__main__":
    main()
