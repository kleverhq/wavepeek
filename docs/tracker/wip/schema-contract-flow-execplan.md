# Rework schema contracts to code-first generated snapshots

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. It is self-contained for this branch and does not require the external proposal file that initiated the work.

## Purpose / Big Picture

After this change, users and release tooling can rely on two stable schema families, `wavepeek.output` for `--json` envelopes and `wavepeek.stream-record` for JSONL records, without tying schema artifact names to the `wavepeek` binary minor version. The runtime machine output will serialize through Rust contract data-transfer objects, and `just update-schema` will regenerate committed schema snapshots from that contract layer instead of round-tripping the schema embedded in the binary.

The behavior is visible by running `just update-schema`, `just check-schema`, `wavepeek schema`, and `wavepeek schema --stream` from the repository root. `wavepeek schema` must print `schema/output.json` byte-for-byte, `wavepeek schema --stream` must print `schema/stream.json` byte-for-byte, and runtime `--json` / `--jsonl` outputs must contain exact schema URLs from `schema/catalog.json`.

## Non-Goals

This plan does not introduce one canonical schema per command. It does not remove historical schema artifacts such as `schema/wavepeek_v1.json` or `schema/wavepeek_v2.0.json`. It does not change the JSON or JSONL public wire shape except for the schema artifact URL values. It does not dynamically generate schemas in normal production CLI execution. It does not automatically decide whether a schema change is major or minor; the version constants remain explicit maintainer-owned values.

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
- [ ] Commit, push, and open a pull request.

## Surprises & Discoveries

- Observation: The current `just update-schema` recipe is artifact-first, not code-first. It runs `cargo run -- schema` and writes the embedded schema output back to the checked-in schema file.
  Evidence: `justfile` defines `update-schema` with `cargo run --quiet -- schema > "$tmp_file"` and `src/schema_contract.rs` embeds `schema/wavepeek_v{major}.{minor}.json` through `include_str!`.

- Observation: The current runtime serializes public JSON and JSONL directly from engine structs.
  Evidence: `src/output.rs::render_json` wraps `result.data` in `OutputEnvelope`, and `src/output.rs::write_jsonl_result` passes engine payloads directly to `JsonlWriter::item`.

- Observation: Current tests already cover many contract semantics that should be preserved: diagnostics rules, extension-friendly objects, command-to-payload JSONL records, scope/signal kind enums, and byte-for-byte schema command output.
  Evidence: `tests/schema_cli.rs`, `tests/jsonl_cli.rs`, and `tools/schema/check_schema_contract.py` validate these constraints.

- Observation: The Python environment used by the local helper tests does not provide the `jsonschema` package.
  Evidence: `python3 -B tools/schema/check_schema_contract.py --schema-dir schema --generated-dir tmp/schema-check` failed with `ModuleNotFoundError: No module named 'jsonschema'` before semantic validation moved into `tools/schema-gen --validate`.

- Observation: Documentation deployment needed explicit catalog artifact handoff; deriving schema artifact names from the CLI version would break independent schema-family versioning.
  Evidence: `tools/docs/check_deploy.py` originally derived `schema-output-v{major}.{minor}.json` from `--version`; it now accepts `--schema-artifact` and `--stream-schema-artifact`, and `tools/docs/workflow_docs.py` passes those values from staged metadata.

- Observation: Publication compatibility for historical artifacts required a separate legacy path.
  Evidence: Post-implementation tooling review found that old no-catalog v2.0 deployments could be blocked by new catalog assumptions and that historical `wavepeek_v*.json` artifacts needed the same immutable overwrite guard as new `schema-output-v*.json` artifacts.

## Decision Log

- Decision: Use a manual deterministic Rust schema builder in `src/contract/schema.rs` rather than introducing a derive-based schema dependency in the first migration.
  Rationale: The current schema has command-to-payload conditionals, extension-friendly objects, diagnostic conditionals, exact URL constants, and stable definition names. A small explicit builder preserves the public schema shape and keeps the migration reviewable. The builder still lives next to the Rust contract DTO and is invoked by `tools/schema-gen`, removing the current binary round-trip.
  Date/Author: 2026-06-26 / coding agent

- Decision: Keep historical `schema/wavepeek_v*.json` and `schema/wavepeek-stream-v*.json` artifacts in the repository while adding new current snapshots `schema/output.json` and `schema/stream.json`.
  Rationale: Historical artifacts are already public contracts. The new family-based snapshots become the embedded current artifacts without deleting prior published files.
  Date/Author: 2026-06-26 / coding agent

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

## Outcomes & Retrospective

The implementation and review milestones are complete. Runtime JSON and JSONL now serialize through `src/contract` DTOs, generated snapshots live at `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`, `just check-schema` validates generated freshness plus runtime embedding, and docs publication handles both catalog-based exact artifacts and historical no-catalog artifacts. Remaining work is commit, push, and PR.

## Context and Orientation

`wavepeek` is a Rust CLI. Its machine output has two formats. The `--json` format is a single JSON object called an envelope. The envelope has a `$schema` URL, a `command` string, a command-specific `data` payload, and a `diagnostics` array. The JSONL format prints one JSON object per line; each line is a stream record with `type` equal to `begin`, `item`, `diagnostic`, or `end`.

The current schema runtime is in `src/schema_contract.rs`. It constructs schema URLs from `CARGO_PKG_VERSION_MAJOR` and `CARGO_PKG_VERSION_MINOR`, embeds `schema/wavepeek_v2.0.json` and `schema/wavepeek-stream-v2.0.json`, and exposes those strings to the CLI. `src/engine/schema.rs` returns the embedded strings for `wavepeek schema` and `wavepeek schema --stream`.

The current JSON and JSONL serialization entrypoint is `src/output.rs`. `render_json` serializes `crate::engine::CommandData` directly inside `OutputEnvelope`. `JsonlWriter::item` serializes any `serde::Serialize` payload passed by engine code. `src/engine/change.rs` and `src/engine/property.rs` stream JSONL rows through sink types that call `JsonlWriter::item`.

The engine modules currently define the payload structs that appear on the wire: `src/engine/info.rs::InfoData`, `src/engine/scope.rs::ScopeEntry`, `src/engine/signal.rs::SignalEntry`, `src/engine/value.rs::ValueSnapshot`, `src/engine/change.rs::ChangeSnapshot`, `src/engine/property.rs::PropertyCaptureRow`, and docs JSON payloads in `src/engine/mod.rs` plus `src/docs/mod.rs`.

The new contract layer will live under `src/contract/`. A contract data-transfer object, abbreviated DTO, is a Rust struct or enum whose purpose is to describe the public wire shape. Engine structs may continue to exist for computation and human rendering, but machine output must convert engine results into contract DTOs before `serde_json` serializes them. The schema generator must be based on the same contract module and explicit schema metadata.

The helper entrypoints are in `justfile`. `just update-schema` must generate `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`. `just check-schema` must generate fresh files under `tmp/schema-check` and compare them with committed snapshots before checking runtime output. `tools/schema/check_schema_contract.py` currently assumes Cargo major/minor artifact names and must become catalog-based.

Docs publication lives in `tools/docs/publish_docs.py`. Deployed-doc validation lives in `tools/docs/check_deploy.py`. Both currently infer artifact names from the CLI version and historical glob patterns. They must read `schema/catalog.json` for current exact family artifacts and preserve historical artifact support.

## Open Questions

There are no blocking open questions. If implementation shows that preserving output schema version `2.0` while changing the `$schema` URL is treated as a semantic contract change by tests or tooling, record that finding here and update the version constants or tests consistently.

## Plan of Work

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

Finally, run formatting, focused tests, `just check-schema`, docs helper tests, and `just check`. Request a post-implementation review, address findings, then commit the branch and open a pull request. Confirm the external proposal file was not added to git.

### Concrete Steps

Run all commands from `/workspaces/wavepeek/.worktrees/feat-rework-schema-flow`.

1. Inspect status before editing:

    git status --short --branch

   Expected result: branch `feat/rework-schema-flow`; no tracked changes except files intentionally edited by this plan.

2. Generate or refresh schema artifacts after implementing the contract generator:

    just update-schema

   Expected result: `schema/output.json`, `schema/stream.json`, and `schema/catalog.json` are created or updated. Historical schema artifacts remain present.

3. Check schema freshness and runtime embed behavior:

    just check-schema

   Expected result: the script reports success. If generated files differ, it fails with a hint to run `just update-schema`.

4. Run focused Rust tests:

    cargo test -q schema_cli jsonl_cli

   Expected result: schema and JSONL CLI tests pass. If the exact filter does not select all desired tests, run `cargo test -q --test schema_cli --test jsonl_cli`.

5. Run helper tests affected by docs publication and schema checks:

    python3 -B -m unittest discover -s tools/docs -p "test_*.py"
    python3 -B -m unittest discover -s tools/schema -p "test_*.py"

   Expected result: all helper tests pass. If `tools/schema` has no tests, the command should report zero tests rather than fail.

6. Run formatting and the local pre-handoff gate:

    just format
    just check

   Expected result: formatting completes and `just check` passes.

7. Inspect final diff and ensure the proposal file is not staged:

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
- Representative runtime `--json` outputs validate against `schema/output.json` and representative runtime JSONL records validate against `schema/stream.json`.
- Historical schema files remain in `schema/` and are not modified by `just update-schema`.
- Docs publication stages exact catalog artifacts for every release and refuses to overwrite an existing exact artifact path with different bytes.

### Idempotence and Recovery

`just update-schema` is safe to run repeatedly; if the Rust contract did not change, it should produce no git diff. `just check-schema` writes disposable generated output under `tmp/schema-check`, which is ignored scratch space. Do not delete arbitrary existing files in `tmp/`; only overwrite or remove paths owned by this plan, such as `tmp/schema-check`.

If generated schemas are wrong, restore committed snapshots with `git checkout -- schema/output.json schema/stream.json schema/catalog.json`, fix the Rust contract schema builder, and rerun `just update-schema`. If docs publication tests fail after tooling changes, keep the old tests as evidence of required behavior and update the code before changing expectations.

### Artifacts and Notes

Current relevant facts before implementation:

    src/schema_contract.rs embeds schema/wavepeek_v{major}.{minor}.json and schema/wavepeek-stream-v{major}.{minor}.json.
    just update-schema runs cargo run -- schema and cargo run -- schema --stream.
    tests/schema_cli.rs expects current artifact paths derived from Cargo package major/minor.
    tools/docs/publish_docs.py copies schema/wavepeek_v*.json and schema/wavepeek-stream-v*.json root artifacts, with latest-promotion behavior.

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

Revision note 2026-06-26: Initial plan created after reading the proposal and current code. The plan records the artifact-first current state and chooses an explicit Rust schema builder to keep the migration narrow and deterministic.

Revision note 2026-06-26: Plan review findings were incorporated. The plan now defines the hidden library boundary used by `tools/schema-gen`, canonical catalog paths independent of `--out`, DTO bypass mitigation, deployed catalog handoff through staged metadata, `just test-aux` coverage for new schema helper tests, `tools/docs/README.md` updates, and an executable proposal-not-committed check.

Revision note 2026-06-26: Implementation progress recorded. The plan now reflects the `families` catalog shape without an `artifact` field, Rust-based schema semantic validation, completed contract/generator/tooling/doc updates, and the remaining validation/review/PR work.

Revision note 2026-06-26: Post-implementation review findings and fixes recorded. The docs publication path now preserves historical no-catalog artifacts, applies immutable overwrite checks to all versioned schema artifacts, and passes exact schema artifact names through staged metadata for new catalog releases.
