# Major-Versioned Schema Artifacts

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek` emits a stable JSON envelope for `--json` commands. Today the envelope's `$schema` URL points at a full semver tag such as `v0.5.0/schema/wavepeek.json`, which makes minor and patch releases look like schema-contract changes even when the JSON contract did not break. After this change, the envelope points at a major-versioned schema artifact on the default branch, for example `https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_v0.json` for all `0.x.y` builds. Users can pin validators to the schema for the tool's major version, and maintainers can keep every major schema in `schema/` permanently.

A user can see the completed behavior by running `wavepeek info --waves tests/fixtures/hand/m2_core.vcd --json` and observing that `$schema` uses `main/schema/wavepeek_v0.json` for the current `0.5.0` package. They can run `wavepeek schema` and observe that the output bytes match `schema/wavepeek_v0.json`. The old `schema/wavepeek.json` artifact is removed.

## Non-Goals

This plan does not shorten or remove the `$schema` URL field from the JSON envelope. It keeps the self-describing JSON Schema link and only changes the versioning policy behind that link.

This plan does not create the future V1 contract. It establishes the path and code policy so a future `1.x.y` build will look for `schema/wavepeek_v1.json` and emit the corresponding URL. The actual V1 schema content will be created during the V1 contract change.

This plan does not relax the current JSON contract. Existing envelope fields, command names, warning behavior, and error behavior remain unchanged except for the schema URL value and canonical artifact path.

## Progress

- [x] (2026-06-07T12:09Z) Loaded repository guidance, the `exec-plan`, `ask-review`, and `git-commit` skills, and inspected current schema URL/artifact references.
- [x] (2026-06-07T12:09Z) Created this ExecPlan with the default branch decision set to `main` and the current schema artifact set to `schema/wavepeek_v0.json`.
- [x] (2026-06-07T12:10Z) Committed the initial ExecPlan as `88070e3 docs(schema): plan major schema artifacts`.
- [x] (2026-06-07T12:20Z) Ran read-only plan review in architecture/code and docs/contract lanes; recorded and incorporated findings.
- [x] (2026-06-07T12:22Z) Renamed `schema/wavepeek.json` to `schema/wavepeek_v0.json` and updated the schema's `$schema` URL pattern.
- [x] (2026-06-07T12:23Z) Updated runtime schema constants, tests, and helper automation to use the Cargo package major version.
- [x] (2026-06-07T12:24Z) Updated public docs, maintainer docs, and breadcrumbs that referenced the old full-semver schema path.
- [x] (2026-06-07T12:25Z) Ran focused tests, acceptance checks, `just check-schema`, and `just check`; all passed.
- [x] (2026-06-07T12:27Z) Committed the implementation as `8d925c5 feat(schema): use major-versioned artifacts`.
- [x] (2026-06-07T12:35Z) Ran implementation review in code/automation and docs/contract lanes; fixed the code/automation findings by enforcing the exact schema URL pattern and canonical path.
- [ ] Run final validation, control review, and record outcomes.

## Surprises & Discoveries

- Observation: The current `just update-schema` recipe copies the runtime output of `wavepeek schema` back into the checked-in artifact, and `wavepeek schema` currently reads that same checked-in artifact through `include_str!`.
  Evidence: `src/schema_contract.rs` defines `CANONICAL_SCHEMA_JSON` with `include_str!(.../schema/wavepeek.json)`, and `src/engine/schema.rs` returns that string as the schema command output.

- Observation: Plan review found that `cargo test schema_cli` is the wrong focused command for the integration test file.
  Evidence: The architecture/code reviewer noted that `cargo test schema_cli` filters test names and does not run `tests/schema_cli.rs`; the correct command is `cargo test --test schema_cli`.

- Observation: Plan review found a validation gap around the schema artifact's own envelope `$schema` regex.
  Evidence: The architecture/code reviewer noted that `check_schema_contract.py` could pass with a stale JSON Schema `properties.$schema.pattern` unless the helper explicitly checks that pattern against the new URL and old URL.

- Observation: `CARGO_PKG_VERSION_MAJOR` works as a compile-time environment variable for this crate.
  Evidence: `cargo test --test schema_cli` and `cargo test --lib schema_contract` both passed after `src/schema_contract.rs` and integration tests used `env!("CARGO_PKG_VERSION_MAJOR")`.

- Observation: Implementation review found the checker should enforce more than representative acceptance/rejection.
  Evidence: The code/automation reviewer noted that `tools/schema/check_schema_contract.py` accepted a broadened artifact regex and did not enforce the canonical current-major path or absence of `schema/wavepeek.json`.

## Decision Log

- Decision: Use `main`, not a release tag, in envelope `$schema` URLs.
  Rationale: The user explicitly chose `main`, and the intended contract is that the current major schema file on the default branch is maintained compatibly across minor and patch releases.
  Date/Author: 2026-06-07 / Grin

- Decision: Store the current 0.x schema as `schema/wavepeek_v0.json` and remove `schema/wavepeek.json`.
  Rationale: Keeping both names would create an extra public path to maintain. The user explicitly requested storing the current contract as `wavepeek_v0.json` and deleting `wavepeek.json`.
  Date/Author: 2026-06-07 / Grin

- Decision: Keep the envelope field named `$schema` and keep it as a URL.
  Rationale: The user deferred URL length changes; this work is about removing minor/patch version noise, not changing the envelope shape.
  Date/Author: 2026-06-07 / Grin

- Decision: Derive the schema artifact path and envelope URL from `CARGO_PKG_VERSION_MAJOR` at compile time.
  Rationale: This makes a `1.x.y` build automatically point at `schema/wavepeek_v1.json` and fail early if that major schema artifact is missing.
  Date/Author: 2026-06-07 / Grin

- Decision: Keep public docs focused on observed behavior and put maintainer release/bootstrap policy in maintainer docs.
  Rationale: Plan review pointed out that public reference docs should not drift into release-planning process language. Users need to know which URL appears and what it means; maintainers need to know how to create or update major artifacts.
  Date/Author: 2026-06-07 / Grin

- Decision: Make `check_schema_contract.py` validate the schema artifact's envelope `$schema` regex directly.
  Rationale: Runtime envelope checks alone do not prove that the JSON Schema artifact will accept the new URL or reject the obsolete full-semver URL. The contract helper should catch both failures.
  Date/Author: 2026-06-07 / Grin

- Decision: Require the artifact regex to equal the exact current-major URL pattern and require the checked artifact path to be `schema/wavepeek_v{major}.json`.
  Rationale: Implementation review showed that acceptance/rejection spot checks still allow broadened patterns and duplicate unversioned artifacts. Exact helper checks better protect the public contract.
  Date/Author: 2026-06-07 / Grin

## Outcomes & Retrospective

Implementation is complete and committed. The current binary emits `https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_v0.json` in JSON envelopes, `wavepeek schema` matches `schema/wavepeek_v0.json`, and `schema/wavepeek.json` is absent from the working tree. Focused schema tests, the schema checker, acceptance commands, and `just check` passed before and after implementation-review fixes. Final control review is still pending.

## Context and Orientation

`wavepeek` is a Rust CLI. Its stable machine-readable command output is a JSON object called an envelope. In this plan, an envelope means the object emitted by stable `--json` commands with fields `$schema`, `command`, `data`, and `warnings`. A JSON Schema artifact means the checked-in JSON file that describes valid envelope shapes and command payloads.

The current schema contract is wired through these files:

- `src/schema_contract.rs` defines `SCHEMA_URL`, currently built from the full Cargo package version, and `CANONICAL_SCHEMA_JSON`, currently loaded from `schema/wavepeek.json`.
- `src/output.rs` uses `SCHEMA_URL` when serializing the envelope.
- `src/engine/schema.rs` returns `CANONICAL_SCHEMA_JSON` for `wavepeek schema`.
- `schema/wavepeek.json` is the current checked-in schema artifact. Its own `$schema` property is the JSON Schema draft identifier, while its `properties.$schema.pattern` validates the envelope `$schema` URL.
- `tests/common/mod.rs` computes the expected schema URL used by integration tests.
- `tests/schema_cli.rs` checks that `wavepeek schema` prints the checked-in artifact bytes.
- `tools/schema/check_schema_contract.py` is the helper behind `just check-schema`; it validates artifact freshness and the runtime envelope URL.
- `justfile` exposes `update-schema` and `check-schema` recipes.
- `docs/public/reference/machine-output.md`, command docs under `docs/public/commands/`, and maintainer docs under `docs/dev/` describe the public and maintainer-facing contract.

The nearest local guidance files require schema changes to keep `docs/public/reference/machine-output.md`, `docs/dev/architecture.md`, tests, and helper automation consistent. Public docs should explain stable behavior and avoid release-planning chatter; maintainer docs can describe release and automation workflow.

## Open Questions

There are no open questions blocking implementation. The user resolved the branch name as `main`, requested `schema/wavepeek_v0.json`, requested removal of `schema/wavepeek.json`, and deferred shortening the envelope URL.

## Plan of Work

First, commit this ExecPlan so the branch has a durable, reviewable specification. Then run a read-only review lane focused on whether the plan covers runtime code, tests, automation, public docs, maintainer docs, and breadcrumbs.

Next, rename the schema artifact from `schema/wavepeek.json` to `schema/wavepeek_v0.json`. In the renamed JSON file, replace the envelope `$schema` property pattern with a pattern that accepts only the current major schema URL for this artifact: `^https://raw\.githubusercontent\.com/kleverhq/wavepeek/main/schema/wavepeek_v0\.json$`. This ensures a V0 envelope validates against the V0 schema and no longer accepts full-semver tag paths.

Then update `src/schema_contract.rs`. Define the schema URL as `https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_v{major}.json`, where `{major}` comes from `env!("CARGO_PKG_VERSION_MAJOR")`. Load `CANONICAL_SCHEMA_JSON` from `schema/wavepeek_v{major}.json` with `include_str!`. The current package version is `0.5.0`, so the current build loads `schema/wavepeek_v0.json`.

Then update tests. In `tests/common/mod.rs`, make `expected_schema_url()` use the same `main/schema/wavepeek_v{major}.json` URL. In `tests/schema_cli.rs`, make `canonical_schema_path()` resolve `schema/wavepeek_v{major}.json`, and add or adjust assertions so the schema's envelope `$schema` pattern matches the runtime URL and does not match the old full-semver path. Existing tests in `src/output.rs`, `tests/*_cli.rs`, and FSDB tests should continue passing because they compare against the shared expected URL.

Then update automation. In `justfile`, change `schema_path` to compute `schema/wavepeek_v{major}.json` from `Cargo.toml`. In `tools/schema/check_schema_contract.py`, compute the package major from `Cargo.toml`, default the artifact path to `schema/wavepeek_v{major}.json`, compare `wavepeek schema` against that path, expect the runtime envelope URL on `main`, validate the runtime URL pattern, and compile the artifact's `properties.$schema.pattern` to confirm it accepts the expected URL and rejects the old full-semver URL. Update `tools/schema/README.md`, `docs/dev/automation.md`, `docs/dev/quality.md`, and `docs/dev/release.md` to describe current-major schema artifacts instead of `schema/wavepeek.json` and full-semver publication endpoints. In `docs/dev/release.md`, describe the manual bootstrap requirement for a new major: create `schema/wavepeek_vN.json` before or with the Cargo version bump because `wavepeek schema` embeds an existing checked-in artifact.

Then update public docs and breadcrumbs. In `docs/public/reference/machine-output.md`, change the envelope example and field description to use `main/schema/wavepeek_v<N>.json` and keep the wording focused on observable user behavior: the schema URL names the running tool's major schema artifact, not the full semver release. Put release/update policy details only in maintainer docs. Update command examples under `docs/public/commands/*.md` from the old `v0.5.0/schema/wavepeek.json` URL to `main/schema/wavepeek_v0.json`. Update `docs/public/commands/schema.md` to say `wavepeek schema` prints the current major schema. Update `docs/public/intro.md`, `docs/dev/architecture.md`, `schema/AGENTS.md`, and `docs/public/reference/AGENTS.md` to remove the stale path.

Finally, run focused validation, commit the implementation, run read-only implementation review, fix any findings, and run final validation. At minimum the focused validation should include `cargo test --test schema_cli`, `cargo test --lib schema_contract`, and `just check-schema` if the container guard allows it. Before final handoff, run `just check` as the repository pre-handoff gate if the environment permits it; otherwise record the exact guard failure.

## Concrete Steps

All commands run from repository root `/workspaces/wavepeek`.

Create and commit this plan:

    git add docs/tracker/wip/schema-major-contract.md
    git commit -m "docs(schema): plan major schema artifacts"

Run a read-only plan review through the available subagent backend. The reviewer should inspect this plan and current repository references, then report concrete findings only.

Apply the implementation edits:

    git mv schema/wavepeek.json schema/wavepeek_v0.json

Then edit the files named in the Plan of Work section. Use precise edits for source and docs. Avoid hand-editing unrelated generated or target files.

Run focused validation while iterating:

    cargo test --test schema_cli
    cargo test --lib schema_contract
    just check-schema

Observed results on 2026-06-07:

    cargo test --test schema_cli
    Test Results: 9 passed

    cargo test --lib schema_contract
    Test Results: 3 passed

    just check-schema
    passed with no stderr

Run the pre-handoff gate:

    just check

Observed result on 2026-06-07 before implementation review: `just check` passed, including rustfmt check, justfile format check, clippy, schema contract validation, actionlint, cargo check, commit-message validation, and FSDB build smoke checks.

Observed result on 2026-06-07 after implementation-review fixes: `just check` passed with the same gate set.

Commit implementation once focused validation passes or any environment limitations are recorded:

    git add <changed files>
    git commit -m "feat(schema): use major-versioned schema artifacts"

Run read-only implementation review against the implementation commit range and fix substantive findings. After any fix, rerun affected checks and commit the fix with an appropriate conventional message.

## Validation and Acceptance

Acceptance is behavioral. After implementation, running this command:

    cargo run --quiet -- info --waves tests/fixtures/hand/m2_core.vcd --json

must print a JSON object whose `$schema` value is exactly:

    https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_v0.json

The old value containing `v0.5.0/schema/wavepeek.json` must not appear in the output.

Running this command:

    cargo run --quiet -- schema > tmp/schema.out

must produce bytes identical to the checked-in file:

    cmp tmp/schema.out schema/wavepeek_v0.json

The old artifact path must be absent from the tracked tree:

    test ! -e schema/wavepeek.json

Focused tests must pass:

    cargo test --test schema_cli
    cargo test --lib schema_contract

The schema contract helper must pass inside the managed container:

    just check-schema

The repository pre-handoff gate should pass inside the managed container:

    just check

## Idempotence and Recovery

The rename from `schema/wavepeek.json` to `schema/wavepeek_v0.json` is safe to repeat only if the old path still exists. If the rename has already happened, do not recreate the old file. If an edit goes wrong, use `git diff` to inspect the affected path and `git restore <path>` to return that file to the last commit.

`tmp/` is the correct location for disposable validation outputs. Do not delete arbitrary existing files in `tmp/`; remove only files created by this plan, such as `tmp/schema.out`.

If `just check-schema` or `just check` fails only because `WAVEPEEK_IN_CONTAINER=1` is not set, record that exact stderr and run the closest cargo tests directly. Do not bypass hooks or quality gates when they are available.

## Artifacts and Notes

Initial inspection found these old-path references that were updated:

    src/schema_contract.rs
    tests/common/mod.rs
    tests/schema_cli.rs
    schema/wavepeek.json -> schema/wavepeek_v0.json
    justfile
    tools/schema/check_schema_contract.py
    tools/schema/README.md
    docs/public/reference/machine-output.md
    docs/public/commands/*.md examples
    docs/dev/architecture.md
    docs/dev/automation.md
    docs/dev/quality.md
    docs/dev/release.md
    schema/AGENTS.md
    docs/public/reference/AGENTS.md

Acceptance output observed on 2026-06-07:

    cargo run --quiet -- info --waves tests/fixtures/hand/m2_core.vcd --json
    {"$schema":"https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_v0.json","command":"info","data":{"time_unit":"1ns","time_start":"0ns","time_end":"10ns"},"warnings":[]}

    cargo run --quiet -- schema > tmp/schema-major-contract.out
    cmp tmp/schema-major-contract.out schema/wavepeek_v0.json
    test ! -e schema/wavepeek.json
    all commands passed

## Interfaces and Dependencies

`src/schema_contract.rs` must expose these stable constants at the end of the work:

    pub const SCHEMA_URL: &str = "https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_v<major>.json";
    pub const CANONICAL_SCHEMA_JSON: &str = include_str!("<repo>/schema/wavepeek_v<major>.json");

The exact implementation should use `concat!` and `env!("CARGO_PKG_VERSION_MAJOR")` so the values are compile-time constants and the binary fails to build if the schema file for its major version is missing.

`tools/schema/check_schema_contract.py` must remain deterministic and non-interactive. It should read `Cargo.toml` with Python's standard-library `tomllib`, compute the major version from the `package.version` string, and validate the same URL pattern used by the runtime envelope.

## Revision Notes

- 2026-06-07T12:09Z: Created the initial self-contained ExecPlan from repository inspection and the user's decision to use `main` plus `schema/wavepeek_v0.json`.
- 2026-06-07T12:20Z: Incorporated read-only plan review findings: corrected the schema integration test command, required helper validation of the schema artifact regex, moved release/update policy detail out of public-doc wording, and documented new-major bootstrap risk in maintainer docs.
- 2026-06-07T12:25Z: Updated implementation progress and validation evidence after code, schema, docs, and helper changes passed focused checks and `just check`.
- 2026-06-07T12:27Z: Recorded implementation commit `8d925c5 feat(schema): use major-versioned artifacts` before starting implementation review.
- 2026-06-07T12:35Z: Recorded implementation review findings and fixes: exact artifact regex enforcement, canonical current-major path enforcement, obsolete unversioned artifact rejection, and rerun validation.
