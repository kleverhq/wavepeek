#!/usr/bin/env python3

from __future__ import annotations

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
    if verdi_home is not None:
        return [verdi_home]
    return [pathlib.Path("/opt/verdi")]


def has_explicit_reader_override() -> bool:
    return env_path("WAVEPEEK_FSDB_READER_LIBDIR") is not None or bool(os.environ.get("WAVEPEEK_FSDB_ABI"))


def find_optional_artifact_dir() -> pathlib.Path | None:
    candidates: list[pathlib.Path] = []
    for name in ("WAVEPEEK_FSDB_ARTIFACTS_DIR", "FSDB_RTL_ARTIFACTS_DIR"):
        value = env_path(name)
        if value is not None:
            candidates.append(value)
    candidates.extend(
        [
            pathlib.Path("/opt/fsdb-rtl-artifacts"),
            pathlib.Path.home() / ".cache" / "wavepeek" / "fsdb-rtl-artifacts",
        ]
    )
    for candidate in candidates:
        if candidate.is_dir():
            return candidate
    return None


def validate_sdk() -> tuple[pathlib.Path, pathlib.Path]:
    candidates = configured_home_candidates()
    explicit_override = has_explicit_reader_override()
    saw_existing_home = False

    for verdi_home in candidates:
        if verdi_home.exists():
            saw_existing_home = True

        header_misses = missing_headers(verdi_home)
        if header_misses:
            if explicit_override:
                fail(
                    "VERDI_HOME does not contain a usable FSDB Reader header root; "
                    f"missing {header_misses[0]}"
                )
            continue

        libdir = selected_libdir(verdi_home)
        library_misses = missing_libraries(libdir)
        if library_misses:
            if env_path("WAVEPEEK_FSDB_READER_LIBDIR") is not None or libdir.exists() or explicit_override:
                fail(
                    "selected FSDB Reader library directory is incomplete; "
                    f"missing {library_misses[0]}; set WAVEPEEK_FSDB_READER_LIBDIR or try WAVEPEEK_FSDB_ABI=linux64_gcc950"
                )
            continue

        return verdi_home, libdir

    if explicit_override:
        fail("explicit FSDB Reader library configuration did not resolve to a usable SDK")

    # An empty /opt/verdi mount, or an intentionally empty temporary VERDI_HOME in
    # tests, is ordinary no-Verdi availability rather than a hard configuration
    # error. The variable is often set by the devcontainer whether a host mount is
    # populated or not. Tiny distinction, large reduction in false alarms.
    _ = saw_existing_home
    skip()


def main() -> None:
    verdi_home, libdir = validate_sdk()
    print(f"ok: fsdb: Verdi FSDB Reader SDK found at {verdi_home} (libdir {libdir})")
    artifact_dir = find_optional_artifact_dir()
    if artifact_dir is not None:
        print(f"info: fsdb: optional artifact directory found at {artifact_dir}")
    else:
        print("info: fsdb: optional artifact directory not found; metadata smoke can still run without WAVEPEEK_FSDB_SMOKE_FILE")


if __name__ == "__main__":
    main()
