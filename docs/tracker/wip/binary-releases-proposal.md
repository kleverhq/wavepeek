# Proposal: binary releases and GitHub-centered publication for wavepeek

Status: draft for review
Scope: release model, binary artifacts, GitHub Release, GitHub Pages, crates.io, installation UX
Out of scope: FSDB-enabled binary builds, Verdi-dependent releases, Homebrew/Scoop/WinGet, container images, self-update

## Summary

`wavepeek` should publish full binary releases: ready-to-run executables for the primary platforms, published in GitHub Releases together with checksums, installer scripts, a release manifest, release notes, and provenance or attestation metadata.

A GitHub Release becomes the primary public release object. A version is considered released when a published GitHub Release exists for tag `vX.Y.Z` and contains the expected core artifacts. Documentation publication and crates.io publication do not define whether the version exists. They are downstream publications that run after the GitHub Release succeeds and can be retried independently.

Target topology:

```text
push tag vX.Y.Z
  └─ release.yml
       ├─ validate tag/version
       ├─ run release quality gate
       ├─ validate crate package
       ├─ build binary release artifacts
       ├─ prepare checksums, installers, manifest, and release notes
       ├─ publish GitHub Release
       └─ dispatch downstream workflows
            ├─ docs.yml
            └─ publish-crate.yml
```

## Reference

Use `uv` as the product and UX reference for this feature:

```text
https://github.com/astral-sh/uv
https://github.com/astral-sh/uv/releases
https://docs.astral.sh/uv/getting-started/installation/
```

`uv` is a reference for the public release experience, not a normative dependency and not a mandatory model for internal implementation. For `wavepeek`, the important contract is comparable user experience:

- users see prebuilt binaries for primary platforms;
- the release page contains a clear platform asset table;
- release notes contain shell and PowerShell installer snippets;
- README/docs show the shortest path for installing the latest version;
- README/docs show pinned installation for a specific version;
- checksums and provenance/attestations are part of the release contract;
- source install through Cargo remains a fallback path, not the primary install path for ordinary users.

For `wavepeek`, the URL model is as important as the release page shape:

- README and long-lived documentation use short GitHub Pages URLs;
- release notes use version-specific GitHub Release URLs;
- the GitHub Release remains self-contained even if the documentation site has not updated yet or is temporarily broken;
- GitHub Pages installer entrypoints are convenient aliases for README/docs, not a new source of truth.

If `uv` implementation details conflict with the `wavepeek` contract, the `wavepeek` contract wins. This especially applies to FSDB: `wavepeek` does not publish FSDB-enabled prebuilt binaries.

## Discussions and decisions

### 2026-06-09: binary distribution backend

Decision: use `cargo-dist` as the backend for binary releases.

`cargo-dist` should own release archive builds, shell and PowerShell installers, the dist manifest, checksums, GitHub Release asset upload, and GitHub artifact attestations where supported. This matches the selected UX reference: `uv` uses `cargo-dist` for release workflow generation, installers, archives, manifest, and attestations.

The `wavepeek` release policy remains a separate contract layered on top of `cargo-dist`: tag/version validation, the release quality gate, `cargo package --locked`, downstream dispatch for docs and crates.io, and the exclusion of FSDB from prebuilt binaries must not be implicitly delegated to the generated workflow without review.

The initial `cargo-dist` configuration should be minimal: `shell` and `powershell` installers, `.tar.gz` for Unix, `.zip` for Windows, the required target set from this proposal, `install-updater = false`, no musl targets, and no package-manager installers. If native `cargo-dist` asset names differ from filenames that include the version, prefer native naming while preserving stable URLs and the manifest/checksum/provenance contract.

### 2026-06-09: installer install path and PATH management

Decision: use generated `cargo-dist` shell/PowerShell installers without a project-local fork, but do not accept the default `install-path = "CARGO_HOME"`.

For `wavepeek`, installer scripts should use an XDG-style cascade modeled after `uv`:

```toml
install-path = ["$XDG_BIN_HOME/", "$XDG_DATA_HOME/../bin", "~/.local/bin"]
install-updater = false
```

The shell installer should rely on standard `cargo-dist` behavior: install the binary into the first available install path, create an environment hook that adds the install path to `PATH`, update the user's shell profile where supported, and print instructions for the current shell session. The PowerShell installer should use standard `cargo-dist` behavior: install the binary and update the user-level `Path` through the Windows environment/registry mechanism where supported.

Installers must support the standard opt-out from PATH modification (`--no-modify-path` and the PowerShell equivalent). The project should not maintain its own installer script fork unless a concrete requirement appears that cannot be expressed through `cargo-dist` configuration.

### 2026-06-09: release manifest, checksums, and release notes

Decision: accept native `cargo-dist` manifest, asset names, and checksum layout instead of introducing a custom `SHA256SUMS` file or manually renaming artifacts.

The public release contract should reference `dist-manifest.json` and checksum artifacts generated by the selected `cargo-dist` version (`*.sha256` and/or an aggregate checksum manifest if it appears in native output). If `cargo-dist` does not include the version in archive filenames, that is acceptable: the immutable version boundary is the GitHub Release URL `/releases/download/vX.Y.Z/...` and `dist-manifest.json`.

Release notes are not a separate release asset. The existing changelog extraction remains the source of truth for the changelog section, and the GitHub Release body is extended with generated sections: install snippets, a platform download table, checksum/provenance verification, an FSDB note, and docs/crates.io links. These generated sections must not be duplicated in `CHANGELOG.md`.

### 2026-06-09: workflow integration strategy

Decision: generate the baseline GitHub Actions workflow with `cargo-dist`, then apply minimal project-specific fixes.

The generated workflow should stay as close as possible to the output of the selected `cargo-dist` version so future regeneration and updates remain practical. Project-specific additions must be small and clearly separated: the release quality gate, `cargo package --locked`, tag/version validation, a generated release body based on `CHANGELOG.md`, downstream dispatch for docs and crates.io, and permission adjustments required by the `wavepeek` contract.

If a change can be expressed through `cargo-dist` config or a custom reusable workflow hook, prefer that over manually rewriting the generated release workflow. Manual edits are allowed only where the generated workflow does not expose the needed hook or violates the project release contract.

### 2026-06-09: platform build strategy

Decision: for the initial target set, use the default host/cross-build strategy generated by the selected `cargo-dist` version.

The project should not add custom runners, Depot/BuildJet-specific configuration, manual `cross`, `zigbuild`, or target-specific workflow glue before there is proven need. If the generated `cargo-dist` workflow can build a target by its standard path, that path is sufficient. If one of the required targets fails under the default strategy, choose the smallest fix after the actual CI/dry-run failure and document it precisely.

The required platform contract remains unchanged: Linux x86_64, Linux arm64, macOS x86_64, macOS arm64, and Windows x86_64. The default strategy is not a reason to silently remove a target from the release matrix.

### 2026-06-09: GitHub Pages installer entrypoints

Decision: Pages installer entrypoints must be byte-for-byte copies of canonical `cargo-dist` installer assets from the corresponding GitHub Release, not project-local wrapper scripts.

Docs publication for version `X.Y.Z` must download `wavepeek-installer.sh` and `wavepeek-installer.ps1` from GitHub Release `vX.Y.Z` and publish copies as `/${X.Y.Z}/install.sh` and `/${X.Y.Z}/install.ps1`. Root aliases `/install.sh` and `/install.ps1` must be copies of the installers for the latest stable version that the docs workflow publishes as `latest`.

This keeps GitHub Release as the source of truth without adding a wrapper layer with its own argument passing, quoting, and PowerShell behavior. The repair docs workflow may republish installer entrypoint copies, but a versioned entrypoint must not change its target version.

### 2026-06-09: artifact signing and attestations

Decision: for the first binary release implementation, use only `cargo-dist`/GitHub Artifact Attestations and checksums. Separate signing or notarization flows are out of scope.

macOS notarization, Windows Authenticode signing, cosign signatures, minisign/signify, and package-manager signing are not readiness criteria for this feature. Release notes must document checksum verification and `gh attestation verify`; they must not promise platform-native code signing until it is explicitly implemented.

### 2026-06-09: downstream publication boundaries

Decision: GitHub Pages documentation and crates.io publication are downstream state derived from a published GitHub Release.

The primary release workflow must first create a GitHub Release with binary assets, installers, manifest, checksums/attestations, and release body. Only after successful GitHub Release publication should docs and crates.io workflows run. These workflows must validate that the GitHub Release for `vX.Y.Z` exists and is published, and must not create or modify the GitHub Release, tag, or binary assets.

`publish-crate.yml` must be idempotent for an already-published version: if `wavepeek X.Y.Z` already exists on crates.io, the workflow exits successfully as a verified no-op. `docs.yml` must remain repairable through explicit `repair_existing_version=true` without issuing a new SemVer tag.

### 2026-06-09: release trigger model

Decision: keep tag-push as the initial release trigger model.

Stable releases are triggered by push events for SemVer tags of the form `vX.Y.Z`. `cargo-dist` `dispatch-releases = true` is not enabled in the first phase. This preserves the existing maintainer mental model and proposal flow: a pushed tag runs the primary release workflow, which publishes the GitHub Release, after which downstream workflows publish docs and the crate.

Switching to a manual `workflow_dispatch` release model remains possible later if the project needs a separate manual release switch or a more complex dry-run/retry procedure.

### 2026-06-09: release body generation helper

Decision: add a new helper `tools/release/render_release_body.py` instead of expanding `tools/release/extract_release_notes.py` into a full GitHub Release body generator.

`extract_release_notes.py` should remain a narrow helper for extracting the changelog section from `CHANGELOG.md`. The new `render_release_body.py` should assemble the complete Markdown body for the GitHub Release: changelog section, install snippets, platform download table, checksum/provenance verification, FSDB note, docs link, and crates.io link/status. The helper must be deterministic, covered by tests under `tools/release`, and invoked by the release workflow instead of writing only the changelog excerpt to `body_path`.

If `dist-manifest.json` is available when the release body is generated, the helper should prefer manifest-derived asset and checksum names. If the generated workflow needs to create the body before the final manifest exists, the helper may use a static expected target mapping from the `cargo-dist` configuration, but the implementation plan must explicitly explain that order and cover it with tests.

## URL policy for installer scripts

Installer scripts have two public URL classes. They must be separated intentionally.

The first class is GitHub Release URLs. They are used in release notes and point to canonical assets of a specific GitHub Release:

```text
https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.sh
https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.ps1
```

Release notes must use these URLs. A specific release page must be self-contained and install that exact release even if GitHub Pages has not updated yet or is temporarily broken.

The second class is GitHub Pages URLs. They are used in README and user documentation as short entrypoints:

```text
https://kleverhq.github.io/wavepeek/install.sh
https://kleverhq.github.io/wavepeek/install.ps1
https://kleverhq.github.io/wavepeek/X.Y.Z/install.sh
https://kleverhq.github.io/wavepeek/X.Y.Z/install.ps1
```

Root-level Pages scripts (`/install.sh`, `/install.ps1`) install the latest stable version. Versioned Pages scripts (`/X.Y.Z/install.sh`, `/X.Y.Z/install.ps1`) live at the root of the specific version directory and install only that version. They must not be hidden deeper under paths such as `/X.Y.Z/assets/...`, because README should show a short and memorable pinned-version URL.

Pages scripts are byte-for-byte copies of release installer assets. The source of truth remains the GitHub Release; Pages provides short addressing for README and docs.

## Feature value

After this feature, a user can install `wavepeek` without a local Rust toolchain.

In README, the primary install path should be short and go through GitHub Pages. Installing the latest stable version uses the root-level Pages alias:

```bash
curl -LsSf https://kleverhq.github.io/wavepeek/install.sh | sh
```

For Windows, README should show the equivalent short PowerShell form:

```powershell
irm https://kleverhq.github.io/wavepeek/install.ps1 | iex
```

Users should also have an explicit pinned install path for a specific version. These scripts are published at the root of the version directory on GitHub Pages:

```bash
curl -LsSf https://kleverhq.github.io/wavepeek/0.5.0/install.sh | sh
```

```powershell
irm https://kleverhq.github.io/wavepeek/0.5.0/install.ps1 | iex
```

`0.5.0` is an example pinned version. The real README should use an actual public version current at the time of editing.

`cargo install` remains the fallback and source-build path:

```bash
cargo install --locked wavepeek
```

FSDB remains a source-only mode on machines with Verdi:

```bash
cargo install --locked wavepeek --features fsdb
```

The resulting `wavepeek` release should look like a modern CLI release: a user opens the GitHub Release, sees a platform table, downloads the right archive or runs an installer, verifies checksum and provenance, and sees docs and crates.io synchronized from the already-published release.

## Target state

In the final state, the release system should have the following properties.

GitHub Release is the only primary release boundary. It contains release notes, binary artifacts, checksums, installers, manifest, and verification instructions. If the GitHub Release is not published, the version is not considered publicly released.

Binary releases are published for default-feature `wavepeek` builds. These builds support VCD/FST. They do not include FSDB.

Documentation is published by a separate workflow after the GitHub Release. The workflow can be run manually to repair the site without issuing a new SemVer tag. Documentation is built from the release tag sources, but tooling is taken from the default branch so publication tooling fixes can be applied without republishing the version.

The crate is published by a separate workflow after the GitHub Release. The workflow can be run manually. If the version is already published on crates.io, the workflow exits successfully as a no-op after verification. There is no repair or overwrite mode for crates.io.

Release workflows use least-privilege permissions. Creating the GitHub Release, dispatching downstream workflows, deploying GitHub Pages, and publishing to crates.io are not mixed into one job/token context.

Release assets are verifiable. SHA-256 checksums are available for archives and installer scripts. Where supported, GitHub artifact attestations are published so users can verify artifact origin.

## Public user contract

### Installation from README

README must show prebuilt-first installation through short GitHub Pages links. README is a long-lived user entrypoint, not a page for a specific GitHub Release, so long `github.com/.../releases/download/...` URLs must not be the primary install path there.

README must contain two install scenarios.

The first scenario is installing the latest stable version. It must appear first and be the shortest install form.

For macOS/Linux:

```bash
curl -LsSf https://kleverhq.github.io/wavepeek/install.sh | sh
```

For Windows PowerShell:

```powershell
irm https://kleverhq.github.io/wavepeek/install.ps1 | iex
```

The second scenario is installing a specific version through the installer script at the root of that version directory on GitHub Pages.

For macOS/Linux:

```bash
curl -LsSf https://kleverhq.github.io/wavepeek/0.5.0/install.sh | sh
```

For Windows PowerShell:

```powershell
irm https://kleverhq.github.io/wavepeek/0.5.0/install.ps1 | iex
```

`0.5.0` is an example pinned version in this proposal. In the real README, the concrete version example must be an actual published version. Do not use an abstract `X.Y.Z` placeholder in README.

GitHub Pages alias contract:

```text
/install.sh              -> installer for the latest stable version
/install.ps1             -> installer for the latest stable version
/X.Y.Z/install.sh         -> installer for version X.Y.Z
/X.Y.Z/install.ps1        -> installer for version X.Y.Z
```

Root-level aliases `/install.sh` and `/install.ps1` are the shortest latest path. Versioned aliases `/<version>/install.sh` and `/<version>/install.ps1` are pinned paths and must live at the root of the corresponding version directory, without additional nesting such as `/assets/`, `/downloads/`, or `/installers/`.

If the project later gains a custom documentation domain, README should use the canonical docs base URL. The contract remains the same: README points to short documentation entrypoints, not long GitHub Release asset URLs.

### Installation from release notes

Release notes must show GitHub Release links, not GitHub Pages links. This separation is mandatory.

Reason: release notes belong to a specific GitHub Release and should point to immutable assets from that same release. They must remain self-contained even if GitHub Pages deployment has not completed, was disabled, or is temporarily broken.

For macOS/Linux for a specific release:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.sh | sh
```

For Windows PowerShell for a specific release:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.ps1 | iex"
```

Release notes may additionally show a latest GitHub Release URL as a convenience snippet:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/kleverhq/wavepeek/releases/latest/download/wavepeek-installer.sh | sh
```

But the version-specific snippet through `/releases/download/vX.Y.Z/...` must be present and must not be replaced by a GitHub Pages link.

### Source install

Cargo install remains the fallback and source-build path:

```bash
cargo install --locked wavepeek
```

### FSDB source-only install

FSDB is not included in prebuilt binaries. The FSDB install block must say explicitly that it requires a Linux x86_64 host with a correct Synopsys Verdi / FSDB Reader SDK environment.

```bash
cargo install --locked wavepeek --features fsdb
```

### Artifact verification

The GitHub Release must contain instructions for verifying checksum artifacts generated by `cargo-dist`. For per-artifact checksum files, the example should look like:

```bash
sha256sum -c wavepeek-x86_64-unknown-linux-gnu.tar.gz.sha256
```

If the selected `cargo-dist` version publishes an aggregate checksum manifest, release notes may additionally show verification through that manifest.

There must also be provenance/attestation verification instructions through the GitHub CLI:

```bash
gh attestation verify <artifact> --repo kleverhq/wavepeek
```

### Release page

The release page must contain:

- the changelog section for the version;
- a platform table and downloadable assets;
- install snippets for macOS/Linux and Windows through version-specific GitHub Release URLs;
- a checksum verification snippet;
- an attestation verification snippet;
- an explicit note that prebuilt binaries are built without FSDB and support VCD/FST;
- a link to the source-build path through Cargo;
- a link to the versioned documentation on GitHub Pages;
- a crates.io link after successful downstream publication, or neutral text before that completes.

## GitHub Release as the primary release object

Tag `vX.Y.Z` starts the primary release workflow. The workflow must first prove that the repository is releasable, then build and publish the core artifacts, then create the GitHub Release. Downstream workflows are dispatched only after the GitHub Release succeeds.

The primary release workflow is responsible for:

```text
validate tag/version
run release quality gate
cargo package --locked
build binary artifacts
build installers
generate checksums
generate release manifest
generate release notes
attest release assets
publish GitHub Release
dispatch docs publication
dispatch crates.io publication
```

`cargo publish` must not run inside the primary release workflow. It is an irreversible downstream publication, so it lives in a separate workflow.

Documentation also must not be part of the primary release boundary. The site is the published representation of an already-released version, not a condition for the version to exist.

## Binary artifacts

Initial required targets:

```text
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
x86_64-pc-windows-msvc
```

Expected archive assets for version `X.Y.Z` must follow native `cargo-dist` naming. Initial expected set:

```text
wavepeek-x86_64-unknown-linux-gnu.tar.gz
wavepeek-aarch64-unknown-linux-gnu.tar.gz
wavepeek-x86_64-apple-darwin.tar.gz
wavepeek-aarch64-apple-darwin.tar.gz
wavepeek-x86_64-pc-windows-msvc.zip
wavepeek-installer.sh
wavepeek-installer.ps1
dist-manifest.json
cargo-dist checksum artifacts
```

Archives must contain the executable `wavepeek` or `wavepeek.exe`, license/readme metadata, and minimal extra content. Asset names must be stable and predictable within the `cargo-dist` manifest so installers and external automation can reference them without heuristics.

Linux musl targets are not part of the required completion contract for this feature. They can be added later as a platform expansion, but the current feature is complete without musl.

## Installer scripts

The release must publish two canonical installer scripts as GitHub Release assets:

```text
wavepeek-installer.sh
wavepeek-installer.ps1
```

These scripts are canonical assets of the specific GitHub Release. Release notes must refer to them through `https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/...`.

Docs/Pages publication must additionally publish short installer entrypoints:

```text
/install.sh
/install.ps1
/X.Y.Z/install.sh
/X.Y.Z/install.ps1
```

These entrypoints must live at the site root and at the root of the specific version directory. For version `X.Y.Z`, pinned installer links must be exactly:

```text
https://kleverhq.github.io/wavepeek/X.Y.Z/install.sh
https://kleverhq.github.io/wavepeek/X.Y.Z/install.ps1
```

Do not hide them deeper under `assets/`, `downloads/`, `installers/`, or under release asset names. The purpose of Pages entrypoints is to provide short, memorable, stable URLs for README.

README must reference GitHub Pages entrypoints, not long GitHub Release download URLs. Release notes, conversely, must reference GitHub Release assets, not Pages entrypoints.

Pages entrypoints are byte-for-byte copies of canonical installer scripts and must install exactly the version implied by the URL:

- `/install.sh` and `/install.ps1` install the latest stable version;
- `/X.Y.Z/install.sh` and `/X.Y.Z/install.ps1` install only `X.Y.Z`;
- a versioned Pages entrypoint must not silently move to a newer version.

The shell installer must:

- detect OS and CPU architecture;
- choose the corresponding GitHub Release asset;
- download the archive;
- verify checksum if the installer has access to manifest/checksum data;
- install the binary into a user bin path;
- print a clear success instruction;
- not require a Rust toolchain.

The PowerShell installer must implement the analogous contract for Windows.

Installers must support latest release URLs and specific version URLs. Users must be able to install either the latest version or a pinned version.

GitHub Pages entrypoints are convenience aliases for README. They do not replace GitHub Release assets and must not become another source of truth. If Pages deployment has not happened, GitHub Release notes must still allow installing the release through GitHub Release URLs.

## Artifact attestations

Release assets should publish attestations where practical. Minimum attestation set:

```text
*.tar.gz
*.zip
*.sh
*.ps1
*.sha256
dist-manifest.json
```

Attestation does not replace checksum verification. Checksum verifies integrity of the downloaded file; attestation verifies artifact origin and its relationship to GitHub Actions build provenance.

Release notes must explain both verification methods briefly and practically.

## FSDB policy

FSDB is not included in binary releases.

Reasons:

- crate default features do not include FSDB;
- FSDB depends on Synopsys Verdi / FSDB Reader SDK;
- the FSDB build contract is limited to Linux x86_64;
- a public prebuilt FSDB binary would create implicit proprietary runtime/linking expectations;
- the user's machine would still need a correct Verdi installation.

Therefore public binaries are VCD/FST-only. FSDB remains a documented source-build path:

```bash
cargo install --locked wavepeek --features fsdb
```

README and release notes must state this distinction explicitly.

## Workflow contracts

### `release.yml`

`release.yml` is the primary tag-triggered workflow.

Trigger:

```yaml
on:
  push:
    tags:
      - v*.*.*
```

Workflow contract:

- accepts only SemVer tags of the form `vX.Y.Z`;
- validates that the tag matches `Cargo.toml` version;
- runs the existing release quality gate;
- runs `cargo package --locked` as source package validation;
- builds default-feature binaries;
- creates release assets;
- creates release notes from the changelog plus generated install/download/verification sections;
- publishes the GitHub Release;
- dispatches `docs.yml` and `publish-crate.yml` through explicit workflow dispatch;
- does not publish the crate to crates.io directly;
- does not deploy GitHub Pages directly.

Downstream dispatch must run only after successful GitHub Release publication. If downstream dispatch fails, the GitHub Release remains valid; the downstream workflow can be run manually with the same `version` and `source_ref`.

### `docs.yml`

`docs.yml` is manually dispatchable.

Inputs:

```yaml
version:
  required: true
  type: string
source_ref:
  required: true
  type: string
repair_existing_version:
  required: false
  default: false
  type: boolean
```

Workflow contract:

- tooling checkout is from the default branch;
- release docs source is taken from `source_ref`;
- `source_ref` must match `version`, meaning `source_ref == v${version}`;
- the GitHub Release for `source_ref` must exist and be published;
- stable docs publication must not require a new SemVer tag by default;
- repair of an existing version is allowed only with `repair_existing_version=true`;
- repair must not modify unrelated versions or unrelated paths;
- GitHub Pages publication runs through `actions/upload-pages-artifact` and `actions/deploy-pages`;
- the workflow must not rely on a push to `gh-pages` from `GITHUB_TOKEN` to trigger a Pages build;
- the workflow publishes Pages installer entrypoints `/install.sh`, `/install.ps1`, `/${version}/install.sh`, `/${version}/install.ps1`.

The `gh-pages` branch may remain a state/history branch for `mike` and reviewability. The actual site deployment must go through Pages deployment actions.

### `publish-crate.yml`

`publish-crate.yml` is manually dispatchable.

Inputs:

```yaml
version:
  required: true
  type: string
source_ref:
  required: true
  type: string
```

Workflow contract:

- tooling checkout is from the default branch;
- release source checkout is from `source_ref`;
- `source_ref` must match `version`, meaning `source_ref == v${version}`;
- the GitHub Release for `source_ref` must exist and be published;
- `Cargo.toml` package version in release source must match `version`;
- the workflow checks whether version `wavepeek` `X.Y.Z` is already published on crates.io;
- if the version is already published, the workflow exits successfully as a verified no-op;
- if the version is not published, the workflow requires `CRATES_IO_TOKEN` and runs `cargo publish --locked`;
- the workflow has no `repair_existing_version` because crates.io versions are immutable.

`publish-crate.yml` may use a protected environment `crates-io` if a manual approval gate is needed. The workflow contract does not change.

## Docs publication contract

Documentation must publish as a versioned GitHub Pages site.

Target result after successful `docs.yml`:

- a site for version `X.Y.Z` is published;
- alias `latest` points at the published version;
- shortest latest installer aliases are available at the Pages site root:
  - `/install.sh`;
  - `/install.ps1`;
- versioned installer aliases are available at the root of the specific version directory, without extra nesting, and are pinned entrypoints:
  - `/X.Y.Z/install.sh`;
  - `/X.Y.Z/install.ps1`;
- explicit latest installer aliases may exist under `/latest/install.sh` and `/latest/install.ps1`, but README should use the shorter root-level aliases;
- root artifacts such as versioned schema JSON are available on the Pages site;
- `versions.json` contains the expected version entry;
- unrelated versions are not modified;
- if the `gh-pages` branch is used, its state matches the deployed site;
- GitHub Pages deployment appears as a separate deployment with environment `github-pages`.

Repairing an existing version must be explicit. Without `repair_existing_version=true`, the workflow must fail if the version already exists in the published docs state.

## Crates.io publication contract

Crates.io publication is a side effect of an already-published GitHub Release.

Target result after successful `publish-crate.yml`:

- crate `wavepeek` version `X.Y.Z` is published on crates.io; or
- the workflow confirms that this version is already published and exits successfully without republishing.

The workflow must not create a new GitHub Release, change release assets, deploy docs, or move a tag. It owns only crates.io state.

## Failure semantics

Expected behavior on failures:

| Failure | Target behavior |
| --- | --- |
| Quality gate fails before GitHub Release | No public release is created. Downstream workflows are not dispatched. |
| Binary build fails before GitHub Release | No public release is created. Downstream workflows are not dispatched. |
| GitHub Release creation/upload fails | Downstream workflows are not dispatched. The version is not released until the GitHub Release succeeds. |
| GitHub Release succeeds, docs dispatch fails | The GitHub Release remains valid. `docs.yml` is run manually with the same inputs. |
| GitHub Release succeeds, crate dispatch fails | The GitHub Release remains valid. `publish-crate.yml` is run manually with the same inputs. |
| Docs publish fails | `docs.yml` is rerun. Replacing an existing version requires `repair_existing_version=true`. |
| Crates.io publish already happened | `publish-crate.yml` detects the existing version and exits success/no-op. |
| Crates.io publish fails ambiguously | Rerunning `publish-crate.yml` either publishes the version or detects that it is already published. |

This contract removes the need to issue a new SemVer tag to repair documentation or retry downstream publication.

## Security and permissions

The release system must separate permissions by task.

`release.yml` needs permissions for GitHub Release assets, workflow dispatch, and artifact attestations. These permissions must not automatically flow into docs deploy and crates.io publish jobs if those jobs do not need them.

`docs.yml` needs `contents: read` for staging, `contents: write` only if it preserves the `gh-pages` state branch, and `pages: write` plus `id-token: write` for actual deployment through GitHub Pages actions.

`publish-crate.yml` must not have `contents: write`. It needs repository read permissions and `CRATES_IO_TOKEN` for publication.

Release source checkout should use `persist-credentials: false` if the workflow must not push from that checkout.

Side workflows must validate dispatch inputs instead of trusting them. Minimum validation:

```text
version is SemVer X.Y.Z
source_ref == v${version}
source_ref resolves to a tag
GitHub Release for source_ref exists and is published
release source Cargo.toml version == version
```

## README requirements

README must change so the primary install path uses prebuilt binaries and short GitHub Pages URLs. The shortest snippet must install the latest stable version through the root-level Pages alias. Nearby, README must provide a pinned snippet for a specific version through the installer script at the root of that version directory on Pages.

README should not use GitHub Release download URLs as the primary install UX. Those URLs belong in release notes, where binding to a specific immutable GitHub Release matters.

Install section structure:

```text
Install
  ├─ Latest release: short command through https://kleverhq.github.io/wavepeek/install.sh
  ├─ Specific version: command through https://kleverhq.github.io/wavepeek/X.Y.Z/install.sh
  ├─ Windows latest/specific through install.ps1
  ├─ Cargo fallback
  └─ FSDB source-only install
```

Minimal latest snippet for macOS/Linux:

```bash
curl -LsSf https://kleverhq.github.io/wavepeek/install.sh | sh
```

Minimal latest snippet for Windows PowerShell:

```powershell
irm https://kleverhq.github.io/wavepeek/install.ps1 | iex
```

Minimal specific-version snippet for macOS/Linux:

```bash
curl -LsSf https://kleverhq.github.io/wavepeek/0.5.0/install.sh | sh
```

Minimal specific-version snippet for Windows PowerShell:

```powershell
irm https://kleverhq.github.io/wavepeek/0.5.0/install.ps1 | iex
```

README must contain both scenarios: installing the latest version and installing a specific version. The shortest install path must be the latest snippet through the root-level Pages alias. Pinned install must go through the script at the root of the specific version directory, for example `/0.5.0/install.sh`, not through a long GitHub Release download URL. The concrete version in the example must be a real published version, not an abstract placeholder.

Cargo install must no longer be the only or first install method for ordinary users.

FSDB must be documented separately and explicitly as a feature requiring Verdi.

## Release notes requirements

Release notes must be self-contained. A user should not need to search README to know which file to download.

Release notes must reference installer scripts through GitHub Release URLs, not GitHub Pages. For release notes, links must be tied to the specific published release.

The split is intentional: README optimizes for a short evergreen command, while release notes optimize for self-contained instructions for a concrete release. Even if GitHub Pages deployment has not run yet, release notes must allow installing exactly that release through GitHub Release assets.

Minimum release notes contents:

```text
What's changed
Installation
Platform assets
Checksums
Artifact attestation verification
FSDB note
Links to docs and crates.io
```

Installation snippets in release notes:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.sh | sh
```

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/kleverhq/wavepeek/releases/download/vX.Y.Z/wavepeek-installer.ps1 | iex"
```

The platform asset table must map OS/architecture, native `cargo-dist` asset name, and checksum link. Example:

```text
Linux x86_64        wavepeek-x86_64-unknown-linux-gnu.tar.gz
Linux arm64         wavepeek-aarch64-unknown-linux-gnu.tar.gz
macOS Intel         wavepeek-x86_64-apple-darwin.tar.gz
macOS Apple Silicon wavepeek-aarch64-apple-darwin.tar.gz
Windows x86_64      wavepeek-x86_64-pc-windows-msvc.zip
```

## Non-goals

This feature must not solve the following tasks:

- FSDB binary releases;
- bundling Synopsys/Verdi runtime libraries;
- Homebrew tap;
- Scoop manifest;
- WinGet package;
- Docker image;
- self-update mechanism;
- musl Linux binaries as a required readiness criterion;
- prerelease docs publication unless a separate explicit requirement appears.

## Feature readiness criteria

The feature is implemented when, for a new stable tag `vX.Y.Z`, the following contract holds:

- GitHub Release is created automatically from the tag;
- GitHub Release contains release notes;
- GitHub Release contains binaries for Linux x86_64, Linux arm64, macOS x86_64, macOS arm64, and Windows x86_64;
- GitHub Release contains checksum artifacts generated by `cargo-dist`;
- GitHub Release contains a shell installer;
- GitHub Release contains a PowerShell installer;
- release assets have provenance/attestation where supported;
- release notes use version-specific GitHub Release links for installer scripts;
- README shows a prebuilt-first install path through short GitHub Pages links;
- README contains examples for installing the latest version and a specific version;
- docs workflow runs after GitHub Release and publishes versioned GitHub Pages through Pages deployment actions;
- docs workflow publishes root latest aliases `/install.sh`, `/install.ps1` and version-root aliases `/X.Y.Z/install.sh`, `/X.Y.Z/install.ps1`;
- docs workflow can be run manually with the same inputs;
- crate publication workflow runs after GitHub Release;
- crate publication workflow can be run manually with the same inputs;
- rerunning the crate publication workflow for an already-published version exits successfully as a no-op;
- FSDB remains available through source build with `--features fsdb`, but is not included in prebuilt binaries.
