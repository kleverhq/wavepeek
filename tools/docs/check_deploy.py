#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Callable, Sequence
from typing import Any

VERSION_RE = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
REPOSITORY_RE = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")
DEFAULT_BASE_URL = "https://kleverhq.github.io/wavepeek"
USER_AGENT = "wavepeek-docs-deploy-check"
SCHEMA_TITLE = "wavepeek JSON output envelope"
STREAM_SCHEMA_TITLE = "wavepeek JSONL stream record"
INPUT_SCHEMA_TITLE = "wavepeek JSON input documents"
BASE_SCHEMA_PROPERTIES = {"$schema", "command", "data"}
STREAM_SCHEMA_MIN_VERSION = (1, 1, 0)
INPUT_SCHEMA_MIN_VERSION = (2, 1, 0)


class DeployCheckError(Exception):
    pass


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check deployed wavepeek GitHub Pages documentation."
    )
    parser.add_argument("--version", required=True)
    parser.add_argument("--base-url", default=DEFAULT_BASE_URL)
    parser.add_argument("--repository", help="GitHub repository, such as kleverhq/wavepeek")
    parser.add_argument(
        "--expect-latest",
        dest="expect_latest",
        action="store_true",
        default=True,
        help="require latest/ and versions.json latest alias to point at --version",
    )
    parser.add_argument(
        "--no-expect-latest",
        dest="expect_latest",
        action="store_false",
        help="check --version without requiring the latest alias",
    )
    parser.add_argument("--retries", type=int, default=10)
    parser.add_argument("--retry-delay", type=float, default=3.0)
    parser.add_argument("--schema-artifact")
    parser.add_argument("--stream-schema-artifact")
    parser.add_argument("--input-schema-artifact")
    parser.add_argument("--timeout", type=float, default=20.0)
    return parser.parse_args(list(argv))


def fail(message: str) -> None:
    raise DeployCheckError(message)


def validate_version(version: str) -> str:
    if VERSION_RE.fullmatch(version) is None:
        fail(f"version must be SemVer X.Y.Z, got {version!r}")
    return version


def major_version(version: str) -> int:
    return int(validate_version(version).split(".", maxsplit=1)[0])


def version_tuple(version: str) -> tuple[int, int, int]:
    parts = version.split(".")
    if len(parts) != 3:
        fail(f"version must use major.minor.patch format, got {version!r}")
    try:
        return tuple(int(part) for part in parts)  # type: ignore[return-value]
    except ValueError:
        fail(f"version must contain numeric components, got {version!r}")


def stream_schema_required(version: str) -> bool:
    return version_tuple(version) >= STREAM_SCHEMA_MIN_VERSION


def input_schema_required(version: str) -> bool:
    return version_tuple(version) >= INPUT_SCHEMA_MIN_VERSION


def schema_artifact_suffix(version: str) -> str:
    major, minor, _patch = version_tuple(version)
    if major >= 2:
        return f"{major}.{minor}"
    return str(major)


def schema_artifact_name(version: str) -> str:
    major, minor, _patch = version_tuple(version)
    if (major, minor) >= (2, 1):
        return f"schema-output-v{major}.{minor}.json"
    if major >= 2:
        return f"wavepeek_v{major}.{minor}.json"
    return f"wavepeek_v{major}.json"


def stream_schema_artifact_name(version: str) -> str:
    major, minor, _patch = version_tuple(version)
    if (major, minor) >= (2, 1):
        return f"schema-stream-v{major}.{minor}.json"
    if major >= 2:
        return f"wavepeek-stream-v{major}.{minor}.json"
    return f"wavepeek-stream-v{major}.json"


def input_schema_artifact_name(version: str) -> str:
    major, minor, _patch = version_tuple(version)
    return f"schema-input-v{major}.{minor}.json"


def artifact_family_tuple(artifact: str, prefix: str) -> tuple[int, int] | None:
    match = re.fullmatch(rf"{re.escape(prefix)}v([1-9][0-9]*)[.]([0-9]+)[.]json", artifact)
    if match is None:
        return None
    return int(match.group(1)), int(match.group(2))


def stream_schema_includes_extract(version: str, stream_schema_artifact: str) -> bool:
    family = artifact_family_tuple(stream_schema_artifact, "schema-stream-")
    if family is not None:
        return family >= (2, 1)
    return version_tuple(version) >= INPUT_SCHEMA_MIN_VERSION


def stream_schema_includes_axi(version: str, stream_schema_artifact: str) -> bool:
    family = artifact_family_tuple(stream_schema_artifact, "schema-stream-")
    if family is not None:
        return family >= (2, 2)
    return version_tuple(version) >= (2, 2, 0)


def input_schema_uses_source_union(version: str, input_schema_artifact: str) -> bool:
    family = artifact_family_tuple(input_schema_artifact, "schema-input-")
    if family is not None:
        return family >= (2, 2)
    return version_tuple(version) >= (2, 2, 0)


def normalize_base_url(base_url: str) -> str:
    value = base_url.strip().rstrip("/")
    parsed = urllib.parse.urlparse(value)
    if parsed.scheme not in {"http", "https"} or not parsed.netloc:
        fail(f"base URL must be an absolute HTTP(S) URL, got {base_url!r}")
    if parsed.query or parsed.fragment:
        fail(f"base URL must not contain query or fragment, got {base_url!r}")
    return value


def page_url(base_url: str, path: str = "") -> str:
    base = normalize_base_url(base_url)
    if not path:
        return base + "/"
    return f"{base}/{path.lstrip('/')}"


def validate_repository(repository: str) -> str:
    if REPOSITORY_RE.fullmatch(repository) is None:
        fail(f"repository must look like owner/name, got {repository!r}")
    return repository


def fetch_bytes(url: str, *, retries: int, retry_delay: float, timeout: float) -> bytes:
    if retries < 1:
        fail("retries must be at least 1")
    last_error: str | None = None
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(1, retries + 1):
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                status = getattr(response, "status", None) or response.getcode()
                if status != 200:
                    last_error = f"HTTP {status}"
                else:
                    return response.read()
        except urllib.error.HTTPError as error:
            last_error = f"HTTP {error.code}"
        except (TimeoutError, urllib.error.URLError) as error:
            last_error = str(error)
        if attempt < retries:
            time.sleep(retry_delay)
    fail(f"{url} did not return HTTP 200 after {retries} attempt(s): {last_error}")


def fetch_json(url: str, *, retries: int, retry_delay: float, timeout: float) -> Any:
    body = fetch_bytes(url, retries=retries, retry_delay=retry_delay, timeout=timeout)
    try:
        return json.loads(body.decode("utf-8"))
    except UnicodeDecodeError as error:
        fail(f"{url} is not valid UTF-8: {error}")
    except json.JSONDecodeError as error:
        fail(f"{url} is not valid JSON: {error}")


def retry_check(
    label: str,
    *,
    retries: int,
    retry_delay: float,
    operation: Callable[[], Any],
) -> Any:
    if retries < 1:
        fail("retries must be at least 1")
    last_error: str | None = None
    for attempt in range(1, retries + 1):
        try:
            return operation()
        except DeployCheckError as error:
            last_error = str(error)
        if attempt < retries:
            time.sleep(retry_delay)
    fail(f"{label} did not pass after {retries} attempt(s): {last_error}")


def aliases(entry: dict[str, Any]) -> set[str]:
    raw = entry.get("aliases", [])
    if not isinstance(raw, list):
        fail("versions.json aliases must be arrays")
    if any(not isinstance(item, str) for item in raw):
        fail("versions.json aliases must contain only strings")
    return set(raw)


def version_entries_by_name(versions: Any) -> dict[str, dict[str, Any]]:
    if not isinstance(versions, list):
        fail("versions.json must contain a list")
    entries: dict[str, dict[str, Any]] = {}
    for entry in versions:
        if not isinstance(entry, dict) or not isinstance(entry.get("version"), str):
            fail("versions.json entries must be objects with a string version")
        version = entry["version"]
        if version in entries:
            fail(f"versions.json contains duplicate version {version!r}")
        entries[version] = entry
    return entries


def validate_versions_json(versions: Any, version: str, *, expect_latest: bool) -> None:
    entries = version_entries_by_name(versions)
    entry = entries.get(version)
    if entry is None:
        fail(f"versions.json does not contain version {version}")
    if entry.get("title") != version:
        fail(f"versions.json entry {version} must have title {version!r}")

    latest_versions = sorted(
        entry_version
        for entry_version, candidate in entries.items()
        if "latest" in aliases(candidate)
    )
    if expect_latest and latest_versions != [version]:
        rendered = ", ".join(latest_versions) if latest_versions else "none"
        fail(f"versions.json latest alias points at {rendered}, expected {version}")


def validate_versions_payload(
    url: str,
    version: str,
    *,
    expect_latest: bool,
    timeout: float,
) -> Any:
    versions = fetch_json(url, retries=1, retry_delay=0.0, timeout=timeout)
    validate_versions_json(versions, version, expect_latest=expect_latest)
    return versions


def validate_latest_matches_version(version_body: bytes, latest_body: bytes) -> None:
    if latest_body != version_body:
        fail("latest docs content does not match version docs content")


def normalize_schema_pattern(pattern: str) -> str:
    return pattern.replace(r"\/", "/").replace(r"\.", ".")


def schema_url_references_expected_artifact(
    schema_property: dict[str, Any], version: str, schema_artifact: str
) -> bool:
    major, _minor, _patch = version_tuple(version)
    if schema_artifact.startswith("schema-output-v"):
        return schema_property.get("const") == page_url(DEFAULT_BASE_URL, schema_artifact)
    pattern = schema_property.get("pattern")
    if not isinstance(pattern, str):
        return False
    normalized = normalize_schema_pattern(pattern)
    if schema_artifact in normalized:
        return True
    if major >= 2:
        return f"wavepeek_v{major}.[0-9]+.json" in normalized
    return major == 0 and "schema/wavepeek.json" in normalized


def stream_schema_url_references_expected_artifact(
    schema_property: dict[str, Any], version: str, stream_schema_artifact: str
) -> bool:
    major, _minor, _patch = version_tuple(version)
    if stream_schema_artifact.startswith("schema-stream-v"):
        return schema_property.get("const") == page_url(DEFAULT_BASE_URL, stream_schema_artifact)
    pattern = schema_property.get("pattern")
    if not isinstance(pattern, str):
        return False
    normalized = normalize_schema_pattern(pattern)
    if stream_schema_artifact in normalized:
        return True
    if major >= 2:
        return f"wavepeek-stream-v{major}.[0-9]+.json" in normalized
    return False


def input_schema_url_references_expected_artifact(
    schema_property: dict[str, Any], input_schema_artifact: str
) -> bool:
    return schema_property.get("const") == page_url(DEFAULT_BASE_URL, input_schema_artifact)


def validate_schema_json(schema: Any, version: str, schema_artifact: str | None = None) -> None:
    if not isinstance(schema, dict):
        fail("schema artifact must contain a JSON object")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        fail("schema artifact must use JSON Schema draft 2020-12")
    if schema.get("title") != SCHEMA_TITLE:
        fail(f"schema artifact title must be {SCHEMA_TITLE!r}")
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        fail("schema artifact properties must be an object")
    expected_properties = set(BASE_SCHEMA_PROPERTIES)
    expected_properties.add("warnings" if major_version(version) == 0 else "diagnostics")
    missing = sorted(expected_properties - set(properties))
    if missing:
        fail("schema artifact is missing properties: " + ", ".join(missing))
    schema_property = properties.get("$schema")
    if not isinstance(schema_property, dict):
        fail("schema artifact $schema property must be an object")
    schema_artifact = schema_artifact or schema_artifact_name(version)
    if not schema_url_references_expected_artifact(schema_property, version, schema_artifact):
        fail(
            "schema artifact $schema property must reference "
            f"{schema_artifact}"
        )


def validate_schema_payload(
    url: str, version: str, *, schema_artifact: str, timeout: float
) -> Any:
    schema = fetch_json(url, retries=1, retry_delay=0.0, timeout=timeout)
    validate_schema_json(schema, version, schema_artifact)
    return schema


def validate_stream_schema_json(
    schema: Any, version: str, stream_schema_artifact: str | None = None
) -> None:
    if not isinstance(schema, dict):
        fail("stream schema artifact must contain a JSON object")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        fail("stream schema artifact must use JSON Schema draft 2020-12")
    if schema.get("title") != STREAM_SCHEMA_TITLE:
        fail(f"stream schema artifact title must be {STREAM_SCHEMA_TITLE!r}")
    defs = schema.get("$defs")
    if not isinstance(defs, dict):
        fail("stream schema artifact must define $defs")
    command = defs.get("streamCommand")
    if not isinstance(command, dict):
        fail("stream schema artifact must define streamCommand")
    expected_commands = ["info", "scope", "signal", "value", "change", "property"]
    stream_schema_artifact = stream_schema_artifact or stream_schema_artifact_name(version)
    if stream_schema_includes_axi(version, stream_schema_artifact):
        expected_commands.append("extract axi")
    if stream_schema_includes_extract(version, stream_schema_artifact):
        expected_commands.append("extract generic")
    if command.get("enum") != expected_commands:
        fail("stream schema artifact command enum mismatch")
    begin = defs.get("beginRecord")
    if not isinstance(begin, dict):
        fail("stream schema artifact must define beginRecord")
    try:
        schema_property = begin["properties"]["$schema"]
    except (KeyError, TypeError):
        fail("stream schema artifact beginRecord must define $schema")
    if not isinstance(schema_property, dict) or not stream_schema_url_references_expected_artifact(
        schema_property, version, stream_schema_artifact
    ):
        fail(
            "stream schema artifact $schema property must reference "
            f"{stream_schema_artifact}"
        )


def validate_stream_schema_payload(
    url: str, version: str, *, stream_schema_artifact: str, timeout: float
) -> Any:
    schema = fetch_json(url, retries=1, retry_delay=0.0, timeout=timeout)
    validate_stream_schema_json(schema, version, stream_schema_artifact)
    return schema


def validate_input_schema_json(
    schema: Any, version: str, input_schema_artifact: str | None = None
) -> None:
    if not isinstance(schema, dict):
        fail("input schema artifact must contain a JSON object")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        fail("input schema artifact must use JSON Schema draft 2020-12")
    if schema.get("title") != INPUT_SCHEMA_TITLE:
        fail(f"input schema artifact title must be {INPUT_SCHEMA_TITLE!r}")
    input_schema_artifact = input_schema_artifact or input_schema_artifact_name(version)
    if input_schema_uses_source_union(version, input_schema_artifact):
        defs = schema.get("$defs")
        if not isinstance(defs, dict):
            fail("input schema artifact must define $defs")
        if schema.get("oneOf") != [
            {"$ref": "#/$defs/extractGenericSourcesInput"},
            {"$ref": "#/$defs/extractAxiSourceInput"},
        ]:
            fail("input schema artifact root must accept generic and AXI source documents")
        generic = defs.get("extractGenericSourcesInput")
        axi = defs.get("extractAxiSourceInput")
        if not isinstance(generic, dict) or not isinstance(axi, dict):
            fail("input schema artifact must define generic and AXI source document defs")
        axi_properties: dict[str, Any] | None = None
        for label, definition, kind_value in [
            ("generic", generic, "extract.generic.sources"),
            ("AXI", axi, "extract.axi.source"),
        ]:
            properties = definition.get("properties")
            if not isinstance(properties, dict):
                fail(f"input schema artifact {label} properties must be an object")
            if label == "AXI":
                axi_properties = properties
            schema_property = properties.get("$schema")
            if not isinstance(schema_property, dict):
                fail(f"input schema artifact {label} $schema property must be an object")
            if not input_schema_url_references_expected_artifact(
                schema_property, input_schema_artifact
            ):
                fail(
                    "input schema artifact $schema property must reference "
                    f"{input_schema_artifact}"
                )
            kind = properties.get("kind")
            if not isinstance(kind, dict) or kind.get("const") != kind_value:
                fail(f"input schema artifact must require {kind_value} kind")
        assert axi_properties is not None
        profile = axi_properties.get("profile")
        if not isinstance(profile, dict) or profile.get("$ref") != "#/$defs/axiProfile":
            fail("input schema artifact AXI profile reference mismatch")
        profile_definition = defs.get("axiProfile")
        if not isinstance(profile_definition, dict) or profile_definition.get("enum") != [
            "axi3",
            "axi4",
            "axi4-lite",
            "axi5",
            "axi5-lite",
            "ace",
            "ace-lite",
            "ace5",
        ]:
            fail("input schema artifact AXI profile enum mismatch")
        return

    properties = schema.get("properties")
    if not isinstance(properties, dict):
        fail("input schema artifact properties must be an object")
    schema_property = properties.get("$schema")
    if not isinstance(schema_property, dict):
        fail("input schema artifact $schema property must be an object")
    if not input_schema_url_references_expected_artifact(
        schema_property, input_schema_artifact
    ):
        fail(
            "input schema artifact $schema property must reference "
            f"{input_schema_artifact}"
        )
    kind = properties.get("kind")
    if not isinstance(kind, dict) or kind.get("const") != "extract.generic.sources":
        fail("input schema artifact must require extract generic kind")


def validate_input_schema_payload(
    url: str, version: str, *, input_schema_artifact: str, timeout: float
) -> Any:
    schema = fetch_json(url, retries=1, retry_delay=0.0, timeout=timeout)
    validate_input_schema_json(schema, version, input_schema_artifact)
    return schema


def load_pages_site(
    repository: str,
    *,
    retries: int,
    retry_delay: float,
    timeout: float,
) -> Any:
    repository = validate_repository(repository)

    def load_once() -> Any:
        try:
            result = subprocess.run(
                ["gh", "api", f"repos/{repository}/pages"],
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=timeout,
            )
        except FileNotFoundError:
            fail("gh CLI is required for GitHub Pages API checks")
        except subprocess.TimeoutExpired:
            fail(f"gh api timed out after {timeout:g}s")
        if result.returncode != 0:
            message = (result.stderr or result.stdout).strip()
            fail(f"failed to load GitHub Pages site for {repository}: {message}")
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError as error:
            fail(f"GitHub Pages API response is not valid JSON: {error}")

    return retry_check(
        "GitHub Pages API check",
        retries=retries,
        retry_delay=retry_delay,
        operation=load_once,
    )


def validate_pages_site(site: Any, base_url: str) -> None:
    if not isinstance(site, dict):
        fail("GitHub Pages API response must be a JSON object")
    if site.get("build_type") != "workflow":
        fail("GitHub Pages must be configured with build_type=workflow")
    html_url = site.get("html_url")
    if not isinstance(html_url, str):
        fail("GitHub Pages API response is missing html_url")
    if normalize_base_url(html_url) != normalize_base_url(base_url):
        fail(f"GitHub Pages html_url {html_url!r} does not match {base_url!r}")


def check_deploy(args: argparse.Namespace) -> None:
    version = validate_version(args.version)
    base_url = normalize_base_url(args.base_url)
    artifact = args.schema_artifact or schema_artifact_name(version)
    stream_artifact = args.stream_schema_artifact or stream_schema_artifact_name(version)
    input_artifact = args.input_schema_artifact or input_schema_artifact_name(version)

    urls = [
        ("site root", page_url(base_url)),
        ("version docs", page_url(base_url, f"{version}/")),
        ("versions.json", page_url(base_url, "versions.json")),
        (artifact, page_url(base_url, artifact)),
    ]
    check_stream_schema = args.stream_schema_artifact is not None or stream_schema_required(version)
    check_input_schema = args.input_schema_artifact is not None or input_schema_required(version)
    if check_stream_schema:
        urls.append((stream_artifact, page_url(base_url, stream_artifact)))
    if check_input_schema:
        urls.append((input_artifact, page_url(base_url, input_artifact)))
    if args.expect_latest:
        urls.insert(2, ("latest docs", page_url(base_url, "latest/")))

    bodies: dict[str, bytes] = {}
    for label, url in urls:
        bodies[label] = fetch_bytes(
            url, retries=args.retries, retry_delay=args.retry_delay, timeout=args.timeout
        )
        print(f"ok: docs-deploy: {label}: {url}")
    if args.expect_latest:
        retry_check(
            "latest docs content check",
            retries=args.retries,
            retry_delay=args.retry_delay,
            operation=lambda: validate_latest_matches_version(
                fetch_bytes(
                    page_url(base_url, f"{version}/"),
                    retries=1,
                    retry_delay=args.retry_delay,
                    timeout=args.timeout,
                ),
                fetch_bytes(
                    page_url(base_url, "latest/"),
                    retries=1,
                    retry_delay=args.retry_delay,
                    timeout=args.timeout,
                ),
            ),
        )
        print("ok: docs-deploy: latest docs match version docs")

    retry_check(
        "versions.json semantic check",
        retries=args.retries,
        retry_delay=args.retry_delay,
        operation=lambda: validate_versions_payload(
            page_url(base_url, "versions.json"),
            version,
            expect_latest=args.expect_latest,
            timeout=args.timeout,
        ),
    )
    print(f"ok: docs-deploy: versions.json contains {version}")

    retry_check(
        "schema artifact semantic check",
        retries=args.retries,
        retry_delay=args.retry_delay,
        operation=lambda: validate_schema_payload(
            page_url(base_url, artifact),
            version,
            schema_artifact=artifact,
            timeout=args.timeout,
        ),
    )
    print(f"ok: docs-deploy: schema artifact {artifact}")

    if check_stream_schema:
        retry_check(
            "stream schema artifact semantic check",
            retries=args.retries,
            retry_delay=args.retry_delay,
            operation=lambda: validate_stream_schema_payload(
                page_url(base_url, stream_artifact),
                version,
                stream_schema_artifact=stream_artifact,
                timeout=args.timeout,
            ),
        )
        print(f"ok: docs-deploy: stream schema artifact {stream_artifact}")

    if check_input_schema:
        retry_check(
            "input schema artifact semantic check",
            retries=args.retries,
            retry_delay=args.retry_delay,
            operation=lambda: validate_input_schema_payload(
                page_url(base_url, input_artifact),
                version,
                input_schema_artifact=input_artifact,
                timeout=args.timeout,
            ),
        )
        print(f"ok: docs-deploy: input schema artifact {input_artifact}")

    if args.repository:
        site = load_pages_site(
            args.repository,
            retries=args.retries,
            retry_delay=args.retry_delay,
            timeout=args.timeout,
        )
        validate_pages_site(site, base_url)
        print(f"ok: docs-deploy: GitHub Pages API for {args.repository}")

    print(f"checked deployed docs for wavepeek {version} at {page_url(base_url)}")


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        check_deploy(args)
        return 0
    except DeployCheckError as error:
        print(f"error: docs-deploy: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
