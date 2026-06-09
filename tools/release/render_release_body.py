#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from dataclasses import dataclass
from typing import Any, Sequence

from extract_release_notes import extract_release_notes

VERSION_RE = re.compile(r"^(0|[1-9][0-9]*)[.](0|[1-9][0-9]*)[.](0|[1-9][0-9]*)$")
REPOSITORY_RE = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")

TARGET_LABELS = {
    "x86_64-unknown-linux-gnu": "Linux x86_64",
    "aarch64-unknown-linux-gnu": "Linux arm64",
    "x86_64-apple-darwin": "macOS Intel",
    "aarch64-apple-darwin": "macOS Apple Silicon",
    "x86_64-pc-windows-msvc": "Windows x86_64",
}
TARGET_ORDER = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
STATIC_ARCHIVES = {
    "x86_64-unknown-linux-gnu": "wavepeek-x86_64-unknown-linux-gnu.tar.gz",
    "aarch64-unknown-linux-gnu": "wavepeek-aarch64-unknown-linux-gnu.tar.gz",
    "x86_64-apple-darwin": "wavepeek-x86_64-apple-darwin.tar.gz",
    "aarch64-apple-darwin": "wavepeek-aarch64-apple-darwin.tar.gz",
    "x86_64-pc-windows-msvc": "wavepeek-x86_64-pc-windows-msvc.zip",
}


@dataclass(frozen=True)
class DownloadArtifact:
    target: str
    platform: str
    archive: str
    checksum: str | None


def fail(message: str) -> None:
    print(f"error: release-body: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_version(version: str) -> str:
    if VERSION_RE.fullmatch(version) is None:
        fail(f"version must be stable SemVer X.Y.Z, got {version!r}")
    return version


def validate_repository(repository: str) -> str:
    if REPOSITORY_RE.fullmatch(repository) is None:
        fail(f"repository must have owner/name form, got {repository!r}")
    return repository


def load_json(path: pathlib.Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        fail(f"failed to read dist manifest {path}: {error}")
    except json.JSONDecodeError as error:
        fail(f"dist manifest {path} is not valid JSON: {error}")
    if not isinstance(data, dict):
        fail(f"dist manifest {path} must contain a JSON object")
    return data


def artifact_target(artifact: dict[str, Any]) -> str | None:
    targets = artifact.get("target_triples")
    if not isinstance(targets, list) or len(targets) != 1:
        return None
    target = targets[0]
    return target if isinstance(target, str) else None


def artifacts_from_manifest(manifest: dict[str, Any]) -> list[DownloadArtifact]:
    raw_artifacts = manifest.get("artifacts")
    if not isinstance(raw_artifacts, dict):
        fail("dist manifest is missing an artifacts object")

    by_target: dict[str, DownloadArtifact] = {}
    for name, raw in raw_artifacts.items():
        if not isinstance(name, str) or not isinstance(raw, dict):
            fail("dist manifest artifacts must be an object of artifact objects")
        if raw.get("kind") != "executable-zip":
            continue
        target = artifact_target(raw)
        if target not in TARGET_LABELS:
            continue
        archive = raw.get("name", name)
        checksum = raw.get("checksum")
        if not isinstance(archive, str) or not archive:
            fail(f"dist manifest artifact for {target} has invalid name")
        if checksum is not None and not isinstance(checksum, str):
            fail(f"dist manifest artifact {archive} has invalid checksum")
        by_target[target] = DownloadArtifact(
            target=target,
            platform=TARGET_LABELS[target],
            archive=archive,
            checksum=checksum,
        )

    missing = [target for target in TARGET_ORDER if target not in by_target]
    if missing:
        fail("dist manifest is missing executable archive(s) for: " + ", ".join(missing))
    return [by_target[target] for target in TARGET_ORDER]


def static_artifacts() -> list[DownloadArtifact]:
    return [
        DownloadArtifact(
            target=target,
            platform=TARGET_LABELS[target],
            archive=STATIC_ARCHIVES[target],
            checksum=f"{STATIC_ARCHIVES[target]}.sha256",
        )
        for target in TARGET_ORDER
    ]


def checksum_assets(manifest: dict[str, Any] | None) -> tuple[bool, list[str]]:
    if manifest is None:
        return True, [f"{archive}.sha256" for archive in STATIC_ARCHIVES.values()]
    raw_artifacts = manifest.get("artifacts")
    if not isinstance(raw_artifacts, dict):
        fail("dist manifest is missing an artifacts object")
    has_unified = "sha256.sum" in raw_artifacts
    per_artifact = sorted(
        name
        for name, raw in raw_artifacts.items()
        if isinstance(name, str)
        and name.endswith(".sha256")
        and isinstance(raw, dict)
        and raw.get("kind") == "checksum"
    )
    if not has_unified and not per_artifact:
        fail("dist manifest does not list checksum artifacts")
    return has_unified, per_artifact


def pages_base(repository: str) -> str:
    owner, name = repository.split("/", maxsplit=1)
    return f"https://{owner.lower()}.github.io/{name}/"


def crate_status_text(crate_published: str, version: str) -> str:
    if crate_published == "true":
        return f"The crate is published at https://crates.io/crates/wavepeek/{version}."
    if crate_published == "false":
        return "The crates.io package has not been published yet; retry the downstream crate workflow."
    return "Crates.io publication is handled by a downstream workflow; check https://crates.io/crates/wavepeek for current status."


def release_asset_url(repository: str, version: str, asset: str) -> str:
    return f"https://github.com/{repository}/releases/download/v{version}/{asset}"


def render_download_table(artifacts: Sequence[DownloadArtifact], repository: str, version: str) -> list[str]:
    lines = ["| Platform | Archive | Checksum |", "| --- | --- | --- |"]
    for artifact in artifacts:
        archive_url = release_asset_url(repository, version, artifact.archive)
        checksum = artifact.checksum or "See `sha256.sum`"
        checksum_url = release_asset_url(repository, version, checksum) if checksum != "See `sha256.sum`" else checksum
        lines.append(
            f"| {artifact.platform} | [`{artifact.archive}`]({archive_url}) | "
            f"[`{checksum}`]({checksum_url}) |"
        )
    return lines


def render_release_body(
    *,
    version: str,
    repository: str,
    changelog_text: str,
    manifest: dict[str, Any] | None,
    crate_published: str,
) -> str:
    version = validate_version(version)
    repository = validate_repository(repository)
    notes = extract_release_notes(changelog_text, version).strip()
    artifacts = artifacts_from_manifest(manifest) if manifest is not None else static_artifacts()
    has_unified_checksum, per_artifact_checksums = checksum_assets(manifest)
    docs_root = pages_base(repository)
    docs_version = f"{docs_root}{version}/"

    shell_url = release_asset_url(repository, version, "wavepeek-installer.sh")
    powershell_url = release_asset_url(repository, version, "wavepeek-installer.ps1")
    sample_archive = artifacts[0].archive

    lines: list[str] = []
    lines.extend(["## Release Notes", "", notes, ""])
    lines.extend(
        [
            f"## Install wavepeek {version}",
            "",
            "macOS and Linux:",
            "",
            "    curl --proto '=https' --tlsv1.2 -LsSf " + shell_url + " | sh",
            "",
            "Windows PowerShell:",
            "",
            "    powershell -ExecutionPolicy Bypass -c \"irm " + powershell_url + " | iex\"",
            "",
            "The installer places `wavepeek` under the configured XDG-style user binary path and does not require a Rust toolchain.",
            "",
            f"## Download wavepeek {version}",
            "",
        ]
    )
    lines.extend(render_download_table(artifacts, repository, version))
    lines.extend(
        [
            "",
            "Additional release assets include `wavepeek-installer.sh`, `wavepeek-installer.ps1`, `dist-manifest.json`, `source.tar.gz`, and checksum files generated by `cargo-dist`.",
            "",
            "## Verify checksums",
            "",
        ]
    )
    if has_unified_checksum:
        lines.extend(
            [
                "Download `sha256.sum` and the artifact you want, then run:",
                "",
                "    sha256sum --check sha256.sum --ignore-missing",
                "",
            ]
        )
    if per_artifact_checksums:
        lines.extend(
            [
                "Per-artifact checksum files are also available. For example:",
                "",
                f"    sha256sum --check {sample_archive}.sha256",
                "",
            ]
        )
    lines.extend(
        [
            "## Verify GitHub Artifact Attestations",
            "",
            "Download an artifact and verify its GitHub attestation with:",
            "",
            f"    gh attestation verify {sample_archive} --repo {repository}",
            "",
            "## FSDB support",
            "",
            "Prebuilt binaries are VCD/FST-only. FSDB support remains source-only and requires a Linux x86_64 host with a valid Synopsys Verdi / FSDB Reader SDK environment:",
            "",
            f"    cargo install --locked wavepeek --version {version} --features fsdb",
            "",
            "## Cargo fallback",
            "",
            "If a prebuilt artifact is not suitable for your platform, install from crates.io with Rust:",
            "",
            f"    cargo install --locked wavepeek --version {version}",
            "",
            "## Documentation",
            "",
            f"Versioned documentation: {docs_version}",
            "",
            f"Latest documentation and installer aliases: {docs_root}",
            "",
            "## crates.io",
            "",
            crate_status_text(crate_published, version),
            "",
        ]
    )
    return "\n".join(lines)


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Render the GitHub Release body for wavepeek.")
    parser.add_argument("--version", required=True, help="Release version without leading v.")
    parser.add_argument("--repository", required=True, help="GitHub repository owner/name.")
    parser.add_argument("--changelog", type=pathlib.Path, default=pathlib.Path("CHANGELOG.md"))
    parser.add_argument("--dist-manifest", type=pathlib.Path)
    parser.add_argument(
        "--crate-published",
        choices=("true", "false", "unknown"),
        default="unknown",
        help="crates.io publication status to render.",
    )
    return parser.parse_args(list(argv))


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        try:
            changelog_text = args.changelog.read_text(encoding="utf-8")
        except OSError as error:
            fail(f"failed to read changelog {args.changelog}: {error}")
        manifest = load_json(args.dist_manifest) if args.dist_manifest else None
        sys.stdout.write(
            render_release_body(
                version=args.version,
                repository=args.repository,
                changelog_text=changelog_text,
                manifest=manifest,
                crate_published=args.crate_published,
            )
        )
    except SystemExit as error:
        return int(error.code) if isinstance(error.code, int) else 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
