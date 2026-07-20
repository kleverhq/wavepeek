from __future__ import annotations

import http.client
import importlib.util
import json
import pathlib
import subprocess
import sys
import unittest
import urllib.error
from unittest import mock

TOOLS_DIR = pathlib.Path(__file__).parent
sys.path.insert(0, str(TOOLS_DIR))
MODULE_PATH = TOOLS_DIR / "check_deploy.py"
SPEC = importlib.util.spec_from_file_location("check_deploy", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
check_deploy = importlib.util.module_from_spec(SPEC)
sys.modules["check_deploy"] = check_deploy
SPEC.loader.exec_module(check_deploy)


class Response:
    def __init__(self, body: bytes = b"ok", status: int = 200) -> None:
        self.body = body
        self.status = status

    def __enter__(self) -> Response:
        return self

    def __exit__(self, *args: object) -> None:
        return None

    def getcode(self) -> int:
        return self.status

    def read(self) -> bytes:
        return self.body


class CheckDeployTests(unittest.TestCase):
    def test_parse_args_defaults_to_latest_and_public_site(self) -> None:
        args = check_deploy.parse_args(["--version", "2.2.0"])
        self.assertEqual(args.base_url, check_deploy.DEFAULT_BASE_URL)
        self.assertTrue(args.expect_latest)
        self.assertEqual(args.retries, 10)
        self.assertEqual(args.timeout, 20.0)

        args = check_deploy.parse_args(
            ["--version", "2.2.0", "--no-expect-latest"]
        )
        self.assertFalse(args.expect_latest)

    def test_version_validation_and_artifact_names_cover_legacy_families(self) -> None:
        self.assertEqual(check_deploy.validate_version("2.2.0"), "2.2.0")
        for invalid in ["v2.2.0", "2.2", "02.2.0", "next"]:
            with self.assertRaises(check_deploy.DeployCheckError):
                check_deploy.validate_version(invalid)

        self.assertEqual(check_deploy.schema_artifact_name("1.0.0"), "wavepeek_v1.json")
        self.assertEqual(check_deploy.schema_artifact_name("2.0.0"), "wavepeek_v2.0.json")
        self.assertEqual(
            check_deploy.schema_artifact_name("2.2.0"),
            "schema-output-v2.2.json",
        )
        self.assertEqual(
            check_deploy.stream_schema_artifact_name("2.2.0"),
            "schema-stream-v2.2.json",
        )
        self.assertEqual(
            check_deploy.input_schema_artifact_name("2.2.0"),
            "schema-input-v2.2.json",
        )

    def test_schema_requirement_thresholds_preserve_historical_checks(self) -> None:
        self.assertFalse(check_deploy.stream_schema_required("1.0.9"))
        self.assertTrue(check_deploy.stream_schema_required("1.1.0"))
        self.assertFalse(check_deploy.input_schema_required("2.0.9"))
        self.assertTrue(check_deploy.input_schema_required("2.1.0"))
        self.assertFalse(check_deploy.axistream_schema_required("2.1.9"))
        self.assertTrue(check_deploy.axistream_schema_required("2.2.0"))

    def test_url_and_repository_validation(self) -> None:
        self.assertEqual(
            check_deploy.normalize_base_url(" https://example.test/docs/ "),
            "https://example.test/docs",
        )
        self.assertEqual(
            check_deploy.page_url("https://example.test/docs", "2.2.0/"),
            "https://example.test/docs/2.2.0/",
        )
        self.assertEqual(check_deploy.validate_repository("owner/repo"), "owner/repo")

        for invalid in ["example.test/docs", "ftp://example.test", "https://x/y?q=1"]:
            with self.assertRaises(check_deploy.DeployCheckError):
                check_deploy.normalize_base_url(invalid)
        with self.assertRaises(check_deploy.DeployCheckError):
            check_deploy.validate_repository("owner")

    def test_fetch_bytes_retries_transient_failures(self) -> None:
        with (
            mock.patch.object(
                check_deploy.urllib.request,
                "urlopen",
                side_effect=[urllib.error.URLError("not ready"), Response(b"ready")],
            ) as urlopen,
            mock.patch.object(check_deploy.time, "sleep") as sleep,
        ):
            body = check_deploy.fetch_bytes(
                "https://example.test/file",
                retries=2,
                retry_delay=0.25,
                timeout=3.0,
            )
        self.assertEqual(body, b"ready")
        self.assertEqual(urlopen.call_count, 2)
        sleep.assert_called_once_with(0.25)

    def test_fetch_bytes_retries_incomplete_response_body(self) -> None:
        incomplete = Response()
        incomplete.read = mock.Mock(side_effect=http.client.IncompleteRead(b"partial"))
        with (
            mock.patch.object(
                check_deploy.urllib.request,
                "urlopen",
                side_effect=[incomplete, Response(b"complete")],
            ),
            mock.patch.object(check_deploy.time, "sleep") as sleep,
        ):
            body = check_deploy.fetch_bytes(
                "https://example.test/file",
                retries=2,
                retry_delay=0.25,
                timeout=3.0,
            )
        self.assertEqual(body, b"complete")
        sleep.assert_called_once_with(0.25)

    def test_fetch_bytes_rejects_non_200_and_invalid_retry_count(self) -> None:
        with mock.patch.object(
            check_deploy.urllib.request,
            "urlopen",
            return_value=Response(status=503),
        ):
            with self.assertRaisesRegex(check_deploy.DeployCheckError, "HTTP 503"):
                check_deploy.fetch_bytes(
                    "https://example.test/file",
                    retries=1,
                    retry_delay=0.0,
                    timeout=3.0,
                )
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "at least 1"):
            check_deploy.fetch_bytes(
                "https://example.test/file",
                retries=0,
                retry_delay=0.0,
                timeout=3.0,
            )

    def test_check_deploy_fetches_current_release_endpoints(self) -> None:
        args = check_deploy.parse_args(
            [
                "--version",
                "2.2.0",
                "--base-url",
                "https://example.test/wavepeek",
                "--retries",
                "1",
            ]
        )
        with (
            mock.patch.object(check_deploy, "fetch_bytes", return_value=b"ok") as fetch,
            mock.patch.object(
                check_deploy, "validate_axistream_deployed_schemas"
            ) as validate,
        ):
            check_deploy.check_deploy(args)

        validate.assert_called_once_with(b"ok", b"ok", b"ok")
        self.assertEqual(
            [call.args[0] for call in fetch.call_args_list],
            [
                "https://example.test/wavepeek/",
                "https://example.test/wavepeek/2.2.0/",
                "https://example.test/wavepeek/latest/",
                "https://example.test/wavepeek/versions.json",
                "https://example.test/wavepeek/schema-output-v2.2.json",
                "https://example.test/wavepeek/schema-stream-v2.2.json",
                "https://example.test/wavepeek/schema-input-v2.2.json",
            ],
        )

    def test_check_deploy_omits_latest_and_optional_legacy_schemas(self) -> None:
        args = check_deploy.parse_args(
            [
                "--version",
                "1.0.0",
                "--base-url",
                "https://example.test/wavepeek",
                "--no-expect-latest",
                "--retries",
                "1",
            ]
        )
        with mock.patch.object(check_deploy, "fetch_bytes", return_value=b"ok") as fetch:
            check_deploy.check_deploy(args)

        urls = [call.args[0] for call in fetch.call_args_list]
        self.assertNotIn("https://example.test/wavepeek/latest/", urls)
        self.assertEqual(
            urls,
            [
                "https://example.test/wavepeek/",
                "https://example.test/wavepeek/1.0.0/",
                "https://example.test/wavepeek/versions.json",
                "https://example.test/wavepeek/wavepeek_v1.json",
            ],
        )

    def test_explicit_schema_artifacts_are_always_fetched(self) -> None:
        args = check_deploy.parse_args(
            [
                "--version",
                "1.0.0",
                "--base-url",
                "https://example.test",
                "--schema-artifact",
                "output.json",
                "--stream-schema-artifact",
                "stream.json",
                "--input-schema-artifact",
                "input.json",
                "--retries",
                "1",
            ]
        )
        with mock.patch.object(check_deploy, "fetch_bytes", return_value=b"ok") as fetch:
            check_deploy.check_deploy(args)
        urls = [call.args[0] for call in fetch.call_args_list]
        self.assertIn("https://example.test/output.json", urls)
        self.assertIn("https://example.test/stream.json", urls)
        self.assertIn("https://example.test/input.json", urls)

    def test_axistream_schema_validation_checks_command_and_source_kind(self) -> None:
        output = json.dumps(
            {"properties": {"command": {"enum": ["extract axistream"]}}}
        ).encode()
        stream = json.dumps(
            {"$defs": {"streamCommand": {"enum": ["extract axistream"]}}}
        ).encode()
        input_schema = json.dumps(
            {
                "oneOf": [{"$ref": "#/$defs/extractAxiStreamSourceInput"}],
                "$defs": {
                    "extractAxiStreamSourceInput": {
                        "properties": {
                            "kind": {"const": "extract.axistream.source"}
                        }
                    }
                },
            }
        ).encode()
        check_deploy.validate_axistream_deployed_schemas(
            output, stream, input_schema
        )

        for invalid in [
            (b"{}", stream, input_schema),
            (output, b"{}", input_schema),
            (output, stream, b"{}"),
            (output, stream, b"not-json"),
        ]:
            with self.assertRaises(check_deploy.DeployCheckError):
                check_deploy.validate_axistream_deployed_schemas(*invalid)

    def test_pages_site_requires_workflow_build_and_matching_url(self) -> None:
        check_deploy.validate_pages_site(
            {"build_type": "workflow", "html_url": "https://example.test/docs/"},
            "https://example.test/docs",
        )
        for site in [
            {"build_type": "legacy", "html_url": "https://example.test/docs"},
            {"build_type": "workflow", "html_url": "https://wrong.test"},
            {"build_type": "workflow"},
            [],
        ]:
            with self.assertRaises(check_deploy.DeployCheckError):
                check_deploy.validate_pages_site(site, "https://example.test/docs")

    def test_check_deploy_runs_optional_pages_api_check(self) -> None:
        args = check_deploy.parse_args(
            [
                "--version",
                "2.2.0",
                "--base-url",
                "https://example.test/docs",
                "--repository",
                "owner/repo",
                "--retries",
                "1",
            ]
        )
        site = {"build_type": "workflow", "html_url": "https://example.test/docs"}
        with (
            mock.patch.object(check_deploy, "fetch_bytes", return_value=b"ok"),
            mock.patch.object(check_deploy, "validate_axistream_deployed_schemas"),
            mock.patch.object(check_deploy, "load_pages_site", return_value=site) as load,
        ):
            check_deploy.check_deploy(args)
        load.assert_called_once_with(
            "owner/repo", retries=1, retry_delay=3.0, timeout=20.0
        )

    def test_load_pages_site_retries_and_parses_json(self) -> None:
        failed = subprocess.CompletedProcess(
            ["gh"], returncode=1, stdout="", stderr="not ready"
        )
        passed = subprocess.CompletedProcess(
            ["gh"],
            returncode=0,
            stdout=json.dumps(
                {"build_type": "workflow", "html_url": "https://example.test"}
            ),
            stderr="",
        )
        with (
            mock.patch.object(
                check_deploy.subprocess,
                "run",
                side_effect=[failed, passed],
            ) as run,
            mock.patch.object(check_deploy.time, "sleep") as sleep,
        ):
            site = check_deploy.load_pages_site(
                "owner/repo", retries=2, retry_delay=0.5, timeout=4.0
            )
        self.assertEqual(site["build_type"], "workflow")
        self.assertEqual(run.call_count, 2)
        sleep.assert_called_once_with(0.5)

    def test_main_reports_deploy_errors(self) -> None:
        with mock.patch.object(
            check_deploy,
            "check_deploy",
            side_effect=check_deploy.DeployCheckError("boom"),
        ):
            self.assertEqual(check_deploy.main(["--version", "2.2.0"]), 1)


if __name__ == "__main__":
    unittest.main()
