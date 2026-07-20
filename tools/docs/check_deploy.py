#!/usr/bin/env python3

from __future__ import annotations

import argparse
import http.client
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
STREAM_SCHEMA_MIN_VERSION = (1, 1, 0)
INPUT_SCHEMA_MIN_VERSION = (2, 1, 0)
AXISTREAM_SCHEMA_MIN_VERSION = (2, 2, 0)


class DeployCheckError(Exception):
    pass


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check deployed wavepeek GitHub Pages endpoints."
    )
    parser.add_argument("--version", required=True)
    parser.add_argument("--base-url", default=DEFAULT_BASE_URL)
    parser.add_argument("--repository", help="GitHub repository, such as kleverhq/wavepeek")
    parser.add_argument(
        "--expect-latest",
        dest="expect_latest",
        action="store_true",
        default=True,
        help="require the latest documentation endpoint",
    )
    parser.add_argument(
        "--no-expect-latest",
        dest="expect_latest",
        action="store_false",
        help="check the version without requiring the latest endpoint",
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


def version_tuple(version: str) -> tuple[int, int, int]:
    validate_version(version)
    major, minor, patch = version.split(".")
    return int(major), int(minor), int(patch)


def stream_schema_required(version: str) -> bool:
    return version_tuple(version) >= STREAM_SCHEMA_MIN_VERSION


def input_schema_required(version: str) -> bool:
    return version_tuple(version) >= INPUT_SCHEMA_MIN_VERSION


def axistream_schema_required(version: str) -> bool:
    return version_tuple(version) >= AXISTREAM_SCHEMA_MIN_VERSION


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
    return base + "/" if not path else f"{base}/{path.lstrip('/')}"


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
                if status == 200:
                    return response.read()
                last_error = f"HTTP {status}"
        except urllib.error.HTTPError as error:
            last_error = f"HTTP {error.code}"
        except (TimeoutError, OSError, http.client.HTTPException) as error:
            last_error = str(error)
        if attempt < retries:
            time.sleep(retry_delay)
    fail(f"{url} did not return HTTP 200 after {retries} attempt(s): {last_error}")


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


def parse_schema_document(body: bytes, label: str) -> dict[str, Any]:
    try:
        document = json.loads(body)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"{label} is not valid JSON: {error}")
    if not isinstance(document, dict):
        fail(f"{label} must contain a JSON object")
    return document


def validate_axistream_deployed_schemas(
    output_body: bytes, stream_body: bytes, input_body: bytes
) -> None:
    output = parse_schema_document(output_body, "output schema")
    stream = parse_schema_document(stream_body, "stream schema")
    input_schema = parse_schema_document(input_body, "input schema")

    if "extract axistream" not in output.get("properties", {}).get("command", {}).get("enum", []):
        fail("output schema must expose extract axistream")
    if "extract axistream" not in stream.get("$defs", {}).get("streamCommand", {}).get("enum", []):
        fail("stream schema must expose extract axistream")
    source_ref = {"$ref": "#/$defs/extractAxiStreamSourceInput"}
    if source_ref not in input_schema.get("oneOf", []):
        fail("input schema must expose extractAxiStreamSourceInput")
    source = input_schema.get("$defs", {}).get("extractAxiStreamSourceInput", {})
    kind = source.get("properties", {}).get("kind", {}).get("const")
    if kind != "extract.axistream.source":
        fail("input schema must expose extract.axistream.source kind")


def check_deploy(args: argparse.Namespace) -> None:
    version = validate_version(args.version)
    base_url = normalize_base_url(args.base_url)
    output_artifact = args.schema_artifact or schema_artifact_name(version)
    stream_artifact = args.stream_schema_artifact or stream_schema_artifact_name(version)
    input_artifact = args.input_schema_artifact or input_schema_artifact_name(version)

    endpoints = [
        ("site root", page_url(base_url)),
        ("version docs", page_url(base_url, f"{version}/")),
        ("versions.json", page_url(base_url, "versions.json")),
        (output_artifact, page_url(base_url, output_artifact)),
    ]
    if args.expect_latest:
        endpoints.insert(2, ("latest docs", page_url(base_url, "latest/")))
    if args.stream_schema_artifact is not None or stream_schema_required(version):
        endpoints.append((stream_artifact, page_url(base_url, stream_artifact)))
    if args.input_schema_artifact is not None or input_schema_required(version):
        endpoints.append((input_artifact, page_url(base_url, input_artifact)))

    fetched: dict[str, bytes] = {}
    for label, url in endpoints:
        fetched[label] = fetch_bytes(
            url,
            retries=args.retries,
            retry_delay=args.retry_delay,
            timeout=args.timeout,
        )
        print(f"ok: docs-deploy: {label}: {url}")

    if axistream_schema_required(version):
        validate_axistream_deployed_schemas(
            fetched[output_artifact], fetched[stream_artifact], fetched[input_artifact]
        )
        print("ok: docs-deploy: AXI-Stream schema contracts")

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
