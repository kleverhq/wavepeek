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


def main() -> None:
    version = package_version()
    major = package_major(version)
    schema_path = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else current_schema_path(major)

    validate_schema_path(schema_path, major)
    schema_bytes, schema = load_schema(schema_path)
    validate_artifact_schema_url_pattern(schema, version, major)
    validate_docs_metadata_schema(schema)
    validate_runtime_schema(schema_path, schema_bytes)
    validate_runtime_envelope_url(version, major)


if __name__ == "__main__":
    main()
