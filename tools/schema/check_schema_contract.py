#!/usr/bin/env python3
"""Validate current wavepeek schema contract artifacts."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

OUTPUT_FAMILY = "wavepeek.output"
STREAM_FAMILY = "wavepeek.stream-record"
INPUT_FAMILY = "wavepeek.input"
EXPECTED_OUTPUT_PATH = "schema/output.json"
EXPECTED_STREAM_PATH = "schema/stream.json"
EXPECTED_INPUT_PATH = "schema/input.json"
EXPECTED_OUTPUT_URL = (
    "https://kleverhq.github.io/wavepeek/schema-output-v2.2.json"
)
EXPECTED_STREAM_URL = (
    "https://kleverhq.github.io/wavepeek/schema-stream-v2.2.json"
)
EXPECTED_INPUT_URL = (
    "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
)
ROOT = Path(__file__).resolve().parents[2]


class ContractError(RuntimeError):
    pass


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "legacy_schema_path",
        nargs="?",
        help="deprecated positional argument kept only to print a useful error",
    )
    parser.add_argument(
        "--schema-dir",
        default="schema",
        help="directory containing committed schema snapshots",
    )
    parser.add_argument(
        "--generated-dir",
        default="tmp/schema-check",
        help="directory containing freshly generated schema snapshots",
    )
    args = parser.parse_args(argv)
    if args.legacy_schema_path:
        raise ContractError(
            "check_schema_contract.py now expects --schema-dir and --generated-dir; "
            "run `just check-schema`"
        )
    return args


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise ContractError(f"missing required schema artifact: {path}") from error
    except json.JSONDecodeError as error:
        raise ContractError(f"invalid JSON in {path}: {error}") from error


def load_catalog(schema_dir: Path) -> dict[str, Any]:
    catalog = load_json(schema_dir / "catalog.json")
    families = catalog.get("families")
    if not isinstance(families, list):
        raise ContractError("schema/catalog.json must contain a families array")
    by_family = {entry.get("id"): entry for entry in families if isinstance(entry, dict)}
    if set(by_family) != {OUTPUT_FAMILY, STREAM_FAMILY, INPUT_FAMILY}:
        raise ContractError(
            "schema/catalog.json must contain exactly wavepeek.output, "
            "wavepeek.stream-record, and wavepeek.input entries"
        )
    return catalog


def validate_catalog_entry(
    entry: dict[str, Any],
    *,
    family: str,
    version: str,
    path: str,
    url: str,
) -> None:
    expected = {
        "id": family,
        "version": version,
        "path": path,
        "url": url,
    }
    for key, value in expected.items():
        if entry.get(key) != value:
            raise ContractError(
                f"catalog entry {family} has {key}={entry.get(key)!r}; expected {value!r}"
            )


def validate_catalog(catalog: dict[str, Any]) -> dict[str, dict[str, Any]]:
    by_family = {entry["id"]: entry for entry in catalog["families"]}
    validate_catalog_entry(
        by_family[OUTPUT_FAMILY],
        family=OUTPUT_FAMILY,
        version="2.2",
        path=EXPECTED_OUTPUT_PATH,
        url=EXPECTED_OUTPUT_URL,
    )
    validate_catalog_entry(
        by_family[STREAM_FAMILY],
        family=STREAM_FAMILY,
        version="2.2",
        path=EXPECTED_STREAM_PATH,
        url=EXPECTED_STREAM_URL,
    )
    validate_catalog_entry(
        by_family[INPUT_FAMILY],
        family=INPUT_FAMILY,
        version="2.2",
        path=EXPECTED_INPUT_PATH,
        url=EXPECTED_INPUT_URL,
    )
    return by_family


def compare_generated(schema_dir: Path, generated_dir: Path) -> None:
    for name in ("output.json", "stream.json", "input.json", "catalog.json"):
        committed = schema_dir / name
        generated = generated_dir / name
        if committed.read_bytes() != generated.read_bytes():
            raise ContractError(
                f"{committed} is stale; run `just update-schema` and commit the result"
            )


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def validate_output_schema(schema: dict[str, Any]) -> None:
    require(schema.get("$id") == EXPECTED_OUTPUT_URL, "output schema $id must be exact URL")
    require(
        schema["properties"]["$schema"].get("const") == EXPECTED_OUTPUT_URL,
        "output schema must require exact $schema URL with const",
    )
    require(schema.get("additionalProperties") is True, "output envelope must allow extensions")
    commands = schema["properties"]["command"]["enum"]
    require(
        commands
        == [
            "info",
            "scope",
            "signal",
            "value",
            "change",
            "property",
            "extract apb",
            "extract axi",
            "extract generic",
            "docs topics",
            "docs search",
        ],
        "output command enum is not the expected stable list",
    )
    require(
        schema["$defs"]["scopeKind"]["enum"],
        "output schema must define scopeKind enum",
    )
    require(
        schema["$defs"]["signalKind"]["enum"],
        "output schema must define signalKind enum",
    )
    require(
        schema["$defs"]["diagnostic"]["additionalProperties"] is True,
        "diagnostic objects must allow extensions",
    )


def validate_stream_schema(schema: dict[str, Any]) -> None:
    require(schema.get("$id") == EXPECTED_STREAM_URL, "stream schema $id must be exact URL")
    require(
        schema["$defs"]["beginRecord"]["properties"]["$schema"].get("const")
        == EXPECTED_STREAM_URL,
        "stream begin record must require exact $schema URL with const",
    )
    require(
        schema["$defs"]["streamCommand"]["enum"]
        == [
            "info",
            "scope",
            "signal",
            "value",
            "change",
            "property",
            "extract apb",
            "extract axi",
            "extract generic",
        ],
        "stream command enum is not the expected stable list",
    )


def run_cargo(args: list[str]) -> bytes:
    process = subprocess.run(
        ["cargo", "run", "--quiet", "--", *args],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if process.returncode != 0:
        raise ContractError(
            f"cargo run -- {' '.join(args)} failed with exit {process.returncode}: "
            f"{process.stderr.decode('utf-8', errors='replace')}"
        )
    if process.stderr:
        raise ContractError(
            f"cargo run -- {' '.join(args)} wrote stderr: "
            f"{process.stderr.decode('utf-8', errors='replace')}"
        )
    return process.stdout


def validate_input_schema(schema: dict[str, Any]) -> None:
    require(schema.get("$id") == EXPECTED_INPUT_URL, "input schema $id must be exact URL")
    require(
        schema.get("oneOf")
        == [
            {"$ref": "#/$defs/extractGenericSourcesInput"},
            {"$ref": "#/$defs/extractApbSourceInput"},
            {"$ref": "#/$defs/extractAxiSourceInput"},
        ],
        "input schema root must accept generic, APB, and AXI source documents",
    )
    generic_def = schema["$defs"]["extractGenericSourcesInput"]
    require(
        generic_def["properties"]["$schema"].get("const") == EXPECTED_INPUT_URL,
        "generic input source must require exact $schema URL with const",
    )
    require(
        generic_def["properties"]["kind"].get("const") == "extract.generic.sources",
        "input schema must require exact extract generic kind",
    )
    require(
        generic_def["properties"]["sources"].get("minItems") == 1,
        "input sources must require at least one source",
    )
    source_def = schema["$defs"]["extractGenericSource"]
    require(
        source_def["properties"]["payload"].get("minItems") == 1,
        "input payload must require at least one signal",
    )
    apb_def = schema["$defs"]["extractApbSourceInput"]
    require(
        apb_def["properties"]["$schema"].get("const") == EXPECTED_INPUT_URL,
        "APB input source must require exact $schema URL with const",
    )
    require(
        apb_def["properties"]["kind"].get("const") == "extract.apb.source",
        "input schema must require exact extract APB kind",
    )
    require(
        schema["$defs"]["apbProfile"].get("enum") == ["apb3", "apb4", "apb5"],
        "APB input profile enum is not the expected stable list",
    )
    require(
        apb_def["properties"]["profile"] == {"$ref": "#/$defs/apbProfile"},
        "APB input profile must reuse the shared profile definition",
    )
    require(
        apb_def["properties"]["pready_mode"]
        == {"$ref": "#/$defs/apbPreadyMode"},
        "APB input PREADY mode must reuse the shared mode definition",
    )
    require(
        "allOf" in apb_def,
        "APB input source must include profile-aware constraints",
    )
    axi_def = schema["$defs"]["extractAxiSourceInput"]
    require(
        axi_def["properties"]["$schema"].get("const") == EXPECTED_INPUT_URL,
        "AXI input source must require exact $schema URL with const",
    )
    require(
        axi_def["properties"]["kind"].get("const") == "extract.axi.source",
        "input schema must require exact extract AXI kind",
    )
    require(
        schema["$defs"]["axiProfile"].get("enum")
        == [
            "axi3",
            "axi4",
            "axi4-lite",
            "axi5",
            "axi5-lite",
            "ace",
            "ace-lite",
            "ace5",
            "ace5-lite",
            "ace5-lite-dvm",
            "ace5-lite-acp",
        ],
        "AXI input profile enum is not the expected stable list",
    )
    require(
        axi_def["properties"]["profile"] == {"$ref": "#/$defs/axiProfile"},
        "AXI input profile must reuse the shared profile definition",
    )
    require(
        "allOf" in axi_def,
        "AXI input source must include profile-aware constraints",
    )


def validate_schema_semantics(schema_dir: Path) -> None:
    process = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            "tools/schema-gen/Cargo.toml",
            "--",
            "--validate",
            str(schema_dir),
        ],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if process.returncode != 0:
        raise ContractError(
            "schema semantic validation failed: "
            f"{process.stderr.decode('utf-8', errors='replace')}"
        )


def validate_runtime(schema_dir: Path) -> None:
    output_bytes = (schema_dir / "output.json").read_bytes()
    stream_bytes = (schema_dir / "stream.json").read_bytes()
    input_bytes = (schema_dir / "input.json").read_bytes()
    if run_cargo(["schema"]) != output_bytes:
        raise ContractError("wavepeek schema output differs from schema/output.json")
    if run_cargo(["schema", "--stream"]) != stream_bytes:
        raise ContractError("wavepeek schema --stream output differs from schema/stream.json")
    if run_cargo(["schema", "--input"]) != input_bytes:
        raise ContractError("wavepeek schema --input output differs from schema/input.json")

    info_stdout = run_cargo(
        ["info", "--waves", "tests/fixtures/generated/m2_core.vcd", "--json"]
    )
    info = json.loads(info_stdout)
    require(info["$schema"] == EXPECTED_OUTPUT_URL, "runtime JSON envelope uses wrong schema URL")

    jsonl_stdout = run_cargo(
        ["info", "--waves", "tests/fixtures/generated/m2_core.vcd", "--jsonl"]
    )
    first = json.loads(jsonl_stdout.splitlines()[0])
    require(
        first["$schema"] == EXPECTED_STREAM_URL,
        "runtime JSONL begin record uses wrong schema URL",
    )
    for line in jsonl_stdout.splitlines():
        record = json.loads(line)
        require(record["command"] == "info", "runtime JSONL record uses wrong command")
        require(record["type"] in {"begin", "item", "diagnostic", "end"}, "runtime JSONL record has invalid type")


def validate_no_obsolete_current_alias(schema_dir: Path) -> None:
    if (schema_dir / "wavepeek.json").exists():
        raise ContractError("schema/wavepeek.json is obsolete; remove it")


def validate_artifact_names(schema_dir: Path) -> None:
    historical_output = re.compile(r"^wavepeek_v(?:\d+|\d+\.\d+)\.json$")
    historical_stream = re.compile(r"^wavepeek-stream-v(?:\d+|\d+\.\d+)\.json$")
    allowed = {"output.json", "stream.json", "input.json", "catalog.json"}
    for path in schema_dir.glob("*.json"):
        if path.name in allowed:
            continue
        if historical_output.match(path.name) or historical_stream.match(path.name):
            continue
        raise ContractError(f"unexpected schema artifact name: {path}")


def main(argv: list[str]) -> int:
    try:
        args = parse_args(argv)
        schema_dir = (ROOT / args.schema_dir).resolve()
        generated_dir = (ROOT / args.generated_dir).resolve()
        validate_no_obsolete_current_alias(schema_dir)
        validate_artifact_names(schema_dir)
        catalog = load_catalog(schema_dir)
        validate_catalog(catalog)
        compare_generated(schema_dir, generated_dir)
        output_schema = load_json(schema_dir / "output.json")
        stream_schema = load_json(schema_dir / "stream.json")
        input_schema = load_json(schema_dir / "input.json")
        validate_output_schema(output_schema)
        validate_stream_schema(stream_schema)
        validate_input_schema(input_schema)
        validate_schema_semantics(schema_dir)
        validate_runtime(schema_dir)
    except ContractError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print("schema contract OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
