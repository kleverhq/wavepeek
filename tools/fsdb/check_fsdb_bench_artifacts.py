#!/usr/bin/env python3
"""Verify converted FSDB benchmark artifacts exist under RTL_ARTIFACTS_DIR."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import sys
from typing import Any

RTL_ARTIFACTS_ENV = "RTL_ARTIFACTS_DIR"


def collect_canonical_fsdb_paths(
    value: Any, out: set[str], artifact_path_re: re.Pattern[str]
) -> None:
    if isinstance(value, str):
        match = artifact_path_re.fullmatch(value)
        if match:
            out.add(match.group(1))
        return
    if isinstance(value, list):
        for item in value:
            collect_canonical_fsdb_paths(item, out, artifact_path_re)
        return
    if isinstance(value, dict):
        for item in value.values():
            collect_canonical_fsdb_paths(item, out, artifact_path_re)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Verify generated RTL FSDB benchmark artifacts exist."
    )
    parser.add_argument("catalog", help="path to bench/e2e/tests_fsdb.json")
    parser.add_argument(
        "--filter",
        help="optional regular expression selecting benchmark test names to verify",
    )
    return parser.parse_args(argv[1:])


def load_catalog(catalog_path: Path) -> Any:
    try:
        return json.loads(catalog_path.read_text(encoding="utf-8"))
    except OSError as error:
        print(f"error: file: failed to read {catalog_path}: {error}", file=sys.stderr)
        return None
    except json.JSONDecodeError as error:
        print(f"error: file: invalid JSON in {catalog_path}: {error}", file=sys.stderr)
        return None


def filter_catalog(catalog: Any, pattern: str | None) -> Any:
    if pattern is None:
        return catalog
    if not isinstance(catalog, dict) or not isinstance(catalog.get("tests"), list):
        print(
            "error: tests: root catalog must be an object with a tests array when --filter is used",
            file=sys.stderr,
        )
        return None
    try:
        regex = re.compile(pattern)
    except re.error as error:
        print(f"error: filter: invalid regex {pattern!r}: {error}", file=sys.stderr)
        return None

    selected = [
        test
        for test in catalog["tests"]
        if isinstance(test, dict) and regex.search(str(test.get("name", "")))
    ]
    if not selected:
        print(
            f"error: filter: no FSDB benchmark tests matched {pattern!r}",
            file=sys.stderr,
        )
        return None
    return {"tests": selected}


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    rtl_dir_value = os.environ.get(RTL_ARTIFACTS_ENV)
    if not rtl_dir_value:
        print(
            f"error: file: {RTL_ARTIFACTS_ENV} is not set by the wavepeek container",
            file=sys.stderr,
        )
        return 1

    rtl_dir = Path(rtl_dir_value)
    catalog_path = Path(args.catalog)
    catalog = load_catalog(catalog_path)
    if catalog is None:
        return 1
    catalog = filter_catalog(catalog, args.filter)
    if catalog is None:
        return 1

    required: set[str] = set()
    artifact_path_re = re.compile(rf"{re.escape(str(rtl_dir))}/([^/]+\.fsdb)\Z")
    collect_canonical_fsdb_paths(catalog, required, artifact_path_re)

    missing = sorted(path for path in required if not (rtl_dir / path).is_file())
    if missing:
        for path in missing:
            print(
                f"error: file: required FSDB benchmark fixture missing at {rtl_dir / path}",
                file=sys.stderr,
            )
        return 1

    print(
        f"info: fsdb fixture: verified {len(required)} RTL benchmark FSDB artifacts under {rtl_dir}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
