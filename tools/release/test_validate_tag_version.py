#!/usr/bin/env python3

from __future__ import annotations

import os
import pathlib
import subprocess
import tempfile
import textwrap
import unittest


SCRIPT_PATH = pathlib.Path(__file__).with_name("validate_tag_version.py")


class ValidateTagVersionCliTest(unittest.TestCase):
    def run_script(
        self, manifest_text: str, tag: str | None
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            manifest_path = pathlib.Path(temp_dir) / "Cargo.toml"
            manifest_path.write_text(manifest_text, encoding="utf-8")
            args = [
                "python3",
                str(SCRIPT_PATH),
                "--manifest",
                str(manifest_path),
            ]
            if tag is not None:
                args.extend(["--tag", tag])
            env = os.environ.copy()
            env.pop("GITHUB_REF_NAME", None)
            return subprocess.run(
                args,
                check=False,
                capture_output=True,
                text=True,
                env=env,
            )

    def test_accepts_matching_v_tag(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3"
                """
            ),
            "v1.2.3",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "tag_version=1.2.3\n")

    def test_fails_when_tag_has_no_v_prefix(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3"
                """
            ),
            "1.2.3",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("expected stable SemVer tag vX.Y.Z", result.stderr)

    def test_fails_when_tag_is_prerelease(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3"
                """
            ),
            "v1.2.3-rc.1",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("expected stable SemVer tag vX.Y.Z", result.stderr)

    def test_fails_when_tag_has_build_metadata(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3"
                """
            ),
            "v1.2.3+build.1",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("expected stable SemVer tag vX.Y.Z", result.stderr)

    def test_fails_when_tag_has_leading_zero_component(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "01.2.3"
                """
            ),
            "v01.2.3",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("expected stable SemVer tag vX.Y.Z", result.stderr)

    def test_fails_when_manifest_version_is_not_stable(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3-rc.1"
                """
            ),
            "v1.2.3",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("Cargo.toml package.version must be stable SemVer", result.stderr)

    def test_fails_when_tag_version_differs_from_manifest(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3"
                """
            ),
            "v1.2.4",
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn(
            "tag version (1.2.4) does not match Cargo.toml version (1.2.3)",
            result.stderr,
        )

    def test_fails_without_tag(self) -> None:
        result = self.run_script(
            textwrap.dedent(
                """\
                [package]
                name = "wavepeek"
                version = "1.2.3"
                """
            ),
            None,
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("tag is required", result.stderr)


if __name__ == "__main__":
    unittest.main()
