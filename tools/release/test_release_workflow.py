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


if __name__ == "__main__":
    unittest.main()
