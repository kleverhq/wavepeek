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
3. Reconcile GitHub Milestones with actual shipped scope. Milestones are planned scope; changelog is factual shipped scope. For the `X.Y.Z` milestone, close shipped issues, move unfinished issues to a future milestone or back to the backlog, then close the completed milestone.
4. Update `Cargo.toml` version to `X.Y.Z`. If this is the first release for minor line `X.Y`, create `schema/wavepeek_vX.Y.json` and `schema/wavepeek-stream-vX.Y.json` before running build-backed schema commands; `wavepeek schema` embeds the checked-in artifact for the package major/minor contract.
5. Refresh the current schema artifacts:

       just update-schema

6. Run the release-quality gate:

       just ci

7. Commit release prep:

       git add CHANGELOG.md Cargo.toml Cargo.lock schema/wavepeek_vX.Y.json schema/wavepeek-stream-vX.Y.json
       git commit -m "chore(release): prepare vX.Y.Z"

8. Run the manual performance gate before pushing a major or minor release. Use the previous release tag as the baseline and the release-prep commit as the revised ref:

       just bench-gate vPREVIOUS HEAD

   For patch releases, either run the gate when the patch may affect performance, or record the skip rationale in the release issue or checklist for clearly non-performance changes such as documentation-only or release-metadata-only updates. The gate writes ignored artifacts under `tmp/bench-gate/`; preserve the summary path or archive externally if it is needed for release review.

9. Push the commit and tag:

       git push origin main
       git tag vX.Y.Z
       git push origin vX.Y.Z

10. Wait for `.github/workflows/release.yml` to finish. It creates a draft GitHub Release, uploads `cargo-dist` archives, shell and PowerShell installers, checksum artifacts, and `dist-manifest.json`, publishes the release, creates GitHub Artifact Attestations, then dispatches downstream workflows.
11. Wait for `.github/workflows/docs.yml` and `.github/workflows/publish-crate.yml` to finish for the same version.
12. Check workflow logs for tag/version validation, `just ci`, `cargo package --locked`, `cargo-dist` matrix builds, release-body rendering from `CHANGELOG.md`, GitHub Release creation, docs staging, staged bundle upload, staged bundle verification, non-forced `gh-pages` push, Pages artifact upload, `actions/deploy-pages` deployment, deployed docs verification, and idempotent `cargo publish --locked` from the release tag checkout.
13. If deployed docs endpoint verification needs to be repeated locally, run `just docs-site-check-deploy X.Y.Z`. Set `DOCS_REPOSITORY=kleverhq/wavepeek` to also check GitHub Pages API state with an authenticated `gh` CLI.
14. Verify final state: the crate is published for `X.Y.Z`, the GitHub Release exists for `vX.Y.Z`, release notes match the changelog section, binary archives and checksum files are attached, `https://kleverhq.github.io/wavepeek/X.Y.Z/` resolves, `https://kleverhq.github.io/wavepeek/latest/` points at the same release, and root schema and installer aliases resolve from Pages.

The release workflow renders notes through the helper group owned by `tools/release/`. The stable release interface remains the workflow and the changelog section, not a hand-run release-note command. Docs publication uses `tools/docs/publish_docs.py` from the trusted branch and copies installer assets from the created GitHub Release.

The docs workflow keeps `gh-pages` as the versioned `mike` state branch, but the public Pages deployment is performed through GitHub Pages Actions (`actions/upload-pages-artifact` and `actions/deploy-pages`). It does not rely on a branch-push-triggered Pages build.

Normal releases do not need local downstream dispatch commands because `.github/workflows/release.yml` dispatches `.github/workflows/docs.yml` and `.github/workflows/publish-crate.yml` on the default branch after the GitHub Release is created. For manual docs repair, first-time bootstrap, or troubleshooting, dispatch the remote docs workflow explicitly from an up-to-date trusted branch:

    just docs-site-dispatch X.Y.Z vX.Y.Z false

Pass `true` as the repair argument only when intentionally replacing an existing Pages snapshot. If the requested version is older than the current `latest` version, or if a repaired version does not currently own the `latest` alias, the docs workflow stages that version without moving `latest`, root installer aliases, or root schema aliases backward. This command requires `gh` authentication and starts a remote GitHub Actions run; it is not a local dry-run check.

For manual crates.io repair, dispatch `.github/workflows/publish-crate.yml` on the default branch with `version=X.Y.Z` and `source_ref=vX.Y.Z`. The workflow checks out trusted tooling from the default branch, checks out release source through `refs/tags/vX.Y.Z`, and exits successfully without requiring `CRATES_IO_TOKEN` if that crate version is already published.

## Rollback

If the tag was wrong before a GitHub Release was published, delete and recreate it after fixes:

    git tag -d vX.Y.Z
    git push origin :refs/tags/vX.Y.Z

If an immutable GitHub Release was published, do not delete it to retry the same tag. A deleted immutable release leaves the tag name unavailable for new releases; publish a new patch version instead.

If a bad crate version was published, publish a new patch version. crates.io versions are immutable and cannot be republished.
