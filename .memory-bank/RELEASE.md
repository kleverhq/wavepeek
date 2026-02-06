# Release Runbook

This runbook covers both dry-run and real release modes for `wavepeek`.

## Preconditions

- `master` is green in pre-merge CI.
- Local branch is up to date with `master`.
- You have permission to push tags and manage repository variables.
- For real releases: `CRATES_IO_TOKEN` exists in repository secrets.

## Release Mode

- Dry-run: set repository variable `RELEASE_DRY_RUN=1`.
- Real release: unset `RELEASE_DRY_RUN` or set `RELEASE_DRY_RUN=0`.

The release workflow always runs checks and packaging. In dry-run mode it skips
crate publish and GitHub Release creation.

## Checklist

1. Choose a new semver version `X.Y.Z` and confirm tag `vX.Y.Z` does not exist.
2. Update `Cargo.toml` version to `X.Y.Z`.
3. Run local checks:

   ```bash
   make ci
   ```

4. Commit release prep:

   ```bash
   git add Cargo.toml Cargo.lock
   git commit -m "chore(release): prepare vX.Y.Z"
   ```

5. Push commit and tag:

   ```bash
   git push origin master
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

6. Wait for `.github/workflows/release.yml` to finish.
7. Check workflow logs for:
   - tag/version validation
   - `make ci`
   - `cargo package --locked`
   - expected mode behavior (skip side effects in dry-run, execute in real mode)
8. Verify final state:
   - Dry-run: no crate publish, no GitHub Release for `vX.Y.Z`.
   - Real release: crate published and GitHub Release created for `vX.Y.Z`.

## Rollback

- If workflow behavior is unstable, force safe mode with `RELEASE_DRY_RUN=1`.
- If tag was wrong, delete and recreate it after fixes:

  ```bash
  git tag -d vX.Y.Z
  git push origin :refs/tags/vX.Y.Z
  ```

- If a bad crate version was published, publish a new patch version; do not try
  to republish the same version.
