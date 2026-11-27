#!/usr/bin/env python3
"""Probe one optional Ondas Xcelium reader."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import platform
import sys

LIBRARIES = {"shm": "libsst2.so", "vwdb": "libvwdbr_analog.so"}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reader", choices=LIBRARIES)
    parser.add_argument("--require", action="store_true")
    parser.add_argument("--print-library", action="store_true")
    args = parser.parse_args()
    reader = args.reader
    home = os.environ.get("XCELIUM_HOME")
    if not home:
        message = "Xcelium reader not configured; set XCELIUM_HOME"
        print(f"{'error' if args.require else 'skip'}: {reader}: {message}",
              file=sys.stderr if args.require else sys.stdout)
        sys.exit(1 if args.require else 77)
    if platform.system() != "Linux" or platform.machine() != "x86_64":
        sys.exit(f"error: {reader}: Ondas requires Linux x86_64 GNU")
    relative_path = Path("tools/lib/64bit") / LIBRARIES[reader]
    library = Path(home).expanduser() / relative_path
    if not library.is_file():
        sys.exit(f"error: {reader}: missing XCELIUM_HOME/{relative_path}: {library}")
    print(library.resolve() if args.print_library else f"ok: {reader}: Xcelium reader found")


if __name__ == "__main__":
    main()
