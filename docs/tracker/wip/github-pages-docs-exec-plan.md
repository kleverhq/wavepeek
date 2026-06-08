# Add versioned GitHub Pages documentation for wavepeek

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, maintainers can build and check a human-friendly web version of the same public documentation that ships inside the `wavepeek` CLI. A release publication will add an immutable documentation snapshot such as `/0.5.0/`, move `/latest/` to the newest stable release, redirect the site root to `/latest/`, and expose raw root artifacts at `/skill.md` and `/wavepeek_vX.json`.

The behavior is observable without publishing anything: from the repository root inside the wavepeek devcontainer, `just docs-site-check` prepares a generated MkDocs staging tree under `tmp/docs-site/`, builds the site in strict mode, and prepares raw root artifacts. Publication uses MkDocs Material for the site and `mike` to manage the cumulative `gh-pages` branch archive.

## Non-Goals

This plan does not backfill web snapshots before `0.5.0`. It does not add a `/dev/` snapshot for `main`. It does not replace docs.rs or Rust API documentation. It does not make `docs/public/` depend on MkDocs at runtime, and it does not make the GitHub Pages site a second authored source of truth. It also does not publish from local machines by default; local deploy commands must have an explicit push flag or workflow context before they update the remote `gh-pages` branch.

## Progress

- [x] (2026-06-07 20:14Z) The Russian proposal from `tmp/proposal.md` was translated to English, stored at `docs/tracker/wip/github-pages-docs-proposal.md`, and committed as `3722338 docs(tracker): add pages docs proposal` before this ExecPlan work began.
- [x] (2026-06-07 20:14Z) Initial repository orientation completed for `docs/public/`, `src/docs/mod.rs`, `tests/docs_cli.rs`, root `justfile`, `.devcontainer/Dockerfile`, `.github/workflows/`, and existing maintainer docs.
- [x] (2026-06-07 20:14Z) Initial ExecPlan drafted at `docs/tracker/wip/github-pages-docs-exec-plan.md`.
- [x] (2026-06-07 20:31Z) Initial review lanes completed for plan compliance, CI/deploy architecture, and docs/runtime compatibility; substantive findings were folded into this revision.
- [x] (2026-06-07 20:49Z) First control review pass completed; findings about deploy-token exposure, stale `gh-pages` state, and optional `see_also` were folded into this revision.
- [x] (2026-06-07 21:05Z) Second control review pass completed; findings about token/process isolation and pushable mutable checkouts were folded into this revision.
- [x] (2026-06-07 21:20Z) Third control review pass completed; finding about poisoned staging workspaces before the token-bearing push step was folded into this revision.
- [x] (2026-06-07 21:35Z) Fourth control review pass completed; findings about same-runner poisoning and push-side bundle verification were folded into this revision.
- [x] (2026-06-07 21:49Z) Fifth control review pass completed; finding about push-side semantic verification of version metadata was folded into this revision.
- [x] (2026-06-07 22:03Z) Sixth control review pass completed with no substantive findings.
- [x] (2026-06-08 05:38Z) User changed the metadata decision: current and future runtime/export JSON should use `description`, with `summary` retained only as a legacy input alias for old Markdown and historical exports.
- [x] (2026-06-08 05:46Z) Focused metadata-rename review completed; finding about updating the JSON schema artifact was folded into this revision.
- [x] (2026-06-08 05:55Z) Focused control review completed; findings about docs-search match reason naming and explicit schema contract checks were folded into this revision.
- [x] (2026-06-08 06:01Z) Second focused control review completed; findings about `docs show --summary` and the checked-in schema artifact flow were folded into this revision.
- [x] (2026-06-08 06:14Z) Third focused control review completed with no substantive findings.
- [x] (2026-06-08 06:21Z) User changed the docs publication trigger: docs must deploy only after a GitHub Release is published, not independently on tag push.
- [x] (2026-06-08 06:28Z) Focused release/docs trigger review completed; findings about explicit repair mode, trusted checkout refs, and release recovery dispatch were folded into this revision.
- [x] (2026-06-08 06:41Z) Focused control review completed; finding about normalizing boolean repair input before just recipes was folded into this revision.
- [x] (2026-06-08 06:44Z) Final focused control review completed with no substantive findings.
- [ ] Implementation has not started.

## Surprises & Discoveries

- Observation: The existing `v0.5.0` tag exists, but it predates the GitHub Pages infrastructure requested in this plan. A normal tag-triggered workflow cannot bootstrap that historical tag because the workflow and scripts are absent from the tag itself.
  Evidence: `git tag --list 'v*'` includes `v0.5.0`; the user confirmed the first publication must use the existing tag even though the site infrastructure was not present then.

- Observation: `docs/public/` currently authors front matter with `summary`, while the desired authored and runtime field is `description`. The current main branch already contains breaking changes and is expected to release as a future major version, so this plan should rename the current docs metadata field end-to-end instead of preserving `summary` in new JSON output.
  Evidence: `docs/public/intro.md` begins with `summary: Start here...`; `src/docs/mod.rs` has `FrontMatter { summary: String }` and `TopicSummary { summary: String }`; `tests/docs_cli.rs` asserts that JSON topic entries contain `summary`; the user explicitly requested the full rename on 2026-06-08.

- Observation: The current devcontainer Docker build context is effectively `.devcontainer/`, because `.devcontainer/Dockerfile` copies `env_contract.sh` without a directory prefix. Installing a root `requirements-docs.txt` into the image from a single source of truth requires either changing the build context or duplicating the file. This plan changes the build context deliberately to avoid duplicate dependency lists.
  Evidence: `.devcontainer/Dockerfile` contains `COPY env_contract.sh /wavepeek-env-contract.sh`; `.devcontainer/devcontainer.json` and `.devcontainer/devcontainer.ci.json` do not set `build.context` today.

- Observation: The historical `v0.5.0` tag uses the older schema artifact name `schema/wavepeek.json`, not the current versioned name `schema/wavepeek_v0.json`.
  Evidence: `git ls-tree -r --name-only v0.5.0 -- schema` lists `schema/wavepeek.json` and no `schema/wavepeek_v*.json` files.

## Decision Log

- Decision: Use MkDocs Material as the static site generator and `mike` as the versioned publishing tool.
  Rationale: The public docs are already topic-oriented Markdown, and `mike` directly supports cumulative version folders and aliases such as `latest` on a dedicated Pages branch.
  Date/Author: 2026-06-07 / Grin, confirmed by user.

- Decision: Add `requirements-docs.txt` at the repository root and install it into the devcontainer/CI image through a Python virtual environment.
  Rationale: The root requirements file is a simple canonical dependency list for non-container users, while installing it into the image keeps `just docs-site-check` fast and available in normal container workflows. A virtual environment avoids Ubuntu's externally managed system Python restrictions.
  Date/Author: 2026-06-07 / Grin, confirmed by user.

- Decision: Change the devcontainer build context to the repository root and add an allowlist-style `.dockerignore` so the Dockerfile can copy root `requirements-docs.txt` without sending `target/`, `.git/`, fixtures, and other large build-context inputs into the build context.
  Rationale: The alternative is duplicating docs requirements under `.devcontainer/`, which violates the single-source dependency goal. The allowlist keeps the larger context change controlled.
  Date/Author: 2026-06-07 / Grin.

- Decision: Publish the first `0.5.0` web snapshot from the existing `v0.5.0` tag using workflow code and scripts from the branch that contains this implementation.
  Rationale: The old tag must define the release docs content, but it cannot define the new publishing machinery because that machinery did not exist. A manual workflow dispatch must keep the working checkout on a trusted branch, normally the default branch after this implementation is merged, that contains `.github/workflows/docs.yml`, just recipes, and `tools/docs/`, while the publication wrapper checks out `v0.5.0` separately as the documentation source.
  Date/Author: 2026-06-07 / Grin, confirmed by user.

- Decision: Rename current docs metadata from `summary` to `description` end-to-end in authored Markdown, Rust runtime structs, docs-search match reason names such as `title_or_summary`, the `docs show --summary` CLI option, `wavepeek docs ... --json` output, export manifests, canonical JSON schema artifacts, docs, and tests. Keep `summary` only as a legacy input alias when reading old Markdown front matter or historical export manifests.
  Rationale: `description` fits MkDocs and web metadata better, and using one current field name avoids unnecessary drift. The current main branch already contains breaking changes and is expected to release as a future major version, so preserving `summary` in new public JSON is not required.
  Date/Author: 2026-06-08 / Grin, confirmed by user and focused review.

- Decision: Generate a temporary MkDocs config under `tmp/docs-site/` that inherits from a small root `mkdocs.yml` and contains generated `docs_dir`, `site_dir`, and `nav` values.
  Rationale: Navigation should come from the CLI export manifest, not a second hand-maintained list. Keeping the generated config in `tmp/` avoids committing site-only generated state.
  Date/Author: 2026-06-07 / Grin.

- Decision: Publish all files matching `schema/wavepeek_v*.json` from the release source checkout to the site root, and publish `docs/skills/wavepeek.md` from that same checkout as `skill.md`. For the legacy `v0.5.0` source only, if no `wavepeek_v*.json` files exist and `schema/wavepeek.json` exists for package major version `0`, copy it to the site root as `wavepeek_v0.json`.
  Rationale: The root artifacts must represent the latest stable release context, and the user explicitly requested all major schema versions rather than a hard-coded single schema. The existing `v0.5.0` tag predates the versioned schema filename, so the first bootstrap needs a narrow compatibility mapping to satisfy the required root URL without moving the historical tag.
  Date/Author: 2026-06-07 / Grin, confirmed by user and review findings.

- Decision: Use `mike set-default latest` for the root redirect behavior.
  Rationale: The user accepted an alias for `/latest/` and requested that `/` redirect to latest. `mike` owns version aliases and default-version redirect generation, so it should own this instead of a separate handwritten root page.
  Date/Author: 2026-06-07 / Grin, confirmed by user.

- Decision: Add `docs-site-check` to both `just check` and `just ci` after dependencies are present in the container image.
  Rationale: The user explicitly approved adding the docs-site check to CI. Adding it to the local pre-handoff gate keeps local and CI results aligned, and the check is lightweight compared with the existing Rust, coverage, and benchmark gates.
  Date/Author: 2026-06-07 / Grin.

- Decision: Treat missing `see_also` fields in exported manifest topic entries as empty lists during site preparation.
  Rationale: `TopicSummary` skips empty `see_also` vectors during serialization, so valid exports may omit the field. Requiring it would make the site generator reject valid CLI exports.
  Date/Author: 2026-06-07 / Grin, from control review.

- Decision: Restrict workflow deployment `source_ref` values to SemVer release tags and keep push credentials away from release-source build/check subprocesses.
  Rationale: The docs workflow compiles code from the selected release source. Limiting refs to release tags, disabling checkout credential persistence, and withholding push tokens from `cargo`, `mkdocs`, and `mike` subprocesses reduces the risk of an accidental or malicious ref using publication credentials.
  Date/Author: 2026-06-07 / Grin, from control review.

- Decision: Fetch and base local `gh-pages` state on `origin/gh-pages` before existing-version checks and deployment.
  Rationale: Cumulative version publishing depends on accurate remote state. A stale local branch could miss an existing snapshot and overwrite or push from the wrong branch state.
  Date/Author: 2026-06-07 / Grin, from control review.

- Decision: Split publication into a no-token staging step and a push-only step.
  Rationale: The staging step compiles and runs code from the selected release tag through `cargo run`. Push credentials must not be present in that process tree. A separate push-only step can receive credentials after all release-source build/check work is complete.
  Date/Author: 2026-06-07 / Grin, from second control review.

- Decision: Require a non-empty SemVer tag source ref for any operation whose result may be pushed.
  Rationale: Versioned publication is intended to create immutable release snapshots. Allowing `SOURCE_REF=""` with a push path would let a mutable current checkout publish a release snapshot.
  Date/Author: 2026-06-07 / Grin, from second control review.

- Decision: Run the credentialed push job from a fresh trusted checkout using only uploaded staging artifacts from the no-token staging job.
  Rationale: Release-source build code runs during staging and has workspace write access. The token-bearing push job must not trust scripts, just recipes, `.git` state, runner state, or metadata from that same mutable workspace except as uploaded data to verify and import.
  Date/Author: 2026-06-07 / Grin, from third and fourth control reviews.

- Decision: Verify staged `gh-pages` bundles against remote base state, version metadata semantics, and an allowed path set before pushing.
  Rationale: The push job receives generated data from a prior job. It must prove the bundle is a fast-forward update from the remote base observed during staging, does not rewrite an existing requested version unless an explicit repair mode is present, preserves unrelated version entries and snapshots, only adds the requested version while moving `latest` and the default to it, and changes only allowed paths.
  Date/Author: 2026-06-07 / Grin, from fourth and fifth control reviews.

- Decision: Start automatic docs publication only after the release workflow has successfully created the GitHub Release, not directly from tag push.
  Rationale: Tag push currently starts the binary release workflow. Starting docs from the same tag event creates independent deployments that can race: docs can publish before the binary release succeeds, or docs can fail while the binary release succeeds. A created GitHub Release is the durable signal that the crate release and user-facing release have completed. Because GitHub Releases created with the default `GITHUB_TOKEN` do not fire downstream `release` workflows, the release workflow should explicitly dispatch the trusted docs workflow after creating the GitHub Release.
  Date/Author: 2026-06-08 / Grin, confirmed by user and release/docs trigger review.

- Decision: Use manual docs workflow dispatch for docs repair and historical bootstrap without moving tags or creating new releases.
  Rationale: If docs scripts fail after the binary release has succeeded, maintainers should fix scripts on the trusted branch and rerun the docs publication for the same `vX.Y.Z` source tag. Tags are immutable release identity and should not be moved to repair documentation automation. If a version snapshot already exists and must be rewritten, the dispatch must set an explicit repair flag that is carried into staging metadata and push-side verification.
  Date/Author: 2026-06-08 / Grin, confirmed by user and release/docs trigger review.

- Decision: Keep the release workflow recoverable enough that GitHub Release creation can be retried after a partial crate publish.
  Rationale: Docs publication depends on the GitHub Release existing and on the release workflow dispatching docs after creating it. If `cargo publish` succeeds but GitHub Release creation fails, maintainers need a safe rerun path that verifies the crate version is already on crates.io and then creates the missing GitHub Release, rather than moving the tag or minting a new release tag. If the original tag-push workflow definition or helper scripts were broken, recovery must be available through a manual dispatch that uses trusted current tooling rather than the old tagged workflow.
  Date/Author: 2026-06-08 / Grin, from release/docs trigger review.

## Outcomes & Retrospective

No implementation outcome exists yet. The ExecPlan has completed focused review and control passes with no substantive findings after the metadata rename and release/docs trigger revisions. The current expected end state is a committed ExecPlan that a future agent or maintainer can execute without relying on this conversation. After implementation, this section must summarize which commands passed, whether the first `0.5.0` bootstrap path was dry-run locally, and any remaining GitHub repository settings needed for Pages.

## Context and Orientation

`wavepeek` is a Rust CLI. Its public user documentation source lives in `docs/public/`. Those Markdown files are embedded into the binary by `src/docs/mod.rs`, exposed through `wavepeek docs`, and exported by `wavepeek docs export`. This means the website must be generated from the exported embedded docs rather than from a separate authored website tree.

The embedded docs loader is in `src/docs/mod.rs`. It loads Markdown files from `docs/public/`, parses YAML front matter, validates that each file path matches its stable topic ID, validates `see_also` references, and currently exposes `TopicSummary` values with `id`, `title`, `summary`, `section`, and `see_also`. This plan renames that current runtime/export field to `description`, while accepting legacy `summary` only at input boundaries. The export path writes topic Markdown files and a `manifest.json` into an output directory. The manifest is the correct contract for site generation because it describes the exact docs embedded in that build.

The CLI docs behavior is tested in `tests/docs_cli.rs`, and additional runtime edge tests for `src/docs/mod.rs` live in `src/tests/docs_runtime_edges.rs`. The current docs CLI includes a `docs show --summary` option in `src/cli/docs.rs`; this plan renames that current user-facing option to `--description` and updates help, docs, and tests. Public docs style conventions currently live in `docs/dev/style.md`, which still describes `summary` front matter. That file must be updated when authored docs and runtime metadata move to `description`.

The packaged agent skill source is `docs/skills/wavepeek.md`. The canonical machine-output schema artifacts live under `schema/`, currently `schema/wavepeek_v0.json` on the working branch. The docs JSON output is covered by that schema, so the `summary` to `description` rename must update the schema artifact as well as code and tests. The current `wavepeek schema` command emits the checked-in schema artifact through `include_str!`, so implementation must update `schema/wavepeek_v0.json` directly or add a real generator before running `just check-schema`; `just update-schema` alone cannot derive the rename from Rust structs. The historical `v0.5.0` tag instead has `schema/wavepeek.json`; publication must normalize that legacy major-0 artifact to the root filename `wavepeek_v0.json`. These schema and skill files are not part of `wavepeek docs export`, but the web publication must expose them as raw static files at the site root.

Build and quality automation is owned by the root `justfile`. Recipes intentionally require `WAVEPEEK_IN_CONTAINER=1`, so new docs-site recipes should follow the same container-first pattern. Auxiliary Python tests are run by `just test-aux`, which currently discovers tests under `bench/e2e`, `bench/expr`, `tools/release`, `tools/coverage`, `tools/fsdb`, and `tools/repo`.

Local and CI containers are defined in `.devcontainer/Dockerfile`, `.devcontainer/devcontainer.json`, and `.devcontainer/devcontainer.ci.json`. The current Dockerfile has a `base` stage shared by the `ci` and `dev` stages. Installing MkDocs dependencies in `base` makes them available to both CI and local development. Because the Dockerfile currently copies files from the `.devcontainer/` directory, changing it to copy root `requirements-docs.txt` requires setting the devcontainer build context to the repository root and updating Dockerfile `COPY` paths.

GitHub Actions workflows currently live in `.github/workflows/ci.yml` and `.github/workflows/release.yml`. They run in the devcontainer baseline through `devcontainers/ci`. The existing release workflow is triggered by `push` tags matching `v*.*.*`; it publishes the crate and creates the GitHub Release. The new docs publication workflow should be separate, should grant write permission only to the push job that updates `gh-pages`, should be dispatched automatically by the release workflow only after the GitHub Release exists, and should also support manual dispatch for the initial historical `v0.5.0` bootstrap and docs repair.

Terminology used in this plan:

A MkDocs staging tree is a generated directory containing Markdown files arranged for website rendering. It is disposable and belongs under `tmp/docs-site/`.

A snapshot is an immutable published version folder such as `/0.5.0/` on GitHub Pages.

An alias is a stable path such as `/latest/` that points to one snapshot. `mike` manages aliases on the Pages branch.

Raw artifacts are static files served directly as Markdown or JSON, not rendered as HTML pages. In this project they are `/skill.md` and `/wavepeek_vX.json`.

The `gh-pages` branch is the dedicated publication branch that GitHub Pages serves. It contains generated HTML and raw artifacts, not source docs.

## Open Questions

There are no blocking product questions remaining from the user. One repository setting remains outside source control: after the workflow exists, a maintainer must ensure GitHub Pages is configured to serve the `gh-pages` branch from the repository root if it is not already configured that way.

## Plan of Work

The work should be implemented in small, independently verifiable slices. Do not skip the validation commands at the end of each slice. Generated files under `tmp/docs-site/` are disposable, but only remove paths owned by this plan, never arbitrary `tmp/` contents.

### Milestone 1: Rename docs metadata from summary to description

At the end of this milestone, `docs/public/` uses `description:` in front matter, and the CLI runtime, JSON output, and export manifest also use `description`. The old `summary` name is accepted only as a legacy input alias for historical Markdown fixtures or old exported manifests.

Edit `src/docs/mod.rs`. In the private front matter struct, rename the Rust field from `summary` to `description` and add serde attributes so it deserializes authored `description` and still accepts older `summary` files from historical releases and tests:

    #[serde(alias = "summary")]
    description: String,

Rename `TopicSummary.summary` and any related docs-search result fields to `description`. Also rename docs-search match reason values that embed the old term, especially `title_or_summary`, to `title_or_description`. Rename the current `wavepeek docs show --summary` option to `--description`, and update CLI help, public docs, and tests accordingly. New JSON output from `wavepeek docs topics --json`, `wavepeek docs search --json`, and `wavepeek docs export` manifests must use `description`, not `summary`, in both field names and match reason values.

Replace the top-level front matter key `summary:` with `description:` in every public topic under `docs/public/**/*.md`. Do not keep both keys. Preserve the text value. Keep `id`, `title`, `section`, and `see_also` unchanged.

Update docs and tests that describe or assert the authored front matter shape. At minimum, update `docs/dev/style.md` so it says public docs use `id`, `title`, `description`, `section`, and optional `see_also`. Update `docs/public/commands/docs.md`, `docs/public/reference/machine-output.md`, and any other public docs that mention docs-command JSON fields, docs-search match reason names, or `docs show --summary`. Update `src/cli/docs.rs`, `src/docs/mod.rs` inline tests, and `src/tests/docs_runtime_edges.rs` fixtures or assertions that deserialize front matter from literal YAML. Update `tests/fixtures/docs_embed/**/*.md` to use `description:` except where a test intentionally proves the legacy `summary` alias still works. Add one focused unit test in `src/docs/mod.rs` or `src/tests/docs_runtime_edges.rs` that deserializes a legacy `summary:` fixture and confirms it maps to the new `description` field. Add `tests/docs_cli.rs` assertions for both `docs topics --json` and `docs search --json` topic objects that `description` is present and `summary` is absent, that docs-search match reasons use `title_or_description` rather than `title_or_summary`, and that `docs show --description` works while `docs show --summary` is absent from current help. Update `schema/wavepeek_v0.json` directly, or add and use a real schema generator if one is introduced, so the checked-in artifact requires `description`, defines no current `summary` property for docs topic metadata, and exposes `title_or_description` rather than `title_or_summary`. Verify the result with `just check-schema`. Update `tools/schema/check_schema_contract.py` or an equivalent schema contract test so `just check-schema` fails if the topic summary schema still requires or defines `summary`, or if the docs-search match reason enum still contains `title_or_summary` instead of `title_or_description`.

Run these commands from the repository root inside the devcontainer:

    cargo test -q docs
    cargo test -q --test docs_cli
    just check-schema

Acceptance for this milestone is that `wavepeek docs topics --json` and `wavepeek docs search --json` include `description` and do not include `summary` in topic objects, docs-search match reasons use `title_or_description` and not `title_or_summary`, `docs show --description` works and `docs show --summary` is absent from current help, `wavepeek docs export` writes manifest topic entries with `description`, current exported Markdown preserves the new `description` front matter from current source, the canonical schema artifact and schema contract checker match the new `description` field and match reason names, and the legacy `summary` alias is covered by a test.

### Milestone 2: Add docs-site dependencies to the repository and container image

At the end of this milestone, `mkdocs`, `mike`, and their Python dependencies are available in both the CI and devcontainer targets without per-command installation. Non-container users can also see the dependency list at the repository root.

Add root `requirements-docs.txt` with the docs-site Python dependencies. Start simple and pin by compatible release series rather than an exact lock file:

    mkdocs-material~=9.6
    mike~=2.1
    PyYAML~=6.0

If implementation discovers that `mkdocs-material` or `mike` already pulls a compatible `PyYAML`, keeping the explicit `PyYAML` line is still acceptable because project helper scripts will import it directly.

Add a root `.dockerignore` because the devcontainer build context will move to the repository root. Use an allowlist pattern so the Docker build sees only files needed by `.devcontainer/Dockerfile`:

    *
    !.devcontainer/
    !.devcontainer/**
    !requirements-docs.txt

Edit `.devcontainer/devcontainer.json` and `.devcontainer/devcontainer.ci.json` so their `build` objects include `"context": ".."`. Then update `.devcontainer/Dockerfile` `COPY` statements that refer to files in `.devcontainer/` to include the `.devcontainer/` prefix. For example, the `env_contract` stage should copy `.devcontainer/env_contract.sh`, and the Verdi wrapper copy should use `.devcontainer/verdi-tool-wrapper.sh`.

In `.devcontainer/Dockerfile`, install `python3-venv` in the `base` apt package list. Copy `requirements-docs.txt` into the image, create a virtual environment such as `/opt/wavepeek-docs-venv`, install the requirements with its `pip`, and add `/opt/wavepeek-docs-venv/bin` to `PATH` before the `ci` and `dev` stages set their final `PATH`. Keep this in `base` so both targets inherit it.

Edit the `dev-setup` recipe in `justfile` to print or verify:

    mkdocs --version
    mike --version

Update `docs/dev/environment.md` to mention that the container image includes docs-site tooling from root `requirements-docs.txt`. Keep the wording short and operational.

Run these commands after rebuilding or within an updated devcontainer image:

    mkdocs --version
    mike --version
    just dev-setup

Acceptance for this milestone is that both commands are on `PATH` in the devcontainer, and `just dev-setup` verifies them without installing anything at recipe runtime.

### Milestone 3: Generate a MkDocs staging tree and generated config from `wavepeek docs export`

At the end of this milestone, a local command can export embedded docs, prepare a website source tree, generate navigation from the manifest, and run `mkdocs build --strict` without touching `docs/public/`.

Add a root `mkdocs.yml` as the committed base configuration. Keep it small and mostly static. It should include project metadata such as `site_name: wavepeek`, `repo_url: https://github.com/kleverhq/wavepeek`, `repo_name: kleverhq/wavepeek`, `theme.name: material`, the built-in `search` plugin, and Material versioning configuration such as `extra.version.provider: mike`. Do not put a hand-maintained full `nav` in this file.

Create `tools/docs/prepare_mkdocs.py`. Its command-line interface should accept an exported docs directory and options similar to:

    python3 -B tools/docs/prepare_mkdocs.py tmp/docs-site/export --output tmp/docs-site/mkdocs-src --config-output tmp/docs-site/mkdocs.yml --version 0.5.0 --force

The script must read `manifest.json`, validate that `kind` is `wavepeek-docs-export`, validate that `export_format_version` is the supported export format, validate that `cli_name` is `wavepeek`, validate `cli_version` against `--version` when a version is supplied, validate that `topics` is a list, and validate every topic entry before using it. Required topic fields are `id`, `title`, a description field, and `section`; current manifests must use `description`, while historical manifests such as `v0.5.0` may use legacy `summary`, which the script should map to `description` internally. The `see_also` field is optional in serialized manifests because empty vectors are omitted by `TopicSummary`; when it is missing, treat it as an empty list, and when it is present, require a list of safe topic IDs. Topic IDs must be safe slash-separated relative paths: no empty segments, no absolute paths, no `.` or `..` segments, and no path separators beyond `/`. Source and destination paths must be resolved and checked to stay inside the export and output roots. After validation, prepare the output directory atomically. If the output directory exists, replacement requires `--force`. The script may remove and recreate only the output paths it owns, such as `tmp/docs-site/mkdocs-src` and `tmp/docs-site/mkdocs.yml`.

For each topic in manifest order, copy its Markdown file from the export directory to the MkDocs source tree. The topic with ID `intro` must become `index.md`. Other topics keep their ID path plus `.md`, for example `commands/change` becomes `commands/change.md`. Exclude `manifest.json` from the MkDocs source tree.

Normalize front matter only in the generated staging file, not in `docs/public/`. If a topic has `summary` but no `description`, write `description` for the site front matter. If a topic already has `description`, keep it. Do not emit both keys. Preserve `id`, `title`, `section`, and `see_also` in staging front matter unless MkDocs strict mode proves a field is harmful. This keeps historical `v0.5.0` exports buildable even though they authored `summary`.

Generate navigation from the manifest. Use this section order and display labels:

    intro -> Introduction
    commands -> Commands
    workflows -> Workflows
    troubleshooting -> Troubleshooting
    reference -> Reference

Within each section, use the manifest order. The manifest is currently produced by `DocsCatalog::logical_topic_summaries` or its renamed equivalent after the metadata refactor, and that path sorts by CLI section rank and ID, so this avoids a second hand-maintained ordering system. For `intro`, create a single top-level `Introduction: index.md` nav entry. For all other sections, create grouped entries using each topic title and generated page path.

Write a generated config at `tmp/docs-site/mkdocs.yml` that inherits from the root `mkdocs.yml` and supplies generated paths and nav. Because the generated config is under `tmp/docs-site/`, the inherited path should point back to the repository root, for example `INHERIT: ../../mkdocs.yml` if the file is `tmp/docs-site/mkdocs.yml`. Set `docs_dir` and `site_dir` relative to the generated config, for example `docs_dir: mkdocs-src` and `site_dir: mkdocs-site`.

Add `tools/docs/test_prepare_mkdocs.py`. Unit tests should create a small fake export tree in a temporary directory, including `manifest.json`, `intro.md`, and at least one nested command topic. Tests must assert that `intro.md` becomes `index.md`, `manifest.json` is excluded, generated front matter uses `description` and not `summary`, nav contains the expected section grouping, missing `see_also` is treated as an empty list, and `--force` is required to replace an existing output directory. Add negative tests for unsupported export format versions, mismatched `cli_name`, mismatched `cli_version`, missing required topic fields other than `see_also`, unsafe topic IDs such as `../escape`, invalid `see_also` values, and source/destination path containment.

Run these commands from the repository root:

    cargo run --quiet -- docs export tmp/docs-site/export --force
    python3 -B tools/docs/prepare_mkdocs.py tmp/docs-site/export --output tmp/docs-site/mkdocs-src --config-output tmp/docs-site/mkdocs.yml --version 0.5.0 --force
    mkdocs build --strict --config-file tmp/docs-site/mkdocs.yml
    python3 -B -m unittest discover -s tools/docs -p "test_*.py"

Acceptance for this milestone is that `tmp/docs-site/mkdocs-src/index.md` exists, `tmp/docs-site/mkdocs-src/manifest.json` does not exist, `mkdocs build --strict` succeeds, and the generated site exists under `tmp/docs-site/mkdocs-site/`.

### Milestone 4: Add a publication wrapper for release sources, mike deployment, and root raw artifacts

At the end of this milestone, one helper can build from the current checkout or from a release ref such as `v0.5.0`, deploy a version snapshot with `mike`, set `latest`, set the root default redirect, and update raw root artifacts without rebuilding old snapshots.

Create `tools/docs/publish_docs.py`. It should be deterministic, non-interactive, and safe to run in CI. It should provide at least these subcommands:

    check: export docs, prepare MkDocs staging, run mkdocs build strict, and prepare root artifacts under tmp/docs-site/root-artifacts without touching gh-pages.
    stage-deploy: perform the check work, run mike deployment for a version against the local gh-pages branch, set latest/default aliases, commit root artifacts to the local gh-pages branch, write a small stage metadata file under tmp/docs-site/, and create a git bundle or equivalent data artifact containing only the staged gh-pages branch state.
    push-staged: from a fresh trusted checkout in a separate push job, verify the stage metadata and bundle, verify the update against the current remote gh-pages state and allowed path set, import the staged gh-pages branch state, and push it to origin. This subcommand must not run cargo, mkdocs, mike, or any command that uses SOURCE_REF.

The wrapper should accept `--version X.Y.Z`, `--source-root PATH`, `--source-ref REF`, and an explicit `--repair-existing-version` flag for `check` and `stage-deploy` where applicable. For any `stage-deploy` run, require `--source-ref` to be a non-empty SemVer release tag in the form `vX.Y.Z` and require it to match `--version`. `check` may accept `--source-root .` for local current-checkout validation, but any check intended to validate a publishable release should also use `--source-ref`. When `--source-ref` is provided, create a private worktree under `tmp/docs-site/source` at that tag and use it as the source root. This is required for the initial `v0.5.0` bootstrap, where current workflow scripts run from the trusted branch but docs content must come from the old tag. Remove only that owned worktree during cleanup.

The wrapper must run the export from the release source, not from the implementation checkout, with a command equivalent to:

    cargo run --quiet --manifest-path SOURCE_ROOT/Cargo.toml -- docs export tmp/docs-site/export --force

This matters because the binary embeds docs at compile time from `SOURCE_ROOT/docs/public`. The wrapper should validate that the release source `Cargo.toml` package version equals `--version`, including repair runs. Source refs used by deployment must be release tags; do not allow arbitrary branches or commit SHAs in the publishing workflow.

Root artifact preparation must read from the same release source root. Copy `SOURCE_ROOT/docs/skills/wavepeek.md` to `tmp/docs-site/root-artifacts/skill.md`. Copy every file matching `SOURCE_ROOT/schema/wavepeek_v*.json` to `tmp/docs-site/root-artifacts/` using the same basename. Sort matches for deterministic logging. If no versioned schema files match and the package major version is `0`, accept the legacy file `SOURCE_ROOT/schema/wavepeek.json` and copy it as `tmp/docs-site/root-artifacts/wavepeek_v0.json`; this fallback exists for the required `v0.5.0` bootstrap. Fail clearly if the skill file is missing or if neither versioned schema files nor the allowed legacy major-0 schema file exist.

`stage-deploy` should run the local strict build before invoking `mike`. It must run without push credentials in its environment. Use the generated config, not the root base config. Before checking existing versions or invoking `mike`, fetch the publication branch state with `git fetch origin gh-pages` when it exists, create or reset the local `gh-pages` branch from `origin/gh-pages`, and handle the absent-branch case as a first publication. Existing-version checks using `mike list` or `versions.json` must run after this fetch/reset so stale local state cannot miss an already-published snapshot. Use commands equivalent to:

    mike deploy --config-file tmp/docs-site/mkdocs.yml --branch gh-pages --remote origin --update-aliases VERSION latest
    mike set-default --config-file tmp/docs-site/mkdocs.yml --branch gh-pages --remote origin latest

Do not pass `--push` to `mike`. The preferred flow is: let `mike` update the local `gh-pages` branch, open a worktree for that branch under `tmp/docs-site/gh-pages`, copy `skill.md` and all `wavepeek_v*.json` files to the branch root, commit them if changed, write stage metadata such as `tmp/docs-site/staged-deploy.json`, and create `tmp/docs-site/gh-pages.bundle` or an equivalent data-only artifact for the staged `gh-pages` branch. The metadata must contain the version, source ref, branch name, remote base commit from `origin/gh-pages` or an explicit null first-publish marker, expected final commit, artifact names, and allowed changed path prefixes. If this preferred flow conflicts with `mike` behavior during implementation, document the discovered behavior in this ExecPlan and use the smallest safe alternative that still keeps the push job separate from build/check work and carries only data into the push job.

The wrapper must not delete old version directories on `gh-pages`. It should rely on `mike deploy VERSION latest --update-aliases` to add or replace only the requested `VERSION` and alias metadata. Replacing the same version requires the explicit `--repair-existing-version` flag. Without that flag, fail if the version already exists in `mike list` or `versions.json`, because silent snapshot rewrites would violate the archive stability requirement. When repair mode is enabled, stage metadata must record that fact, and push-side verification must still preserve unrelated versions and aliases while allowing only the requested version snapshot to be replaced.

Add `tools/docs/test_publish_docs.py`. Tests should avoid real network pushes and should not require `mike` to modify a real branch. Cover version extraction from tags, rejection of non-SemVer or empty source refs in `stage-deploy`, acceptance of `--source-root .` only in non-pushing `check`, Cargo version validation from a temporary `Cargo.toml`, source root raw artifact collection for current `wavepeek_v*.json` files, the legacy `schema/wavepeek.json` to `wavepeek_v0.json` fallback for major version `0`, failure when the skill file is missing, failure when no schema files exist, command construction for `mike deploy`/`set-default`, the required `gh-pages` fetch/reset preflight, stage metadata validation including the repair flag, bundle creation/import command construction, remote-base verification, fast-forward/no-force push behavior, rejection of bundles that change disallowed paths or unrelated version directories, rejection when requested `VERSION` already exists in the remote base without an explicit repair flag, acceptance of replacing only the requested version when the explicit repair flag is present, verification that unrelated `versions.json` entries and aliases are preserved, verification that only `VERSION` is added or repaired and `latest`/default move to it, and the fact that `push-staged` does not call cargo, mkdocs, mike, or use SOURCE_REF.

Run:

    python3 -B -m unittest discover -s tools/docs -p "test_*.py"
    python3 -B tools/docs/publish_docs.py check --version 0.5.0 --source-root .
    python3 -B tools/docs/publish_docs.py check --version 0.5.0 --source-ref v0.5.0

Acceptance for this milestone is that both check commands build a strict site, prepare `tmp/docs-site/root-artifacts/skill.md`, prepare at least `tmp/docs-site/root-artifacts/wavepeek_v0.json`, and do not touch the remote repository.

### Milestone 5: Add just recipes and keep quality gates aligned

At the end of this milestone, maintainers have stable root recipes for site build, serve, check, and deploy, and the standard gates run the docs-site check.

Edit the root `justfile`. Add variables for owned generated paths under `tmp/docs-site/`, such as export directory, MkDocs source directory, generated config, site output directory, and root artifact output directory. Add a version expression that reads `Cargo.toml` through Python and returns the package version, similar to the existing schema path expression.

Add recipes with names and behavior matching the proposal:

    docs-site-build VERSION=<current Cargo.toml version>

This exports docs from the current checkout, prepares the MkDocs staging tree with `--force`, and runs `mkdocs build --strict --config-file tmp/docs-site/mkdocs.yml`.

    docs-site-serve VERSION=<current Cargo.toml version>

This prepares staging the same way as build and runs `mkdocs serve --config-file tmp/docs-site/mkdocs.yml`. It is acceptable for this recipe to be interactive and therefore not used by CI.

    docs-site-check VERSION=<current Cargo.toml version>

This runs the same strict build as `docs-site-build`, then prepares root artifacts from the current checkout and verifies that `skill.md` plus at least one `wavepeek_v*.json` exist under `tmp/docs-site/root-artifacts/`.

    docs-site-deploy VERSION SOURCE_REF="" REPAIR="0"

This is a local non-pushing convenience wrapper. It should require an explicit version argument. If `SOURCE_REF` is empty, run `tools/docs/publish_docs.py check --version VERSION --source-root .` against the current checkout. If `SOURCE_REF` is set, run `tools/docs/publish_docs.py stage-deploy --version VERSION --source-ref SOURCE_REF`, adding `--repair-existing-version` only when `REPAIR` is truthy (`1` or `true`), without push credentials and leave the local `gh-pages` branch staged but unpushed.

    docs-site-stage-deploy VERSION SOURCE_REF REPAIR="0"

This is the CI staging entrypoint. It must require a non-empty SemVer release tag `SOURCE_REF` and run `tools/docs/publish_docs.py stage-deploy --version VERSION --source-ref SOURCE_REF`, adding `--repair-existing-version` only when `REPAIR` is truthy (`1` or `true`), without push credentials.

    docs-site-push-staged VERSION REPAIR="0"

This is the push-only entrypoint. It must run `tools/docs/publish_docs.py push-staged --version VERSION`, adding `--repair-existing-version` only when `REPAIR` is truthy (`1` or `true`), and must not run cargo, mkdocs, mike, or any source-ref checkout. In the workflow it must run in a separate push job from a fresh trusted checkout and consume only uploaded staged metadata and bundle artifacts as data. It may receive push credentials because all release-source build/check subprocesses have already completed in a different job.

Add `just docs-site-check` to both the `check` and `ci` aggregate recipes. Add `python3 -B -m unittest discover -s tools/docs -p "test_*.py"` to `test-aux`.

Run:

    just format-justfile-check
    just docs-site-check
    just test-aux

Acceptance for this milestone is that `just docs-site-check` leaves a complete generated site under `tmp/docs-site/mkdocs-site/`, leaves raw root artifacts under `tmp/docs-site/root-artifacts/`, and `just test-aux` runs the new `tools/docs` tests.

### Milestone 6: Gate docs publication on published GitHub Releases

At the end of this milestone, future docs snapshots publish only after the binary release has succeeded and a GitHub Release has been created. Maintainers can manually bootstrap `0.5.0` from the existing `v0.5.0` tag and can repair docs publication for an already-published release without moving tags or creating new release tags.

Keep `.github/workflows/release.yml` as the only workflow that triggers directly on `push` tags matching `v*.*.*`. Its normal tag-push order should remain: validate tag/version, run release quality gates, package, publish to crates.io, then create the GitHub Release. Do not create or publish the GitHub Release before the crate publish step succeeds. After the GitHub Release step succeeds, explicitly dispatch `.github/workflows/docs.yml` on the trusted default branch with `source_ref=vX.Y.Z`, `version=X.Y.Z`, and `repair_existing_version=false`. Use the repository `GITHUB_TOKEN` for this dispatch only if the workflow has the required `actions: write` permission; `workflow_dispatch` is the supported downstream event for `GITHUB_TOKEN`, unlike passive `release` events created by that token.

Add a manual `workflow_dispatch` recovery mode to the release workflow, or a dedicated release recovery workflow if that is clearer. The recovery mode must use trusted current workflow/helper code, validate an immutable `source_ref=vX.Y.Z`, verify that the tag's `Cargo.toml` version matches `X.Y.Z`, verify that the crate version is already present on crates.io, extract release notes from the tag source, and create the missing GitHub Release. After creating the missing GitHub Release, it must dispatch the docs workflow with `repair_existing_version=false`. This covers both transient GitHub Release creation failures and failures caused by old broken tagged workflow/helper code. It must fail if the crate version is not on crates.io, because docs publication should not bypass the binary release.

Create `.github/workflows/docs.yml`. Use a separate workflow from pre-merge CI and from release packaging. It should trigger on:

    workflow_dispatch with inputs source_ref, version, and repair_existing_version

Do not rely on `release: published` as the automatic trigger while GitHub Releases are created by `GITHUB_TOKEN`, because those token-created release events do not start downstream workflows. If the project later switches Release creation to a GitHub App token or PAT that intentionally triggers workflows, a `release: published` trigger can be added, but it must share the same validation path and avoid duplicate publication with the release-workflow dispatch.

The docs workflow must not trigger directly on tag push. A tag push starts the binary release workflow; the created GitHub Release plus explicit dispatch from the release workflow is the signal that docs may publish. If the binary release fails before the GitHub Release is created, docs do not run. If docs fail after the GitHub Release is created, maintainers fix the docs workflow/scripts on the trusted branch and run the docs workflow manually for the same `source_ref=vX.Y.Z` and `version=X.Y.Z`. If the version snapshot already exists and must be replaced, the manual dispatch must set `repair_existing_version=true`; otherwise the push-side verifier must reject rewriting that version.

For `workflow_dispatch`, default `source_ref` to `v0.5.0`, allow `version` to be optional, and add a boolean `repair_existing_version` input defaulting to `false`. Normalize that boolean to a simple `DOCS_REPAIR_EXISTING_VERSION` value accepted by just recipes, such as `1` for true and `0` for false, before invoking any just recipe. Require `source_ref` to match the release-tag pattern `v[0-9]+.[0-9]+.[0-9]+`. If `version` is empty, derive it by stripping the leading `v`; if `version` is present, require it to match the tag. Before entering the devcontainer, verify that a GitHub Release for `source_ref` exists, is published, is not a draft, and is not a prerelease. This preflight applies to automatic dispatch from the release workflow and to manual dispatch. If `v0.5.0` lacks a GitHub Release, run the release recovery path first or add a separately named emergency override with explicit maintainer approval; do not silently bypass the release gate.

For docs workflow dispatch runs, the Actions checkout used for workflow code, just recipes, and `tools/docs/` must be a trusted branch checkout, normally the repository default branch. Configure `actions/checkout` with an explicit `ref`, such as `${{ github.event.repository.default_branch }}`, and with full history and tags. It must not check out the release tag over the working tree before `devcontainers/ci`. Manual bootstrap and repair require the workflow file and scripts to exist on the trusted branch being checked out; after merge this should be the repository default branch. The publication wrapper is the only component that should check out `source_ref`, under `tmp/docs-site/source`, so docs content and root artifacts come from the release tag while publication scripts come from trusted current code. This is what makes repair possible after script bugs: fix scripts on the trusted branch, rerun manual dispatch for the same immutable tag.

Set workflow permissions at the workflow or job level so the staging job has only what it needs for checkout, artifact upload, and optional GHCR cache access, while the push job is the only job with `contents: write` for pushing `gh-pages`. Add `packages: write` only to jobs that write the GHCR devcontainer cache; if the docs workflow deliberately uses cache read-only mode, document that choice and omit `packages: write`. Add a concurrency group such as `docs-pages` so two docs publications cannot push over each other.

Use the same devcontainer baseline pattern as existing workflows: checkout trusted code with full history and tags, set up Docker Buildx, log in to GHCR for cache when allowed, create `.devcontainer/.devcontainer.json` as a symlink to `devcontainer.ci.json`, print Dockerfile provenance, and run the docs staging command through `devcontainers/ci`. Configure `actions/checkout` with `persist-credentials: false` so an input-selected release source cannot read stored push credentials while `cargo run` builds the release binary. Pass only `DOCS_VERSION`, `DOCS_SOURCE_REF`, and `DOCS_REPAIR_EXISTING_VERSION` into the container through the devcontainers action `env:` forwarding or interpolate their resolved values directly into `runCmd`. Do not pass `GITHUB_TOKEN`, `GH_TOKEN`, `DOCS_PUSH_TOKEN`, or equivalent push credentials to the devcontainer staging step.

The in-container staging run command should configure a CI git identity and then call:

    just docs-site-stage-deploy "$DOCS_VERSION" "$DOCS_SOURCE_REF" "$DOCS_REPAIR_EXISTING_VERSION"

After that devcontainer step succeeds, upload the staged deploy metadata and `gh-pages` bundle as a workflow artifact. Add a separate push-only job, not merely a later step in the same job. The push job must start from a fresh clean checkout of the trusted workflow ref, not from the staging workspace, and must download only the staged data artifact from the staging job. That push job may receive `GITHUB_TOKEN`, should verify `staged-deploy.json`, verify the bundle ref and expected final commit, fetch current `origin/gh-pages`, verify that the remote base commit still matches the metadata or that the metadata explicitly marks first publish, verify the staged commit is a fast-forward update, reject the push if requested `VERSION` already exists in the remote base unless the explicit repair input is true and the staged metadata also records repair mode, compare remote-base and staged `versions.json` so unrelated version entries and aliases are preserved, verify the only semantic version metadata changes are adding or repairing `VERSION` and moving `latest` plus the default to it, verify that changed paths are limited to the requested version directory, `latest` alias/default files, mike metadata such as `versions.json`, `.nojekyll`, and root `skill.md` plus `wavepeek_v*.json`, and then push without force. It may run either a second devcontainer invocation that calls:

    just docs-site-push-staged "$DOCS_VERSION" "$DOCS_REPAIR_EXISTING_VERSION"

or an equivalent direct runner command:

    python3 -B tools/docs/publish_docs.py push-staged --version "$DOCS_VERSION" --metadata PATH/staged-deploy.json --bundle PATH/gh-pages.bundle [--repair-existing-version]

This push-only job must not run cargo, mkdocs, mike, any script from the staging workspace, or any command that checks out or executes `SOURCE_REF`.

The workflow must publish root raw artifacts from the same source ref as the docs snapshot. For the initial manual dispatch, that means the checkout used by Actions remains on the trusted branch containing this implementation, normally the default branch after merge, while source content comes from `v0.5.0` through `tools/docs/publish_docs.py --source-ref`. For future automatic publication, the release workflow dispatch supplies the release tag as `source_ref`, but workflow code and scripts still come from the trusted checkout described above.

Run:

    just check-actions
    python3 -B -m unittest discover -s tools/release -p "test_*.py"

Acceptance for this milestone is that `actionlint` passes, any release recovery helper tests pass, the docs workflow has no tag-push trigger, the release workflow dispatches the docs workflow only after creating the GitHub Release, the docs workflow preflights that the GitHub Release exists and is stable, trusted checkouts use explicit refs rather than accidentally checking out the release tag, the release recovery path can create a missing GitHub Release from trusted current tooling after verifying crates.io already has the version and then dispatch docs, and the workflow clearly shows how to publish `0.5.0` or repair a failed docs publication from manual dispatch without requiring the old tag to contain `.github/workflows/docs.yml`.

### Milestone 7: Update maintainer docs and final validation

At the end of this milestone, contributors know how to build, serve, check, and publish the docs site, and all standard gates pass.

Update maintainer docs under `docs/dev/`. The likely files are `docs/dev/environment.md`, `docs/dev/quality.md`, `docs/dev/automation.md`, and `docs/dev/release.md` if it documents release ordering. Keep these updates concise. They should state that docs-site Python dependencies live in `requirements-docs.txt`, are installed into the devcontainer image, `just docs-site-check` is part of the standard gates, and publication is handled by `.github/workflows/docs.yml` plus the split `just docs-site-stage-deploy` and `just docs-site-push-staged` flow. They should also state that automatic docs publication starts from a published GitHub Release, not from tag push, and that manual docs dispatch is the repair path for a release tag whose docs failed.

If implementation changes source module boundaries beyond the front matter alias in `src/docs/mod.rs`, update `docs/dev/architecture.md`. If only helper scripts and docs metadata change, architecture updates are probably unnecessary.

Run the final validation from the repository root inside the devcontainer:

    just docs-site-check
    python3 -B tools/docs/publish_docs.py check --version 0.5.0 --source-ref v0.5.0
    just check
    just ci

If `just ci` is too expensive during early iteration, run the focused commands first and record the skipped full gate in `Progress`; before handoff, run `just ci` unless the user explicitly waives it.

Acceptance for the full plan is:

`docs/public/` authors `description` front matter, while `wavepeek docs topics --json`, `wavepeek docs search --json`, export manifests, and the canonical schema artifact expose `description`, not `summary`.

`just docs-site-check` builds a strict MkDocs Material site from `wavepeek docs export` and prepares raw root artifacts.

The generated staging source has `index.md` for the intro topic and does not commit generated files.

`tools/docs/publish_docs.py check --version 0.5.0 --source-ref v0.5.0` proves the historical-tag bootstrap path can export and build from the existing tag.

`.github/workflows/docs.yml` can publish snapshots for published GitHub Releases with `mike`, move `latest`, set the root default redirect, and update root `skill.md` plus every `wavepeek_v*.json` from the release source using a no-token staging step followed by a push-only step. It must not trigger directly on tag push.

`just ci` passes.

## Concrete Steps

The implementation agent should proceed in this order from the repository root:

1. Update front matter compatibility, docs metadata, CLI option names, and the JSON schema artifact.

    cargo test -q docs
    cargo test -q --test docs_cli
    just check-schema

2. Add root docs dependencies and container installation, then rebuild or enter an updated container.

    mkdocs --version
    mike --version
    just dev-setup

3. Add `mkdocs.yml`, `tools/docs/prepare_mkdocs.py`, and tests.

    cargo run --quiet -- docs export tmp/docs-site/export --force
    python3 -B tools/docs/prepare_mkdocs.py tmp/docs-site/export --output tmp/docs-site/mkdocs-src --config-output tmp/docs-site/mkdocs.yml --version 0.5.0 --force
    mkdocs build --strict --config-file tmp/docs-site/mkdocs.yml
    python3 -B -m unittest discover -s tools/docs -p "test_*.py"

4. Add `tools/docs/publish_docs.py` and tests.

    python3 -B -m unittest discover -s tools/docs -p "test_*.py"
    python3 -B tools/docs/publish_docs.py check --version 0.5.0 --source-root .
    python3 -B tools/docs/publish_docs.py check --version 0.5.0 --source-ref v0.5.0

5. Add just recipes and wire gates.

    just format-justfile-check
    just docs-site-check
    just test-aux

6. Add the docs workflow, release workflow recovery behavior, and maintainer docs updates.

    just check-actions
    python3 -B -m unittest discover -s tools/release -p "test_*.py"

7. Run final gates.

    just docs-site-check
    python3 -B tools/docs/publish_docs.py check --version 0.5.0 --source-ref v0.5.0
    just check
    just ci

## Validation and Acceptance

A human can verify local behavior by running `just docs-site-check` inside the devcontainer. The command must exit successfully. After it runs, these files should exist:

    tmp/docs-site/mkdocs-src/index.md
    tmp/docs-site/mkdocs-site/index.html
    tmp/docs-site/root-artifacts/skill.md
    tmp/docs-site/root-artifacts/wavepeek_v0.json

The generated MkDocs source must not contain `manifest.json`. The generated intro page should be `index.md`, not `intro.md`. Current generated topic front matter should contain `description`, not `summary`.

A human can verify the historical release path without pushing by running:

    python3 -B tools/docs/publish_docs.py check --version 0.5.0 --source-ref v0.5.0

That command must export docs by compiling the release source from `v0.5.0`, build a strict site using the current scripts, and prepare root artifacts from the `v0.5.0` source checkout.

A human can verify the CLI contract by running:

    cargo run --quiet -- docs topics --json
    cargo run --quiet -- docs search --json docs

The JSON topic entries for both `docs topics --json` and `docs search --json` must contain `description` and must not contain `summary`, because `description` is the current runtime contract field after this refactor. Docs-search match reasons must use `title_or_description`, not `title_or_summary`. `wavepeek docs show --description intro` must work, and `wavepeek docs show --help` must not list `--summary`.

The final repository gates are:

    just check
    just ci

Both must pass before handoff unless the user explicitly accepts a narrower validation result.

## Idempotence and Recovery

All local generated outputs must live under `tmp/docs-site/`, which this plan owns. Scripts may remove and recreate their own subpaths under `tmp/docs-site/`, but they must not clean the entire `tmp/` directory.

`wavepeek docs export ... --force` is safe only for empty or managed export roots. The docs-site recipes should always use owned paths under `tmp/docs-site/` so `--force` cannot replace user files.

The MkDocs staging script should use a temporary sibling directory and atomic rename when replacing an output directory. If a run fails halfway, rerun the same command with `--force`; the script should clean only its own temporary path.

The publish wrapper should separate staging from pushing. `check` must not touch `gh-pages`. `stage-deploy` may update only local `gh-pages` state, stage metadata, and a staged branch bundle, and it must run without push credentials. `push-staged` is the only subcommand that may push. In CI it must run in a separate push job from a fresh trusted checkout and consume only uploaded metadata plus the bundle from staging; it must not rebuild, restage, check out `SOURCE_REF`, run cargo, mkdocs, or mike, or execute scripts from the staging workspace.

When a `stage-deploy` run starts, fetch `origin/gh-pages` and base local work on the fetched remote branch. If the remote branch does not exist, treat the run as the first publication and let `mike` initialize it. Record the fetched base commit or first-publish marker in metadata. When staging fails after `mike` updates a local branch but before push, inspect the local `gh-pages` branch and rerun the staging command; the rerun must fetch/reset from remote before checking existing versions. If the version already exists remotely and the rerun is intentionally repairing the same version, require the explicit repair flag described in Milestone 4. Otherwise do not overwrite existing snapshots. The push-only job must verify stage metadata, remote base, fast-forward ancestry, requested-version absence unless repair mode is explicit, `versions.json` semantic changes, changed paths, and the bundle in a fresh trusted checkout before pushing without force, and it must not rebuild or restage.

For the first `0.5.0` bootstrap, use manual workflow dispatch with `source_ref=v0.5.0` and `version=0.5.0`. If it fails before pushing, fix the workflow/scripts on the trusted branch and rerun dispatch; do not move the historical tag. For future releases, docs publication is repairable the same way after the GitHub Release exists: fix trusted docs scripts and manually dispatch the docs workflow for the same release tag. If the binary release workflow publishes the crate but fails before creating the GitHub Release, rerun the release workflow through its recovery path so it creates the missing GitHub Release; do not start docs manually until the GitHub Release exists unless a maintainer explicitly decides to bypass the release gate for an emergency repair.

## Artifacts and Notes

The proposal that motivated this plan is tracked at:

    docs/tracker/wip/github-pages-docs-proposal.md

It was committed before this ExecPlan as:

    3722338 docs(tracker): add pages docs proposal

Expected generated local layout after `just docs-site-check`:

    tmp/docs-site/export/          exported embedded CLI docs plus manifest.json
    tmp/docs-site/mkdocs-src/      generated MkDocs Markdown source
    tmp/docs-site/mkdocs.yml       generated MkDocs config with nav
    tmp/docs-site/mkdocs-site/     strict MkDocs HTML build output
    tmp/docs-site/root-artifacts/  generated skill.md and wavepeek_v*.json root files

Expected public Pages layout after publishing `0.5.0`:

    /latest/          alias managed by mike
    /0.5.0/           version snapshot managed by mike
    /                 default redirect managed by mike
    /skill.md         raw latest stable packaged skill
    /wavepeek_v0.json raw latest stable schema for major version 0

## Interfaces and Dependencies

The Rust interface change is centered in `src/docs/mod.rs` and `src/cli/docs.rs`. The private front matter type must deserialize current `description` and legacy `summary` into a `description` field. Public `TopicSummary`, docs-search JSON structs, docs-search match reason names, the `docs show --description` CLI option, export manifest topic entries, and the docs-topic portion of the canonical schema artifact must expose `description` instead of `summary` in current output. The schema contract checker must enforce this so stale checked-in schema cannot pass review.

The Python dependency interface is root `requirements-docs.txt`. It must include MkDocs Material, mike, and PyYAML. The devcontainer image must expose `mkdocs` and `mike` on `PATH` in both `ci` and `dev` targets.

`tools/docs/prepare_mkdocs.py` must expose a CLI that accepts an export directory, output directory, generated config path, version string, and `--force`. It must also be structured so its pure functions can be unit-tested without invoking MkDocs.

`tools/docs/publish_docs.py` must expose `check`, `stage-deploy`, and `push-staged` subcommands. `check` accepts `--version`, `--source-root`, and optionally `--source-ref`. `stage-deploy` requires `--version` and a non-empty SemVer `--source-ref`, accepts `--repair-existing-version` only when explicitly requested, and writes metadata plus a staged `gh-pages` bundle. `push-staged` accepts `--version`, `--metadata`, `--bundle`, and optional `--repair-existing-version`, validates both inputs, imports the staged `gh-pages` state into the fresh checkout, fetches current `origin/gh-pages`, verifies remote base and fast-forward ancestry, rejects existing requested versions unless repair mode is explicit, verifies allowed `versions.json` semantic changes, checks changed paths against the allowed set, and pushes without force. Subprocess invocation must stay in small functions so tests can mock command execution.

The just interface must include these recipes:

    just docs-site-build
    just docs-site-serve
    just docs-site-check
    just docs-site-deploy VERSION=0.5.0 SOURCE_REF=v0.5.0 REPAIR=0
    just docs-site-stage-deploy VERSION=0.5.0 SOURCE_REF=v0.5.0 REPAIR=0
    just docs-site-push-staged VERSION=0.5.0 REPAIR=0

The GitHub Actions interface is `.github/workflows/docs.yml`, with `workflow_dispatch` used both by the release workflow after GitHub Release creation and by maintainers for `v0.5.0` bootstrap or docs repair for an existing release tag. Manual dispatch includes an explicit `repair_existing_version` input. The release workflow remains `.github/workflows/release.yml`, triggered by tag push and responsible for publishing the crate before creating the GitHub Release signal and dispatching docs; it also needs a trusted-code manual recovery path for creating a missing GitHub Release after verifying the crate version already exists on crates.io.

## Revision Notes

2026-06-07 20:31Z: Incorporated initial review findings. The plan now handles the legacy `v0.5.0` schema filename by normalizing `schema/wavepeek.json` to `wavepeek_v0.json`, requires stronger manifest/path validation in the MkDocs preparation script, makes manual-dispatch checkout behavior explicit, requires explicit CI env forwarding or interpolation, covers GHCR cache permissions, added then-current CLI JSON field checks, and removes non-neutral phrasing from repository artifacts.

2026-06-07 20:49Z: Incorporated first control-pass findings. The plan now treats missing manifest `see_also` as an empty list, requires release-tag-only `source_ref` values for deploy workflow paths, prevents checkout credential persistence, requires push tokens to be withheld from build/check subprocesses that execute release-source code, and requires fetching/resetting local `gh-pages` from `origin/gh-pages` before existing-version checks and deployment.

2026-06-07 21:05Z: Incorporated second control-pass findings. The plan now splits publication into a no-token `stage-deploy` step and a separate `push-staged` step, and requires any pushable publication path to use a non-empty SemVer release tag rather than a mutable current checkout.

2026-06-07 21:20Z: Incorporated third control-pass findings. The plan now requires the credentialed push step to run from a fresh trusted checkout and consume only staged metadata plus a `gh-pages` bundle produced by the no-token staging step.

2026-06-07 21:35Z: Incorporated fourth control-pass findings. The plan now requires staging and pushing to run as separate workflow jobs, limits `contents: write` to the push job, and requires push-side verification of remote base, fast-forward ancestry, bundle commit, and path-limited changes before pushing.

2026-06-07 21:49Z: Incorporated fifth control-pass findings. The plan now requires push-side semantic verification of `versions.json`, independent rejection of existing requested versions unless repair mode is explicit, and preservation of unrelated version entries and aliases.

2026-06-07 22:03Z: Recorded sixth control pass. The reviewer reported no substantive findings, so the plan was ready for handoff or implementation at that point.

2026-06-08 05:38Z: Updated the metadata direction after user clarification. The plan now requires a full current-field rename from `summary` to `description` across authored docs, Rust runtime structs, CLI JSON, export manifests, docs, and tests, while keeping `summary` only as a legacy input alias for old Markdown and historical manifests.

2026-06-08 05:46Z: Incorporated focused metadata-rename review. The plan now requires updating and checking the canonical schema artifact so docs-topic JSON schema uses `description` and does not continue to require `summary`.

2026-06-08 05:55Z: Incorporated focused control-review findings. The plan now explicitly renames docs-search match reason values such as `title_or_summary` to `title_or_description`, and requires an explicit schema contract check so stale schema definitions cannot continue to require `summary` or expose old match reason names.

2026-06-08 06:01Z: Incorporated second focused control-review findings. The plan now renames the current `docs show --summary` option to `--description`, updates docs/tests for that CLI surface, and states that the checked-in schema artifact must be edited directly or generated by a newly introduced generator before `just check-schema` can verify it.

2026-06-08 06:14Z: Recorded third focused control pass after the metadata/schema fixes. The reviewer reported no substantive findings.

2026-06-08 06:21Z: Updated the release/docs trigger model after user clarification. The plan now requires automatic docs publication to start only after a GitHub Release is created rather than directly from tag push, keeps tag push exclusively for the binary release workflow, and adds a recoverable path for docs script failures and partial crate-publish/GitHub-Release failures.

2026-06-08 06:28Z: Incorporated focused release/docs trigger review. The plan now requires explicit `repair_existing_version` flow for rewriting an already-published docs snapshot, explicit trusted-branch checkout refs for docs automation, and a manual release recovery path using trusted current tooling when the original tag-push release workflow cannot create the missing GitHub Release.

2026-06-08 06:41Z: Incorporated focused control-review finding. The plan now requires workflow boolean repair input to be normalized before invoking just recipes, and the recipes treat `1` or `true` as repair mode.

2026-06-08 06:44Z: Recorded final focused control pass after the release/docs dispatch fixes. The reviewer reported no substantive findings.
