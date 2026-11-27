#!/usr/bin/env python3
"""Generate source-backed waveform fixtures for integration tests."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
POLICY_PATH = ROOT / "tests" / "fixtures" / "waveform_policy.json"
GENERATED_DIR = ROOT / "tests" / "fixtures" / "generated"
GHW_GENERATED_DIR = GENERATED_DIR / "ghw"
WORK_DIR = ROOT / "tmp" / "waveform-fixtures"


class FixtureError(RuntimeError):
    """Raised when fixture generation cannot proceed."""


@dataclass
class ScopeNode:
    kind: str
    name: str
    variables: list[str] = field(default_factory=list)
    children: list["ScopeNode"] = field(default_factory=list)
    child_indexes: dict[tuple[str, str], int] = field(default_factory=dict)

    def child(self, kind: str, name: str) -> "ScopeNode":
        key = (kind, name)
        index = self.child_indexes.get(key)
        if index is not None:
            return self.children[index]
        node = ScopeNode(kind=kind, name=name)
        self.child_indexes[key] = len(self.children)
        self.children.append(node)
        return node


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ghw", action="store_true", help="prepare only GHW and its reference VCD")
    args = parser.parse_args()

    try:
        tools = ("ghdl",) if args.ghw else ("iverilog", "vvp", "vcd2fst")
        for tool in tools:
            require_tool(tool)
        GENERATED_DIR.mkdir(parents=True, exist_ok=True)
        WORK_DIR.mkdir(parents=True, exist_ok=True)
        # All runs use the same work directories and staged output names.
        with (WORK_DIR / ".lock").open("a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            policy = load_policy()
            reconcile_generated_outputs(policy, ghw=args.ghw)
            for entry in policy.get("source_backed", []):
                is_vhdl = Path(expect_str(entry, "source")).suffix == ".vhd"
                if is_vhdl == args.ghw:
                    generate_source_backed(entry)
            if not args.ghw:
                for entry in policy.get("derived_outputs", []):
                    generate_derived_output(entry)
    except FixtureError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


def require_tool(name: str) -> None:
    if shutil.which(name) is None:
        raise FixtureError(
            f"required tool '{name}' is not on PATH; run inside the devcontainer "
            "or install the waveform fixture toolchain"
        )


def load_policy() -> dict[str, Any]:
    try:
        with POLICY_PATH.open("r", encoding="utf-8") as handle:
            policy = json.load(handle)
    except OSError as error:
        raise FixtureError(f"failed to read {relative(POLICY_PATH)}: {error}") from error
    except json.JSONDecodeError as error:
        raise FixtureError(f"failed to parse {relative(POLICY_PATH)}: {error}") from error
    for key in ["hand_dumps", "source_backed", "derived_outputs"]:
        if not isinstance(policy.get(key), list):
            raise FixtureError(f"{relative(POLICY_PATH)} must contain an array named '{key}'")
    return policy


def reconcile_generated_outputs(policy: dict[str, Any], *, ghw: bool) -> None:
    expected = expected_generated_outputs(policy)
    directory = GHW_GENERATED_DIR if ghw else GENERATED_DIR
    for path in sorted(directory.rglob("*")):
        if not ghw and path.is_relative_to(GHW_GENERATED_DIR):
            continue
        if path.is_file() and path.suffix in {".vcd", ".fst", ".ghw"} and path not in expected:
            path.unlink()
            print(f"removed stale generated fixture {relative(path)}")


def expected_generated_outputs(policy: dict[str, Any]) -> set[Path]:
    expected: set[Path] = set()
    for entry in policy.get("source_backed", []):
        for output in expect_str_list(entry, "outputs"):
            path = root_path(output)
            ensure_generated_output(path)
            expected.add(path)
    for entry in policy.get("derived_outputs", []):
        path = root_path(expect_str(entry, "output"))
        ensure_generated_output(path)
        expected.add(path)
    return expected


def generate_source_backed(entry: dict[str, Any]) -> None:
    name = expect_str(entry, "name")
    source = root_path(expect_str(entry, "source"))
    outputs = expect_str_list(entry, "outputs")
    if not source.is_file():
        raise FixtureError(f"source-backed fixture '{name}' source is missing: {relative(source)}")

    output_paths = [root_path(output) for output in outputs]
    for output in output_paths:
        ensure_generated_output(output)

    fixture_work_dir = work_dir_for_name(name)
    if fixture_work_dir.exists():
        shutil.rmtree(fixture_work_dir)
    fixture_work_dir.mkdir(parents=True)

    if source.suffix == ".v":
        generate_verilog(source, output_paths, fixture_work_dir)
    elif source.suffix == ".vhd":
        generate_vhdl(source, output_paths, fixture_work_dir)
    else:
        raise FixtureError(f"unsupported fixture source: {relative(source)}")

    print(
        "generated "
        + ", ".join(relative(path) for path in output_paths)
        + f" from {relative(source)}"
    )


def generate_verilog(source: Path, output_paths: list[Path], fixture_work_dir: Path) -> None:
    vcd_outputs = [path for path in output_paths if path.suffix == ".vcd"]
    fst_outputs = [path for path in output_paths if path.suffix == ".fst"]
    unsupported = [path for path in output_paths if path.suffix not in {".vcd", ".fst"}]
    if unsupported:
        raise FixtureError(
            f"source-backed fixture '{fixture_work_dir.name}' has unsupported outputs: "
            + ", ".join(relative(path) for path in unsupported)
        )
    if len(vcd_outputs) != 1:
        raise FixtureError(f"source-backed fixture '{fixture_work_dir.name}' must declare exactly one VCD output")

    compiled = fixture_work_dir / f"{fixture_work_dir.name}.vvp"
    run(["iverilog", "-g2012", "-o", str(compiled), str(source)], cwd=ROOT)
    run(["vvp", str(compiled)], cwd=fixture_work_dir)

    raw_vcd = fixture_work_dir / vcd_outputs[0].name
    if not raw_vcd.is_file():
        raise FixtureError(
            f"source-backed fixture '{fixture_work_dir.name}' did not produce expected VCD {raw_vcd.name}"
        )
    replace_text(vcd_outputs[0], canonicalize_vcd(raw_vcd.read_text(encoding="utf-8")))

    for fst_output in fst_outputs:
        tmp_fst = fixture_work_dir / fst_output.name
        run(["vcd2fst", str(vcd_outputs[0]), str(tmp_fst)], cwd=ROOT)
        replace_binary(tmp_fst, fst_output)


def generate_vhdl(source: Path, output_paths: list[Path], fixture_work_dir: Path) -> None:
    dumps = sorted(output_paths, key=lambda path: path.suffix)
    if [path.suffix for path in dumps] != [".ghw", ".vcd"]:
        raise FixtureError(f"VHDL fixture {relative(source)} must declare one GHW and one VCD output")
    ghw, vcd = dumps
    run(["ghdl", "-a", "--std=08", str(source)], cwd=fixture_work_dir)
    run(
        [
            "ghdl", "-r", "--std=08", "--time-resolution=fs", source.stem,
            f"--wave={ghw.name}", f"--vcd={vcd.name}", "--vcd-nodate",
        ],
        cwd=fixture_work_dir,
    )
    for output in dumps:
        if not (fixture_work_dir / output.name).is_file():
            raise FixtureError(f"VHDL fixture {relative(source)} did not produce {output.name}")
    for output in dumps:
        replace_binary(fixture_work_dir / output.name, output)


def generate_derived_output(entry: dict[str, Any]) -> None:
    source = root_path(expect_str(entry, "source"))
    output = root_path(expect_str(entry, "output"))
    if not source.is_file():
        raise FixtureError(f"derived fixture source is missing: {relative(source)}")
    ensure_generated_output(output)
    if source.suffix != ".vcd" or output.suffix != ".fst":
        raise FixtureError(
            f"derived fixture must convert VCD to FST: {relative(source)} -> {relative(output)}"
        )

    fixture_work_dir = work_dir_for_name(output.stem)
    if fixture_work_dir.exists():
        shutil.rmtree(fixture_work_dir)
    fixture_work_dir.mkdir(parents=True)
    tmp_fst = fixture_work_dir / output.name
    run(["vcd2fst", str(source), str(tmp_fst)], cwd=ROOT)
    replace_binary(tmp_fst, output)
    print(f"generated {relative(output)} from {relative(source)}")


def canonicalize_vcd(text: str) -> str:
    lines = text.splitlines(keepends=True)
    result: list[str] = []
    index = 0
    while index < len(lines):
        directive = lines[index].strip()
        if directive in {"$date", "$version"}:
            replacement = (
                ["$date\n", "  generated by tools/waveform/prepare_fixtures.py\n", "$end\n"]
                if directive == "$date"
                else ["$version\n", "  Icarus Verilog\n", "$end\n"]
            )
            result.extend(replacement)
            index += 1
            while index < len(lines) and lines[index].strip() != "$end":
                index += 1
            if index == len(lines):
                raise FixtureError(f"unterminated {directive} block in generated VCD")
            index += 1
            continue
        result.append(lines[index])
        index += 1
    return merge_vcd_scopes(
        "".join(result).replace(
            "$comment Show the parameter values. $end\n$dumpall\n$end\n",
            "",
        )
    )


def merge_vcd_scopes(text: str) -> str:
    lines = text.splitlines(keepends=True)
    first_scope = next(
        (index for index, line in enumerate(lines) if line.strip().startswith("$scope ")),
        None,
    )
    end_definitions = next(
        (index for index, line in enumerate(lines) if line.strip() == "$enddefinitions $end"),
        None,
    )
    if first_scope is None or end_definitions is None or first_scope >= end_definitions:
        return text

    root = ScopeNode(kind="", name="")
    stack = [root]
    for line in lines[first_scope:end_definitions]:
        stripped = line.strip()
        if stripped.startswith("$scope ") and stripped.endswith(" $end"):
            parts = stripped.split()
            if len(parts) != 4:
                raise FixtureError(f"unsupported generated VCD scope directive: {stripped}")
            stack.append(stack[-1].child(kind=parts[1], name=parts[2]))
        elif stripped == "$upscope $end":
            if len(stack) == 1:
                raise FixtureError("generated VCD contains an unmatched $upscope directive")
            stack.pop()
        elif stripped.startswith("$var "):
            stack[-1].variables.append(line)
        elif stripped == "":
            continue
        else:
            raise FixtureError(f"unsupported generated VCD definition directive: {stripped}")
    if len(stack) != 1:
        raise FixtureError("generated VCD contains an unterminated $scope directive")

    merged_definitions: list[str] = []
    for child in root.children:
        emit_scope(child, merged_definitions)
    return "".join(lines[:first_scope] + merged_definitions + lines[end_definitions:])


def emit_scope(node: ScopeNode, output: list[str]) -> None:
    output.append(f"$scope {node.kind} {node.name} $end\n")
    output.extend(node.variables)
    for child in node.children:
        emit_scope(child, output)
    output.append("$upscope $end\n")


def work_dir_for_name(name: str) -> Path:
    if not name or not all(
        char.isascii() and (char.isalnum() or char in "_-") for char in name
    ):
        raise FixtureError(
            f"fixture work directory name must be a simple basename: {name!r}"
        )
    path = (WORK_DIR / name).resolve()
    try:
        path.relative_to(WORK_DIR.resolve())
    except ValueError as error:
        raise FixtureError(f"fixture work directory escapes {relative(WORK_DIR)}: {name}") from error
    return path


def replace_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.tmp")
    tmp.write_text(content, encoding="utf-8")
    os.replace(tmp, path)


def replace_binary(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    tmp = destination.with_name(f".{destination.name}.tmp")
    shutil.copyfile(source, tmp)
    os.replace(tmp, destination)


def run(argv: list[str], cwd: Path) -> None:
    process = subprocess.run(
        argv,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if process.returncode != 0:
        stdout = process.stdout.strip()
        stderr = process.stderr.strip()
        details = "\n".join(part for part in [stdout, stderr] if part)
        raise FixtureError(
            "command failed: "
            + shell_join(argv)
            + (f"\n{details}" if details else "")
        )


def expect_str(entry: dict[str, Any], key: str) -> str:
    value = entry.get(key)
    if not isinstance(value, str) or not value:
        raise FixtureError(f"manifest entry must contain non-empty string '{key}'")
    return value


def expect_str_list(entry: dict[str, Any], key: str) -> list[str]:
    value = entry.get(key)
    if not isinstance(value, list) or not value:
        raise FixtureError(f"manifest entry must contain non-empty string array '{key}'")
    if not all(isinstance(item, str) and item for item in value):
        raise FixtureError(f"manifest entry array '{key}' must contain only non-empty strings")
    return value


def root_path(path: str) -> Path:
    candidate = (ROOT / path).resolve()
    try:
        candidate.relative_to(ROOT)
    except ValueError as error:
        raise FixtureError(f"path escapes repository root: {path}") from error
    return candidate


def ensure_generated_output(path: Path) -> None:
    try:
        path.relative_to(GENERATED_DIR)
    except ValueError as error:
        raise FixtureError(f"generated output must be under {relative(GENERATED_DIR)}: {relative(path)}") from error


def relative(path: Path) -> str:
    return str(path.relative_to(ROOT))


def shell_join(argv: list[str]) -> str:
    return " ".join(sh_quote(arg) for arg in argv)


def sh_quote(value: str) -> str:
    if not value:
        return "''"
    safe = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_./:-")
    if all(char in safe for char in value):
        return value
    return "'" + value.replace("'", "'\\''") + "'"


if __name__ == "__main__":
    raise SystemExit(main())
