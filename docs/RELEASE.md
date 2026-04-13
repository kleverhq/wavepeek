# Release Runbook

This runbook covers the production release flow for `wavepeek`.

## Preconditions

- `main` is green in pre-merge CI.
- Local branch is up to date with `main`.
- You have permission to push tags and create releases.
- `CRATES_IO_TOKEN` exists in repository secrets.

## Checklist

1. Choose a new semver version `X.Y.Z` and confirm tag `vX.Y.Z` does not exist.
2. Update `CHANGELOG.md` using [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/):
    - move finalized entries from `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD`;
    - keep a fresh `## [Unreleased]` section for future changes;
    - update bottom links for `Unreleased` and the new version tag.
    - treat existing released sections as immutable historical records; do not
      retroactively rewrite older shipped scope while preparing a new release.
    - if a factual correction is needed after release, record it in
      `## [Unreleased]` or a later release instead of editing the old shipped
      section.
   The GitHub Release body is published from the `CHANGELOG.md` section for
   `X.Y.Z`, so only shipped notes for that version should remain in that section.
3. Reconcile `docs/ROADMAP.md` with actual shipped scope:
   - roadmap is planned scope; `CHANGELOG.md` is factual shipped scope;
   - if a feature planned for a future milestone shipped in `X.Y.Z`, move it to
     the current milestone section in roadmap;
   - remove duplicates from future milestone sections so roadmap reflects
     remaining work only.
4. Update `Cargo.toml` version to `X.Y.Z`.
5. Regenerate canonical schema artifact:

   ```bash
   make update-schema
   ```

6. Run local checks:

   ```bash
   make ci
   ```

7. Commit release prep:

   ```bash
   git add CHANGELOG.md docs/ROADMAP.md Cargo.toml Cargo.lock schema/wavepeek.json
   git commit -m "chore(release): prepare vX.Y.Z"
   ```

8. Push commit and tag:

   ```bash
   git push origin main
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

9. Wait for `.github/workflows/release.yml` to finish.
10. Check workflow logs for:
    - tag/version validation
    - `make ci`
    - `cargo package --locked`
    - release-note extraction from `CHANGELOG.md`
    - `cargo publish --locked`
    - GitHub Release creation with the extracted changelog body
11. Validate schema publication endpoint for the tag (no extra asset upload required):
    - `https://raw.githubusercontent.com/kleverhq/wavepeek/vX.Y.Z/schema/wavepeek.json` resolves to the committed schema artifact.
12. Verify final state:
    - crate published for `X.Y.Z`;
    - GitHub Release created for `vX.Y.Z` with notes matching `CHANGELOG.md` `## [X.Y.Z] - YYYY-MM-DD`;
    - schema published implicitly via the tagged source raw URL.

## Rollback

- If tag was wrong, delete and recreate it after fixes:

  ```bash
  git tag -d vX.Y.Z
  git push origin :refs/tags/vX.Y.Z
  ```

- If a bad crate version was published, publish a new patch version; do not try
  to republish the same version.
