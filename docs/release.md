# Release Runbook

Release from green `main` with permission to push tags and create GitHub Releases. GitHub Pages must use Actions deployments. The publication workflow needs the repository's `CRATES_IO_TOKEN` for a new crate version.

## Prepare and publish

1. Choose an unused stable SemVer tag `vX.Y.Z`. Set the matching version in `Cargo.toml`.
2. Move shipped changes from `Unreleased` in `CHANGELOG.md` into a dated version section. Use Keep a Changelog headings. Keep an empty `Unreleased` section and update comparison links. GitHub Release notes come from the matching section. Keep published sections intact and record corrections in a later section. Add user-visible changes during normal development.
3. Reconcile the GitHub Milestone with shipped scope. Close shipped issues and move unfinished work before closing the milestone.
4. Run `./dev just ci` and commit `CHANGELOG.md`, `Cargo.toml`, and `Cargo.lock`. For a major or minor release, run `./dev just bench-gate vPREVIOUS HEAD` from the committed tree. For a patch, run it if performance can change. Record the reason in the release checklist if you skip the gate or explicitly disable FSDB capture.
5. Push the release commit to `main` and push the tag for that commit. Wait for the Release, Publish Docs, and Publish Crate workflows to succeed for the same version.
6. Verify the release notes and assets on GitHub. Verify the version on crates.io. Check Pages with `./dev just docs-site-check-deploy X.Y.Z`.

## Publication and repair

The Release workflow dispatches docs and crate publication after the GitHub Release exists. The downstream workflows use trusted tooling from the default branch and source from the release tag. For manual repair, dispatch `.github/workflows/docs.yml` or `.github/workflows/publish-crate.yml` on the default branch with `version=X.Y.Z` and `source_ref=vX.Y.Z`.

For docs, set `repair_existing_version=true` only to replace an existing version snapshot. Historical repairs keep `latest`, the root Playground, and root installer aliases on the current release. The `gh-pages` branch holds the versioned state. A Pages Actions deployment publishes that state. The crate workflow treats an already published version as a successful run.

Before publication, correct a wrong tag and rerun the release from the corrected commit. After an immutable GitHub Release or a crate version is published, issue a new patch version to correct it. Deleting an immutable release does not free its tag.
