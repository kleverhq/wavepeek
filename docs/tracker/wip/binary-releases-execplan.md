# Implement binary releases and GitHub-centered publication for wavepeek

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. It is self-contained: a contributor should be able to start from this file and the current repository and implement the feature without relying on chat history.

## Purpose / Big Picture

After this change, a user can install `wavepeek` without installing the Rust toolchain. A stable tag such as `v0.6.0` will produce a GitHub Release that contains prebuilt VCD/FST-only binaries for Linux x86_64, Linux arm64, macOS Intel, macOS Apple Silicon, and Windows x86_64; shell and PowerShell installers; `cargo-dist` checksums and manifest; GitHub artifact attestations; and release notes with install, download, checksum, attestation, Cargo fallback, FSDB source-only, docs, and crates.io information.

GitHub Release is the primary release boundary. Documentation on GitHub Pages and crates.io publication are downstream state derived from the published GitHub Release and can be repaired or retried without issuing a new SemVer tag.

The feature is working when pushing a stable tag `vX.Y.Z` creates a GitHub Release with those artifacts, then dispatches docs and crate workflows; when README shows the short Pages install commands; and when the release page itself is sufficient to install and verify that exact version.

## Non-Goals

This plan does not add FSDB-enabled prebuilt binaries. FSDB remains source-only through `cargo install --locked wavepeek --features fsdb` on a Linux x86_64 host with a valid Synopsys Verdi / FSDB Reader SDK environment.

This plan does not add Homebrew, Scoop, WinGet, Docker images, musl Linux binaries as a readiness requirement, self-update commands, macOS notarization, Windows Authenticode signing, cosign, minisign, or package-manager signing. The first release implementation uses `cargo-dist` checksums and GitHub Artifact Attestations only.

This plan keeps tag push as the release trigger. It does not enable `cargo-dist` `dispatch-releases = true`, though that remains a future maintainer workflow option.

## Progress

- [x] (2026-06-09T10:53:57Z) Created and committed `docs/tracker/wip/binary-releases-proposal.md`, an English WIP proposal that records the release contract and design decisions.
- [x] (2026-06-09T10:53:57Z) Started this ExecPlan from the committed proposal and the current repository state.
- [x] (2026-06-09T11:05:00Z) Ran focused architecture and docs review lanes for the initial ExecPlan draft, then incorporated required clarifications for milestones, crates.io source checkout, docs repair/latest semantics, workflow permissions, README first-binary-release sequencing, and concrete guidance-reading commands.
- [x] (2026-06-09T11:20:00Z) Ran a fresh control-pass review and incorporated remaining release-safety clarifications for tag-ref checkout, stable SemVer validation, and default-branch downstream dispatch.
- [x] (2026-06-09T11:35:00Z) Ran a final clean control-pass review; it reported no substantive findings.
- [x] (2026-06-09T15:05:00Z) Installed `cargo-dist 0.32.0`, added `dist-workspace.toml`, ran `dist init --yes`, generated the baseline release workflow, ran `dist plan --tag v0.5.0 --output-format=json > tmp/dist-plan.json`, and verified generated workflow YAML with `actionlint .github/workflows/*.yml`.
- [x] (2026-06-09T15:05:00Z) Recorded generated `cargo-dist` job names: `plan`, `build-local-artifacts`, `build-global-artifacts`, `host`, and `announce`; generated release artifact names include the five required archives, `wavepeek-installer.sh`, `wavepeek-installer.ps1`, `sha256.sum`, per-artifact `*.sha256` files, `source.tar.gz`, `source.tar.gz.sha256`, and `dist-manifest.json` from the host upload path.
- [x] (2026-06-09T15:55:00Z) Replaced the generated release workflow with the `cargo-dist` baseline plus stable tag validation, a devcontainer `just ci`/`cargo package --locked` gate, rendered release notes, and default-branch downstream dispatch for docs and crates.io publication.
- [x] (2026-06-09T15:55:00Z) Added `tools/release/render_release_body.py` and tests covering changelog extraction, installer snippets, artifact table rendering, checksum and attestation instructions, FSDB source-only guidance, docs links, crates.io status text, and manifest validation.
- [x] (2026-06-09T15:55:00Z) Moved crates.io publication out of `.github/workflows/release.yml` into `.github/workflows/publish-crate.yml` with `tools/release/publish_crate.py` tests for default-branch dispatch, GitHub Release validation, exact tag checkout, idempotent already-published no-op behavior, missing-token failure, and `cargo publish --locked` cwd.
- [x] (2026-06-09T15:55:00Z) Extended docs publication to download release installer assets and copy them byte-for-byte into versioned Pages entrypoints, promoting root `/install.sh` and `/install.ps1` only when the staged version owns `latest`; tests cover alias preservation during non-latest repair.
- [x] (2026-06-09T15:55:00Z) Updated README, release/automation maintainer docs, release/docs helper READMEs, and `just docs-site-stage-deploy` for prebuilt-first install commands and the new GitHub Release/downstream workflow boundaries.
- [x] (2026-06-09T15:55:00Z) Ran focused helper and workflow checks: `python3 -B -m unittest discover -s tools/release -p 'test_*.py'`, `python3 -B -m unittest discover -s tools/docs -p 'test_*.py'`, `just test-aux`, `just check-actions`, `dist host --steps=create --tag v0.5.0 --output-format=json`, and `tools/release/render_release_body.py` against the resulting manifest.
- [ ] Perform at least one focused review pass, apply substantive fixes, rerun affected checks, and update this plan with review outcomes.

## Surprises & Discoveries

- Observation: The current `.github/workflows/release.yml` publishes to crates.io before creating the GitHub Release.
  Evidence: The workflow currently checks crates.io, requires `CRATES_IO_TOKEN`, runs `cargo publish --locked`, and only then uses `softprops/action-gh-release@v3`.

- Observation: The current docs workflow already treats GitHub Release as a prerequisite for docs publication.
  Evidence: `tools/docs/workflow_docs.py validate-dispatch` loads `repos/{repository}/releases/tags/{source_ref}` through `gh api` and rejects draft or prerelease releases.

- Observation: The docs publication helper already protects unrelated Pages paths.
  Evidence: `tools/docs/publish_docs.py allowed_path_patterns()` currently allows only the requested version tree, `latest/**`, root docs files, `versions.json`, and root schema artifacts; adding root `/install.sh` and `/install.ps1` requires extending this allowlist.

- Observation: `uv` uses `cargo-dist` and configures XDG-style install paths rather than the default Cargo home path.
  Evidence: `uv` has `installers = ["shell", "powershell"]`, `install-updater = false`, and `install-path = ["$XDG_BIN_HOME/", "$XDG_DATA_HOME/../bin", "~/.local/bin"]` in `dist-workspace.toml`.

- Observation: Review identified that docs repair semantics must not move the public latest installer aliases backward when repairing an older version.
  Evidence: Both review lanes flagged the conflict between root `/install.sh` as latest stable and a repair workflow that always refreshes root/latest aliases for the requested version.

- Observation: Control review identified three release-safety gaps: checkout of ambiguous `source_ref`, stable SemVer validation, and downstream dispatch ref selection.
  Evidence: The plan previously said to check out `source_ref` without requiring `refs/tags/${source_ref}`, relied on current tag/Cargo equality validation that accepts any `v...` shape, and omitted dispatching downstream workflows on the default branch.

- Observation: `cargo-dist 0.32.0` generated jobs named `plan`, `build-local-artifacts`, `build-global-artifacts`, `host`, and `announce`.
  Evidence: `.github/workflows/release.yml` generated by `dist init --yes` contains those job keys.

- Observation: `dist plan --tag v0.5.0 --output-format=json` with this configuration reports native checksum artifacts as both a unified `sha256.sum` and per-artifact `*.sha256` files.
  Evidence: `tmp/dist-plan.json` has artifacts named `sha256.sum`, `wavepeek-<target>.<archive>.sha256`, and `source.tar.gz.sha256`.

- Observation: `dist init --yes` added a `[profile.dist]` section to `Cargo.toml` and normalized the target order in `dist-workspace.toml`.
  Evidence: The generated diff shows `[profile.dist] inherits = "release"` with `lto = "thin"`, and the generated target list is sorted alphabetically by target triple.

- Observation: `cargo-dist` refuses to plan or host releases when generated CI output differs from its expected file.
  Evidence: `dist plan --tag v0.5.0 --output-format=json` failed with an out-of-date `.github/workflows/release.yml` diff after local policy hooks were applied. Adding `allow-dirty = ["ci"]` to `dist-workspace.toml` allowed `dist plan` and `dist host --steps=create` to succeed while preserving the local release workflow policy hooks.

- Observation: Docs repair required separate verification for `versions.json` latest semantics and root installer aliases.
  Evidence: New tests in `tools/docs/test_publish_docs.py` cover preserving the existing `latest` holder and rejecting root `/install.sh` changes while repairing a non-latest version.

## Decision Log

- Decision: Use `cargo-dist` as the binary distribution backend.
  Rationale: It owns the common release mechanics needed here: archive packaging, shell and PowerShell installers, checksums, dist manifest, GitHub Release upload, and GitHub Artifact Attestations. This avoids maintaining a custom release packager.
  Date/Author: 2026-06-09 / Grin

- Decision: Pin `cargo-dist` to `0.32.0` for the first binary-release implementation.
  Rationale: `cargo install cargo-dist --locked --version 0.32.0`, `dist init --yes`, and `dist plan --tag v0.5.0 --output-format=json` succeeded in the devcontainer. Pinning the observed generator version keeps the workflow shape reviewable.
  Date/Author: 2026-06-09 / Grin

- Decision: Generate the baseline release workflow with `cargo-dist`, then make minimal project-specific edits.
  Rationale: Staying close to generated output makes future `cargo-dist` updates practical. Project policy is layered on top through small hooks or jobs rather than a hand-rolled workflow that slowly becomes a worse private fork of `cargo-dist`.
  Date/Author: 2026-06-09 / Grin

- Decision: Keep tag-push release triggering.
  Rationale: The repository already uses tag push for stable release intent, and switching to manual `workflow_dispatch` can be done later if release operations need a manual approval switch.
  Date/Author: 2026-06-09 / Grin

- Decision: Use the XDG-style install path cascade and disable the `cargo-dist` updater.
  Rationale: Prebuilt installs should not imply a Rust toolchain or Cargo home. Updates are initially handled by rerunning the installer; self-update is explicitly out of scope.
  Date/Author: 2026-06-09 / Grin

- Decision: Accept native `cargo-dist` asset names, `dist-manifest.json`, and checksum layout.
  Rationale: The immutable version boundary is the GitHub Release URL for `vX.Y.Z`; manually renaming artifacts or inventing a custom checksum manifest creates drift against installers, manifest, and attestations.
  Date/Author: 2026-06-09 / Grin

- Decision: GitHub Pages installer entrypoints are byte-for-byte copies of the canonical release installer assets.
  Rationale: Wrappers add quoting, argument propagation, and PowerShell behavior risks. Copies preserve generated installer behavior while providing short README URLs.
  Date/Author: 2026-06-09 / Grin

- Decision: Add `tools/release/render_release_body.py` rather than expanding `extract_release_notes.py`.
  Rationale: `extract_release_notes.py` has a narrow, tested purpose. The full GitHub Release body is a separate artifact that combines changelog content with generated install, download, verification, FSDB, docs, and crates.io sections.
  Date/Author: 2026-06-09 / Grin

- Decision: GitHub Pages and crates.io publication are downstream state derived from a published GitHub Release.
  Rationale: Documentation can be repaired and crate publication can be retried without changing the public release identity. A version is public when the GitHub Release exists with assets, not when every downstream publication has completed.
  Date/Author: 2026-06-09 / Grin

- Decision: `publish-crate.yml` must run every Cargo command from the checked-out release source path, not from the trusted tooling checkout.
  Rationale: crates.io publication is irreversible. Publishing from the default branch when the intended source is `source_ref` would publish the wrong package contents under the release version.
  Date/Author: 2026-06-09 / Grin

- Decision: Docs repair must preserve the current public latest aliases when repairing a non-latest version.
  Rationale: `/install.sh`, `/install.ps1`, and the `latest` docs alias are public latest-stable entrypoints. Repairing an older version should update only that version's pinned docs and installer copies unless the repaired version is already the current latest.
  Date/Author: 2026-06-09 / Grin

- Decision: The regenerated release workflow must include an explicit permissions review before handoff.
  Rationale: `cargo-dist` attestations, GitHub Release writes, downstream workflow dispatch, and devcontainer cache access require different permissions. Generated defaults may not cover `actions: write`, `contents: write`, `id-token: write`, `attestations: write`, and `packages: write` in the jobs that need them.
  Date/Author: 2026-06-09 / Grin

- Decision: Release automation must reject anything except stable SemVer tags `vX.Y.Z` and matching Cargo package version `X.Y.Z`.
  Rationale: The proposal scope is stable releases only. A tag such as `v1.2.3-rc.1` or `vfoo` must not create a public binary GitHub Release even if it matches a package version string.
  Date/Author: 2026-06-09 / Grin

- Decision: Downstream workflows must run from trusted default-branch workflow definitions and check out release source through an unambiguous tag ref.
  Rationale: The tooling source and release source are intentionally separate. Dispatching on a feature branch or checking out an ambiguous branch/tag name could run stale workflow logic or publish the wrong immutable crates.io source.
  Date/Author: 2026-06-09 / Grin

- Decision: Set `allow-dirty = ["ci"]` in `dist-workspace.toml`.
  Rationale: The repository intentionally carries policy hooks around the generated `cargo-dist` jobs. Without this setting, `dist plan` and `dist host` fail before producing a manifest because `.github/workflows/release.yml` is not byte-for-byte generated output.
  Date/Author: 2026-06-09 / Grin

## Outcomes & Retrospective

Initial planning outcome: the release contract is documented in `docs/tracker/wip/binary-releases-proposal.md`, and this plan breaks the implementation into independently verifiable milestones. No implementation work has started yet.

Review outcome, 2026-06-09: the first architecture and docs review pass found missing milestone structure and several release safety clarifications. The plan now explicitly covers source-checkout working directory for crates.io publication, non-latest docs repair semantics, release workflow permissions, and first-binary-release README sequencing.

Control review outcome, 2026-06-09: the follow-up control pass found no implementation code issues because implementation has not started, but it found plan gaps around ambiguous source checkout, stable tag validation, and downstream workflow refs. Those are now explicit requirements.

Final review outcome, 2026-06-09: a fresh control reviewer inspected the proposal and ExecPlan after the hardening changes and reported no substantive findings.

Implementation outcome, 2026-06-09: binary-release workflow implementation is in the working tree. Focused helper tests, workflow linting, `just test-aux`, `dist host --steps=create --tag v0.5.0 --output-format=json`, and release-body rendering from the resulting manifest pass. The remaining work is a full `just check`, focused review, fixes from that review, final commit(s), push, and PR creation.

As milestones complete, add dated entries here summarizing what changed, which commands proved it, and which risks remain.

## Context and Orientation

`wavepeek` is a Rust CLI. The root `Cargo.toml` defines package `wavepeek`, version `0.5.0`, one binary named `wavepeek` at `src/main.rs`, default features with VCD/FST support, and optional feature `fsdb`. Optional FSDB support depends on Synopsys Verdi/FSDB Reader SDK and must not be included in public prebuilt binaries.

Repository automation is container-first and exposed through the root `justfile`. The standard quality gate is `just ci`; the local pre-handoff gate is `just check`. Workflow linting is `just check-actions`. Focused Python helper tests are run through `python3 -B -m unittest discover -s tools/release -p "test_*.py"` and `python3 -B -m unittest discover -s tools/docs -p "test_*.py"`; `just test-aux` also runs those helper tests as part of the auxiliary suite.

Current release automation lives in `.github/workflows/release.yml`. It is tag-triggered on `v*.*.*`, validates tag/version using `tools/release/validate_tag_version.py`, runs `just ci` and `cargo package --locked` through `devcontainers/ci`, extracts a changelog excerpt through `tools/release/extract_release_notes.py`, publishes the crate to crates.io inline, creates a GitHub Release with `softprops/action-gh-release`, and dispatches docs publication. This workflow must be replaced or regenerated around `cargo-dist` so binary artifacts are published before downstream docs/crates.io state.

Current docs automation lives in `.github/workflows/docs.yml`, `tools/docs/workflow_docs.py`, and `tools/docs/publish_docs.py`. `docs.yml` is manual-only, checks out trusted tooling from the default branch, validates that a stable GitHub Release exists for `source_ref`, stages a versioned `mike` docs update, pushes a verified `gh-pages` state branch, and deploys through `actions/upload-pages-artifact` and `actions/deploy-pages`. The helper already supports repair through `repair_existing_version=true` and verifies that unrelated versions and paths are not changed. This area must be extended to copy release installer assets into GitHub Pages paths `/install.sh`, `/install.ps1`, `/${version}/install.sh`, and `/${version}/install.ps1`.

Current release helper code under `tools/release/` contains `validate_tag_version.py`, `extract_release_notes.py`, their unit tests, and `README.md`. New release helper logic belongs in this group, must have local tests next to it, and must be mentioned in `tools/release/README.md` when it becomes a normal workflow entrypoint.

README currently lists Cargo install as the first install path. It must be changed to show prebuilt-first installation through short GitHub Pages URLs, with Cargo install as fallback and FSDB as source-only.

`docs/dev/release.md` and `docs/dev/automation.md` describe the current maintainer workflow and must be updated because crates.io publication will move out of `release.yml`, binary assets become part of the GitHub Release, and docs/crates.io become downstream workflows.

A GitHub Release is the release page and asset container at `https://github.com/kleverhq/wavepeek/releases/tag/vX.Y.Z`. A GitHub Pages site is the documentation site under `https://kleverhq.github.io/wavepeek/`. crates.io is the Rust package registry. `cargo-dist` is the release packaging tool that generates GitHub Actions workflows and release artifacts for Rust applications.

## Open Questions

The implementation must answer these by inspection or dry-run and then update this plan:

1. Which exact `cargo-dist` version is pinned? The proposal uses `cargo-dist` as the backend and current crates.io state shows `0.32.0` as a candidate, but the implementer must confirm the selected version before generating workflow files.
2. What job names does the selected `cargo-dist` version generate for plan/build/host/announce? Record them after generation so downstream jobs can use precise `needs:` edges.
3. Does generated `cargo-dist` output provide a clean hook for replacing the GitHub Release body before publication? If not, use a minimal `gh release edit --notes-file` step immediately after release creation and before downstream dispatch, and record that choice in the Decision Log.
4. Does generated `cargo-dist` produce per-artifact `*.sha256` files, an aggregate checksum manifest, or both for this configuration? The release body renderer must reflect the actual output.
5. Does default `cargo-dist` build strategy succeed for `aarch64-unknown-linux-gnu` in GitHub Actions? If it fails, fix only the failing target path and record the evidence.

## Milestones

### Milestone 1: Generate and understand the `cargo-dist` baseline

At the end of this milestone, the repository has a pinned `cargo-dist` configuration and a generated release workflow whose job graph is understood. The implementer runs `dist init --yes` and a non-publishing `dist plan` command, records the selected `cargo-dist` version, generated job names, artifact naming, checksum layout, and any workflow hook limitations in this plan, and commits only the configuration and generated workflow changes that match the release contract.

Acceptance for this milestone is observable in `dist-workspace.toml`, `.github/workflows/release.yml`, and this plan. `dist plan --tag vX.Y.Z --output-format=json` or the selected version's equivalent plan command succeeds without publishing, and `just check-actions` or `actionlint .github/workflows/*.yml` passes after any workflow edits.

### Milestone 2: Render and finalize GitHub Release content

At the end of this milestone, release body generation is deterministic and tested. `tools/release/render_release_body.py` can produce a complete GitHub Release body from `CHANGELOG.md` plus either `dist-manifest.json` or the configured static target mapping. The release workflow uses this body before downstream dispatch, either through a native generated hook or a minimal `gh release edit --notes-file` step immediately after release creation.

Acceptance is that `python3 -B -m unittest discover -s tools/release -p "test_*.py"` passes and the rendered Markdown contains the changelog, installer snippets, platform table, checksum instructions, attestation instructions, FSDB source-only note, Cargo fallback, docs link, and crates.io status/link sections.

### Milestone 3: Split and verify downstream publication

At the end of this milestone, crates.io publication and docs publication are independent downstream workflows derived from a published GitHub Release. `.github/workflows/publish-crate.yml` validates its inputs and publishes from the checked-out release source path only, where that source is resolved from `refs/tags/${source_ref}` or the corresponding tag commit SHA rather than an ambiguous branch/tag name. `.github/workflows/docs.yml` downloads release installer assets and `tools/docs/publish_docs.py` copies them into the correct Pages root and versioned paths while preserving latest aliases during non-latest repairs.

Acceptance is that release and docs helper tests pass, workflow linting passes, `publish-crate.yml` can prove already-published versions are success/no-op in tests, publish tests prove `cwd` and tag-ref checkout selection, and docs tests prove byte-for-byte installer copies plus safe repair/latest behavior.

### Milestone 4: Update public and maintainer documentation

At the end of this milestone, README presents prebuilt binary installation first, Cargo install as fallback, and FSDB as source-only. Maintainer docs explain that `release.yml` creates the GitHub Release and binary assets, while docs and crates.io workflows run downstream and can be retried according to their contracts.

Acceptance is that README uses GitHub Pages latest install URLs, the pinned README example is tied to the first version that actually has binary-release Pages installer aliases, and `docs/dev/release.md`, `docs/dev/automation.md`, and `tools/release/README.md` describe current workflow behavior without stale references to inline crates.io publication.

### Milestone 5: Validate, review, and hand off

At the end of this milestone, all focused tests and workflow lint checks pass, the devcontainer `just check` pre-handoff gate passes, and this plan records final decisions, surprises, test output summaries, and review outcomes. At least one focused review pass must inspect the consolidated implementation, and substantive findings must be fixed or documented with rationale.

Acceptance is a clean `git status --short`, successful recorded validation commands, and an updated `Outcomes & Retrospective` entry explaining what was implemented and what remains for the first real binary release tag.

## Plan of Work

Start by creating the `cargo-dist` baseline. Install the selected `cargo-dist` version locally in the devcontainer or CI-equivalent environment. Add root `dist-workspace.toml` with a `[workspace]` section containing `members = ["cargo:."]` and a `[dist]` section that pins `cargo-dist-version`, sets `ci = "github"`, `installers = ["shell", "powershell"]`, `unix-archive = ".tar.gz"`, `windows-archive = ".zip"`, the five required targets, `create-release = true`, `dispatch-releases = false`, `github-attestations = true`, attestation filters for JSON, shell, PowerShell, zip, tar.gz, and checksum artifacts, `install-updater = false`, and the XDG-style `install-path`. Do not add musl targets, package-manager installers, custom runners, or self-update.

Generate the baseline release workflow with `dist init --yes` after the config is present. Inspect every generated file. If `cargo-dist` wants to migrate configuration, accept only changes that preserve this plan's contract. Keep the generated `.github/workflows/release.yml` close to generated output; put longer project-specific logic into helper scripts or small reusable jobs instead of inline shell. Run `dist plan --tag v0.5.0 --output-format=json` or the selected version's equivalent non-publishing plan command and record the important output shape in this plan. The command should not publish anything.

Next, restore `wavepeek` release policy on top of generated `cargo-dist` output. The release workflow must still validate that `GITHUB_REF_NAME` is a stable SemVer tag `vX.Y.Z` and matches `Cargo.toml` package version `X.Y.Z`, run `just ci` in the CI devcontainer, and run `cargo package --locked` before creating the public GitHub Release. Keep the existing devcontainer cache pattern unless `cargo-dist` generated output conflicts with it. If generated workflow has custom job hooks, use them; otherwise add minimal jobs with explicit `needs:` relationships after the generated plan job and before the generated publish/announce job. Avoid long inline scripts in `.github/workflows/release.yml`; `.github/workflows/AGENTS.md` limits inline shell or Python to short glue. After generation, explicitly verify permissions for each job: release creation needs `contents: write`, downstream dispatch needs `actions: write`, attestations need `id-token: write` and `attestations: write`, and the existing devcontainer cache flow may still need `packages: write`.

Add `tools/release/render_release_body.py`. It should import or reuse the changelog extraction logic from `tools/release/extract_release_notes.py` rather than duplicating parser behavior. Its CLI should accept `--version X.Y.Z`, `--repository kleverhq/wavepeek`, `--changelog CHANGELOG.md`, optional `--dist-manifest path`, and optional `--crate-published true|false|unknown`. It writes Markdown to stdout. The output must contain a `## Release Notes` heading with the changelog body, an `## Install wavepeek X.Y.Z` section with version-specific GitHub Release installer snippets, a `## Download wavepeek X.Y.Z` section with a platform table, a checksum verification section matching actual `cargo-dist` checksum artifacts, an attestation verification section using `gh attestation verify <artifact> --repo kleverhq/wavepeek`, an FSDB note that prebuilt binaries are VCD/FST-only, a Cargo fallback snippet, and docs/crates.io links or neutral status text.

If `dist-manifest.json` is available before the release body is finalized, parse it to discover asset and checksum names. Keep the parser conservative: accept only the asset fields needed for the table, fail loudly on malformed JSON, and fall back to no guessing when the manifest is present but incomplete. If workflow ordering means the body must be generated before final manifest upload, use a static expected table derived from `dist-workspace.toml`; tests must prove this static table matches the configured targets. Update the Decision Log with the selected ordering.

Move crates.io publication into a new `.github/workflows/publish-crate.yml`. Add a small tested helper under `tools/release/`, for example `tools/release/publish_crate.py` or `tools/release/workflow_crate.py`, to keep workflow YAML short. The helper should validate `version` and `source_ref`, load and verify the stable GitHub Release for `source_ref`, verify that `source_ref` resolves to a Git tag `refs/tags/${source_ref}`, read `Cargo.toml` from an explicit release source path, check whether `wavepeek` version `X.Y.Z` exists on crates.io through the crates.io API, and either return a verified no-op or run `cargo publish --locked` with `CARGO_REGISTRY_TOKEN`. The workflow should check out trusted tooling from the default branch, check out release source through an unambiguous tag ref such as `refs/tags/${{ inputs.source_ref }}` or a resolved tag commit SHA into a separate path such as `release-source` with `persist-credentials: false`, pass that path to the helper as `--source-root`, require `CRATES_IO_TOKEN` only when the version is not already published, avoid `contents: write` permissions, and ensure every Cargo command runs with `cwd` set to the release source path. Add helper tests that fail if the publish command would run from the tooling checkout or if a branch named like the tag could be selected instead of `refs/tags/${source_ref}`.

Modify the generated release workflow so it dispatches `.github/workflows/docs.yml` and `.github/workflows/publish-crate.yml` only after the GitHub Release body and assets are in their final expected state. Dispatch inputs are `version` without leading `v` and `source_ref` with the leading `v`. The dispatch `ref` must be the repository default branch so downstream workflows use trusted current workflow definitions and helper tooling; mirror the existing docs dispatch validation for `publish-crate.yml` so a workflow run started from a non-default branch is rejected. A failed downstream dispatch must not invalidate the GitHub Release; maintainers can rerun downstream workflows manually with the same inputs.

Extend docs publication. In `.github/workflows/docs.yml`, after dispatch validation and before staging docs in the devcontainer, download release installer assets from GitHub Release `source_ref` into `tmp/docs-site/release-assets` using a short `gh release download` command or an equivalent helper. Pass the needed token into only the step that needs it. In `tools/docs/publish_docs.py`, add a `release_assets` path to `Paths`, copy `wavepeek-installer.sh` to `${version}/install.sh`, and copy `wavepeek-installer.ps1` to `${version}/install.ps1`. Root `install.sh`, root `install.ps1`, and the `latest` alias must be refreshed only when publishing a new version or when repairing the version that already owns the current `latest` alias. Repairing a non-latest version must preserve the existing root installer aliases and `latest` target. These copies must happen in the `gh-pages` worktree before metadata is written and bundled. Add root `install.sh` and `install.ps1` to the allowed path patterns; the versioned paths are already covered by `${version}/**`. Add tests that prove byte-for-byte copies are staged, missing installer assets fail with a clear error, allowed path verification accepts the new root files, non-latest repair preserves root/latest aliases, and latest repair refreshes root/latest aliases.

Update README so the first install path is prebuilt-first through Pages. Show latest macOS/Linux and Windows commands through `https://kleverhq.github.io/wavepeek/install.sh` and `https://kleverhq.github.io/wavepeek/install.ps1`. Show pinned commands only for a version that actually has binary-release Pages installer aliases. If this implementation lands before the first binary release is published, make the README pinned example part of the first binary release preparation commit and use that release version; do not point at `0.5.0` unless `0.5.0` has been backfilled with installer aliases. Move Cargo install below as fallback and keep FSDB as an explicit source-only feature requiring Verdi.

Update maintainer documentation. In `docs/dev/automation.md`, describe that release workflow now uses `cargo-dist`, builds binary artifacts, creates the GitHub Release, and dispatches docs and crate workflows; crates.io no longer publishes inside `release.yml`. In `docs/dev/release.md`, update the release checklist and verification steps: after pushing the tag, wait for binary release assets, release body, attestations, docs dispatch, and crate dispatch; verify Pages installer aliases and crates.io state. Update `tools/release/README.md` with normal helper entrypoints, including `render_release_body.py` and any crates.io helper.

Keep implementation commits small enough to review. A good sequence is: cargo-dist config and generated workflow baseline; release body helper and tests; crates.io workflow/helper and tests; docs installer entrypoint helper and tests; README/dev docs updates; final workflow wiring and validation fixes. Update this ExecPlan's `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` at each stopping point.

### Concrete Steps

Work from repository root `/workspaces/wavepeek`.

1. Confirm a clean base and read local guidance. Use the available file-reading tool, or use shell commands such as `python3 - <<'PY'` snippets, to inspect these files before editing: `docs/tracker/wip/AGENTS.md`, `.github/workflows/AGENTS.md`, and `tools/AGENTS.md`. Then run:

       git status --short

   Expected: only intentional worktree changes appear. Do not delete files under `tmp/` because that directory may contain user or agent scratch state.

2. Install and inspect `cargo-dist`:

       cargo install cargo-dist --locked --version 0.32.0
       dist --version

   If version `0.32.0` is no longer the selected version, update the pinned version in this plan before continuing. Record the selected version in `Decision Log`.

3. Add `dist-workspace.toml` with the minimal configuration. Use this shape and adjust only if `cargo-dist` rejects a field:

       [workspace]
       members = ["cargo:."]

       [dist]
       cargo-dist-version = "0.32.0"
       ci = "github"
       installers = ["shell", "powershell"]
       unix-archive = ".tar.gz"
       windows-archive = ".zip"
       targets = [
           "x86_64-unknown-linux-gnu",
           "aarch64-unknown-linux-gnu",
           "x86_64-apple-darwin",
           "aarch64-apple-darwin",
           "x86_64-pc-windows-msvc",
       ]
       create-release = true
       dispatch-releases = false
       github-attestations = true
       github-attestations-filters = ["*.json", "*.sh", "*.ps1", "*.zip", "*.tar.gz", "*.sha256"]
       install-updater = false
       install-path = ["$XDG_BIN_HOME/", "$XDG_DATA_HOME/../bin", "~/.local/bin"]

4. Generate or refresh the `cargo-dist` workflow:

       dist init --yes
       git diff -- .github/workflows/release.yml dist-workspace.toml Cargo.toml
       dist plan --tag v0.5.0 --output-format=json > tmp/dist-plan.json

   If `dist plan` needs a different non-publishing command in the selected version, use the command printed by `dist --help`, record the difference in `Surprises & Discoveries`, and keep `tmp/dist-plan.json` ignored scratch. Do not commit files under `tmp/`.

5. Update `tools/release/validate_tag_version.py` and its tests so the helper rejects non-stable tags and versions. It must accept only `vX.Y.Z` where each component is a non-negative decimal integer with no prerelease or build suffix, and it must still require exact equality with `Cargo.toml` package version `X.Y.Z`. Then add and test `tools/release/render_release_body.py` and `tools/release/test_render_release_body.py`:

       python3 -B -m unittest discover -s tools/release -p "test_*.py"

   Expected: all release helper tests pass, including existing `extract_release_notes` and `validate_tag_version` tests.

6. Add crates.io helper and `.github/workflows/publish-crate.yml`. Test helper behavior with mocked HTTP or local fixture JSON; do not hit crates.io in unit tests. Include tests that record the `cargo publish --locked` command and assert its working directory is the release source checkout path passed as `--source-root`, not the trusted tooling checkout, and that source checkout uses `refs/tags/${source_ref}` or the resolved tag commit SHA instead of an ambiguous `source_ref` name. Run:

       python3 -B -m unittest discover -s tools/release -p "test_*.py"
       actionlint .github/workflows/*.yml

   Expected: already-published versions are success/no-op in tests, unpublished versions require a token before publish, malformed dispatch inputs fail, publishing cannot run from the wrong checkout, and an ambiguous branch/tag name cannot select release source.

7. Extend docs installer publication in `tools/docs/publish_docs.py`, `tools/docs/workflow_docs.py` if needed, `.github/workflows/docs.yml`, and docs helper tests. Run:

       python3 -B -m unittest discover -s tools/docs -p "test_*.py"
       actionlint .github/workflows/*.yml

   Expected: tests prove installer asset copies are byte-for-byte, missing assets fail clearly, allowed paths accept root installer aliases, new-version publication promotes latest/root aliases, repairing the current latest refreshes latest/root aliases, and repairing an older non-latest version preserves existing latest/root aliases.

8. Update README and maintainer docs:

       README.md
       docs/dev/automation.md
       docs/dev/release.md
       tools/release/README.md

   Keep wording concise and describe current behavior, not the history of why it changed. Do not commit a pinned README installer URL for a version unless that version will have Pages installer aliases when the change is released.

9. Run focused validation, then the pre-handoff gate in the devcontainer/CI image:

       python3 -B -m unittest discover -s tools/release -p "test_*.py"
       python3 -B -m unittest discover -s tools/docs -p "test_*.py"
       just check-actions
       just check

   If the environment is not inside the devcontainer and `just check` fails with the container guard, run these commands in the repository devcontainer/CI image. Do not bypass hooks unless explicitly instructed.

10. Before handoff, update this ExecPlan with actual test output summaries, generated job names, final decisions, and any remaining gaps. Run `git status --short` and ensure only intentional tracked changes remain.

### Validation and Acceptance

Focused helper acceptance:

- `python3 -B -m unittest discover -s tools/release -p "test_*.py"` passes. The updated tag validation tests reject non-stable tags and mismatched versions. The new release body tests verify changelog extraction, version-specific installer snippets, platform table rows for all five targets, checksum instructions matching generated/native artifact names, attestation instructions, FSDB source-only text, and docs/crates.io link/status rendering.
- `python3 -B -m unittest discover -s tools/docs -p "test_*.py"` passes. New docs tests prove installer assets are copied byte-for-byte to root and versioned Pages paths, missing release installer assets are fatal, and root installer aliases are allowed while unrelated paths remain rejected.
- `actionlint .github/workflows/*.yml` or `just check-actions` passes for the regenerated and edited workflows.

Release dry-run acceptance:

- `dist plan --tag vX.Y.Z --output-format=json` or the selected version's equivalent non-publishing command succeeds and shows archive/installers/manifest artifacts for the five required targets.
- The generated or edited release workflow contains tag-push trigger for `v*.*.*`, validates stable SemVer tags before release creation, does not publish crates.io inline, dispatches docs and crate workflows on the repository default branch after GitHub Release publication, and has verified permissions for GitHub Release writes, workflow dispatch, artifact attestations, and any devcontainer cache package access.

End-to-end release acceptance on the first real stable tag after implementation:

- GitHub Release `vX.Y.Z` exists and is published.
- Release assets include `wavepeek-x86_64-unknown-linux-gnu.tar.gz`, `wavepeek-aarch64-unknown-linux-gnu.tar.gz`, `wavepeek-x86_64-apple-darwin.tar.gz`, `wavepeek-aarch64-apple-darwin.tar.gz`, `wavepeek-x86_64-pc-windows-msvc.zip`, `wavepeek-installer.sh`, `wavepeek-installer.ps1`, `dist-manifest.json`, and native `cargo-dist` checksum artifacts.
- The release body contains the changelog section plus install, download, checksum, attestation, FSDB, Cargo fallback, docs, and crates.io sections.
- `gh attestation verify <downloaded-artifact> --repo kleverhq/wavepeek` succeeds for an attested artifact.
- The docs workflow publishes `https://kleverhq.github.io/wavepeek/install.sh`, `https://kleverhq.github.io/wavepeek/install.ps1`, `https://kleverhq.github.io/wavepeek/X.Y.Z/install.sh`, and `https://kleverhq.github.io/wavepeek/X.Y.Z/install.ps1` for a new latest release, and preserves root latest aliases when repairing an older non-latest version.
- The versioned Pages installer script is byte-for-byte equal to the corresponding GitHub Release installer asset for `vX.Y.Z`.
- `publish-crate.yml` publishes the crate from the release source checked out via `refs/tags/vX.Y.Z` or the resolved tag commit SHA, or confirms it is already published as a success/no-op.

Repository gate acceptance:

- Before handoff, run `just check` in the devcontainer/CI image and expect success. If a full `just ci` run is available and not prohibitively expensive, run `just ci` as the standard quality gate and record its result.

### Idempotence and Recovery

All local generation should be repeatable. `dist init --yes` is designed to be rerun; after rerunning, inspect the diff and keep only changes that preserve this plan's contract. Do not edit generated workflow sections casually. If a `cargo-dist` update rewrites large parts of `.github/workflows/release.yml`, regenerate in a clean worktree and compare before applying project-specific patches.

Docs publication repair is explicit. Rerunning `docs.yml` for an existing version must fail unless `repair_existing_version=true`. With repair enabled, it may replace the requested version. It refreshes `latest` and root installer aliases only if the requested version already owns the current `latest` alias; repairing an older non-latest version must preserve the existing public latest/root aliases and must not modify unrelated versions or paths.

Crates.io publication is immutable. Rerunning `publish-crate.yml` after successful publication must detect the existing version and exit success/no-op. If a publish attempt fails ambiguously, rerun the workflow; it should either publish or detect that publication already happened.

If a tag was pushed incorrectly before GitHub Release creation, delete and recreate the tag after fixing the release prep commit. If a GitHub Release was created with bad artifacts, delete or supersede according to maintainer policy and prefer a new patch version once public users may have consumed the release.

Scratch artifacts such as `tmp/dist-plan.json`, local release downloads, and test output belong under repository-root `tmp/`. Never delete arbitrary existing files there because they may belong to the user or another agent.

### Artifacts and Notes

The design source for this plan is committed at:

    docs/tracker/wip/binary-releases-proposal.md

Current release workflow behavior to replace:

    .github/workflows/release.yml
      validates tag/version
      runs just ci
      runs cargo package --locked
      extracts changelog notes
      publishes crates.io inline
      creates GitHub Release
      dispatches docs

Current docs workflow behavior to extend:

    .github/workflows/docs.yml
      workflow_dispatch inputs: version, source_ref, repair_existing_version
      validates stable GitHub Release existence
      stages mike docs
      pushes verified gh-pages state
      deploys through Pages actions

Expected public installer URLs after implementation:

    https://kleverhq.github.io/wavepeek/install.sh
    https://kleverhq.github.io/wavepeek/install.ps1
    https://kleverhq.github.io/wavepeek/X.Y.Z/install.sh
    https://kleverhq.github.io/wavepeek/X.Y.Z/install.ps1
    https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.sh
    https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.ps1

### Interfaces and Dependencies

Add root `dist-workspace.toml` as the `cargo-dist` configuration file. The selected `cargo-dist` version must be pinned in this file through `cargo-dist-version`.

Keep `tools/release/extract_release_notes.py` public function:

    def extract_release_notes(changelog_text: str, version: str) -> str

Add `tools/release/render_release_body.py` with a deterministic CLI:

    python3 -B tools/release/render_release_body.py \
        --version X.Y.Z \
        --repository kleverhq/wavepeek \
        --changelog CHANGELOG.md \
        [--dist-manifest path] \
        [--crate-published true|false|unknown]

The helper writes Markdown to stdout and exits non-zero with an `error: release-body:` prefix on invalid input.

Add a crates.io workflow helper under `tools/release/`. Its exact filename may be `publish_crate.py` or `workflow_crate.py`, but it must accept an explicit `--source-root` path and expose tested behavior for:

    validate version/source_ref as stable SemVer X.Y.Z and vX.Y.Z
    verify source_ref resolves to refs/tags/${source_ref}
    verify stable GitHub Release exists
    verify release source Cargo.toml package.version from --source-root
    check crates.io version existence
    publish with cargo publish --locked only when needed, using cwd = --source-root

Extend `tools/docs/publish_docs.py` paths with a release assets directory under the existing work dir, for example:

    release_assets = work_dir / "release-assets"

Add or update functions with behavior equivalent to:

    copy_installer_entrypoints(version: str, run_paths: Paths, runner: CommandRunner) -> None

This function copies `wavepeek-installer.sh` and `wavepeek-installer.ps1` from `run_paths.release_assets` into the `gh-pages` worktree as versioned pinned aliases, and conditionally as root latest aliases when the requested version is being promoted or is already the current latest. It then stages the copied files with git. It must fail if either source asset is missing or is not a regular file.

Update `.github/workflows/docs.yml` so the stage job downloads release installer assets before invoking `tools/docs/workflow_docs.py stage-deploy`. Keep inline YAML glue short; move longer logic into `tools/docs` or `tools/release` helpers.

Update `.github/workflows/release.yml` so the generated `cargo-dist` workflow creates binary artifacts and the GitHub Release, then finalizes release notes and dispatches downstream workflows. If job names differ from this plan after generation, update this file's Open Questions, Concrete Steps, and Validation sections with the actual names before continuing.

Revision note, 2026-06-09: Initial ExecPlan written from the committed binary release proposal. The plan deliberately includes a discovery milestone for generated `cargo-dist` job names because those names are version-specific and must be verified from actual generated output before workflow wiring is finalized.

Revision note, 2026-06-09: Incorporated first review pass findings. Added narrative milestones, explicit crates.io release-source working-directory requirements, safe docs repair/latest semantics, release workflow permissions review, README first-binary-release sequencing, and valid guidance-reading instructions.

Revision note, 2026-06-09: Incorporated control-pass findings. Added requirements for stable `vX.Y.Z` tag validation, unambiguous `refs/tags/${source_ref}` release-source checkout, and default-branch downstream workflow dispatch.

Revision note, 2026-06-09: Recorded final clean control-pass review result. No plan changes were required beyond documenting that review outcome.
