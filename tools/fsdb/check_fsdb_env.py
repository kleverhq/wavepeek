#!/usr/bin/env python3

from __future__ import annotations

import argparse
import os
import pathlib
import sys

SKIP_STATUS = 77
SKIP_MESSAGE = "skip: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run FSDB build checks"
REQUIRED_HEADERS = ("ffrAPI.h", "ffrKit.h", "fsdbShr.h")
REQUIRED_LIBRARIES = ("libnffr.so", "libnsys.so")


def eprint(message: str) -> None:
    print(message, file=sys.stderr)


def fail(message: str) -> None:
    eprint(f"error: fsdb: {message}")
    raise SystemExit(1)


def skip() -> None:
    print(SKIP_MESSAGE)
    raise SystemExit(SKIP_STATUS)


def env_path(name: str) -> pathlib.Path | None:
    value = os.environ.get(name)
    if value is None or value == "":
        return None
    return pathlib.Path(value).expanduser()


def reader_root(verdi_home: pathlib.Path) -> pathlib.Path:
    return verdi_home / "share" / "FsdbReader"


def missing_headers(verdi_home: pathlib.Path) -> list[pathlib.Path]:
    root = reader_root(verdi_home)
    return [root / name for name in REQUIRED_HEADERS if not (root / name).is_file()]


def selected_libdir(verdi_home: pathlib.Path) -> pathlib.Path:
    for abi in ("linux64", "LINUX64"):
        libdir = reader_root(verdi_home) / abi
        if not missing_libraries(libdir):
            return libdir
    return reader_root(verdi_home) / "linux64"


def missing_libraries(libdir: pathlib.Path) -> list[pathlib.Path]:
    return [libdir / name for name in REQUIRED_LIBRARIES if not (libdir / name).is_file()]


def verbose_output_enabled() -> bool:
    return os.environ.get("WAVEPEEK_FSDB_ENV_VERBOSE") == "1"


def unavailable(required: bool) -> None:
    if required:
        fail("Verdi FSDB Reader SDK not found; set VERDI_HOME to run this target")
    skip()


def validate_sdk(required: bool) -> tuple[pathlib.Path, pathlib.Path]:
    for name in (
        "WAVEPEEK_FSDB_ABI",
        "WAVEPEEK_FSDB_READER_LIBDIR",
        "WAVEPEEK_FSDB_EMBED_RPATH",
    ):
        if os.environ.get(name):
            fail(f"{name} is not supported by Ondas; unset it and select the SDK with VERDI_HOME")

    verdi_home = env_path("VERDI_HOME")
    if verdi_home is None or missing_headers(verdi_home):
        unavailable(required)

    libdir = selected_libdir(verdi_home)
    library_misses = missing_libraries(libdir)
    if library_misses:
        if libdir.exists():
            missing = library_misses[0]
            missing_text = str(missing) if verbose_output_enabled() else missing.name
            fail(
                "selected FSDB Reader library directory is incomplete; "
                f"missing {missing_text}; Ondas requires linux64 or LINUX64 under share/FsdbReader"
            )
        unavailable(required)
    return verdi_home, libdir


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="check local Verdi FSDB Reader SDK availability")
    parser.add_argument(
        "--require",
        action="store_true",
        help="treat missing Verdi as an error instead of an optional skip",
    )
    parser.add_argument(
        "--print-libdir",
        action="store_true",
        help="print the selected FSDB Reader library directory after validation",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    verdi_home, libdir = validate_sdk(required=args.require)
    if args.print_libdir:
        print(libdir)
        return

    verbose = verbose_output_enabled()
    if verbose:
        print(f"ok: fsdb: Verdi FSDB Reader SDK found at {verdi_home} (libdir {libdir})")
    else:
        print("ok: fsdb: Verdi FSDB Reader SDK found")


if __name__ == "__main__":
    main()
