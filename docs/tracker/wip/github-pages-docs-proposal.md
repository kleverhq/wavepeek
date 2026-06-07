# Proposal: Versioned wavepeek docs on GitHub Pages

## Purpose

This document describes the desired direction for integrating the public `wavepeek` documentation with GitHub Pages. It is not an exhaustive specification and should not block simpler or cleaner technical solutions if they preserve the overall intent: the public docs remain part of the CLI, and the website becomes a convenient web presentation of the same documentation corpus.

The target model is to design publication as **versioned and cumulative** from the start. It must be possible to publish the current version, `0.5.0`, and to add new snapshots seamlessly for later SemVer releases without breaking already published versions.

Backfilling releases before `0.5.0` is not required. The starting point for the web archive is the current version, `0.5.0`.

## Current Context

The repository already has a separate public documentation corpus under `docs/public/`. These Markdown files are embedded into the CLI and are available to users through the `wavepeek docs` command. This is an important architectural constraint: the website documentation must not become a separate independent source of truth.

The CLI can already export embedded documentation through `wavepeek docs export`. This export provides a useful contract layer between runtime documentation and the site generator: what is published on the website for version `X.Y.Z` must correspond to what is actually embedded in the CLI/crate for version `X.Y.Z`.

Two additional public artifacts are important:

```text
docs/skills/wavepeek.md   # packaged agent skill
schema/wavepeek_vX.json   # canonical machine-output schema
```

They must be published from the site root as raw text:

```text
/skill.md
/wavepeek_vX.json
```

These files must be directly available as text artifacts, not only as pages inside the HTML documentation. All major schema versions must be published.

## Desired Result

The final result should look approximately like this:

```text
docs/public/                 # source of truth for CLI documentation
docs/skills/wavepeek.md      # source of truth for raw skill.md
schema/wavepeek_vX.json      # source of truth or checked/generated schema artifact
mkdocs.yml                   # site configuration
tools/docs/prepare_mkdocs.py # prepares the staging corpus for MkDocs
tools/docs/publish_docs.py   # optional wrapper for versioned deploy/root artifacts
.github/workflows/docs.yml   # GitHub Pages publication
```

The published website should be convenient for humans: it should provide section navigation, search, correct page titles, repository links, and a clean presentation of the public documentation. It should not require manual duplication of content, navigation, or metadata where those can be derived from the existing topic files or export manifest.

The desired published site structure is:

```text
/                 # entry point, usually redirect/default to latest
/latest/          # alias for the latest published stable release
/0.5.0/           # immutable documentation snapshot for wavepeek 0.5.0
/0.6.0/           # future snapshot, added at the 0.6.0 release
/skill.md         # raw skill Markdown for latest/current stable
/wavepeek_vX.json # raw JSON schema for major version X
```

The exact URL aliases can be refined during implementation, but two ideas must be preserved: versions accumulate, and root-level raw artifacts are available at stable paths.

## Main Approach

The recommended approach is to generate the website through an intermediate staging directory instead of making MkDocs work directly with `docs/public/`.

An approximate local build flow is:

```bash
cargo run --quiet -- docs export tmp/mkdocs-src --force
python3 tools/docs/prepare_mkdocs.py tmp/mkdocs-src --version 0.5.0
mkdocs build --strict
```

Staging is useful for several reasons:

- `docs/public/intro.md` should remain `intro.md`, because that is a stable topic ID for the CLI, but on the website it should naturally become `index.md`.
- `manifest.json`, generated navigation, and other site-only artifacts must not be written back into the CLI docs corpus.
- The website may need small presentation-layer transformations that should not pollute the source format of embedded documentation.
- Versioned publication should take the snapshot from the release/tag context, not accidentally from the current `main` checkout.

`docs/public/` remains the authored source. `tmp/mkdocs-src/` or an equivalent directory is disposable generated output.

## MkDocs Material and Versioned Publishing

MkDocs Material is a natural choice for this project. The `wavepeek` documentation is already organized as topic-oriented Markdown files: introduction, commands, workflows, troubleshooting, and reference. This maps well to a static site with navigation, search, and metadata-driven pages.

Versioned accumulation should be designed in from the start. In practice, this can be implemented with `mike` or an equivalent publication flow on a dedicated Pages branch. The specific tool is less important than the behavior:

```text
release v0.5.0 -> publish /0.5.0/, latest points to 0.5.0
release v0.6.0 -> publish /0.6.0/, latest moves to 0.6.0, /0.5.0/ remains available
release v0.6.1 -> publish /0.6.1/, latest moves to 0.6.1, old snapshots remain available
```

Old snapshots must not be rebuilt and overwritten on every new release. The exception is an explicit repair operation for a specific version.

Rustdoc remains appropriate for API documentation and docs.rs, but it does not replace product CLI documentation. The GitHub Pages site should be the user-facing web presentation of `docs/public`, not a replacement for crate API documentation.

## Versions and Release Policy

The canonical version path must use the full SemVer without the `v` prefix:

```text
/0.5.0/
/0.6.0/
/0.6.1/
```

Git tags may remain in the usual form:

```text
v0.5.0
v0.6.0
v0.6.1
```

The publication workflow must be able to extract the version from a release tag or from `Cargo.toml` and use it as the snapshot name. For the current state, it must support publishing `0.5.0` as the first version of the web archive.

Backfilling versions before `0.5.0` is out of scope. If earlier versions need to be restored later, that should be handled by a separate decision that does not complicate this integration.

An optional `/dev/` snapshot for `main` is not required.

## Root Raw Artifacts

The root of the published site must expose raw files:

```text
/skill.md
/wavepeek_vX.json # one file for each major version
```

`skill.md` must correspond to the packaged skill for the current/latest stable release. Its source is `docs/skills/wavepeek.md` from the same release context that publishes the documentation.

`wavepeek_vX.json` must correspond to the latest canonical machine-output schema for major release X. Its source is `schema/wavepeek_vX.json`.

These root-level files must be updated when a new stable release is published, together with the `latest` alias. Versioned snapshots may additionally contain copies of these artifacts under `/X.Y.Z/` if that is convenient for the implementation, but the required behavior is stable root paths.

Raw artifacts must be served as ordinary static files, not MkDocs-rendered HTML pages. A user or agent must be able to fetch them directly by URL and use them as Markdown or JSON.

## Front Matter

The current front matter is already close to the desired format. The authored metadata should preferably be adjusted to a more MkDocs/Material-compatible shape without turning `docs/public` into a MkDocs-specific format.

The recommended canonical front matter is:

```yaml
---
id: commands/change
title: Change command
description: Inspect transitions over a time range.
section: commands
see_also:
  - reference/expression-language
---
```

The `description` field should become the authored field instead of `summary`, because it aligns better with web and MkDocs metadata. Rust API and JSON shapes may continue to use `summary` as the runtime/internal term. In practice this can be implemented through serde mapping:

```rust
#[serde(rename = "description", alias = "summary")]
summary: String,
```

In the current `main`, the docs should move to `description` and should not keep both fields at the same time.

The `id`, `section`, and `see_also` fields should be preserved. They describe the `wavepeek docs` domain model, not only the web presentation. Site-only fields such as `nav_title`, `order`, `slug`, `template`, or `icon` should not be added up front if navigation can be generated from existing metadata.

## Site Navigation

Navigation should preferably be generated from topic metadata and/or the export manifest, rather than maintained manually in two places. The base grouping can follow the existing sections:

```text
Introduction
Commands
Workflows
Troubleshooting
Reference
```

Ordering within sections can be stable lexicographic ordering or can be defined by a small generator based on the current CLI logic. If the user experience requires manual ordering later, it should be introduced carefully and explicitly without breaking CLI topic IDs.

## CI, Local Commands, and Publication

Add separate commands for building, checking, serving, and publishing the site. Approximately:

```text
just docs-site-build
just docs-site-serve
just docs-site-check
just docs-site-deploy VERSION=0.5.0
```

`docs-site-check` should be lightweight and understandable: prepare the staging directory, build MkDocs in strict mode, verify that navigation and links are not broken, and ensure root raw artifacts can be prepared.

The GitHub Actions workflow for docs should be separate from pre-merge CI. Publication should preferably trigger on release tags matching `v*.*.*` and support manual dispatch for the first publication of `0.5.0`.

An approximate release workflow is:

```text
1. Check out the release tag.
2. Determine VERSION, for example 0.5.0.
3. Run cargo run --quiet -- docs export tmp/mkdocs-src --force.
4. Prepare the MkDocs staging directory.
5. Build the site in strict mode.
6. Publish the versioned snapshot at /VERSION/.
7. Update the latest alias.
8. Update root /skill.md and /wavepeek_vX.json files.
9. Do not touch already published snapshots for other versions.
```

Publication can use a dedicated `gh-pages` branch because the versioned archive and root artifacts require cumulative state across releases.

## Non-Functional Expectations

The integration should be simple and predictable. Generated staging/output directories should not be committed unless there is an explicit reason. The public docs corpus should remain suitable for embedded CLI use without depending on the MkDocs runtime.

The solution should minimize duplication: one authored Markdown corpus, one set of semantic metadata, and generated site artifacts as output.

Published version snapshots must be stable. A new release adds a new version and updates aliases/root artifacts, but must not silently change HTML for old versions.

## Decisions

The implementer may choose the exact implementation details:

- use `mike` or a thin custom wrapper for versioned publishing;
- store Python dependencies in `requirements-docs.txt`, `uv.lock`, or another accepted format;
- generate `mkdocs.yml`/navigation completely or keep a small static `mkdocs.yml` plus generated `docs_dir`;
- include docs-site checks in the general `just ci` immediately or after stabilization;
- decide whether to add extra copies of `skill.md` and `wavepeek_vX.json` inside each `/X.Y.Z/` snapshot;
- add a `/dev/` snapshot later if it becomes useful.

The main criterion is that the website must be derived from the same documentation that the CLI user receives, with versioned accumulation starting at `0.5.0`, without manual copying and without drift between CLI docs and web docs.

## Follow-Up Decisions

The following decisions were confirmed after the initial proposal:

- Use `mike` for MkDocs versioned publishing.
- Start with `requirements-docs.txt` as the canonical Python dependency list, and install that file into the devcontainer/CI image so local container runs do not need per-command dependency installation.
- Publish the first `0.5.0` snapshot from the existing tag context even though the site infrastructure did not exist in that tag.
- Use a `latest` alias rather than a physical copy.
- Redirect the site root to `/latest/`.
- Publish every `schema/wavepeek_v*.json` file found in the release checkout as root raw artifacts.
- Include the docs-site check in the standard `just ci` quality gate.
