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

    def test_github_release_uses_draft_first_asset_upload(self) -> None:
        text = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        create = 'gh release create "${{ needs.plan.outputs.tag }}"'
        upload = 'gh release upload "${{ needs.plan.outputs.tag }}" --clobber artifacts/*'
        publish = 'gh release edit "${{ needs.plan.outputs.tag }}" --draft=false'
        self.assertIn("name: Create draft GitHub Release", text)
        self.assertIn(create, text)
        self.assertIn("--draft", text)
        self.assertIn(upload, text)
        self.assertIn(publish, text)
        self.assertLess(text.index(create), text.index(upload))
        self.assertLess(text.index(upload), text.index(publish))

    def test_downstream_dispatch_is_explicitly_scoped_to_repository(self) -> None:
        text = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("GH_REPO: ${{ github.repository }}", text)
        self.assertIn('gh workflow run docs.yml --repo "$GH_REPO"', text)
        self.assertIn('gh workflow run publish-crate.yml --repo "$GH_REPO"', text)


if __name__ == "__main__":
    unittest.main()
