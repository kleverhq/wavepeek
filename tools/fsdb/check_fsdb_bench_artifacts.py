#!/usr/bin/env python3
"""Verify converted FSDB benchmark artifacts exist under RTL_ARTIFACTS_DIR."""

from __future__ import annotations

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


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check_fsdb_bench_artifacts.py <tests_fsdb.json>", file=sys.stderr)
        return 2

    rtl_dir_value = os.environ.get(RTL_ARTIFACTS_ENV)
    if not rtl_dir_value:
        print(
            f"error: file: {RTL_ARTIFACTS_ENV} is not set by the wavepeek container",
            file=sys.stderr,
        )
        return 1

    rtl_dir = Path(rtl_dir_value)
    catalog_path = Path(argv[1])
    try:
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    except OSError as error:
        print(f"error: file: failed to read {catalog_path}: {error}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as error:
        print(f"error: file: invalid JSON in {catalog_path}: {error}", file=sys.stderr)
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
