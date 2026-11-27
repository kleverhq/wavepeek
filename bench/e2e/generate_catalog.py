#!/usr/bin/env python3
"""Write a format-specific E2E catalog and print its path."""

from __future__ import annotations

import argparse
import json
import pathlib
import tempfile


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
DEFAULT_SOURCE = SCRIPT_DIR / "tests.json"
OUTPUT_ROOT = SCRIPT_DIR.parents[1] / "tmp/bench-catalogs"
FST_SUFFIX = ".fst"


def fail(message: str) -> None:
    raise SystemExit(message)


def generate_catalog(source_path: pathlib.Path, target: str) -> str:
    try:
        source = source_path.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"error: {target} catalog: failed to read {source_path}: {error}")

    try:
        json.loads(source)
    except json.JSONDecodeError as error:
        fail(f"error: {target} catalog: invalid JSON in {source_path}: {error}")

    if FST_SUFFIX not in source:
        fail(f"error: {target} catalog: no {FST_SUFFIX} suffixes found in {source_path}")

    generated = source.replace(FST_SUFFIX, f".{target}")
    if target in ("fsdb", "shm"):
        generated = generated.replace("tests/fixtures/generated/", f"tests/fixtures/{target}/")
    return generated


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", choices=("vcd", "fsdb", "shm"), help="target waveform format")
    parser.add_argument("--source", type=pathlib.Path, default=DEFAULT_SOURCE)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    generated = generate_catalog(args.source, args.target)
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    # Keep the catalog referenced by each run manifest available and stable.
    output_dir = pathlib.Path(tempfile.mkdtemp(prefix=f"{args.target}-", dir=OUTPUT_ROOT))
    output_path = output_dir / "tests.json"
    output_path.write_text(generated, encoding="utf-8")
    print(output_path)


if __name__ == "__main__":
    main()
