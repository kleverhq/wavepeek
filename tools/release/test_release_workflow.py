#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import unittest

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
RELEASE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release.yml"


class ReleaseWorkflowContractTest(unittest.TestCase):
    def test_release_workflow_uses_tag_version_output_as_raw_version(self) -> None:
        text = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("version: ${{ steps.validate-tag.outputs.tag_version }}", text)
        self.assertIn("python3 -B tools/release/validate_tag_version.py >> \"$GITHUB_OUTPUT\"", text)
        self.assertNotIn("steps.validate-tag.outputs.version", text)

    def test_plan_job_uses_python_with_tomllib(self) -> None:
        text = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("  plan:\n    runs-on: ubuntu-24.04\n", text)

    def test_global_and_manifest_assets_are_attested(self) -> None:
        text = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("name: Attest global artifacts", text)
        self.assertIn("target/distrib/wavepeek-installer.sh", text)
        self.assertIn("target/distrib/wavepeek-installer.ps1", text)
        self.assertIn("target/distrib/sha256.sum", text)
        self.assertIn("name: Attest dist manifest", text)
        self.assertIn("subject-path: dist-manifest.json", text)


if __name__ == "__main__":
    unittest.main()
