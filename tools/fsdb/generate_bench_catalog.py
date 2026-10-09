#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import pathlib
import sys


DEFAULT_SOURCE = pathlib.Path("bench/e2e/tests.json")
FST_SUFFIX = ".fst"
OUTPUTS = {
    "fsdb": pathlib.Path("bench/e2e/tests_fsdb.json"),
    "vcd": pathlib.Path("bench/e2e/tests_vcd.json"),
}


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def generate_catalog(source_path: pathlib.Path, target: str = "fsdb") -> tuple[str, int]:
    try:
        source = source_path.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"error: {target} catalog: failed to read {source_path}: {error}")

    try:
        json.loads(source)
    except json.JSONDecodeError as error:
        fail(f"error: {target} catalog: invalid JSON in {source_path}: {error}")

    count = source.count(FST_SUFFIX)
    if count == 0:
        fail(f"error: {target} catalog: no {FST_SUFFIX} suffixes found in {source_path}")

    generated = source.replace(FST_SUFFIX, f".{target}")
    if target == "fsdb":
        generated = generated.replace("tests/fixtures/generated/", "tests/fixtures/fsdb/")
    return generated, count


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate an e2e benchmark catalog from the FST catalog."
    )
    parser.add_argument(
        "--source",
        default=str(DEFAULT_SOURCE),
        help=f"Source FST benchmark catalog (default: {DEFAULT_SOURCE}).",
    )
    parser.add_argument(
        "--target",
        choices=tuple(OUTPUTS),
        default="fsdb",
        help="Target waveform format (default: fsdb).",
    )
    parser.add_argument(
        "--output",
        help="Output catalog (default: bench/e2e/tests_<target>.json).",
    )
    parser.add_argument(
        "--artifact-dir",
        default=os.environ.get("RTL_ARTIFACTS_DIR", "/opt/rtl-artifacts"),
        help="Compatibility option retained for existing callers; catalog generation "
        "replaces every .fst suffix in the source text.",
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
    output_path = pathlib.Path(args.output) if args.output else OUTPUTS[args.target]
    generated, count = generate_catalog(source_path, args.target)

    if args.check:
        try:
            current = output_path.read_text(encoding="utf-8")
        except OSError as error:
            fail(f"error: {args.target} catalog: failed to read {output_path}: {error}")
        if current != generated:
            fail(
                f"error: {args.target} catalog: "
                f"{output_path} is stale; run `just update-bench-e2e-{args.target}-catalog`"
            )
        print(f"ok: {args.target} catalog: {output_path} matches {source_path} ({count} suffixes)")
        return

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(generated, encoding="utf-8")
    print(f"info: {args.target} catalog: wrote {output_path} from {source_path} ({count} suffixes)")


if __name__ == "__main__":
    main()
