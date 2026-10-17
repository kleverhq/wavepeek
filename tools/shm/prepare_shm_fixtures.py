#!/usr/bin/env python3
"""Prepare ignored SHM databases from test VCDs and benchmark FSTs."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
TEST_FIXTURES = (
    "m2_core", "value_vectors", "same_time_updates",
    "change_property_core", "change_property_real_output",
    "change_property_events",
    "extract_apb3", "extract_ahb_lite", "extract_atb", "extract_axistream",
    "extract_axi5_lite", "rich_names",
)
STAMP = ".wavepeek-source.json"
NATIVE_PROBE = "probe -create top -all -dynamic -depth all"


def complete_database(path: Path) -> bool:
    designs = list(path.glob("*.dsn"))
    transitions = list(path.glob("*.trn"))
    return len(designs) == 1 and designs[0].with_suffix(".trn") in transitions and all(
        p.is_file() and p.stat().st_size > 0 for p in designs + transitions
    )


def identity(path: Path) -> list[str | int]:
    stat = path.stat()
    return [str(path.resolve()), stat.st_size, stat.st_mtime_ns]


def publish(staged: Path, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    transaction = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    previous = transaction / "previous"
    published = False
    try:
        replacement = transaction / "database"
        shutil.copytree(staged, replacement)
        if output.exists():
            output.rename(previous)
        try:
            replacement.rename(output)
            published = True
        except BaseException:
            if previous.exists():
                previous.rename(output)
            raise
    finally:
        # Retain the backup if rollback itself fails.
        if published or not previous.exists():
            shutil.rmtree(transaction)


def prepare(source: Path, output: Path, converter: Path) -> None:
    converter = converter.expanduser().resolve()
    stamp = {"source": identity(source), "converter": identity(converter)}
    if source.suffix == ".sv":
        stamp["probe"] = NATIVE_PROBE
    marker = output / STAMP
    if output.exists():
        if not marker.is_file():
            raise RuntimeError(f"refusing to replace an unowned database: {output}")
        if json.loads(marker.read_text()) == stamp and complete_database(output):
            print(f"info: shm fixture: up to date {output}", flush=True)
            return
    scratch = ROOT / "tmp/shm-fixtures"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch) as temporary:
        work = Path(temporary)
        vcd = source
        if source.suffix == ".fst":
            persistent = source.with_suffix(".vcd")
            if persistent.is_file() and persistent.stat().st_mtime_ns >= source.stat().st_mtime_ns:
                vcd = persistent
            else:
                vcd = work / "input.vcd"
                with vcd.open("wb") as stream:
                    subprocess.run(["fst2vcd", "-f", str(source)], stdout=stream, check=True)
        staged = work / output.name
        env = dict(os.environ)
        home = Path(env["XCELIUM_HOME"]).expanduser().resolve()
        env["XCELIUM_HOME"] = str(home)
        env["PATH"] = f"{home}/tools/bin/64bit:{home}/tools/bin:{env.get('PATH', '')}"
        if source.suffix == ".sv":
            script = work / "record.tcl"
            script.write_text(
                f"database -open recording -shm -into {{{staged}}} -default\n"
                f"{NATIVE_PROBE}\nrun\nexit\n"
            )
            command = [str(converter), "-64bit", "-sv", "-access", "+r", "-input", str(script), str(source)]
        else:
            command = [str(converter), "-nocopyright", "-nolog", "-shm", "-output", str(staged), str(vcd)]
        result = subprocess.run(command, cwd=work, env=env, capture_output=True, text=True)
        if result.returncode or not complete_database(staged):
            raise RuntimeError(f"conversion failed for {source}:\n{result.stdout}\n{result.stderr}")
        (staged / STAMP).write_text(json.dumps(stamp) + "\n")
        publish(staged, output)
    print(f"info: shm fixture: converted {source} -> {output}", flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--tests-only", action="store_true")
    mode.add_argument("--bench-only", action="store_true")
    parser.add_argument("--filter", default="", help="benchmark test-name regex")
    args = parser.parse_args()
    home = Path(os.environ["XCELIUM_HOME"]).expanduser().resolve()
    converter = home / "tools/bin/64bit/simvisdbutil"
    if not converter.is_file():
        parser.error(f"missing converter: {converter}")
    if not args.bench_only:
        for name in TEST_FIXTURES:
            sources = [ROOT / "tests/fixtures" / group / f"{name}.vcd" for group in ("generated", "hand")]
            source = next((p for p in sources if p.is_file()), None)
            if source is None:
                parser.error(f"missing test VCD: {name}; run just prepare-waveform-fixtures")
            prepare(source, ROOT / "tests/fixtures/shm" / f"{name}.shm", converter)
        prepare(
            ROOT / "tests/fixtures/source/shm_rich.sv",
            ROOT / "tests/fixtures/shm/shm_rich.shm",
            home / "tools/bin/64bit/xrun",
        )
    if not args.tests_only:
        catalog = json.loads((ROOT / "bench/e2e/tests.json").read_text())
        sources = sorted({
            test["meta"]["waves"] for test in catalog["tests"]
            if re.search(args.filter, test["name"])
        })
        if not sources:
            parser.error(f"no benchmark tests match {args.filter!r}")
        for name in sources:
            source = ROOT / name
            output = source.with_suffix(".shm")
            if source.is_relative_to(ROOT / "tests/fixtures/generated"):
                output = ROOT / "tests/fixtures/shm" / output.name
            prepare(source, output, converter)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"error: shm fixture: {error}") from error
