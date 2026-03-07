#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import subprocess
import tempfile
import textwrap
import unittest


SCRIPT_PATH = pathlib.Path(__file__).with_name("extract_release_notes.py")


class ExtractReleaseNotesCliTest(unittest.TestCase):
    def run_script(self, changelog_text: str, version: str) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            changelog_path = pathlib.Path(temp_dir) / "CHANGELOG.md"
            changelog_path.write_text(changelog_text, encoding="utf-8")
            return subprocess.run(
                [
                    "python3",
                    str(SCRIPT_PATH),
                    "--changelog",
                    str(changelog_path),
                    "--version",
                    version,
                ],
                check=False,
                capture_output=True,
                text=True,
            )

    def test_extracts_matching_version_section_body_only(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                # Changelog

                ## [Unreleased]

                ### Added
                - Future work.

                ## [1.2.3] - 2026-03-07

                ### Added
                - First shipped item.

                ### Fixed
                - Second shipped item.

                ## [1.2.2] - 2026-03-01

                ### Fixed
                - Older item.
                """
            ),
            "1.2.3",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            result.stdout,
            textwrap.dedent(
                """\
                ### Added
                - First shipped item.

                ### Fixed
                - Second shipped item.
                """
            ),
        )

    def test_fails_when_version_heading_is_missing(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                # Changelog

                ## [Unreleased]

                ### Added
                - Future work.
                """
            ),
            "9.9.9",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("error: release-notes: version 9.9.9 not found", result.stderr)

    def test_fails_when_version_section_has_no_release_notes(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                # Changelog

                ## [1.2.3] - 2026-03-07

                ## [1.2.2] - 2026-03-01

                ### Fixed
                - Older item.
                """
            ),
            "1.2.3",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("error: release-notes: version 1.2.3 section is empty", result.stderr)

    def test_ignores_changelog_reference_links_after_last_release_section(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                # Changelog

                ## [1.2.3] - 2026-03-07

                ### Fixed
                - Last shipped item.

                [Unreleased]: https://example.com/compare/v1.2.3...HEAD
                [1.2.3]: https://example.com/releases/tag/v1.2.3
                """
            ),
            "1.2.3",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            result.stdout,
            textwrap.dedent(
                """\
                ### Fixed
                - Last shipped item.
                """
            ),
        )


if __name__ == "__main__":
    unittest.main()
