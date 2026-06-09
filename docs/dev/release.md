# Release Runbook

This runbook covers production releases for `wavepeek`. Use it with `changelog.md`, `quality.md`, and the release workflow in `.github/workflows/release.yml`.

## Preconditions

- `main` is green in pre-merge CI.
- The local branch is up to date with `main`.
- You have permission to push tags and create GitHub Releases.
- `CRATES_IO_TOKEN` exists in repository secrets for downstream crates.io publication.
- GitHub Pages is configured for GitHub Actions deployments.
- The release tag is stable SemVer only: `vX.Y.Z`. Prerelease and build-metadata tags are rejected.

## Checklist

1. Choose a new semver version `X.Y.Z` and confirm tag `vX.Y.Z` does not exist.
2. Update `CHANGELOG.md` using Keep a Changelog:
   - move finalized entries from `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD`;
   - keep a fresh `## [Unreleased]` section for future changes;
   - update bottom links for `Unreleased` and the new version tag;
   - treat released sections as immutable historical records.
3. Reconcile `docs/tracker/roadmap.md` with actual shipped scope. Roadmap is planned scope; changelog is factual shipped scope. If planned work shipped in `X.Y.Z`, remove it from future roadmap notes so remaining work stays visible.
4. Update `Cargo.toml` version to `X.Y.Z`. If this is the first release for major version `X`, create `schema/wavepeek_vX.json` before running build-backed schema commands; `wavepeek schema` embeds the checked-in artifact for the package major version.
5. Refresh the current major schema artifact:

       just update-schema

6. Run the release-quality gate:

       just ci

7. Commit release prep:

       git add CHANGELOG.md docs/tracker/roadmap.md Cargo.toml Cargo.lock schema/wavepeek_vX.json
       git commit -m "chore(release): prepare vX.Y.Z"

8. Push the commit and tag:

       git push origin main
       git tag vX.Y.Z
       git push origin vX.Y.Z

9. Wait for `.github/workflows/release.yml` to finish. It creates the GitHub Release with `cargo-dist` archives, shell and PowerShell installers, checksum artifacts, `dist-manifest.json`, and GitHub Artifact Attestations before dispatching downstream workflows.
10. Wait for `.github/workflows/docs.yml` and `.github/workflows/publish-crate.yml` to finish for the same version.
11. Check workflow logs for tag/version validation, `just ci`, `cargo package --locked`, `cargo-dist` matrix builds, release-body rendering from `CHANGELOG.md`, GitHub Release creation, docs staging, staged bundle upload, staged bundle verification, non-forced `gh-pages` push, Pages artifact upload, `actions/deploy-pages` deployment, and idempotent `cargo publish --locked` from the release tag checkout.
12. Validate Pages endpoints: `https://kleverhq.github.io/wavepeek/wavepeek_vX.json`, `https://kleverhq.github.io/wavepeek/install.sh`, and `https://kleverhq.github.io/wavepeek/install.ps1` should resolve to the staged root artifacts.
13. Verify final state: the crate is published for `X.Y.Z`, the GitHub Release exists for `vX.Y.Z`, release notes match the changelog section, binary archives and checksum files are attached, `https://kleverhq.github.io/wavepeek/X.Y.Z/` resolves, `https://kleverhq.github.io/wavepeek/latest/` points at the same release, and root schema and installer aliases resolve from Pages.

The release workflow renders notes through the helper group owned by `tools/release/`. The stable release interface remains the workflow and the changelog section, not a hand-run release-note command. Docs publication uses `tools/docs/publish_docs.py` from the trusted branch and copies installer assets from the created GitHub Release.

The docs workflow keeps `gh-pages` as the versioned `mike` state branch, but the public Pages deployment is performed through GitHub Pages Actions (`actions/upload-pages-artifact` and `actions/deploy-pages`). It does not rely on a branch-push-triggered Pages build.

Normal releases do not need local downstream dispatch commands because `.github/workflows/release.yml` dispatches `.github/workflows/docs.yml` and `.github/workflows/publish-crate.yml` on the default branch after the GitHub Release is created. For manual docs repair, first-time bootstrap, or troubleshooting, dispatch the remote docs workflow explicitly from an up-to-date trusted branch:

    just docs-site-dispatch version=X.Y.Z source_ref=vX.Y.Z repair=false

Use `repair=true` only when intentionally replacing an existing Pages snapshot. If the requested version is older than the current `latest` version, or if a repaired version does not currently own the `latest` alias, the docs workflow stages that version without moving `latest`, root installer aliases, or root schema aliases backward. This command requires `gh` authentication and starts a remote GitHub Actions run; it is not a local dry-run check.

For manual crates.io repair, dispatch `.github/workflows/publish-crate.yml` on the default branch with `version=X.Y.Z` and `source_ref=vX.Y.Z`. The workflow checks out trusted tooling from the default branch, checks out release source through `refs/tags/vX.Y.Z`, and exits successfully without requiring `CRATES_IO_TOKEN` if that crate version is already published.

## Rollback

If the tag was wrong, delete and recreate it after fixes:

    git tag -d vX.Y.Z
    git push origin :refs/tags/vX.Y.Z

If a bad crate version was published, publish a new patch version. crates.io versions are immutable and cannot be republished.
