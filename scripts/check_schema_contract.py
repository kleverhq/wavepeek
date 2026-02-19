#!/usr/bin/env python3

from __future__ import annotations

import json
import pathlib
import re
import subprocess
import sys
import tomllib


def fail(message: str, *, hint_update_schema: bool = False) -> None:
    print(message, file=sys.stderr)
    if hint_update_schema:
        print("hint: run make update-schema", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    schema_path = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else pathlib.Path("schema/wavepeek.json")
    if not schema_path.exists():
        fail(
            f"error: schema: missing canonical schema artifact at {schema_path}",
            hint_update_schema=True,
        )

    schema_bytes = schema_path.read_bytes()
    try:
        json.loads(schema_bytes.decode("utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: schema: canonical schema is not valid JSON: {error}")

    if not schema_bytes.endswith(b"\n"):
        fail(
            "error: schema: canonical schema must end with trailing newline",
            hint_update_schema=True,
        )

    runtime_schema = subprocess.run(
        ["cargo", "run", "--quiet", "--", "schema"],
        check=True,
        stdout=subprocess.PIPE,
        text=False,
    ).stdout
    if runtime_schema != schema_bytes:
        fail(
            "error: schema: canonical schema mismatch between schema/wavepeek.json and 'wavepeek schema' output",
            hint_update_schema=True,
        )

    cargo_toml = tomllib.loads(pathlib.Path("Cargo.toml").read_text(encoding="utf-8"))
    version = cargo_toml["package"]["version"]
    expected_schema_url = (
        f"https://github.com/kleverhq/wavepeek/blob/v{version}/schema/wavepeek.json"
    )
    url_pattern = re.compile(
        r"^https://github.com/kleverhq/wavepeek/blob/v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?/schema/wavepeek\.json$"
    )

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

    if actual_schema_url != expected_schema_url:
        fail(
            "error: schema: envelope $schema URL mismatch: "
            f"expected {expected_schema_url}, got {actual_schema_url}"
        )

    if actual_schema_url is None or url_pattern.match(actual_schema_url) is None:
        fail(
            "error: schema: envelope $schema URL does not match required pattern: "
            f"{actual_schema_url}"
        )

    if "schema_version" in envelope:
        fail("error: schema: legacy schema_version key is still present in JSON envelope")


if __name__ == "__main__":
    main()
