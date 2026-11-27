#!/usr/bin/env python3
"""Locate the reader files required by Ondas."""

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
    home = os.environ.get("QUESTA_HOME")
    if not home:
        if args.require:
            sys.exit("error: wlf: set QUESTA_HOME to a compatible Questa installation")
        print("skip: wlf: Questa reader not configured; set QUESTA_HOME")
        sys.exit(77)
    if platform.system() != "Linux" or platform.machine() != "x86_64":
        sys.exit("error: wlf: Ondas requires Linux x86_64 GNU")
    root = Path(home).expanduser()
    library = root / "linux_x86_64/libwlf.so"
    for path in (root / "include/wlf_api.h", library):
        if not path.is_file():
            sys.exit(f"error: wlf: missing reader file: {path}")
    print(library.resolve() if args.print_library else "ok: wlf: Questa reader found")


if __name__ == "__main__":
    main()
