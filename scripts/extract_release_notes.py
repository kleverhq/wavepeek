#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import re
import sys


VERSION_HEADING_TEMPLATE = r"^## \[{version}\] - (?P<date>\d{{4}}-\d{{2}}-\d{{2}})$"
NEXT_VERSION_HEADING_RE = re.compile(r"^## \[[^\]]+\](?: - \d{4}-\d{2}-\d{2})?$")
REFERENCE_LINK_RE = re.compile(r"^\[[^\]]+\]:\s+https?://")


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def extract_release_notes(changelog_text: str, version: str) -> str:
    version_heading_re = re.compile(VERSION_HEADING_TEMPLATE.format(version=re.escape(version)))
    lines = changelog_text.splitlines()

    start_index: int | None = None
    for index, line in enumerate(lines):
        if version_heading_re.match(line):
            start_index = index + 1
            break

    if start_index is None:
        fail(
            "error: release-notes: version "
            f"{version} not found in changelog heading '## [{version}] - YYYY-MM-DD'"
        )
    assert start_index is not None
    release_body_start = start_index

    end_index = len(lines)
    for index in range(release_body_start, len(lines)):
        if NEXT_VERSION_HEADING_RE.match(lines[index]):
            end_index = index
            break
        if REFERENCE_LINK_RE.match(lines[index]):
            end_index = index
            break

    body_lines = lines[release_body_start:end_index]
    while body_lines and not body_lines[0].strip():
        body_lines.pop(0)
    while body_lines and not body_lines[-1].strip():
        body_lines.pop()

    if not body_lines:
        fail(f"error: release-notes: version {version} section is empty")

    return "\n".join(body_lines) + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract release notes for a version from CHANGELOG.md."
    )
    parser.add_argument(
        "--changelog",
        default="CHANGELOG.md",
        help="Path to changelog file (default: CHANGELOG.md).",
    )
    parser.add_argument(
        "--version",
        required=True,
        help="Version to extract, without the leading v (example: 1.2.3).",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    changelog_path = pathlib.Path(args.changelog)
    if not changelog_path.exists():
        fail(f"error: release-notes: changelog file not found: {changelog_path}")

    changelog_text = changelog_path.read_text(encoding="utf-8")
    sys.stdout.write(extract_release_notes(changelog_text, args.version))


if __name__ == "__main__":
    main()
