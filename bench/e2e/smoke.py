#!/usr/bin/env python3
"""Check five CLI results on equivalent dumps of tests/fixtures/source/m2_core.v."""

import argparse
from pathlib import Path
import tempfile

from perf import REPO_ROOT, run_functional_capture, write_wavepeek_artifact


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("format", choices=("vcd", "fst", "fsdb", "shm", "wlf"))
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    format_name = args.format
    fixtures = "generated" if format_name in ("vcd", "fst") else format_name
    waves = REPO_ROOT / f"tests/fixtures/{fixtures}/m2_core.{format_name}"
    path = "top.cpu.valid"
    signals = [{"path": path, "value": "1'h1"}]
    cases = (
        ("info", [], [{"time_unit": "1ns", "time_start": "0ns", "time_end": "10ns"}]),
        ("scope", ["--filter", "^top[.]cpu$"], [{"path": "top.cpu", "depth": 1, "kind": "module"}]),
        (
            "signal",
            ["--scope", "top", "--recursive", "--filter", "^top[.]cpu[.]valid$"],
            [{"name": "valid", "path": path, "relative_path": "cpu.valid", "width": 1}],
        ),
        ("value", ["--signals", path, "--at", "5ns"], [{"time": "5ns", "signals": signals}]),
        (
            "change",
            ["--signals", path, "--from", "0ns", "--to", "5ns", "--on", "*", "--sample-mode", "native"],
            [{"time": "5ns", "sample_time": "5ns", "signals": signals}],
        ),
    )

    output_root = REPO_ROOT / "tmp/bench-smoke" / format_name
    output_root.mkdir(parents=True, exist_ok=True)
    run_dir = Path(tempfile.mkdtemp(dir=output_root))
    for command, options, expected in cases:
        test = {
            "name": command,
            "command": ["{wavepeek_bin}", command, "--waves", str(waves), *options],
        }
        payload, failure = run_functional_capture(
            test, str(args.binary.resolve()), f"{format_name} smoke", 15, fail_on_error=False
        )
        if failure:
            raise SystemExit(f"error: {format_name} smoke: {command}: {failure['message']}")
        write_wavepeek_artifact(run_dir, command, payload)
        actual = payload["data"]
        if command == "signal":
            # SHM exposes generic declaration kinds for converted signals.
            actual = [{key: value for key, value in row.items() if key != "kind"} for row in actual]
        if payload["diagnostics"] or actual != expected:
            raise SystemExit(f"error: {format_name} smoke: unexpected {command} result; see {run_dir}")
    print(f"ok: {format_name} smoke: {len(cases)} checks; results in {run_dir}")


if __name__ == "__main__":
    main()
