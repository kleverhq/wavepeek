from __future__ import annotations

import importlib.util
import json
import pathlib
import sys
import unittest

TOOLS_DIR = pathlib.Path(__file__).parent
sys.path.insert(0, str(TOOLS_DIR))
MODULE_PATH = TOOLS_DIR / "check_deploy.py"
SPEC = importlib.util.spec_from_file_location("check_deploy", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
check_deploy = importlib.util.module_from_spec(SPEC)
sys.modules["check_deploy"] = check_deploy
SPEC.loader.exec_module(check_deploy)


class CheckDeployTests(unittest.TestCase):
    def test_page_url_normalizes_base_url(self) -> None:
        self.assertEqual(
            check_deploy.page_url("https://kleverhq.github.io/wavepeek/", "0.5.0/"),
            "https://kleverhq.github.io/wavepeek/0.5.0/",
        )
        self.assertEqual(
            check_deploy.page_url("https://kleverhq.github.io/wavepeek"),
            "https://kleverhq.github.io/wavepeek/",
        )

    def test_page_url_rejects_relative_base_url(self) -> None:
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "absolute"):
            check_deploy.page_url("kleverhq.github.io/wavepeek", "0.5.0/")

    def test_page_url_rejects_query_or_fragment(self) -> None:
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "query"):
            check_deploy.page_url("https://kleverhq.github.io/wavepeek?x=1", "0.5.0/")
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "fragment"):
            check_deploy.page_url("https://kleverhq.github.io/wavepeek#docs", "0.5.0/")

    def test_stream_schema_requirement_starts_after_published_v1_0_1(self) -> None:
        self.assertFalse(check_deploy.stream_schema_required("1.0.1"))
        self.assertTrue(check_deploy.stream_schema_required("1.1.0"))

    def test_schema_artifact_name_uses_legacy_major_then_family_v2_minor(self) -> None:
        self.assertEqual(check_deploy.schema_artifact_name("0.5.0"), "wavepeek_v0.json")
        self.assertEqual(check_deploy.schema_artifact_name("1.1.0"), "wavepeek_v1.json")
        self.assertEqual(check_deploy.schema_artifact_name("2.0.0"), "wavepeek_v2.0.json")
        self.assertEqual(check_deploy.schema_artifact_name("2.1.0"), "schema-output-v2.1.json")
        self.assertEqual(check_deploy.schema_artifact_name("12.3.1"), "schema-output-v12.3.json")
        self.assertEqual(
            check_deploy.stream_schema_artifact_name("0.5.0"),
            "wavepeek-stream-v0.json",
        )
        self.assertEqual(
            check_deploy.stream_schema_artifact_name("1.1.0"),
            "wavepeek-stream-v1.json",
        )
        self.assertEqual(
            check_deploy.stream_schema_artifact_name("2.0.0"),
            "wavepeek-stream-v2.0.json",
        )
        self.assertEqual(
            check_deploy.stream_schema_artifact_name("2.1.0"),
            "schema-stream-v2.1.json",
        )
        self.assertEqual(
            check_deploy.stream_schema_artifact_name("12.3.1"),
            "schema-stream-v12.3.json",
        )
        self.assertEqual(
            check_deploy.input_schema_artifact_name("2.1.0"),
            "schema-input-v2.1.json",
        )

    def test_stream_schema_expectations_follow_explicit_artifact_family(self) -> None:
        self.assertFalse(
            check_deploy.stream_schema_includes_extract("2.0.0", "schema-stream-v2.0.json")
        )
        self.assertTrue(
            check_deploy.stream_schema_includes_extract("2.0.0", "schema-stream-v2.1.json")
        )

    def test_retry_check_retries_stale_then_fresh(self) -> None:
        attempts = 0

        def operation() -> str:
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                raise check_deploy.DeployCheckError("stale")
            return "fresh"

        self.assertEqual(
            check_deploy.retry_check(
                "state", retries=2, retry_delay=0.0, operation=operation
            ),
            "fresh",
        )
        self.assertEqual(attempts, 2)

    def test_retry_check_reports_exhausted_failures(self) -> None:
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "state"):
            check_deploy.retry_check(
                "state",
                retries=2,
                retry_delay=0.0,
                operation=lambda: (_ for _ in ()).throw(
                    check_deploy.DeployCheckError("still stale")
                ),
            )

    def test_validate_versions_json_requires_version_and_latest_alias(self) -> None:
        check_deploy.validate_versions_json(
            [
                {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
            ],
            "0.5.0",
            expect_latest=True,
        )

        with self.assertRaisesRegex(check_deploy.DeployCheckError, "latest"):
            check_deploy.validate_versions_json(
                [
                    {"version": "0.4.0", "title": "0.4.0", "aliases": ["latest"]},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": []},
                ],
                "0.5.0",
                expect_latest=True,
            )

    def test_validate_versions_json_allows_old_version_without_latest(self) -> None:
        check_deploy.validate_versions_json(
            [
                {"version": "0.4.0", "title": "0.4.0", "aliases": []},
                {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
            ],
            "0.4.0",
            expect_latest=False,
        )

    def test_validate_versions_json_rejects_duplicate_versions(self) -> None:
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "duplicate"):
            check_deploy.validate_versions_json(
                [
                    {"version": "0.5.0", "title": "0.5.0", "aliases": []},
                    {"version": "0.5.0", "title": "0.5.0", "aliases": ["latest"]},
                ],
                "0.5.0",
                expect_latest=False,
            )

    def test_validate_latest_matches_version_rejects_stale_latest(self) -> None:
        check_deploy.validate_latest_matches_version(b"same", b"same")

        with self.assertRaisesRegex(check_deploy.DeployCheckError, "latest"):
            check_deploy.validate_latest_matches_version(b"version", b"latest")

    def test_validate_schema_json_checks_contract_shape(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON output envelope",
            "properties": {
                "$schema": {
                    "pattern": r"^https://kleverhq\.github\.io/wavepeek/wavepeek_v1\.json$"
                },
                "command": {},
                "data": {},
                "diagnostics": {},
            },
        }

        check_deploy.validate_schema_json(schema, "1.0.0")

        broken = json.loads(json.dumps(schema))
        del broken["properties"]["diagnostics"]
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "diagnostics"):
            check_deploy.validate_schema_json(broken, "1.0.0")

    def test_validate_schema_json_allows_legacy_major_zero_self_reference(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON output envelope",
            "properties": {
                "$schema": {
                    "pattern": r"^https://raw\.githubusercontent\.com/kleverhq/wavepeek/v[0-9]+\.[0-9]+\.[0-9]+/schema/wavepeek\.json$"
                },
                "command": {},
                "data": {},
                "warnings": {},
            },
        }

        check_deploy.validate_schema_json(schema, "0.5.0")

        with self.assertRaisesRegex(check_deploy.DeployCheckError, "diagnostics"):
            check_deploy.validate_schema_json(schema, "1.0.0")

    def test_validate_schema_json_rejects_wrong_major_self_reference(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON output envelope",
            "properties": {
                "$schema": {"pattern": "wavepeek_v0.json"},
                "command": {},
                "data": {},
                "diagnostics": {},
            },
        }

        with self.assertRaisesRegex(check_deploy.DeployCheckError, "wavepeek_v1"):
            check_deploy.validate_schema_json(schema, "1.0.0")

    def test_validate_schema_json_accepts_legacy_v2_pattern_artifact(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON output envelope",
            "properties": {
                "$schema": {
                    "pattern": r"^https://kleverhq\.github\.io/wavepeek/wavepeek_v2\.[0-9]+\.json$"
                },
                "command": {},
                "data": {},
                "diagnostics": {},
            },
        }

        check_deploy.validate_schema_json(schema, "2.0.0", "wavepeek_v2.0.json")

    def test_validate_schema_json_accepts_v2_exact_family_const(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON output envelope",
            "properties": {
                "$schema": {
                    "const": "https://kleverhq.github.io/wavepeek/schema-output-v2.0.json"
                },
                "command": {},
                "data": {},
                "diagnostics": {},
            },
        }

        check_deploy.validate_schema_json(schema, "2.0.0", "schema-output-v2.0.json")

    def test_validate_schema_json_rejects_stale_v2_2_axi_profiles(self) -> None:
        schema_path = TOOLS_DIR.parent.parent / "schema" / "output.json"
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        check_deploy.validate_schema_json(
            schema, "2.1.0", "schema-output-v2.2.json"
        )

        stale_profiles = json.loads(json.dumps(schema))
        stale_profiles["$defs"]["axiProfile"]["enum"] = [
            "axi3",
            "axi4",
            "axi4-lite",
            "ace",
            "ace-lite",
            "ace5",
        ]
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "AXI profile"):
            check_deploy.validate_schema_json(
                stale_profiles, "2.1.0", "schema-output-v2.2.json"
            )

        stale_issue = json.loads(json.dumps(schema))
        axi5_branch = next(
            branch
            for branch in stale_issue["$defs"]["extractAxiData"]["oneOf"]
            if branch["properties"]["profile"]["const"] == "axi5"
        )
        axi5_branch["properties"]["issue"]["const"] = "H.c"
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "Issue L"):
            check_deploy.validate_schema_json(
                stale_issue, "2.1.0", "schema-output-v2.2.json"
            )

        stale_transfers = json.loads(json.dumps(schema))
        stale_transfers["$defs"]["extractAxi5Transfer"]["oneOf"] = stale_transfers[
            "$defs"
        ]["extractAxi5Transfer"]["oneOf"][:5]
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "AXI5 transfer"):
            check_deploy.validate_schema_json(
                stale_transfers, "2.1.0", "schema-output-v2.2.json"
            )

    def test_validate_schema_json_requires_schema_property_object(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON output envelope",
            "properties": {
                "$schema": "not an object",
                "command": {},
                "data": {},
                "warnings": {},
            },
        }

        with self.assertRaisesRegex(check_deploy.DeployCheckError, "object"):
            check_deploy.validate_schema_json(schema, "0.5.0")

    def test_validate_stream_schema_json_checks_contract_shape(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSONL stream record",
            "$defs": {
                "streamCommand": {
                    "enum": ["info", "scope", "signal", "value", "change", "property"]
                },
                "beginRecord": {
                    "properties": {
                        "$schema": {
                            "pattern": r"^https://kleverhq\.github\.io/wavepeek/wavepeek-stream-v1\.json$"
                        }
                    }
                },
            },
        }

        check_deploy.validate_stream_schema_json(schema, "1.0.0")

        broken = json.loads(json.dumps(schema))
        broken["$defs"]["streamCommand"]["enum"] = ["info"]
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "command enum"):
            check_deploy.validate_stream_schema_json(broken, "1.0.0")

    def test_validate_stream_schema_json_rejects_stale_v2_2_axi_profiles(self) -> None:
        schema_path = TOOLS_DIR.parent.parent / "schema" / "stream.json"
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        check_deploy.validate_stream_schema_json(
            schema, "2.1.0", "schema-stream-v2.2.json"
        )

        stale_profiles = json.loads(json.dumps(schema))
        stale_profiles["$defs"]["axiProfile"]["enum"] = [
            "axi3",
            "axi4",
            "axi4-lite",
            "ace",
            "ace-lite",
            "ace5",
        ]
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "AXI profile"):
            check_deploy.validate_stream_schema_json(
                stale_profiles, "2.1.0", "schema-stream-v2.2.json"
            )

        stale_issue = json.loads(json.dumps(schema))
        axi5_lite_branch = next(
            branch
            for branch in stale_issue["$defs"]["extractAxiContext"]["oneOf"]
            if branch["properties"]["profile"]["const"] == "axi5-lite"
        )
        axi5_lite_branch["properties"]["issue"]["const"] = "H.c"
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "Issue L"):
            check_deploy.validate_stream_schema_json(
                stale_issue, "2.1.0", "schema-stream-v2.2.json"
            )

        stale_transfers = json.loads(json.dumps(schema))
        stale_transfers["$defs"]["extractAxi5LiteTransfer"]["oneOf"] = []
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "AXI5-Lite transfer"):
            check_deploy.validate_stream_schema_json(
                stale_transfers, "2.1.0", "schema-stream-v2.2.json"
            )

    def test_validate_stream_schema_json_accepts_legacy_v2_pattern_artifact(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSONL stream record",
            "$defs": {
                "streamCommand": {
                    "enum": ["info", "scope", "signal", "value", "change", "property"]
                },
                "beginRecord": {
                    "properties": {
                        "$schema": {
                            "pattern": r"^https://kleverhq\.github\.io/wavepeek/wavepeek-stream-v2\.[0-9]+\.json$"
                        }
                    }
                },
            },
        }

        check_deploy.validate_stream_schema_json(
            schema, "2.0.0", "wavepeek-stream-v2.0.json"
        )

    def test_validate_stream_schema_json_accepts_v2_exact_family_const(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSONL stream record",
            "$defs": {
                "streamCommand": {
                    "enum": ["info", "scope", "signal", "value", "change", "property"]
                },
                "beginRecord": {
                    "properties": {
                        "$schema": {
                            "const": "https://kleverhq.github.io/wavepeek/schema-stream-v2.0.json"
                        }
                    }
                },
            },
        }

        check_deploy.validate_stream_schema_json(schema, "2.0.0", "schema-stream-v2.0.json")

    def test_validate_stream_schema_json_accepts_explicit_v2_1_family_for_older_package_version(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSONL stream record",
            "$defs": {
                "streamCommand": {
                    "enum": ["info", "scope", "signal", "value", "change", "property", "extract generic"]
                },
                "beginRecord": {
                    "properties": {
                        "$schema": {
                            "const": "https://kleverhq.github.io/wavepeek/schema-stream-v2.1.json"
                        }
                    }
                },
            },
        }

        check_deploy.validate_stream_schema_json(schema, "2.0.0", "schema-stream-v2.1.json")

    def test_validate_stream_schema_json_accepts_explicit_v2_2_axi_family_for_older_package_version(self) -> None:
        schema_path = TOOLS_DIR.parent.parent / "schema" / "stream.json"
        schema = json.loads(schema_path.read_text(encoding="utf-8"))

        check_deploy.validate_stream_schema_json(schema, "2.0.0", "schema-stream-v2.2.json")

    def test_validate_schema_json_requires_schema_pattern(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON output envelope",
            "properties": {
                "$schema": {},
                "command": {},
                "data": {},
                "warnings": {},
            },
        }

        with self.assertRaisesRegex(check_deploy.DeployCheckError, "reference"):
            check_deploy.validate_schema_json(schema, "0.5.0")

    def test_validate_input_schema_json_checks_legacy_contract_shape(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON input documents",
            "properties": {
                "$schema": {
                    "const": "https://kleverhq.github.io/wavepeek/schema-input-v2.1.json"
                },
                "kind": {"const": "extract.generic.sources"},
            },
        }

        check_deploy.validate_input_schema_json(schema, "2.1.0", "schema-input-v2.1.json")

        broken = json.loads(json.dumps(schema))
        broken["properties"]["kind"]["const"] = "wrong"
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "kind"):
            check_deploy.validate_input_schema_json(broken, "2.1.0", "schema-input-v2.1.json")

    def test_validate_input_schema_json_checks_v2_2_union_contract_shape(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "wavepeek JSON input documents",
            "oneOf": [
                {"$ref": "#/$defs/extractGenericSourcesInput"},
                {"$ref": "#/$defs/extractAxiSourceInput"},
            ],
            "$defs": {
                "axiProfile": {
                    "enum": [
                        "axi3",
                        "axi4",
                        "axi4-lite",
                        "axi5",
                        "axi5-lite",
                        "ace",
                        "ace-lite",
                        "ace5",
                    ]
                },
                "extractGenericSourcesInput": {
                    "properties": {
                        "$schema": {
                            "const": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
                        },
                        "kind": {"const": "extract.generic.sources"},
                    }
                },
                "extractAxiSourceInput": {
                    "properties": {
                        "$schema": {
                            "const": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json"
                        },
                        "kind": {"const": "extract.axi.source"},
                        "profile": {"$ref": "#/$defs/axiProfile"},
                    }
                },
            },
        }

        check_deploy.validate_input_schema_json(schema, "2.1.0", "schema-input-v2.2.json")

        broken_reference = json.loads(json.dumps(schema))
        broken_reference["$defs"]["extractAxiSourceInput"]["properties"]["profile"][
            "$ref"
        ] = "#/$defs/wrong"
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "profile reference"):
            check_deploy.validate_input_schema_json(
                broken_reference, "2.1.0", "schema-input-v2.2.json"
            )

        broken = json.loads(json.dumps(schema))
        broken["$defs"]["axiProfile"]["enum"] = ["axi5"]
        with self.assertRaisesRegex(check_deploy.DeployCheckError, "profile enum"):
            check_deploy.validate_input_schema_json(broken, "2.1.0", "schema-input-v2.2.json")

    def test_validate_input_schema_json_accepts_current_canonical_artifact(self) -> None:
        schema_path = TOOLS_DIR.parent.parent / "schema" / "input.json"
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        check_deploy.validate_input_schema_json(
            schema, "2.1.0", "schema-input-v2.2.json"
        )

    def test_load_pages_site_retries_and_uses_timeout(self) -> None:
        calls = 0
        original_run = check_deploy.subprocess.run

        def fake_run(*args: object, **kwargs: object) -> object:
            nonlocal calls
            calls += 1
            self.assertEqual(kwargs["timeout"], 2.0)
            if calls == 1:
                return check_deploy.subprocess.CompletedProcess(
                    args[0], 1, "", "temporary"
                )
            return check_deploy.subprocess.CompletedProcess(
                args[0],
                0,
                '{"html_url":"https://kleverhq.github.io/wavepeek/","build_type":"workflow"}',
                "",
            )

        check_deploy.subprocess.run = fake_run
        try:
            site = check_deploy.load_pages_site(
                "kleverhq/wavepeek", retries=2, retry_delay=0.0, timeout=2.0
            )
        finally:
            check_deploy.subprocess.run = original_run

        self.assertEqual(site["build_type"], "workflow")
        self.assertEqual(calls, 2)

    def test_validate_pages_site_requires_workflow_build_type_and_url(self) -> None:
        check_deploy.validate_pages_site(
            {
                "html_url": "https://kleverhq.github.io/wavepeek/",
                "build_type": "workflow",
            },
            "https://kleverhq.github.io/wavepeek",
        )

        with self.assertRaisesRegex(check_deploy.DeployCheckError, "workflow"):
            check_deploy.validate_pages_site(
                {
                    "html_url": "https://kleverhq.github.io/wavepeek/",
                    "build_type": "legacy",
                },
                "https://kleverhq.github.io/wavepeek",
            )


if __name__ == "__main__":
    unittest.main()
