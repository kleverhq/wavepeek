# Release Runbook

This runbook covers production releases for `wavepeek`. Use it with `changelog.md`, `quality.md`, and the release workflow in `.github/workflows/release.yml`.

## Preconditions

- `main` is green in pre-merge CI.
- The local branch is up to date with `main`.
- You have permission to push tags and create GitHub Releases.
- `CRATES_IO_TOKEN` exists in repository secrets.
- GitHub Pages is configured to publish from the `gh-pages` branch root.

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

9. Wait for `.github/workflows/release.yml` to finish. It dispatches docs publication only after the GitHub Release is created.
10. Wait for `.github/workflows/docs.yml` to finish for the same version.
11. Check workflow logs for tag/version validation, `just ci`, `cargo package --locked`, release-note extraction from `CHANGELOG.md`, `cargo publish --locked`, GitHub Release creation, docs staging, staged bundle upload, staged bundle verification, and non-forced `gh-pages` push.
12. Validate the schema publication endpoint on `main`: `https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_vX.json` should resolve to the committed schema artifact.
13. Verify final state: the crate is published for `X.Y.Z`, the GitHub Release exists for `vX.Y.Z`, release notes match the changelog section, `https://kleverhq.github.io/wavepeek/X.Y.Z/` resolves, `https://kleverhq.github.io/wavepeek/latest/` points at the same release, and root artifacts such as `skill.md` and `wavepeek_vX.json` resolve from Pages.

The release workflow extracts notes through the helper group owned by `tools/release/`. The stable release interface remains the workflow and the changelog section, not a hand-run release-note command. Docs publication uses `tools/docs/publish_docs.py` from the trusted branch; for a manual repair or first-time bootstrap, run `.github/workflows/docs.yml` with `version`, `source_ref`, and `repair_existing_version` only when intentionally replacing an existing Pages snapshot.

## Rollback

If the tag was wrong, delete and recreate it after fixes:

    git tag -d vX.Y.Z
    git push origin :refs/tags/vX.Y.Z

If a bad crate version was published, publish a new patch version. Do not try to republish the same version; crates.io is not a time machine, despite everyone occasionally needing one.
