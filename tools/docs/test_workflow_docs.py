from __future__ import annotations

import importlib.util
import json
import os
import pathlib
import sys
import tempfile
import unittest

TOOLS_DIR = pathlib.Path(__file__).parent
sys.path.insert(0, str(TOOLS_DIR))
MODULE_PATH = TOOLS_DIR / "workflow_docs.py"
SPEC = importlib.util.spec_from_file_location("workflow_docs", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
workflow_docs = importlib.util.module_from_spec(SPEC)
sys.modules["workflow_docs"] = workflow_docs
SPEC.loader.exec_module(workflow_docs)


class WorkflowDocsTests(unittest.TestCase):
    def test_validate_dispatch_accepts_default_branch_and_stable_release(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            release_json = pathlib.Path(temp) / "release.json"
            release_json.write_text(
                json.dumps({"tag_name": "v0.5.0", "draft": False, "prerelease": False}), encoding="utf-8"
            )

            workflow_docs.validate_dispatch(
                workflow_docs.parse_args(
                    ["validate-dispatch", "--release-json", str(release_json)]
                ),
                {
                    "VERSION": "0.5.0",
                    "SOURCE_REF": "v0.5.0",
                    "DISPATCH_REF": "main",
                    "DEFAULT_BRANCH": "main",
                    "GITHUB_REPOSITORY": "kleverhq/wavepeek",
                },
            )

    def test_validate_dispatch_rejects_non_default_branch(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            release_json = pathlib.Path(temp) / "release.json"
            release_json.write_text(
                json.dumps({"tag_name": "v0.5.0", "draft": False, "prerelease": False}), encoding="utf-8"
            )

            with self.assertRaisesRegex(workflow_docs.WorkflowError, "main"):
                workflow_docs.validate_dispatch(
                    workflow_docs.parse_args(
                        ["validate-dispatch", "--release-json", str(release_json)]
                    ),
                    {
                        "VERSION": "0.5.0",
                        "SOURCE_REF": "v0.5.0",
                        "DISPATCH_REF": "docs/branch",
                        "DEFAULT_BRANCH": "main",
                        "GITHUB_REPOSITORY": "kleverhq/wavepeek",
                    },
                )

    def test_validate_dispatch_rejects_mismatched_source_ref(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            release_json = pathlib.Path(temp) / "release.json"
            release_json.write_text(
                json.dumps({"tag_name": "v0.5.0", "draft": False, "prerelease": False}), encoding="utf-8"
            )

            with self.assertRaisesRegex(workflow_docs.publish_docs.PublishError, "does not match"):
                workflow_docs.validate_dispatch(
                    workflow_docs.parse_args(
                        ["validate-dispatch", "--release-json", str(release_json)]
                    ),
                    {
                        "VERSION": "0.5.0",
                        "SOURCE_REF": "v0.5.1",
                        "DISPATCH_REF": "main",
                        "DEFAULT_BRANCH": "main",
                        "GITHUB_REPOSITORY": "kleverhq/wavepeek",
                    },
                )

    def test_validate_dispatch_rejects_release_tag_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            release_json = pathlib.Path(temp) / "release.json"
            release_json.write_text(
                json.dumps({"tag_name": "v0.4.0", "draft": False, "prerelease": False}),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(workflow_docs.WorkflowError, "tag"):
                workflow_docs.validate_dispatch(
                    workflow_docs.parse_args(
                        ["validate-dispatch", "--release-json", str(release_json)]
                    ),
                    {
                        "VERSION": "0.5.0",
                        "SOURCE_REF": "v0.5.0",
                        "DISPATCH_REF": "main",
                        "DEFAULT_BRANCH": "main",
                        "GITHUB_REPOSITORY": "kleverhq/wavepeek",
                    },
                )

    def test_validate_dispatch_rejects_draft_or_prerelease(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            release_json = pathlib.Path(temp) / "release.json"
            base_env = {
                "VERSION": "0.5.0",
                "SOURCE_REF": "v0.5.0",
                "DISPATCH_REF": "main",
                "DEFAULT_BRANCH": "main",
                "GITHUB_REPOSITORY": "kleverhq/wavepeek",
            }

            release_json.write_text(
                json.dumps({"tag_name": "v0.5.0", "draft": True, "prerelease": False}), encoding="utf-8"
            )
            with self.assertRaisesRegex(workflow_docs.WorkflowError, "stable"):
                workflow_docs.validate_dispatch(
                    workflow_docs.parse_args(
                        ["validate-dispatch", "--release-json", str(release_json)]
                    ),
                    base_env,
                )

            release_json.write_text(
                json.dumps({"tag_name": "v0.5.0", "draft": False, "prerelease": True}), encoding="utf-8"
            )
            with self.assertRaisesRegex(workflow_docs.WorkflowError, "stable"):
                workflow_docs.validate_dispatch(
                    workflow_docs.parse_args(
                        ["validate-dispatch", "--release-json", str(release_json)]
                    ),
                    base_env,
                )

    def test_workflow_publish_args_translate_env_and_repair_flag(self) -> None:
        env = {
            "VERSION": "0.5.0",
            "SOURCE_REF": "v0.5.0",
            "REPAIR_EXISTING_VERSION": "true",
        }

        self.assertEqual(
            workflow_docs.workflow_publish_args("stage-deploy", env),
            [
                "stage-deploy",
                "--version",
                "0.5.0",
                "--work-dir",
                "tmp/docs-site",
                "--source-ref",
                "v0.5.0",
                "--repair-existing-version",
            ],
        )
        self.assertEqual(
            workflow_docs.workflow_publish_args(
                "push-staged", {"VERSION": "0.5.0", "REPAIR_EXISTING_VERSION": "false"}
            ),
            ["push-staged", "--version", "0.5.0", "--work-dir", "tmp/docs-site"],
        )

    def test_workflow_publish_args_reject_unknown_command(self) -> None:
        with self.assertRaisesRegex(workflow_docs.WorkflowError, "unsupported"):
            workflow_docs.workflow_publish_args("check", {"VERSION": "0.5.0"})

    def test_check_deploy_args_follow_promote_latest_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            temp_path = pathlib.Path(temp)
            metadata_dir = temp_path / "tmp" / "docs-site"
            metadata_dir.mkdir(parents=True)
            metadata_path = metadata_dir / workflow_docs.publish_docs.METADATA_NAME
            metadata_path.write_text(
                json.dumps({
                    "promote_latest": False,
                    "schema_artifacts": ["schema-output-v2.0.json", "schema-stream-v2.0.json"],
                }),
                encoding="utf-8",
            )
            original_cwd = pathlib.Path.cwd()
            try:
                os.chdir(temp_path)
                args = workflow_docs.parse_args(
                    [
                        "check-deploy",
                        "--base-url",
                        "https://kleverhq.github.io/wavepeek",
                        "--repository",
                        "kleverhq/wavepeek",
                    ]
                )
                self.assertEqual(
                    workflow_docs.check_deploy_args(args, {"VERSION": "0.4.0"}),
                    [
                        "--version",
                        "0.4.0",
                        "--base-url",
                        "https://kleverhq.github.io/wavepeek",
                        "--no-expect-latest",
                        "--schema-artifact",
                        "schema-output-v2.0.json",
                        "--stream-schema-artifact",
                        "schema-stream-v2.0.json",
                        "--repository",
                        "kleverhq/wavepeek",
                    ],
                )
                metadata_path.write_text(
                    json.dumps({"promote_latest": True, "schema_artifacts": []}),
                    encoding="utf-8",
                )
                self.assertIn(
                    "--expect-latest",
                    workflow_docs.check_deploy_args(args, {"VERSION": "0.5.0"}),
                )
            finally:
                os.chdir(original_cwd)

    def test_check_deploy_args_require_promote_latest_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            temp_path = pathlib.Path(temp)
            metadata_dir = temp_path / "tmp" / "docs-site"
            metadata_dir.mkdir(parents=True)
            (metadata_dir / workflow_docs.publish_docs.METADATA_NAME).write_text(
                json.dumps({}), encoding="utf-8"
            )
            original_cwd = pathlib.Path.cwd()
            try:
                os.chdir(temp_path)
                args = workflow_docs.parse_args(
                    ["check-deploy", "--base-url", "https://kleverhq.github.io/wavepeek"]
                )
                with self.assertRaisesRegex(workflow_docs.WorkflowError, "promote_latest"):
                    workflow_docs.check_deploy_args(args, {"VERSION": "0.5.0"})
            finally:
                os.chdir(original_cwd)


if __name__ == "__main__":
    unittest.main()
