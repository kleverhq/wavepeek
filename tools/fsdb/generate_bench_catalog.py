#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import pathlib
import sys
from typing import Any, NamedTuple


DEFAULT_SOURCE = pathlib.Path("bench/e2e/tests.json")
DEFAULT_OUTPUT = pathlib.Path("bench/e2e/tests_fsdb.json")
FST_SUFFIX = ".fst"
FSDB_SUFFIX = ".fsdb"


class Conversion(NamedTuple):
    value: Any
    count: int


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def convert_artifact_paths(value: Any, artifact_dir: str) -> Conversion:
    artifact_prefix = artifact_dir.rstrip("/") + "/"

    if isinstance(value, str):
        if value.startswith(artifact_prefix) and value.endswith(FST_SUFFIX):
            return Conversion(value[: -len(FST_SUFFIX)] + FSDB_SUFFIX, 1)
        return Conversion(value, 0)

    if isinstance(value, list):
        converted_items: list[Any] = []
        count = 0
        for item in value:
            converted = convert_artifact_paths(item, artifact_dir)
            converted_items.append(converted.value)
            count += converted.count
        return Conversion(converted_items, count)

    if isinstance(value, dict):
        converted_items: dict[str, Any] = {}
        count = 0
        for key, item in value.items():
            converted = convert_artifact_paths(item, artifact_dir)
            converted_items[str(key)] = converted.value
            count += converted.count
        return Conversion(converted_items, count)

    return Conversion(value, 0)


def generate_catalog(source_path: pathlib.Path, artifact_dir: str) -> tuple[str, int]:
    try:
        payload = json.loads(source_path.read_text(encoding="utf-8"))
    except OSError as error:
        fail(f"error: fsdb catalog: failed to read {source_path}: {error}")
    except json.JSONDecodeError as error:
        fail(f"error: fsdb catalog: invalid JSON in {source_path}: {error}")

    converted = convert_artifact_paths(payload, artifact_dir)
    if converted.count == 0:
        fail(
            "error: fsdb catalog: no FST artifact paths found under "
            f"{artifact_dir.rstrip('/')}/ in {source_path}"
        )

    return json.dumps(converted.value, indent=2) + "\n", converted.count


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate the FSDB e2e benchmark catalog from the FST catalog."
    )
    parser.add_argument(
        "--source",
        default=str(DEFAULT_SOURCE),
        help=f"Source FST benchmark catalog (default: {DEFAULT_SOURCE}).",
    )
    parser.add_argument(
        "--output",
        default=str(DEFAULT_OUTPUT),
        help=f"Output FSDB benchmark catalog (default: {DEFAULT_OUTPUT}).",
    )
    parser.add_argument(
        "--artifact-dir",
        default=os.environ.get("RTL_ARTIFACTS_DIR", "/opt/rtl-artifacts"),
        help="RTL artifact directory used in benchmark catalogs "
        "(default: RTL_ARTIFACTS_DIR or /opt/rtl-artifacts).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check that the output catalog is up to date without rewriting it.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    source_path = pathlib.Path(args.source)
    output_path = pathlib.Path(args.output)
    generated, count = generate_catalog(source_path, str(args.artifact_dir))

    if args.check:
        try:
            current = output_path.read_text(encoding="utf-8")
        except OSError as error:
            fail(f"error: fsdb catalog: failed to read {output_path}: {error}")
        if current != generated:
            fail(
                "error: fsdb catalog: "
                f"{output_path} is stale; run `just update-bench-e2e-fsdb-catalog`"
            )
        print(f"ok: fsdb catalog: {output_path} matches {source_path} ({count} paths)")
        return

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(generated, encoding="utf-8")
    print(f"info: fsdb catalog: wrote {output_path} from {source_path} ({count} paths)")


if __name__ == "__main__":
    main()
