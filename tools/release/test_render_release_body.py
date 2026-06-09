#!/usr/bin/env python3

from __future__ import annotations

import json
import pathlib
import subprocess
import tempfile
import textwrap
import unittest

SCRIPT_PATH = pathlib.Path(__file__).with_name("render_release_body.py")


class RenderReleaseBodyCliTest(unittest.TestCase):
    def changelog(self) -> str:
        return textwrap.dedent(
            """\
            # Changelog

            ## [1.2.3] - 2026-06-09

            ### Added
            - Binary release assets.

            ## [1.2.2] - 2026-05-01

            ### Fixed
            - Older work.
            """
        )

    def manifest(self) -> dict[str, object]:
        artifacts: dict[str, object] = {
            "sha256.sum": {"name": "sha256.sum", "kind": "unified-checksum"},
            "source.tar.gz": {
                "name": "source.tar.gz",
                "kind": "source-tarball",
                "checksum": "source.tar.gz.sha256",
            },
            "source.tar.gz.sha256": {"name": "source.tar.gz.sha256", "kind": "checksum"},
            "wavepeek-installer.sh": {"name": "wavepeek-installer.sh", "kind": "installer"},
            "wavepeek-installer.ps1": {"name": "wavepeek-installer.ps1", "kind": "installer"},
        }
        for target, archive in {
            "x86_64-unknown-linux-gnu": "wavepeek-x86_64-unknown-linux-gnu.tar.gz",
            "aarch64-unknown-linux-gnu": "wavepeek-aarch64-unknown-linux-gnu.tar.gz",
            "x86_64-apple-darwin": "wavepeek-x86_64-apple-darwin.tar.gz",
            "aarch64-apple-darwin": "wavepeek-aarch64-apple-darwin.tar.gz",
            "x86_64-pc-windows-msvc": "wavepeek-x86_64-pc-windows-msvc.zip",
        }.items():
            artifacts[archive] = {
                "name": archive,
                "kind": "executable-zip",
                "target_triples": [target],
                "checksum": f"{archive}.sha256",
            }
            artifacts[f"{archive}.sha256"] = {
                "name": f"{archive}.sha256",
                "kind": "checksum",
                "target_triples": [target],
            }
        return {"artifacts": artifacts}

    def run_script(
        self,
        *,
        manifest: dict[str, object] | None = None,
        version: str = "1.2.3",
        repository: str = "kleverhq/wavepeek",
        crate_published: str = "unknown",
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            changelog_path = temp_path / "CHANGELOG.md"
            changelog_path.write_text(self.changelog(), encoding="utf-8")
            args = [
                "python3",
                str(SCRIPT_PATH),
                "--changelog",
                str(changelog_path),
                "--version",
                version,
                "--repository",
                repository,
                "--crate-published",
                crate_published,
            ]
            if manifest is not None:
                manifest_path = temp_path / "dist-manifest.json"
                manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
                args.extend(["--dist-manifest", str(manifest_path)])
            return subprocess.run(args, check=False, capture_output=True, text=True)

    def test_renders_release_notes_installers_downloads_and_verification(self) -> None:
        result = self.run_script(manifest=self.manifest(), crate_published="true")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("## Release Notes", result.stdout)
        self.assertIn("- Binary release assets.", result.stdout)
        self.assertIn(
            "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/kleverhq/wavepeek/releases/download/v1.2.3/wavepeek-installer.sh | sh",
            result.stdout,
        )
        self.assertIn(
            'powershell -ExecutionPolicy Bypass -c "irm https://github.com/kleverhq/wavepeek/releases/download/v1.2.3/wavepeek-installer.ps1 | iex"',
            result.stdout,
        )
        self.assertIn("| Linux x86_64 | [`wavepeek-x86_64-unknown-linux-gnu.tar.gz`]", result.stdout)
        self.assertIn("| Linux arm64 | [`wavepeek-aarch64-unknown-linux-gnu.tar.gz`]", result.stdout)
        self.assertIn("| macOS Intel | [`wavepeek-x86_64-apple-darwin.tar.gz`]", result.stdout)
        self.assertIn("| macOS Apple Silicon | [`wavepeek-aarch64-apple-darwin.tar.gz`]", result.stdout)
        self.assertIn("| Windows x86_64 | [`wavepeek-x86_64-pc-windows-msvc.zip`]", result.stdout)
        self.assertIn("sha256sum --check sha256.sum --ignore-missing", result.stdout)
        self.assertIn(
            "gh attestation verify wavepeek-x86_64-unknown-linux-gnu.tar.gz --repo kleverhq/wavepeek",
            result.stdout,
        )
        self.assertIn("Prebuilt binaries are VCD/FST-only", result.stdout)
        self.assertIn("cargo install --locked wavepeek --version 1.2.3 --features fsdb", result.stdout)
        self.assertIn("https://kleverhq.github.io/wavepeek/1.2.3/", result.stdout)
        self.assertIn("https://crates.io/crates/wavepeek/1.2.3", result.stdout)

    def test_renders_static_artifacts_without_manifest(self) -> None:
        result = self.run_script()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("wavepeek-aarch64-apple-darwin.tar.gz", result.stdout)
        self.assertIn("wavepeek-x86_64-pc-windows-msvc.zip", result.stdout)

    def test_fails_when_manifest_misses_required_target(self) -> None:
        manifest = self.manifest()
        assert isinstance(manifest["artifacts"], dict)
        del manifest["artifacts"]["wavepeek-x86_64-pc-windows-msvc.zip"]
        result = self.run_script(manifest=manifest)

        self.assertEqual(result.returncode, 1)
        self.assertIn("missing executable archive(s) for: x86_64-pc-windows-msvc", result.stderr)

    def test_fails_for_prerelease_version(self) -> None:
        result = self.run_script(version="1.2.3-rc.1")

        self.assertEqual(result.returncode, 1)
        self.assertIn("version must be stable SemVer X.Y.Z", result.stderr)

    def test_fails_for_invalid_repository(self) -> None:
        result = self.run_script(repository="not a repo")

        self.assertEqual(result.returncode, 1)
        self.assertIn("repository must have owner/name form", result.stderr)


if __name__ == "__main__":
    unittest.main()
