#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="timeout wrapper for wavepeek benchmark calls")
    parser.add_argument("--timeout-seconds", required=True, type=int)
    parser.add_argument("--label", default="unknown-test")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    timeout_seconds = int(args.timeout_seconds)
    if timeout_seconds < 1:
        print("error: timeout wrapper: --timeout-seconds must be >= 1", file=sys.stderr)
        return 2

    command = list(args.command)
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        print("error: timeout wrapper: command is required", file=sys.stderr)
        return 2

    try:
        result = subprocess.run(
            command,
            check=False,
            cwd=REPO_ROOT,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired:
        print(
            f"warning: run: benchmark timeout cap reached for `{args.label}` "
            f"({timeout_seconds}s)",
            file=sys.stderr,
        )
        return 0

    return int(result.returncode)


if __name__ == "__main__":
    raise SystemExit(main())
