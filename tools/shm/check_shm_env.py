#!/usr/bin/env python3
"""Probe the optional Ondas Xcelium reader; ABI checks remain owned by Ondas."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import platform
import sys


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--require", action="store_true")
    parser.add_argument("--print-library", action="store_true")
    args = parser.parse_args()
    home = os.environ.get("XCELIUM_HOME")
    if not home:
        message = "Xcelium reader not configured; set XCELIUM_HOME"
        print(f"{'error' if args.require else 'skip'}: shm: {message}")
        sys.exit(1 if args.require else 77)
    if platform.system() != "Linux" or platform.machine() != "x86_64":
        sys.exit("error: shm: Ondas requires Linux x86_64 GNU")
    library = Path(home).expanduser() / "tools/lib/64bit/libsst2.so"
    if not library.is_file():
        sys.exit(f"error: shm: missing XCELIUM_HOME/tools/lib/64bit/libsst2.so: {library}")
    print(library.resolve() if args.print_library else "ok: shm: Xcelium reader found")


if __name__ == "__main__":
    main()
