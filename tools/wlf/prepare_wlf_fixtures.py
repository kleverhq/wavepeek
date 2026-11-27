#!/usr/bin/env python3
"""Prepare WLF files used by the WavePeek tests and benchmark catalog."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[2]
GENERATED = ROOT / "tests/fixtures/generated"
WLF_FIXTURES = ROOT / "tests/fixtures/wlf"
TEST_INPUTS = ("m2_core", "value_vectors", "change_property_core", "extract_global_include")
# FST export can include SystemVerilog declaration kinds that the VCD importer
# rejects. Use standard VCD kinds with the same value encoding and width.
VCD_DECLARATIONS = (
    r"s/^(\$var )(logic|bit) /\1reg /; "
    r"s/^(\$var )(int|longint) /\1integer /; "
    r"s/^(\$var )real_parameter /\1real /"
)


def identity(path: Path) -> list[str | int]:
    stat = path.stat()
    return [str(path.resolve()), stat.st_size, stat.st_mtime_ns]


def input_identity(path: Path) -> list[str | int]:
    if path.is_relative_to(GENERATED):
        # Fixture generation can replace a file with the same content.
        with path.open("rb") as stream:
            return [str(path), hashlib.file_digest(stream, "sha256").hexdigest()]
    return identity(path)


def fixture_paths(source: Path) -> tuple[Path, Path]:
    output = source.with_suffix(".wlf")
    if source.is_relative_to(GENERATED):
        return source.with_suffix(".vcd"), WLF_FIXTURES / output.name
    return source, output


def prepare(source: Path, output: Path, converter: Path) -> None:
    marker = output.with_suffix(".wlf.source.json")
    stamp = json.dumps({
        "input": input_identity(source),
        "converter": identity(converter),
        "vcd_declarations": VCD_DECLARATIONS if source.suffix == ".fst" else None,
    })
    if output.is_file() and output.stat().st_size and marker.is_file() and marker.read_text() == stamp:
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".wlf-", dir=output.parent) as temporary:
        staged = Path(temporary) / output.name
        if source.suffix == ".vcd":
            command = [str(converter), str(source), str(staged)]
        else:
            command = [
                "bash", "-o", "pipefail", "-c",
                'fst2vcd -f "$1" | sed -E "$2" | "$3" - "$4"', "prepare-wlf",
                str(source), VCD_DECLARATIONS, str(converter), str(staged),
            ]
        subprocess.run(command, check=True)
        if not staged.is_file() or not staged.stat().st_size:
            raise RuntimeError(f"converter produced no waveform for {source}")
        staged_marker = staged.parent / marker.name
        staged_marker.write_text(stamp)
        staged.replace(output)
        staged_marker.replace(marker)
    print(f"ok: wlf fixture: {output}", flush=True)


def benchmark_sources(pattern: str) -> list[Path]:
    catalog = json.loads((ROOT / "bench/e2e/tests.json").read_text())
    names = {
        entry["meta"]["waves"]
        for entry in catalog["tests"]
        if re.search(pattern, entry["name"])
    }
    if not names:
        raise RuntimeError(f"no benchmark tests match {pattern!r}")
    return [ROOT / name for name in sorted(names)]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--tests-only", action="store_true")
    mode.add_argument("--bench-only", action="store_true")
    parser.add_argument("--filter", default="")
    args = parser.parse_args()
    converter = Path(os.environ["QUESTA_HOME"]).expanduser() / "bin/vcd2wlf"
    sources = (
        [GENERATED / f"{name}.vcd" for name in TEST_INPUTS]
        if args.tests_only else benchmark_sources(args.filter)
    )
    for source in sources:
        input_path, output_path = fixture_paths(source)
        prepare(input_path, output_path, converter)


if __name__ == "__main__":
    try:
        main()
    except (KeyError, OSError, RuntimeError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"error: wlf fixture: {error}") from error
