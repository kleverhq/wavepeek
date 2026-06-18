#!/usr/bin/env python3

from __future__ import annotations

import json
import pathlib
import re
import subprocess
import sys
import tomllib


SCHEMA_PAGES_BASE = "https://kleverhq.github.io/wavepeek"
RAW_REPOSITORY = "https://raw.githubusercontent.com/kleverhq/wavepeek"


def fail(message: str, *, hint_update_schema: bool = False) -> None:
    print(message, file=sys.stderr)
    if hint_update_schema:
        print("hint: run just update-schema", file=sys.stderr)
    raise SystemExit(1)


def package_version() -> str:
    cargo_toml = tomllib.loads(pathlib.Path("Cargo.toml").read_text(encoding="utf-8"))
    return cargo_toml["package"]["version"]


def package_major(version: str) -> str:
    return version.split(".", maxsplit=1)[0]


def current_schema_path(major: str) -> pathlib.Path:
    return pathlib.Path("schema") / f"wavepeek_v{major}.json"


def expected_schema_url(major: str) -> str:
    return f"{SCHEMA_PAGES_BASE}/wavepeek_v{major}.json"


def expected_schema_url_pattern(major: str) -> str:
    return rf"^{re.escape(SCHEMA_PAGES_BASE)}/wavepeek_v{re.escape(major)}\.json$"


def validate_schema_path(schema_path: pathlib.Path, major: str) -> None:
    expected_path = current_schema_path(major)
    if schema_path != expected_path and schema_path.resolve() != expected_path.resolve():
        fail(
            "error: schema: canonical schema path mismatch: "
            f"expected {expected_path}, got {schema_path}"
        )

    obsolete_path = pathlib.Path("schema/wavepeek.json")
    if obsolete_path.exists():
        fail(f"error: schema: obsolete unversioned schema artifact exists at {obsolete_path}")


def load_schema(schema_path: pathlib.Path) -> tuple[bytes, dict[str, object]]:
    if not schema_path.exists():
        fail(
            f"error: schema: missing canonical schema artifact at {schema_path}",
            hint_update_schema=True,
        )

    schema_bytes = schema_path.read_bytes()
    try:
        schema = json.loads(schema_bytes.decode("utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: schema: canonical schema is not valid JSON: {error}")

    if not schema_bytes.endswith(b"\n"):
        fail(
            "error: schema: canonical schema must end with trailing newline",
            hint_update_schema=True,
        )

    if not isinstance(schema, dict):
        fail("error: schema: canonical schema root must be a JSON object")

    return schema_bytes, schema


def schema_url_pattern(schema: dict[str, object]) -> str:
    try:
        pattern = schema["properties"]["$schema"]["pattern"]  # type: ignore[index]
    except (KeyError, TypeError):
        fail("error: schema: canonical schema is missing properties.$schema.pattern")

    if not isinstance(pattern, str):
        fail("error: schema: canonical schema properties.$schema.pattern must be a string")

    return pattern


def validate_artifact_schema_url_pattern(schema: dict[str, object], version: str, major: str) -> None:
    pattern = schema_url_pattern(schema)
    expected_pattern = expected_schema_url_pattern(major)
    if pattern != expected_pattern:
        fail(
            "error: schema: canonical schema properties.$schema.pattern mismatch: "
            f"expected {expected_pattern}, got {pattern}"
        )

    try:
        artifact_url_pattern = re.compile(pattern)
    except re.error as error:
        fail(f"error: schema: canonical schema properties.$schema.pattern is invalid: {error}")

    expected_url = expected_schema_url(major)
    if artifact_url_pattern.fullmatch(expected_url) is None:
        fail(
            "error: schema: canonical schema properties.$schema.pattern does not accept "
            f"expected URL {expected_url}"
        )

    old_url = f"{RAW_REPOSITORY}/v{version}/schema/wavepeek.json"
    if artifact_url_pattern.fullmatch(old_url) is not None:
        fail(
            "error: schema: canonical schema properties.$schema.pattern still accepts "
            f"obsolete URL {old_url}"
        )


def validate_runtime_schema(schema_path: pathlib.Path, schema_bytes: bytes) -> None:
    runtime_schema = subprocess.run(
        ["cargo", "run", "--quiet", "--", "schema"],
        check=True,
        stdout=subprocess.PIPE,
        text=False,
    ).stdout
    if runtime_schema != schema_bytes:
        fail(
            "error: schema: canonical schema mismatch between "
            f"{schema_path} and 'wavepeek schema' output",
            hint_update_schema=True,
        )


def validate_diagnostic_schema(schema: dict[str, object]) -> None:
    required = schema.get("required")
    properties = schema.get("properties")
    defs = schema.get("$defs")
    if not isinstance(required, list):
        fail("error: schema: canonical schema required must be an array")
    if not isinstance(properties, dict):
        fail("error: schema: canonical schema properties must be an object")
    if not isinstance(defs, dict):
        fail("error: schema: canonical schema $defs must be an object")

    if "diagnostics" not in required:
        fail("error: schema: canonical schema must require diagnostics")
    if "warnings" in required:
        fail("error: schema: canonical schema must not require legacy warnings")
    if "diagnostics" not in properties:
        fail("error: schema: canonical schema must define diagnostics")
    if "warnings" in properties:
        fail("error: schema: canonical schema must not define legacy warnings")

    data = properties.get("data")
    if not isinstance(data, dict):
        fail("error: schema: data property must be an object")
    if "oneOf" in data:
        fail("error: schema: data property must not use oneOf because empty arrays match multiple command payloads")
    if "anyOf" not in data:
        fail("error: schema: data property must use anyOf for command payload variants")

    diagnostics = properties["diagnostics"]
    if not isinstance(diagnostics, dict):
        fail("error: schema: diagnostics property must be an object")
    if diagnostics.get("type") != "array":
        fail("error: schema: diagnostics property must be an array")
    items = diagnostics.get("items")
    if not isinstance(items, dict) or items.get("$ref") != "#/$defs/diagnostic":
        fail("error: schema: diagnostics items must reference $defs.diagnostic")

    diagnostic = defs.get("diagnostic")
    if not isinstance(diagnostic, dict):
        fail("error: schema: canonical schema must define $defs.diagnostic")
    if diagnostic.get("type") != "object":
        fail("error: schema: diagnostic definition must be an object")
    if diagnostic.get("additionalProperties") is not False:
        fail("error: schema: diagnostic definition must reject additional properties")
    if diagnostic.get("required") != ["kind", "message"]:
        fail("error: schema: diagnostic definition must require kind and message")

    diagnostic_properties = diagnostic.get("properties")
    if not isinstance(diagnostic_properties, dict):
        fail("error: schema: diagnostic properties must be an object")
    if set(diagnostic_properties) != {"kind", "code", "message"}:
        fail("error: schema: diagnostic properties must be exactly kind, code, and message")
    kind = diagnostic_properties.get("kind")
    code = diagnostic_properties.get("code")
    message = diagnostic_properties.get("message")
    if not isinstance(kind, dict) or kind.get("enum") != ["info", "warning", "error"]:
        fail("error: schema: diagnostic kind enum mismatch")
    if not isinstance(code, dict) or code.get("pattern") != r"^WPK-[WE][0-9]{4}$":
        fail("error: schema: diagnostic code pattern mismatch")
    if not isinstance(message, dict) or message.get("type") != "string":
        fail("error: schema: diagnostic message must be a string")

    rules = diagnostic.get("allOf")
    if not isinstance(rules, list):
        fail("error: schema: diagnostic definition must use allOf conditionals")
    found_warning = False
    found_error = False
    found_info = False
    for rule in rules:
        if not isinstance(rule, dict):
            continue
        try:
            kind_const = rule["if"]["properties"]["kind"]["const"]  # type: ignore[index]
        except (KeyError, TypeError):
            continue
        then = rule.get("then")
        if not isinstance(then, dict):
            continue
        code_properties = then.get("properties")
        if not isinstance(code_properties, dict):
            code_pattern = None
        else:
            code_schema = code_properties.get("code")
            code_pattern = code_schema.get("pattern") if isinstance(code_schema, dict) else None
        if (
            kind_const == "warning"
            and then.get("required") == ["code"]
            and code_pattern == r"^WPK-W[0-9]{4}$"
        ):
            found_warning = True
        if (
            kind_const == "error"
            and then.get("required") == ["code"]
            and code_pattern == r"^WPK-E[0-9]{4}$"
        ):
            found_error = True
        if kind_const == "info" and then.get("not") == {"required": ["code"]}:
            found_info = True
    if not found_warning:
        fail("error: schema: warning diagnostics must require a WPK-W code")
    if not found_error:
        fail("error: schema: error diagnostics must require a WPK-E code")
    if not found_info:
        fail("error: schema: info diagnostics must reject code")


def validate_docs_metadata_schema(schema: dict[str, object]) -> None:
    try:
        topic_summary = schema["$defs"]["topicSummary"]  # type: ignore[index]
        topic_required = topic_summary["required"]  # type: ignore[index]
        topic_properties = topic_summary["properties"]  # type: ignore[index]
        match_kind = schema["$defs"]["docsSearchMatch"]["properties"]["match_kind"]  # type: ignore[index]
        match_kind_enum = match_kind["enum"]  # type: ignore[index]
    except (KeyError, TypeError):
        fail("error: schema: canonical schema is missing docs metadata definitions")

    if not isinstance(topic_required, list):
        fail("error: schema: topicSummary.required must be an array")
    if not isinstance(topic_properties, dict):
        fail("error: schema: topicSummary.properties must be an object")
    if not isinstance(match_kind_enum, list):
        fail("error: schema: docsSearchMatch.match_kind.enum must be an array")

    if "description" not in topic_required:
        fail("error: schema: topicSummary must require description")
    if "summary" in topic_required:
        fail("error: schema: topicSummary must not require legacy summary")
    if "description" not in topic_properties:
        fail("error: schema: topicSummary must define description")
    if "summary" in topic_properties:
        fail("error: schema: topicSummary must not define current summary property")
    if "title_or_description" not in match_kind_enum:
        fail("error: schema: docs search match kind enum must include title_or_description")
    if "title_or_summary" in match_kind_enum:
        fail("error: schema: docs search match kind enum must not include title_or_summary")


def validate_runtime_envelope_url(version: str, major: str) -> None:
    expected_url = expected_schema_url(major)
    runtime_url_pattern = re.compile(expected_schema_url_pattern(major))

    info_json_stdout = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--",
            "info",
            "--waves",
            "tests/fixtures/hand/m2_core.vcd",
            "--json",
        ],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout
    envelope = json.loads(info_json_stdout)
    actual_schema_url = envelope.get("$schema")

    if actual_schema_url != expected_url:
        fail(
            "error: schema: envelope $schema URL mismatch: "
            f"expected {expected_url}, got {actual_schema_url}"
        )

    if actual_schema_url is None or runtime_url_pattern.fullmatch(actual_schema_url) is None:
        fail(
            "error: schema: envelope $schema URL does not match required pattern: "
            f"{actual_schema_url}"
        )

    obsolete_url = f"{RAW_REPOSITORY}/v{version}/schema/wavepeek.json"
    if actual_schema_url == obsolete_url:
        fail("error: schema: envelope $schema URL still uses obsolete full-semver path")

    if "schema_version" in envelope:
        fail("error: schema: legacy schema_version key is still present in JSON envelope")

    if envelope.get("diagnostics") != []:
        fail("error: schema: info JSON envelope must contain empty diagnostics")
    if "warnings" in envelope:
        fail("error: schema: legacy warnings key is still present in JSON envelope")


def main() -> None:
    version = package_version()
    major = package_major(version)
    schema_path = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else current_schema_path(major)

    validate_schema_path(schema_path, major)
    schema_bytes, schema = load_schema(schema_path)
    validate_artifact_schema_url_pattern(schema, version, major)
    validate_diagnostic_schema(schema)
    validate_docs_metadata_schema(schema)
    validate_runtime_schema(schema_path, schema_bytes)
    validate_runtime_envelope_url(version, major)


if __name__ == "__main__":
    main()
