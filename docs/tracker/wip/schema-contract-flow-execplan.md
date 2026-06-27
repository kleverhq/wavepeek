# Rework schema contracts to code-first generated snapshots

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. It is self-contained for this branch and does not require the external proposal file that initiated the work.

## Purpose / Big Picture

After this change, users and release tooling can rely on two stable schema families, `wavepeek.output` for `--json` envelopes and `wavepeek.stream-record` for JSONL records, without tying schema artifact names to the `wavepeek` binary minor version. The runtime machine output serializes through Rust contract data-transfer objects. The remaining work is to make committed schema snapshots more honestly derived from those contract DTOs, and to add runtime validation coverage for every JSON-capable CLI command before changing the generator internals.

The behavior is visible by running `just update-schema`, `just check-schema`, `wavepeek schema`, and `wavepeek schema --stream` from the repository root. `wavepeek schema` must print `schema/output.json` byte-for-byte, `wavepeek schema --stream` must print `schema/stream.json` byte-for-byte, and runtime `--json` / `--jsonl` outputs must contain exact schema URLs from `schema/catalog.json`. This plan also records the follow-up policy that current branch checkouts keep only generated current schema snapshots in `schema/`; historical schema URLs remain public through GitHub Pages and release tags, not through duplicate files in the current tree.

## Non-Goals

This plan does not introduce one canonical schema per command. It does not remove already published historical schema URLs from GitHub Pages and does not rewrite historical release tags. It does remove duplicate historical schema files such as `schema/wavepeek_v1.json` and `schema/wavepeek_v2.0.json` from the current branch checkout, because current runtime and current schema checks use `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`. It does not change the JSON or JSONL public wire shape except for the schema artifact URL values. It does not dynamically generate schemas in normal production CLI execution. It does not automatically decide whether a schema change is major or minor; the version constants remain explicit maintainer-owned values.

The external proposal file at `/workspaces/wavepeek/tmp/schema-contract-proposal.v4.md` is source material only and must not be committed.

## Progress

- [x] (2026-06-26T21:09:51Z) Read the proposal, repository breadcrumbs, current schema/runtime flow, and helper-tooling entrypoints.
- [x] (2026-06-26T21:09:51Z) Created this ExecPlan with the implementation sequence, acceptance criteria, and current findings.
- [x] (2026-06-26T21:26:00Z) Reviewed this ExecPlan with architecture and tooling review lanes; recorded findings and plan changes.
- [x] (2026-06-26T22:05:51Z) Added the Rust contract layer under `src/contract/` and routed JSON / JSONL serialization through it.
- [x] (2026-06-26T22:05:51Z) Added deterministic schema generation and the `tools/schema-gen` utility.
- [x] (2026-06-26T22:05:51Z) Regenerated and embedded `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`.
- [x] (2026-06-26T22:05:51Z) Updated schema checks, just recipes, tests, docs publication tooling, and documentation.
- [x] (2026-06-26T22:39:28Z) Ran focused validation and the repository quality gate.
- [x] (2026-06-26T22:39:28Z) Performed post-implementation reviews, fixed findings, and completed a clean control pass.
- [x] (2026-06-26T22:48:00Z) Committed, pushed, and opened pull request 45 for the schema contract migration.
- [x] (2026-06-27T11:22:00Z) Restored this ExecPlan as requested in a separate commit before starting the historical schema artifact cleanup.
- [x] (2026-06-27T11:22:00Z) Reviewed the updated plan for removing duplicate checked-in historical schema artifacts; review returned no substantive findings.
- [x] (2026-06-27T11:22:00Z) Removed checked-in historical schema artifacts from `schema/` while preserving current generated snapshots.
- [x] (2026-06-27T11:22:00Z) Updated schema/tooling documentation to describe where historical schema artifacts are preserved.
- [x] (2026-06-27T11:22:00Z) Ran focused validation and post-change review; fixed the review finding about stale pre-migration wording.
- [x] (2026-06-27T11:24:00Z) Committed and pushed the historical schema cleanup to pull request 45.
- [x] (2026-06-27T11:42:26Z) Reopened the plan after identifying that `src/contract/schema.rs` is still a manual schema builder and that JSON CLI runtime validation is weaker than JSONL validation.
- [x] (2026-06-27T11:42:26Z) Prepared this ExecPlan update as a separate plan-only change before adding more code.
- [x] (2026-06-27T11:42:26Z) Added runtime JSON validation coverage for `info`, `scope`, and `signal` `--json` outputs.
- [x] (2026-06-27T11:42:26Z) Added runtime JSON validation coverage for `value`, `change`, and `property` `--json` outputs.
- [x] (2026-06-27T11:42:26Z) Added runtime JSON validation coverage for `docs topics` and `docs search` `--json` outputs.
- [x] (2026-06-27T11:42:26Z) Reviewed the JSON validation tests, fixed the helper diagnostics finding, ran focused Rust tests, and committed the test slices.
- [x] (2026-06-27T11:58:06Z) Expanded this ExecPlan with the exact `schemars` implementation details after the test safety net was in place.
- [x] (2026-06-27T11:58:06Z) Refactored schema generation so DTO-owned `JsonSchema` derives or custom implementations produce payload definitions, with hand-written code limited to catalog metadata and root command/data composition that JSON Schema cannot infer safely.
- [x] (2026-06-27T11:58:06Z) Regenerated schema snapshots, ran focused schema validation, reviewed the schema-generation refactor, and fixed the architecture documentation finding.
- [x] (2026-06-27T11:58:06Z) Ran `just check` after the schema-generation refactor.
- [x] (2026-06-27T11:58:06Z) Committed the schema-generation refactor.
- [ ] Push all follow-up commits and update pull request 45.

## Surprises & Discoveries

- Observation: Before this branch, the `just update-schema` recipe was artifact-first, not code-first. It ran `cargo run -- schema` and wrote the embedded schema output back to the checked-in schema file.
  Evidence: the old `justfile` recipe used `cargo run --quiet -- schema > "$tmp_file"` and `src/schema_contract.rs` embedded `schema/wavepeek_v{major}.{minor}.json` through `include_str!`.

- Observation: Before this branch, runtime public JSON and JSONL serialization bypassed a dedicated contract DTO layer.
  Evidence: the old `src/output.rs::render_json` wrapped `result.data` in `OutputEnvelope`, and the old `src/output.rs::write_jsonl_result` passed engine payloads directly to `JsonlWriter::item`.

- Observation: Current tests already cover many contract semantics that should be preserved: diagnostics rules, extension-friendly objects, command-to-payload JSONL records, scope/signal kind enums, and byte-for-byte schema command output.
  Evidence: `tests/schema_cli.rs`, `tests/jsonl_cli.rs`, and `tools/schema/check_schema_contract.py` validate these constraints.

- Observation: The Python environment used by the local helper tests does not provide the `jsonschema` package.
  Evidence: `python3 -B tools/schema/check_schema_contract.py --schema-dir schema --generated-dir tmp/schema-check` failed with `ModuleNotFoundError: No module named 'jsonschema'` before semantic validation moved into `tools/schema-gen --validate`.

- Observation: Documentation deployment needed explicit catalog artifact handoff; deriving schema artifact names from the CLI version would break independent schema-family versioning.
  Evidence: `tools/docs/check_deploy.py` originally derived `schema-output-v{major}.{minor}.json` from `--version`; it now accepts `--schema-artifact` and `--stream-schema-artifact`, and `tools/docs/workflow_docs.py` passes those values from staged metadata.

- Observation: Publication compatibility for historical artifacts required a separate legacy path.
  Evidence: Post-implementation tooling review found that old no-catalog v2.0 deployments could be blocked by new catalog assumptions and that historical `wavepeek_v*.json` artifacts needed the same immutable overwrite guard as new `schema-output-v*.json` artifacts.

- Observation: Current runtime and schema freshness checks no longer depend on checked-in historical schema files.
  Evidence: `src/schema_contract.rs` embeds only `schema/output.json` and `schema/stream.json`; `tools/schema/check_schema_contract.py` compares only `output.json`, `stream.json`, and `catalog.json`; `tools/docs/publish_docs.py` publishes catalog-listed current artifacts when `schema/catalog.json` exists.

- Observation: The current branch still has two schema sources for field-level shape: runtime DTOs in `src/contract/output.rs` and `src/contract/stream.rs`, and a manual schema builder in `src/contract/schema.rs`.
  Evidence: `OutputEnvelope`, `OutputData`, `BeginRecord`, `ItemRecord`, and related DTOs derive `serde::Serialize` and are used on the stdout path, while `output_schema_value`, `stream_schema_value`, `output_defs`, and `stream_defs` hand-build JSON Schema definitions with `serde_json::json!`.

- Observation: JSONL CLI runtime outputs are validated against `schema/stream.json` more comprehensively than JSON CLI runtime outputs are validated against `schema/output.json`.
  Evidence: `tests/jsonl_cli.rs::parse_stream` validates every JSONL record it parses with a compiled stream schema validator. `tests/schema_cli.rs` validates representative hand-built output samples and checks one runtime `info --json` envelope, but it does not validate every JSON-capable command's actual `--json` stdout.

- Observation: The `schemars` refactor can make schema definitions DTO-owned without changing the generated public schema snapshots.
  Evidence: after deriving or implementing `JsonSchema` on contract DTOs, `just update-schema` left `schema/output.json` and `schema/stream.json` with no git diff, and `just check-schema` passed.

## Decision Log

- Decision: Use a manual deterministic Rust schema builder in `src/contract/schema.rs` only for the first migration slice, not as the final contract source-of-truth design.
  Rationale: The current schema has command-to-payload conditionals, extension-friendly objects, diagnostic conditionals, exact URL constants, and stable definition names. A small explicit builder preserved the public schema shape and kept the initial migration reviewable, but it can still drift from the runtime DTOs. The remaining work moves field-level schema definitions to DTO-owned `schemars` derives or custom `JsonSchema` implementations.
  Date/Author: 2026-06-26, revised 2026-06-27 / coding agent

- Decision: Keep already published historical schema URLs resolvable through GitHub Pages and release tags, but remove duplicate historical schema files from current branch checkouts.
  Rationale: Historical artifacts are public contracts, but the current branch does not need duplicate copies once the runtime embeds generated family snapshots and publication uses `schema/catalog.json`. Keeping old files in `main` makes the current schema directory look like it has multiple active sources of truth.
  Date/Author: 2026-06-27 / user and coding agent

- Decision: Set the initial explicit schema family versions to output `2.0` and stream-record `2.0`.
  Rationale: The migration is intended to preserve the existing wire shape, with only the schema artifact URL moving to the new exact family URL names. There is no maintainer-directed schema semantic bump in the proposal.
  Date/Author: 2026-06-26 / coding agent

- Decision: Publish/check exact current family artifacts from `schema/catalog.json`, while preserving support for already published historical root artifacts.
  Rationale: The proposal requires new exact URLs such as `/schema-output-v2.0.json` and append-only behavior for exact artifacts, but old URLs must remain resolvable.
  Date/Author: 2026-06-26 / coding agent

- Decision: Expose the contract schema generator from the main library through a `#[doc(hidden)] pub mod contract` boundary rather than duplicating generator code in the helper crate.
  Rationale: `tools/schema-gen` is a separate Cargo package, so it cannot access private crate modules. A hidden public module keeps one implementation shared by runtime and tooling while documenting that it is an internal support API, not a public semver commitment.
  Date/Author: 2026-06-26 / coding agent

- Decision: Keep catalog `path` values canonical and repository-relative, independent of the generator `--out` directory.
  Rationale: `tools/schema-gen --out tmp/schema-check` must produce a catalog identical to `schema/catalog.json`; temporary output directories must never leak into committed catalogs or docs publication.
  Date/Author: 2026-06-26 / coding agent

- Decision: Use the proposal catalog shape with a top-level `families` array, each entry containing `id`, `version`, `path`, and `url`; do not store a separate `artifact` field.
  Rationale: The URL already determines the published artifact filename. Keeping the catalog minimal avoids redundant state and matches the source proposal more closely.
  Date/Author: 2026-06-26 / coding agent

- Decision: Perform JSON Schema semantic sample validation in `tools/schema-gen --validate` rather than importing Python `jsonschema` in `tools/schema/check_schema_contract.py`.
  Rationale: The Python test environment lacks `jsonschema`, while the Rust helper can depend on the existing Rust `jsonschema` crate and validate generated schemas without adding a new Python runtime dependency.
  Date/Author: 2026-06-26 / coding agent

- Decision: Add runtime JSON `--json` schema validation tests before refactoring schema generation.
  Rationale: JSONL already has a runtime validation safety net. Adding the same kind of safety net for JSON output gives immediate protection against `output.rs` and `schema.rs` drift and makes the later `schemars` refactor safer to review.
  Date/Author: 2026-06-27 / user and coding agent

- Decision: Use `schemars` for DTO-owned field and payload definitions, while retaining a small hand-written root composition layer for the global output and stream schemas.
  Rationale: Rust DTOs can own field names, optionality, value constraints, descriptions, stable kind enums, and diagnostic schema rules through derives or custom `JsonSchema` implementations. The global schemas still need explicit root semantics that a plain untagged DTO cannot safely infer, especially `command -> data` conditionals for JSON envelopes and command-specific JSONL item records.
  Date/Author: 2026-06-27 / user and coding agent

- Decision: Keep the follow-up commit series reviewable: one plan commit, small JSON test commits by command group, then one or more schema-generation refactor commits.
  Rationale: The work touches public machine-output contracts. Splitting tests from generator changes makes regressions easier to isolate and gives reviewers a stable test safety net before the internal refactor.
  Date/Author: 2026-06-27 / user and coding agent

- Decision: Preserve generated `schema/output.json` and `schema/stream.json` byte-for-byte during the `schemars` refactor.
  Rationale: The goal is to make the source of truth more honest, not to revise the public schema artifact. Keeping snapshots unchanged proves that this slice is an internal generator refactor guarded by runtime validation tests.
  Date/Author: 2026-06-27 / coding agent

## Outcomes & Retrospective

The initial migration delivered the stable schema family flow, runtime serialization through `src/contract` DTOs, current snapshots at `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`, catalog-aware docs publication, and removal of duplicate historical schema artifacts from the current tree. Review then identified two remaining contract-quality gaps: `src/contract/schema.rs` still hand-builds field-level schema definitions instead of deriving them from the DTOs, and JSON `--json` runtime outputs do not have the same comprehensive schema-validation coverage as JSONL outputs. This plan was reopened to close those gaps before pull request 45 is considered complete. The JSON runtime validation tests and `schemars` refactor are now implemented and reviewed. Remaining work is to commit and push the schema-generation refactor and update the pull request.

## Context and Orientation

`wavepeek` is a Rust CLI. Its machine output has two formats. The `--json` format is a single JSON object called an envelope. The envelope has a `$schema` URL, a `command` string, a command-specific `data` payload, and a `diagnostics` array. The JSONL format prints one JSON object per line; each line is a stream record with `type` equal to `begin`, `item`, `diagnostic`, or `end`.

Before this branch, the schema runtime in `src/schema_contract.rs` constructed schema URLs from `CARGO_PKG_VERSION_MAJOR` and `CARGO_PKG_VERSION_MINOR`, embedded `schema/wavepeek_v2.0.json` and `schema/wavepeek-stream-v2.0.json`, and exposed those strings to the CLI. In the current branch, `src/schema_contract.rs` embeds `schema/output.json` and `schema/stream.json`, while URL and family metadata come from `src/contract/schema.rs`. `src/engine/schema.rs` returns the embedded current snapshots for `wavepeek schema` and `wavepeek schema --stream`.

The current JSON and JSONL serialization entrypoint is `src/output.rs`. `render_json` converts `crate::engine::CommandResult` into `contract::output::OutputEnvelope` before calling `serde_json`. `JsonlWriter::begin`, `JsonlWriter::item`, `JsonlWriter::diagnostic`, and `JsonlWriter::end` serialize records from `contract::stream`; the `StreamItem` trait restricts which engine values can become JSONL item payloads. `src/engine/change.rs` and `src/engine/property.rs` stream JSONL rows through sink types that call this typed item API.

The engine modules still own computation-oriented structs: `src/engine/info.rs::InfoData`, `src/engine/scope.rs::ScopeEntry`, `src/engine/signal.rs::SignalEntry`, `src/engine/value.rs::ValueSnapshot`, `src/engine/change.rs::ChangeSnapshot`, `src/engine/property.rs::PropertyCaptureRow`, and docs JSON payloads in `src/engine/mod.rs` plus `src/docs/mod.rs`. Before machine-output serialization, those engine structs are converted to contract DTOs in `src/contract/output.rs`, `src/contract/stream.rs`, and `src/contract/common.rs`.

The current contract layer lives under `src/contract/`. A contract data-transfer object, abbreviated DTO, is a Rust struct or enum whose purpose is to describe the public wire shape. Engine structs may continue to exist for computation and human rendering, but machine output must convert engine results into contract DTOs before `serde_json` serializes them. The schema generator must be based on the same contract module. Today `src/contract/schema.rs` hand-builds schema definitions next to the DTOs; the next refactor moves those definitions onto the DTOs via `schemars` derives or custom `JsonSchema` implementations.

The helper entrypoints are in `justfile`. `just update-schema` generates `schema/output.json`, `schema/stream.json`, and `schema/catalog.json` by running `tools/schema-gen`. `just check-schema` generates fresh files under `tmp/schema-check`, compares them with committed snapshots, runs semantic schema validation through `tools/schema-gen --validate`, and checks runtime `wavepeek schema` output. `tools/schema/check_schema_contract.py` is catalog-based.

Docs publication lives in `tools/docs/publish_docs.py`. Deployed-doc validation lives in `tools/docs/check_deploy.py`. Current catalog-based releases read `schema/catalog.json` for exact family artifacts. Historical release repair and deployed checks still understand legacy artifact names such as `wavepeek_v2.0.json`, but those historical files do not need to be present in the current branch checkout.

## Open Questions

There are no blocking open questions. The chosen policy is that `main`/current branch checkouts keep generated current snapshots only, while GitHub Pages and release tags preserve historical artifacts. The chosen generator policy is that DTO-owned `schemars` output should produce field-level and payload definitions, while explicit root composition remains allowed for global schema semantics that cannot be derived safely from an untagged runtime enum.

## Plan of Work

The initial migration sequence in this section has already been executed and is retained so a new reader can understand how the branch reached its current state. The active follow-up starts after the historical schema cleanup paragraph below.

First, review this plan against the current source. The review should check whether all runtime paths that emit JSON or JSONL are covered, whether docs publication and deployed-doc checks are included, and whether the migration leaves historical artifacts intact.

Next, add `src/contract/mod.rs`, `src/contract/common.rs`, `src/contract/output.rs`, `src/contract/stream.rs`, and `src/contract/schema.rs`, and expose the module from `src/lib.rs` as `#[doc(hidden)] pub mod contract;` for `tools/schema-gen`. `common.rs` should define contract diagnostics and helper wrappers for stable scope and signal kind strings. These wrappers must validate values against `crate::waveform::STABLE_SCOPE_KIND_ALIASES` and `crate::waveform::STABLE_SIGNAL_KIND_ALIASES` before serialization. `output.rs` should define the JSON envelope DTO and borrowed command payload DTOs. `stream.rs` should define begin, item, diagnostic, and end JSONL record DTOs. `schema.rs` should define the family IDs, exact URLs, canonical repository-relative artifact paths, and deterministic functions that return the output schema, stream schema, and catalog as pretty JSON with a trailing newline. Catalog `path` entries must always be `schema/output.json` and `schema/stream.json`, even when the generator writes to `tmp/schema-check`.

Then change `src/output.rs` so `render_json` converts `CommandResult` into `contract::output::OutputEnvelope` before serializing. Change `JsonlWriter` so `begin`, `item`, `diagnostic`, and `end` serialize `contract::stream` records. The item method should accept engine values through a new conversion trait or typed enum so callers cannot accidentally bypass the contract DTO. Update `src/engine/change.rs` and `src/engine/property.rs` JSONL sinks if their item calls need the new typed item API. Where possible, remove `Serialize` derives from engine-only aggregate types after conversion so future code cannot use them as an alternate machine-output source; where a derive must remain for internal tests or fixtures, add tests that public machine-output code emits contract DTO shapes.

After runtime conversion works, change `src/schema_contract.rs` so it embeds `schema/output.json` and `schema/stream.json` directly and takes URL/version metadata from `crate::contract::schema`. Keep tests that compare scope/signal alias inventories, but update them for the new artifact names and exact `$schema` URL constants.

Add `tools/schema-gen/Cargo.toml` and `tools/schema-gen/src/main.rs`. This utility should accept `--out <dir>` and write `output.json`, `stream.json`, and `catalog.json` under that directory. It should also support writing to `schema/` through `just update-schema`. The utility must use the hidden Rust contract schema functions, not `wavepeek schema` runtime output, and generated catalog contents must be byte-identical regardless of the `--out` directory.

Update `justfile`: remove `schema_path` and `stream_schema_path` variables that derive names from `Cargo.toml`; add a `schema_check_dir := "tmp/schema-check"` variable; make `update-schema` run `cargo run --quiet --manifest-path tools/schema-gen/Cargo.toml -- --out schema`; make `check-schema` run the same generator into `tmp/schema-check` and invoke `tools/schema/check_schema_contract.py` with the committed schema directory and generated directory, or let the script do generation comparison itself. Keep the user-facing recipes unchanged.

Refactor `tools/schema/check_schema_contract.py` to load `schema/catalog.json`, verify exactly the two families `wavepeek.output` and `wavepeek.stream-record`, verify paths and exact URLs, compare committed artifacts to generated artifacts under `tmp/schema-check`, invoke `tools/schema-gen --validate` for JSON Schema semantic samples, and verify `wavepeek schema` / `wavepeek schema --stream` byte-for-byte against the committed snapshots. Runtime JSON and JSONL `$schema` values must equal the exact catalog URLs, not a same-major pattern. If new Python unit tests are added under `tools/schema`, wire them into the `just test-aux` recipe so CI and pre-commit run them.

Update docs publication. In `tools/docs/publish_docs.py`, read the catalog from the source root, copy catalog-listed artifacts to root publication artifacts by deriving artifact filenames from catalog URLs, and keep existing historical `wavepeek_v*.json` and `wavepeek-stream-v*.json` support only for already checked-in historical files. Exact catalog artifacts must be staged even when a release is not promoted to `latest`; root aliases and installer entrypoints remain latest-promotion behavior. Existing exact artifact paths on `gh-pages` must be byte-for-byte immutable: if a staged exact path already exists with different bytes, fail before committing. Persist the staged catalog artifact names in `staged-deploy.json` so `push-staged` and workflow checks do not infer schema filenames from CLI SemVer. Update `allowed_path_patterns`, `path_allowed`, and `required_pages_artifact_paths` for `schema-output-v*.json` and `schema-stream-v*.json`.

Update `tools/docs/check_deploy.py` so deployed release checks fetch and validate exact catalog-style artifact names for current releases. The deployed check should get current artifact names from a published catalog when available or from catalog metadata carried by the docs workflow/staged deploy metadata; historical version fallback can continue to validate old artifact names if a catalog is unavailable for old tags. Update helper unit tests under `tools/docs/` accordingly.

Update Rust and integration tests. `tests/common/mod.rs` should return exact output and stream schema URLs from the new family names. `tests/schema_cli.rs` and `tests/jsonl_cli.rs` should read `schema/output.json` and `schema/stream.json`, check exact URL `const` values instead of same-major patterns, and retain validation of diagnostics, command branches, extension friendliness, and representative runtime outputs. Add tests that `schema/catalog.json` points to both committed snapshots.

Update documentation and breadcrumbs. Refresh `tools/schema/README.md`, `tools/docs/README.md`, `schema/AGENTS.md`, `docs/dev/release.md`, `docs/dev/architecture.md`, `docs/dev/automation.md`, `docs/dev/quality.md`, `docs/public/reference/machine-output.md`, and `docs/public/commands/schema.md` to describe code-first generation, `schema/output.json`, `schema/stream.json`, `schema/catalog.json`, and exact family URLs. Keep historical examples only where they explicitly discuss old releases.

Finally, run formatting, focused tests, `just check-schema`, docs helper tests, and `just check`. Request a post-implementation review, address findings, then commit the branch and update pull request 45. Confirm the external proposal file was not added to git.

Follow-up cleanup for checked-in historical schema artifacts is now part of this plan. Remove these five files from the current tree: `schema/wavepeek_v0.json`, `schema/wavepeek_v1.json`, `schema/wavepeek_v2.0.json`, `schema/wavepeek-stream-v1.json`, and `schema/wavepeek-stream-v2.0.json`. Do not remove `schema/output.json`, `schema/stream.json`, or `schema/catalog.json`. Update `schema/AGENTS.md` and `tools/schema/README.md` to say that current checkouts contain only generated current snapshots and that historical artifacts are preserved by release tags and GitHub Pages. Update `tools/docs/README.md` to distinguish publishing current catalog artifacts from preserving already deployed historical artifacts. Keep `tools/docs/publish_docs.py` legacy support intact for old source refs and tests, because old tags still contain no-catalog historical artifacts.

The active follow-up starts by adding JSON runtime validation tests in `tests/schema_cli.rs`. Add a helper that compiles `schema/output.json` with the Rust `jsonschema` crate, executes a CLI command with `--json`, parses stdout, validates the parsed value, and asserts that `$schema` and `command` match the expected values. Keep command fixtures small and deterministic. Use `tests/fixtures/hand/m2_core.vcd` for waveform commands, a tiny temporary VCD for property capture when a purpose-built sequence is clearer, and the embedded docs commands for `docs topics` and `docs search`. Split the test additions into reviewable command groups: `info`/`scope`/`signal`, then `value`/`change`/`property`, then `docs topics`/`docs search`. After each group, run `cargo test -q --test schema_cli` and commit the passing slice.

The `schemars` refactor has the following implementation shape. Add `schemars = "1.2.1"` to the main crate dependencies in `Cargo.toml`; `tools/schema-gen` already depends on `wavepeek`, so it should not need its own direct `schemars` dependency unless it starts naming schemars types itself. In `src/contract/common.rs`, keep `ContractDiagnostic` as the runtime diagnostic DTO and add a custom `JsonSchema` implementation for it that encodes the existing public rules: `kind` is one of `info`, `warning`, or `error`; `message` is always required; `code` is required for warnings and errors, forbidden for info diagnostics, and matches the existing `^WPK-[WE][0-9]{4}$` pattern with branch-specific `W` or `E` prefixes. Also add transparent newtype DTOs for recurring scalar contracts: `NormalizedTime<'a>`, `CanonicalPath<'a>`, `SampledValue<'a>`, `ScopeKind<'a>`, and `SignalKind<'a>`. These newtypes must serialize as strings, but their `JsonSchema` implementations must supply the stable definition names and schema rules. `ScopeKind` and `SignalKind` must use `STABLE_SCOPE_KIND_ALIASES` and `STABLE_SIGNAL_KIND_ALIASES` for their enum values, so the runtime validator and schema share the same source.

In `src/contract/output.rs`, derive `schemars::JsonSchema` on output DTO structs and enums where derive is sufficient: `OutputEnvelope`, `OutputData`, `InfoData`, `ScopeEntry`, `SignalEntry`, `SampledSignalValue`, `ValueSnapshot`, `ChangeSnapshot`, `PropertyKind`, `PropertyRow`, `TopicSummary`, `DocsTopicsData`, `DocsSearchMatch`, and `DocsSearchData`. Replace raw string fields that are semantic scalar contracts with the transparent newtypes from `common.rs`: timestamps use `NormalizedTime`, paths use `CanonicalPath`, sampled values use `SampledValue`, scope kinds use `ScopeKind`, and signal kinds use `SignalKind`. Preserve all `serde` wire behavior, including `$schema` renaming, untagged enums, `skip_serializing_if` on `see_also` and optional `width`, and lowercase or snake_case enum names. Put `#[schemars(range(min = 1))]` or equivalent DTO-owned constraints on fields such as `SignalEntry::width` and `DocsSearchMatch::matched_tokens`; use a DTO-owned custom schema only where derive attributes cannot express the rule cleanly.

In `src/contract/stream.rs`, derive `JsonSchema` for `BeginRecord`, `ItemRecord`, `DiagnosticRecord`, `EndRecord`, `StreamSummary`, and `StreamItemData`. Keep the runtime constructors that validate stream command support and record sequencing unchanged. Record-type constants such as `"begin"`, `"item"`, `"diagnostic"`, and `"end"`, exact schema URL constants, and command-specific item branches still need explicit root composition in `schema.rs`; do not expect derive output for `ItemRecord<'a>` alone to express the command-to-item relationship.

In `src/contract/schema.rs`, replace `common_payload_defs`, `diagnostic_def`, `topic_summary_def`, `docs_search_match_def`, and the hand-written record definition bodies with helpers that collect schemars definitions. Use `schemars::SchemaGenerator::default()`, call `subschema_for::<T>()` for each DTO whose definition must be present, then use `take_definitions(true)` to obtain a `serde_json::Map<String, Value>` keyed by the DTO-owned schema names. Use explicit `schema_name()` implementations or `#[schemars(rename = "...")]` so public `$defs` names remain stable: `diagnostic`, `normalizedTime`, `canonicalPath`, `sampledValue`, `scopeKind`, `signalKind`, `infoData`, `scopeEntry`, `signalEntry`, `sampledSignalValue`, `valueSnapshot`, `changeSnapshot`, `propertyRow`, `topicSummary`, `docsSearchMatch`, `beginRecord`, `itemRecord`, `diagnosticRecord`, `endRecord`, and `streamSummary`. Keep `commandData` arrays, `outputCommands`, `streamCommand`, `sequence`, exact `$schema` URL constants, root `anyOf` / `allOf`, `command -> data` branches, and command-specific JSONL item records as hand-written root composition because those are global schema semantics rather than individual DTO field definitions.

After implementing the refactor, run `just update-schema`. The generated `schema/output.json` and `schema/stream.json` should stay semantically equivalent to the current snapshots; small ordering or formatting differences are acceptable only if deterministic and reviewable. If schemars emits an undesired default such as `additionalProperties: false`, fix it at the DTO schema level or with a small normalization helper so public objects remain extension-friendly. Do not manually edit generated schema snapshots. Run `cargo test -q --test schema_cli`, `cargo test -q --test jsonl_cli`, `just check-schema`, and then request a schema-generation review before committing.

### Concrete Steps

Run all commands from `/workspaces/wavepeek/.worktrees/feat-rework-schema-flow`.

1. Inspect status before editing:

    git status --short --branch

   Expected result: branch `feat/rework-schema-flow`; no tracked changes except files intentionally edited by this plan.

2. Generate or refresh schema artifacts after implementing the contract generator:

    just update-schema

   Expected result: `schema/output.json`, `schema/stream.json`, and `schema/catalog.json` are created or updated. No `schema/wavepeek_v*.json` or `schema/wavepeek-stream-v*.json` files are created in the current checkout.

3. Check schema freshness and runtime embed behavior:

    just check-schema

   Expected result: the script reports success. If generated files differ, it fails with a hint to run `just update-schema`.

4. Run focused Rust tests after each JSON validation test slice:

    cargo test -q --test schema_cli

   Expected result: the newly added runtime `--json` validation tests pass for the command group just added.

5. After all JSON validation slices are present, run the schema and JSONL integration tests together:

    cargo test -q --test schema_cli --test jsonl_cli

   Expected result: schema, JSON runtime validation, and JSONL runtime validation tests pass.

6. After the later `schemars` refactor, regenerate and check schema artifacts:

    just update-schema
    just check-schema

   Expected result: generated snapshots are deterministic, runtime `wavepeek schema` output still matches committed snapshots, and semantic schema validation passes.

7. Run helper tests affected by docs publication and schema checks:

    python3 -B -m unittest discover -s tools/docs -p "test_*.py"
    python3 -B -m unittest discover -s tools/schema -p "test_*.py"

   Expected result: all helper tests pass. If `tools/schema` has no tests, the command should report zero tests rather than fail.

8. Run formatting and the local pre-handoff gate:

    just format
    just check

   Expected result: formatting completes and `just check` passes.

9. Inspect final diff and ensure the proposal file is not staged:

    git status --short
    git diff --stat
    git -C /workspaces/wavepeek ls-files -- tmp/schema-contract-proposal.v4.md

   Expected result: the proposal file is not tracked by the source checkout.

### Validation and Acceptance

The change is accepted when a human can verify these behaviors:

- `just update-schema` regenerates `schema/output.json`, `schema/stream.json`, and `schema/catalog.json` from Rust contract code, not from `wavepeek schema` output.
- `just check-schema` fails when any committed current schema snapshot or the catalog differs from freshly generated output, and the failure tells the user to run `just update-schema`.
- `wavepeek schema` prints exactly the bytes in `schema/output.json`, including the trailing newline.
- `wavepeek schema --stream` prints exactly the bytes in `schema/stream.json`, including the trailing newline.
- Runtime `wavepeek info --waves tests/fixtures/hand/m2_core.vcd --json` contains `$schema` equal to the `wavepeek.output` URL in `schema/catalog.json`.
- Runtime `wavepeek info --waves tests/fixtures/hand/m2_core.vcd --jsonl` begins with a `begin` record whose `$schema` equals the `wavepeek.stream-record` URL in `schema/catalog.json`.
- Runtime `--json` outputs for `info`, `scope`, `signal`, `value`, `change`, `property`, `docs topics`, and `docs search` validate against `schema/output.json`.
- Representative runtime JSONL records validate against `schema/stream.json`.
- Current branch checkouts contain no `schema/wavepeek_v*.json` or `schema/wavepeek-stream-v*.json` files.
- `just update-schema` does not recreate historical schema files.
- Docs publication stages exact catalog artifacts for every current release, preserves already deployed historical artifacts on GitHub Pages, and refuses to overwrite an existing exact artifact path with different bytes.
- Field-level schema definitions for contract payloads are owned by `src/contract` DTOs through `schemars` derives or custom `JsonSchema` implementations, not duplicated as independent hand-written objects in `src/contract/schema.rs`.
- `src/contract/schema.rs` remains responsible for schema-family metadata, deterministic output, catalog JSON, and root-level composition that expresses global schema semantics.

### Idempotence and Recovery

`just update-schema` is safe to run repeatedly; if the Rust contract did not change, it should produce no git diff. `just check-schema` writes disposable generated output under `tmp/schema-check`, which is ignored scratch space. Do not delete arbitrary existing files in `tmp/`; only overwrite or remove paths owned by this plan, such as `tmp/schema-check`.

If generated schemas are wrong, restore committed snapshots with `git checkout -- schema/output.json schema/stream.json schema/catalog.json`, fix the Rust contract schema generator, and rerun `just update-schema`. If a new JSON runtime validation test fails before the `schemars` refactor, inspect whether the runtime DTO or the schema is wrong; do not weaken the test without identifying which contract is intended. If deleting historical schema files breaks validation, first determine whether the failing path is a current-catalog path or a historical-tag repair path. Current-catalog paths should not require historical files. Historical-tag repair behavior should be covered by fixtures or tests without requiring historical artifacts in the current `schema/` directory. If docs publication tests fail after tooling changes, keep the old tests as evidence of required behavior and update the code before changing expectations.

### Artifacts and Notes

Historical relevant facts before the first migration:

    src/schema_contract.rs embedded schema/wavepeek_v{major}.{minor}.json and schema/wavepeek-stream-v{major}.{minor}.json.
    just update-schema ran cargo run -- schema and cargo run -- schema --stream.
    tests/schema_cli.rs expected current artifact paths derived from Cargo package major/minor.
    tools/docs/publish_docs.py copied schema/wavepeek_v*.json and schema/wavepeek-stream-v*.json root artifacts, with latest-promotion behavior.

Current relevant facts before the JSON validation and `schemars` follow-up:

    src/output.rs routes JSON and JSONL stdout through src/contract DTOs.
    src/contract/schema.rs manually builds output and stream schema definitions with serde_json::json!.
    tests/jsonl_cli.rs validates actual JSONL runtime records against schema/stream.json.
    tests/schema_cli.rs does not yet validate every JSON-capable command's actual --json stdout against schema/output.json.
    tools/schema-gen calls wavepeek::contract::schema::{output_schema_json, stream_schema_json, catalog_json}.

### Interfaces and Dependencies

At the end of the implementation, `src/contract/schema.rs` must expose these constants or equivalent names used by runtime and tooling:

    pub const OUTPUT_SCHEMA_ID: &str = "wavepeek.output";
    pub const STREAM_SCHEMA_ID: &str = "wavepeek.stream-record";
    pub const OUTPUT_SCHEMA_VERSION_STR: &str = "2.0";
    pub const STREAM_SCHEMA_VERSION_STR: &str = "2.0";
    pub const OUTPUT_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-output-v2.0.json";
    pub const STREAM_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-stream-v2.0.json";
    pub fn output_schema_json() -> String;
    pub fn stream_schema_json() -> String;
    pub fn catalog_json() -> String;

`src/contract/output.rs` must expose a conversion from `crate::engine::CommandResult` or its parts into a serializable output envelope. `src/contract/stream.rs` must expose typed constructors for begin, item, diagnostic, and end records so `src/output.rs` no longer serializes raw engine payloads for machine output.

`tools/schema-gen/src/main.rs` must write exactly three files named `output.json`, `stream.json`, and `catalog.json` under the requested output directory.

The `schemars` refactor must keep these public interfaces stable while changing how `output_schema_json()` and `stream_schema_json()` obtain field-level definitions. DTO modules should expose only the schema support needed by `src/contract/schema.rs`; hidden helper types or functions are acceptable when they prevent duplicate hand-written schema objects.

Revision note 2026-06-26: Initial plan created after reading the proposal and current code. The plan records the artifact-first current state and chooses an explicit Rust schema builder to keep the migration narrow and deterministic.

Revision note 2026-06-26: Plan review findings were incorporated. The plan now defines the hidden library boundary used by `tools/schema-gen`, canonical catalog paths independent of `--out`, DTO bypass mitigation, deployed catalog handoff through staged metadata, `just test-aux` coverage for new schema helper tests, `tools/docs/README.md` updates, and an executable proposal-not-committed check.

Revision note 2026-06-26: Implementation progress recorded. The plan now reflects the `families` catalog shape without an `artifact` field, Rust-based schema semantic validation, completed contract/generator/tooling/doc updates, and the remaining validation/review/PR work.

Revision note 2026-06-26: Post-implementation review findings and fixes recorded. The docs publication path now preserves historical no-catalog artifacts, applies immutable overwrite checks to all versioned schema artifacts, and passes exact schema artifact names through staged metadata for new catalog releases.

Revision note 2026-06-27: The plan was restored in a separate commit at the user's request and updated for a follow-up cleanup. Current branch checkouts should contain only generated current schema snapshots; historical schema artifacts remain available through GitHub Pages and release tags rather than duplicate files in `schema/`. The cleanup intentionally does not add a `check-schema` guard against reintroducing old files.

Revision note 2026-06-27: Historical schema cleanup was implemented after plan review. The current tree now removes `schema/wavepeek_v0.json`, `schema/wavepeek_v1.json`, `schema/wavepeek_v2.0.json`, `schema/wavepeek-stream-v1.json`, and `schema/wavepeek-stream-v2.0.json`, and documentation now states that historical artifacts live in release tags and GitHub Pages.

Revision note 2026-06-27: Post-change review found stale pre-migration wording in this plan. The context now distinguishes the pre-branch artifact-first runtime from the current `schema/output.json` and `schema/stream.json` embedding.

Revision note 2026-06-27: User review identified two remaining gaps: the schema builder is still manual and can drift from the runtime DTOs, and JSON CLI runtime outputs have weaker schema-validation coverage than JSONL outputs. The plan is reopened to add JSON `--json` validation tests first, then refactor schema generation toward DTO-owned `schemars` definitions with explicit root composition.

Revision note 2026-06-27: The first JSON validation slice added `tests/schema_cli.rs` coverage for `info`, `scope`, and `signal` runtime `--json` stdout against `schema/output.json`.

Revision note 2026-06-27: The second JSON validation slice added `tests/schema_cli.rs` coverage for `value`, `change`, and `property` runtime `--json` stdout against `schema/output.json`.

Revision note 2026-06-27: The third JSON validation slice added `tests/schema_cli.rs` coverage for `docs topics` and `docs search` runtime `--json` stdout against `schema/output.json`. Review found weak helper failure diagnostics; the helper now includes args, status, stdout, stderr, and parsed value context in failures, and a control pass found no code issues.

Revision note 2026-06-27: The `schemars` follow-up is now specified in detail. DTO-owned derives or custom `JsonSchema` implementations should produce field-level definitions, while `src/contract/schema.rs` keeps only schema family metadata, deterministic output, and root-level composition for command/data and JSONL record semantics.

Revision note 2026-06-27: The `schemars` refactor was implemented with byte-identical generated schema snapshots. Code review returned no substantive findings. Architecture review requested documenting the new `schemars` dependency in `docs/dev/architecture.md`; that finding was fixed.
