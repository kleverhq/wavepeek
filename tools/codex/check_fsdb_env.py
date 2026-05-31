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
    explicit_libdir = env_path("WAVEPEEK_FSDB_READER_LIBDIR")
    if explicit_libdir is not None:
        return explicit_libdir

    abi = os.environ.get("WAVEPEEK_FSDB_ABI") or "linux64"
    return reader_root(verdi_home) / abi


def missing_libraries(libdir: pathlib.Path) -> list[pathlib.Path]:
    return [libdir / name for name in REQUIRED_LIBRARIES if not (libdir / name).is_file()]


def configured_home_candidates() -> list[pathlib.Path]:
    verdi_home = env_path("VERDI_HOME")
    if verdi_home is None:
        return []
    return [verdi_home]


def has_explicit_reader_override() -> bool:
    return env_path("WAVEPEEK_FSDB_READER_LIBDIR") is not None or bool(
        os.environ.get("WAVEPEEK_FSDB_ABI")
    )


def verbose_output_enabled() -> bool:
    return os.environ.get("WAVEPEEK_FSDB_ENV_VERBOSE") == "1"


def unavailable(required: bool) -> None:
    if required:
        fail("Verdi FSDB Reader SDK not found; set VERDI_HOME to run this target")
    skip()


def validate_sdk(required: bool) -> tuple[pathlib.Path, pathlib.Path]:
    candidates = configured_home_candidates()
    explicit_override = has_explicit_reader_override()

    if not candidates:
        if explicit_override:
            fail("VERDI_HOME is required when FSDB Reader library overrides are set")
        unavailable(required)

    for verdi_home in candidates:
        header_misses = missing_headers(verdi_home)
        if header_misses:
            if explicit_override:
                missing = header_misses[0]
                missing_text = str(missing) if verbose_output_enabled() else missing.name
                fail(
                    "VERDI_HOME does not contain a usable FSDB Reader header root; "
                    f"missing {missing_text}"
                )
            continue

        libdir = selected_libdir(verdi_home)
        library_misses = missing_libraries(libdir)
        if library_misses:
            if (
                env_path("WAVEPEEK_FSDB_READER_LIBDIR") is not None
                or libdir.exists()
                or explicit_override
            ):
                missing = library_misses[0]
                missing_text = str(missing) if verbose_output_enabled() else missing.name
                fail(
                    "selected FSDB Reader library directory is incomplete; "
                    f"missing {missing_text}; set WAVEPEEK_FSDB_READER_LIBDIR or try WAVEPEEK_FSDB_ABI=linux64_gcc950"
                )
            continue

        return verdi_home, libdir

    if explicit_override:
        fail("explicit FSDB Reader library configuration did not resolve to a usable SDK")

    # A devcontainer may set VERDI_HOME to an empty host mount. That is ordinary
    # no-Verdi availability for optional discovery, but a hard error for targets
    # that explicitly require Verdi. Yes, this distinction is annoying. So are
    # proprietary SDKs wired through environment variables.
    unavailable(required)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="check local Verdi FSDB Reader SDK availability")
    parser.add_argument(
        "--require",
        action="store_true",
        help="treat missing Verdi as an error instead of an optional skip",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    verdi_home, libdir = validate_sdk(required=args.require)
    verbose = verbose_output_enabled()
    if verbose:
        print(f"ok: fsdb: Verdi FSDB Reader SDK found at {verdi_home} (libdir {libdir})")
    else:
        print("ok: fsdb: Verdi FSDB Reader SDK found")


if __name__ == "__main__":
    main()
