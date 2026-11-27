#!/usr/bin/env python3
"""Write a format-specific E2E catalog and print its path."""

from __future__ import annotations

import argparse
import json
import pathlib
import tempfile


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
DEFAULT_SOURCE = SCRIPT_DIR / "tests.json"
OUTPUT_ROOT = SCRIPT_DIR.parents[1] / "tmp/bench-catalogs"
FST_SUFFIX = ".fst"
VWDB_PARITY = SCRIPT_DIR / "vwdb-parity.json"
VWDB_RTL_FIXTURES = {
    "fst0000-chipyard-clusteredrocketconfig-dhrystone": "vwdb0047-chipyard-clusteredrocketconfig-dhrystone",
    "fst0002-chipyard-clusteredrocketconfig-mt-memcpy": "vwdb0049-chipyard-clusteredrocketconfig-mt-memcpy",
    "fst0006-chipyard-dualrocketconfig-dhrystone": "vwdb0053-chipyard-dualrocketconfig-dhrystone",
    "fst0012-picorv32-test-ez-vcd": "vwdb0060-picorv32-test-ez",
    "fst0013-picorv32-test-vcd": "vwdb0059-picorv32-test",
    "fst0022-scr1-max-axi-coremark": "vwdb0069-scr1-axi-coremark",
    "fst0025-scr1-max-axi-isr-sample": "vwdb0072-scr1-axi-isr_sample",
    "fst0027-scr1-max-axi-riscv-compliance": "vwdb0074-scr1-axi-riscv_compliance",
}
VWDB_OPB_GROUPS = (
    "all_x64", "background", "constant64", "dense64", "mixed_xz64", "payload",
    "prior", "regular64", "scalar", "sparse64", "staggered64", "wide1024",
)
# These batches select parameters and declarations absent from the VWDB RTL recordings.
# Use distinct recorded signals of the same cardinality from the OPB point workload.
VWDB_POINT_CASES = {
    "value_scr1_signals_10": 7,
    "value_scr1_signals_100": 10,
    "value_scr1_signals_1000": 0,
    "value_chipyard_dualrocketconfig_dhrystone_signals_1000": 1,
    "value_chipyard_clusteredrocketconfig_dhrystone_signals_1000": 2,
}
OPB_S_VWDB = "vwdb/vwdb0046-opb-s/waveform.vwdb"


def fail(message: str) -> None:
    raise SystemExit(message)


def adapt_vwdb_test(test: dict, recording: str) -> dict:
    command = test["command"].copy()
    index = command.index("--waves") + 1
    source = pathlib.PurePosixPath(command[index])
    if source.as_posix() == "tests/fixtures/generated/extract_global_include.fst":
        waves = recording
    elif test["name"] in VWDB_POINT_CASES:
        waves = str(source.parents[2] / OPB_S_VWDB)
        offset = VWDB_POINT_CASES[test["name"]]
        count = test["meta"]["signal_count"]
        command[command.index("--signals") + 1] = ",".join(
            f"tb_opb.{VWDB_OPB_GROUPS[offset + signal // 100]}.word[{signal % 100}].data.q"
            for signal in range(count)
        )
        command[command.index("--at") + 1] = "50ns"
    else:
        fixture = VWDB_RTL_FIXTURES.get(source.parent.name)
        if fixture is None:
            fail(f"error: vwdb catalog: no recording for {source}")
        waves = source.parents[2] / "vwdb" / fixture / "waveform.vwdb"
        waves = str(waves)
        if "-scr1-" in fixture:
            # The simulator records the testbench at the root. The FST also
            # contains a wrapper-level alias for its clock.
            command = [
                "scr1_top_tb_axi" if arg == "TOP" else
                arg.replace("TOP.clk", "scr1_top_tb_axi.clk").replace("TOP.", "")
                for arg in command
            ]
            # SCR1's source clock period is 10ns. The historical FST records
            # that period as 10ps, so one FST tick maps to one VWDB nanosecond.
            for position, arg in enumerate(command[1:], 1):
                if command[position - 1] in ("--from", "--to", "--at"):
                    if not arg.endswith("ps"):
                        fail(f"error: vwdb catalog: expected SCR1 time in ps for {test['name']}: {arg}")
                    command[position] = f"{arg[:-2]}ns"
            if test["name"].startswith("change_scr1_signals_100"):
                signals = command.index("--signals") + 1
                # Replace two undumped memory parameters with recorded inputs.
                for parameter, signal in (("N_IF", "arvalid"), ("SIZE", "araddr")):
                    old = f".i_memory_tb.{parameter}"
                    if old not in command[signals]:
                        fail(f"error: vwdb catalog: expected SCR1 parameter {parameter} for {test['name']}")
                    command[signals] = command[signals].replace(old, f".i_top.i_imem_axi.{signal}")
        if "-chipyard-" in fixture:
            command = [arg.replace("TOP.TestDriver", "TestDriver") for arg in command]
    command[index] = waves
    meta = {key: value for key, value in test["meta"].items() if key != "size"}
    return {**test, "command": command, "meta": {**meta, "waves": waves}}


def vwdb_parity_tests(target: str) -> list[dict]:
    tests = json.loads(VWDB_PARITY.read_text(encoding="utf-8"))["tests"]
    if target == "vwdb":
        for test in tests:
            command = test["command"]
            index = command.index("--waves") + 1
            source = pathlib.PurePosixPath(command[index])
            command[index] = str(source.parents[2] / OPB_S_VWDB)
            test["meta"]["waves"] = command[index]
    return tests


def generate_catalog(source_path: pathlib.Path, target: str, recording: str = "") -> str:
    try:
        source = source_path.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"error: {target} catalog: failed to read {source_path}: {error}")

    try:
        catalog = json.loads(source)
    except json.JSONDecodeError as error:
        fail(f"error: {target} catalog: invalid JSON in {source_path}: {error}")

    if target == "vwdb-fst":
        return json.dumps({"tests": vwdb_parity_tests(target)}, indent=2) + "\n"
    if target == "vwdb":
        if not recording:
            fail("error: vwdb catalog: --vwdb-recording is required")
        tests = [adapt_vwdb_test(test, recording) for test in catalog["tests"]]
        return json.dumps({"tests": tests + vwdb_parity_tests(target)}, indent=2) + "\n"
    if FST_SUFFIX not in source:
        fail(f"error: {target} catalog: no {FST_SUFFIX} suffixes found in {source_path}")

    generated = source.replace(FST_SUFFIX, f".{target}")
    if target in ("fsdb", "shm", "wlf"):
        generated = generated.replace("tests/fixtures/generated/", f"tests/fixtures/{target}/")
    return generated


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", choices=("vcd", "fsdb", "shm", "wlf", "vwdb", "vwdb-fst"), help="target waveform format")
    parser.add_argument("--source", type=pathlib.Path, default=DEFAULT_SOURCE)
    parser.add_argument("--vwdb-recording", help="path to the source-backed AXI recording")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    generated = generate_catalog(args.source, args.target, args.vwdb_recording)
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    # Keep the catalog referenced by each run manifest available and stable.
    output_dir = pathlib.Path(tempfile.mkdtemp(prefix=f"{args.target}-", dir=OUTPUT_ROOT))
    output_path = output_dir / "tests.json"
    output_path.write_text(generated, encoding="utf-8")
    print(output_path)


if __name__ == "__main__":
    main()
