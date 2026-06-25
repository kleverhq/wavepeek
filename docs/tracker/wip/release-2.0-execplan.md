# Prepare wavepeek 2.0.0 with v2 schema and RTL-default sampling

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. It is intentionally self-contained so a contributor with only this branch can continue the release-preparation work.

## Purpose / Big Picture

This work prepares `wavepeek` for a `2.0.0` release instead of the unsafe `1.1.0` minor release candidate. Users gain a clearer RTL debugging default: `change` and `property` require an explicit event trigger, and clock-edge queries sample values immediately before the selected edge by default. Machine clients gain a v2 JSON contract that names exact major.minor schema artifacts and permits additive fields, so future `2.x` minor releases can add metadata with less risk to pinned validators.

The behavior is visible after implementation by running commands such as `wavepeek property --waves tests/fixtures/hand/m2_core.vcd --scope top --on 'posedge clk' --eval "data == 8'h00" --capture match --json` and observing rows with both `time` and `sample_time`, where `sample_time` is the value-sampling timestamp. Raw wildcard scans remain available, but they must be explicit, for example `wavepeek change --waves tests/fixtures/hand/m2_core.vcd --scope top --signals data --on '*' --sample-mode native --json`.

## Non-Goals

This plan does not remove or rewrite historical v0 or v1 schema artifacts. It does not close or modify the existing `rc/1.1.0` pull request unless a maintainer explicitly requests that later. It does not optimize the `change` engines or resolve unrelated benchmark regressions except where benchmark catalogs must be updated to exercise the intended native or pre-edge behavior. It does not change the expression language grammar except for command-level requirements around explicit `--on` and sampling-mode validation.

## Progress

- [x] (2026-06-24 19:47Z) Created branch `rc/2.0.0` from the current `rc/1.1.0` branch at commit `050cf81`.
- [x] (2026-06-24 20:00Z) Ran read-only exploration for schema-versioning impact and CLI sampling impact.
- [x] (2026-06-24 20:04Z) Drafted this initial ExecPlan.
- [x] (2026-06-24 20:09Z) Committed the ExecPlan as `7b646ea docs(tracker): plan v2 release prep`; commit hooks passed.
- [x] (2026-06-24 20:23Z) Ran read-only plan review and recorded required plan fixes.
- [x] (2026-06-24 20:30Z) Incorporated plan-review findings about validation commands, help-contract staging, benchmark catalogs, milestone reconciliation, root schema aliases, and superseding PR #43.
- [x] (2026-06-24 21:48Z) Milestone 1 implementation completed locally: release metadata now targets `2.0.0`, v2.0 schema artifacts exist, runtime/tooling/tests use exact major.minor artifact names, and targeted schema/helper checks pass.
- [x] (2026-06-24 22:10Z) Reviewed Milestone 1 and fixed findings: schema contract now smokes runtime JSONL begin `$schema`, and docs publish tooling rejects invalid `wavepeek_v2.json` / `wavepeek_v1.1.json` aliases.
- [x] (2026-06-24 22:29Z) Milestone 2 implementation completed locally: `change` and `property` require `--on`, default to `pre-edge`, keep explicit wildcard/native flows, and targeted plus full Rust tests pass.
- [x] (2026-06-24 22:46Z) Reviewed Milestone 2 and fixed coverage findings: default pre-edge success cases now omit `--sample-mode`, `change` default pre-edge rejects wildcard/plain/mixed non-edge triggers, and stale parity test naming was corrected.
- [x] (2026-06-24 23:31Z) Milestone 3 implementation completed locally: public docs, packaged skill, maintainer docs, changelog, benchmark catalogs, and benchmark catalog tests now reflect required `--on`, default pre-edge sampling, explicit native raw scans, and v2.0 schema artifacts.
- [x] (2026-06-25 00:02Z) Reviewed Milestone 3 and fixed findings: expression-language wildcard guidance now includes `--sample-mode native`, packaged skill routing uses returned `sample_time` for follow-up `value` queries, and ExecPlan examples are runnable against `tests/fixtures/hand/m2_core.vcd`.
- [x] (2026-06-25 00:18Z) Ran a fresh Milestone 3 control review on commits `d0e49f8..9355b2d`; no substantive findings.
- [x] (2026-06-25 00:40Z) Reconciled GitHub milestone state: retitled closed milestone #1 from `v1.1` to `v2.0.0`, kept its 7 shipped issues closed, and confirmed open issue #35 remains assigned to `v1.2` while other open backlog issues remain unmilestoned.
- [x] (2026-06-25 00:40Z) Ran Milestone 4 release gates: `just update-schema`, `just check-schema`, `just check`, and `just ci` passed with no schema drift.
- [x] (2026-06-25 00:40Z) Ran `just bench-gate v1.0.1 HEAD`; gate failed as expected for this major release. Artifacts are under `tmp/bench-gate/gates/20260625T000806Z-81777d4e939e..fa23afbbf5d5`.
- [ ] Milestone 4 final review and `rc/2.0.0` pull request creation.

## Surprises & Discoveries

- Observation: `wavepeek schema` does not generate a schema from Rust types; it prints a checked-in artifact embedded by `include_str!` through `src/schema_contract.rs`.
  Evidence: `src/engine/schema.rs` returns `CANONICAL_SCHEMA_JSON`, and `src/schema_contract.rs` builds that constant with `include_str!(.../schema/wavepeek_v<major>.json)`.

- Observation: changing Cargo version and schema artifact naming can break compilation before any test runs if the new `schema/wavepeek_v2.0.json` and `schema/wavepeek-stream-v2.0.json` files do not already exist.
  Evidence: `src/schema_contract.rs` embeds schema files at compile time.

- Observation: `change` optimizer and benchmark tests must pin `--sample-mode native` for wildcard and non-edge trigger workloads after the default becomes pre-edge, or they will stop exercising the optimized native engines.
  Evidence: `src/engine/change.rs` routes `SampleMode::PreEdge` to `run_pre_edge_emit`, while native modes select among baseline, fused, and edge-fast engines.

- Observation: the release performance gate uses current benchmark catalogs against both old and new binaries, so v2-only flags such as `--sample-mode` make `v1.0.1` captures unsupported instead of comparable.
  Evidence: the earlier `just bench-gate v1.0.1 HEAD` run on `rc/1.1.0` skipped new `--sample-mode` cases because `v1.0.1` reported `unexpected argument '--sample-mode'`.

- Observation: `just update-schema` is a freshness check for embedded artifacts rather than a schema generator for v2 contract policy.
  Evidence: after manually creating and editing `schema/wavepeek_v2.0.json` and `schema/wavepeek-stream-v2.0.json`, `just update-schema` produced no further schema diff because `wavepeek schema` prints those embedded files.

- Observation: integration tests were not the only callers affected by changing `ChangeArgs.on` and `PropertyArgs.on` from `Option<String>` to `String`; internal unit tests construct those structs directly and must use explicit wildcard strings when bypassing clap.
  Evidence: `cargo test --lib change` initially failed with `expected String, found Option<_>` errors in `src/engine/change.rs` and `src/engine/property.rs` until unit-test struct literals were updated.

- Observation: because pre-commit runs the full Rust and optional FSDB suites, test updates planned for Milestone 3 had to move into Milestone 2 for any code commit to pass hooks.
  Evidence: full `cargo test -q` initially failed in `tests/change_opt_equivalence.rs`, `tests/change_vcd_fst_parity.rs`, `tests/expression_event_runtime.rs`, `tests/fsdb_disabled_cli.rs`, `tests/jsonl_cli.rs`, and `tests/property_vcd_fst_parity.rs`; optional `just run-if-verdi test-fsdb` initially failed in `tests/fsdb_cli.rs`. These suites now pin native sampling or explicit wildcard triggers where their purpose is not default pre-edge behavior.

- Observation: benchmark catalog commands also need contract coverage, not just one-time JSON edits.
  Evidence: Milestone 3 added a benchmark harness unit test that requires every `change` and `property` catalog command to pass `--on`, and requires wildcard, plain-signal, or mixed-trigger workloads to pass `--sample-mode native`. It covers `bench/e2e/tests.json`, `bench/e2e/tests_commit.json`, and `bench/e2e/tests_fsdb.json`.

## Decision Log

- Decision: Prepare `2.0.0`, not `1.1.0`, because the current shipped scope changes the stable machine-output contract by requiring `sample_time` in `change` and `property` rows.
  Rationale: A required field in a strict v1 schema is a breaking contract change for strict clients and belongs in a major release.
  Date/Author: 2026-06-24 / Grin with user direction.

- Decision: Use exact major.minor schema artifact names starting in v2, such as `schema/wavepeek_v2.0.json` and `schema/wavepeek-stream-v2.0.json`.
  Rationale: Exact minor artifacts make schema-visible changes explicit while preserving historical v0/v1 artifacts.
  Date/Author: 2026-06-24 / Grin with user direction.

- Decision: v2 schemas should be extension-friendly by allowing additional object properties while preserving required fields, command discriminators, payload references, enum constraints, and diagnostic invariants.
  Rationale: Allowing additive fields lets pinned v2.0 validators accept compatible future v2.x output. Keeping discriminators and required fields prevents the schema from becoming decorative fog.
  Date/Author: 2026-06-24 / Grin with user direction.

- Decision: v2 schema URL patterns should accept any same-major minor artifact, while runtime output should point at the exact current artifact.
  Rationale: A pinned v2.0 schema must not reject v2.1 output solely because `$schema` points at `wavepeek_v2.1.json`.
  Date/Author: 2026-06-24 / Grin with user direction.

- Decision: `change` and `property` should require `--on`, remove the implicit `*` fallback, and default `--sample-mode` to `pre-edge`.
  Rationale: RTL debugging should default to the values engineers expect at a clock edge, while raw wildcard dump scans should be an explicit native-mode choice.
  Date/Author: 2026-06-24 / Grin with user direction.

- Decision: root Pages artifacts should preserve historical `wavepeek_v0.json`, `wavepeek_v1.json`, and `wavepeek-stream-v1.json` files and should publish/validate exact current v2 artifacts such as `wavepeek_v2.0.json` and `wavepeek-stream-v2.0.json`; there should not be a new ambiguous `wavepeek_v2.json` alias.
  Rationale: historical contracts must remain resolvable, while exact minor artifact names keep the current contract unambiguous.
  Date/Author: 2026-06-24 / Grin after plan review.

- Decision: the final `2.0.0` PR must explicitly say it supersedes PR #43 without pushing to or modifying `rc/1.1.0`.
  Rationale: two release-prep PRs for adjacent versions are easy to merge in the wrong order unless the newer PR states the intended release path.
  Date/Author: 2026-06-24 / Grin after plan review.

## Outcomes & Retrospective

Planning review completed and found fixable gaps in the initial plan: an invalid cargo test command, stale help-contract staging, missing `tests_commit.json` coverage, missing milestone reconciliation, vague root schema alias policy, benchmark uncomparability, and PR #43 supersession wording. Those gaps have been folded into this revision before implementation starts.

Milestone 1 implementation and review are complete. It converts release metadata to `2.0.0`, adds exact-minor v2 schema artifacts, switches runtime/schema tooling/docs deployment helpers/tests to the new artifact names, and validates that v2 schemas remain embedded and extension-friendly. Review follow-up tightened docs artifact-name handling and added a JSONL runtime `$schema` smoke to `check-schema`. Targeted checks passed: `cargo check`, `just check-schema`, `cargo test --test schema_cli --test jsonl_cli`, `just test-aux`, docs helper tests, `cargo fmt --check`, and `just update-schema` without schema drift.

Milestone 2 implementation and review are complete. It makes `--on` required in `change` and `property`, switches `SampleMode` default to `pre-edge`, removes engine-level implicit wildcard fallback, updates command help source, pins old raw/native tests to explicit `--on '*' --sample-mode native`, and adds missing-`--on` coverage. Because commit hooks run the full Rust suite and this environment has Verdi, Milestone 2 also updates optimizer, backend-parity, JSONL, FSDB-disabled, FSDB-enabled, and expression-shadow tests that need explicit native sampling. Review follow-up strengthened default-pre-edge positive coverage and non-edge rejection coverage. Checks passed: `cargo test --test cli_contract --test change_cli --test property_cli`, `cargo test --lib change`, `cargo test --lib property`, full `cargo test -q`, `just run-if-verdi test-fsdb`, `cargo check`, `cargo fmt --check`, and `git diff --check`.

Milestone 3 implementation and review are complete. Public command/reference/troubleshooting docs, maintainer docs, packaged skill guidance, changelog entries, and benchmark catalogs now describe the v2 behavior: `change` and `property` require explicit `--on`, default to pre-edge sampling for edge-only triggers, use `--sample-mode native` for wildcard/plain/mixed triggers, and emit v2.0 schema URLs. Benchmark catalogs now pin native mode for non-edge workloads and include a unit-test invariant to keep that policy from drifting. Review follow-up corrected wildcard guidance, packaged skill follow-up sampling instructions, and runnable ExecPlan examples. A fresh control review found no substantive issues. Checks passed: `python3 -m unittest bench.e2e.test_perf`, `just docs-site-check`, `just check-bench-e2e-fsdb-catalog`, full `cargo test`, `just pre-commit`, and review-fix checks consisting of two fixture-backed example commands, `just docs-site-check`, `python3 -m unittest bench.e2e.test_perf`, and `git diff --check`.

Milestone 4 release gates are partially complete and awaiting final review/PR creation. GitHub milestone #1 is now closed `v2.0.0` with 7 shipped issues and no open issues; `v1.2` still contains open issue #35, and other open backlog items remain unmilestoned. `just update-schema`, `just check-schema`, `just check`, and `just ci` passed with no schema drift. `just bench-gate v1.0.1 HEAD` wrote artifacts to `tmp/bench-gate/gates/20260625T000806Z-81777d4e939e..fa23afbbf5d5` and failed as expected for the major release: `e2e-fst` had 67 comparable tests, 83 skipped v2-only tests, 36 functional mismatches, and 8 timing failures; `e2e-fsdb` had 61 comparable tests, 77 skipped v2-only tests, 33 functional mismatches, and 13 timing failures. All functional mismatches and timing failures were `change` workloads whose default edge sampling changed from native to pre-edge. Revised FST-vs-FSDB parity passed with 138 comparable tests and the existing 5 documented metadata ignores.

## Context and Orientation

`wavepeek` is a Rust CLI for deterministic waveform inspection. The repository uses a root `justfile`; normal local gates are `just check` before handoff and `just ci` for CI-equivalent validation. Binary waveform fixtures such as `.fst` and `.fsdb` are not read as text.

The current branch `rc/2.0.0` was created from `rc/1.1.0`, whose top commit `050cf81 chore(release): prepare v1.1.0` bumps Cargo metadata to `1.1.0` and moves changelog entries into a `1.1.0` section. This plan intentionally converts that release prep to `2.0.0`; it does not modify the existing `rc/1.1.0` branch or PR.

A schema artifact is a checked-in JSON Schema file under `schema/` that `wavepeek schema` prints exactly. In v1 these artifacts are named by major version, for example `schema/wavepeek_v1.json` and `schema/wavepeek-stream-v1.json`. Starting in v2 this plan changes the current artifact naming to exact major.minor, for example `schema/wavepeek_v2.0.json`. Historical v0 and v1 files remain in the repository because they are public contracts.

`additionalProperties` is a JSON Schema keyword. When it is `false`, validators reject object fields that are not listed in `properties`. That strictness caught accidental drift, but it also means adding a new field in a minor release can break pinned validators. In v2, public object shapes should allow additional fields so compatible future minor releases can add metadata. This does not permit deleting required fields or changing existing field meanings.

`sample_time` is the timestamp where values are read or evaluated. `time` is the selected event timestamp. In native sampling these are equal. In pre-edge sampling, `time` remains the selected clock edge while `sample_time` is immediately before that edge.

Key files for schema behavior are `src/schema_contract.rs`, `src/engine/schema.rs`, `src/output.rs`, `schema/`, `tools/schema/check_schema_contract.py`, `tests/schema_cli.rs`, `tests/jsonl_cli.rs`, `tests/common/mod.rs`, `tools/docs/check_deploy.py`, and `tools/docs/publish_docs.py`.

Key files for CLI sampling behavior are `src/cli/sampling.rs`, `src/cli/change.rs`, `src/cli/property.rs`, `src/cli/mod.rs`, `src/engine/change.rs`, `src/engine/property.rs`, integration tests under `tests/`, benchmark catalogs under `bench/e2e/`, public docs under `docs/public/`, and the packaged skill at `docs/skills/wavepeek.md`.

## Open Questions

- Whether the branch-local ExecPlan should remain in the final PR or be removed before merge. The default repository guidance says tracked WIP artifacts should be removed before merging unless a maintainer wants handoff context. Until that decision, keep this file updated and committed.
- How much additional current-only benchmark evidence maintainers want for v2-only `change` and `property` behaviors that cannot be fairly compared to `v1.0.1`. The plan requires the final PR to record which release-gate comparisons remain comparable and which workloads are intentionally uncomparable because of the major CLI/schema break.

## Plan of Work

Milestone 0 is planning. Commit this file, run read-only review on the plan, fix any findings, and only then start implementation. The acceptance for this milestone is a committed plan that names affected files, known risks, validation, and staged implementation order.

Milestone 1 establishes the v2 release and schema foundation. Change `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md` from `1.1.0` release prep to `2.0.0`. Add `schema/wavepeek_v2.0.json` and `schema/wavepeek-stream-v2.0.json` before changing compile-time include paths. Update `src/schema_contract.rs` so runtime URLs and embedded artifact names use major.minor for the current package. Update the `justfile` schema path, `tools/schema/check_schema_contract.py`, `tests/common/mod.rs`, `tests/schema_cli.rs`, `tests/jsonl_cli.rs`, docs deployment helpers, and helper tests to expect exact v2.0 artifact names. For v2 artifacts, set `$schema` URL patterns to accept same-major minor URLs, make object shapes extension-friendly, and keep old v0/v1 artifacts untouched. Validate with `just check-schema`, targeted schema tests, and helper tests.

Milestone 2 changes `change` and `property` command behavior together with generated-help source and help-contract tests. Update `src/cli/sampling.rs` so `SampleMode::PreEdge` is the default. Update `src/cli/change.rs`, `src/cli/property.rs`, and `src/cli/mod.rs` so `--on` is required, help no longer says omission means `*`, default sampling is documented as pre-edge, and wildcard/plain/mixed triggers tell users to pass `--sample-mode native`. Update `src/engine/change.rs` and `src/engine/property.rs` to remove `unwrap_or("*")` fallback, keep wildcard expression support when users explicitly pass `--on '*'`, and emit clear validation when pre-edge is used with wildcard, plain signal, or mixed non-edge triggers. Acceptance is that missing `--on` fails at the CLI layer, `--on 'posedge clk'` defaults to pre-edge, `--on '*' --sample-mode native` preserves raw wildcard scans, and `cli_contract` validates the new help surface in the same reviewed slice.

Milestone 3 updates evidence around the new behavior. Update integration tests, command fixture manifests, snapshots, and benchmark catalogs including both `bench/e2e/tests.json` and `bench/e2e/tests_commit.json`. Tests that are meant to exercise old raw wildcard/native behavior must explicitly pass `--on '*' --sample-mode native`. Tests that are meant to cover RTL clock-edge defaults should omit `--sample-mode` and expect pre-edge rows. Benchmark commands that are intended for the `v1.0.1` release comparison should not be blindly converted to v2-only flags; instead, mark or summarize uncomparable `change` and `property` workloads and preserve comparable evidence for unaffected command families. Update public docs, maintainer docs, schema docs, architecture docs, and `docs/skills/wavepeek.md`. Acceptance is that targeted CLI/doc/schema tests pass, pre-commit benchmark smoke passes, and examples in docs match the actual CLI behavior.

Milestone 4 performs release validation and PR creation. Reconcile the GitHub Milestone for `2.0.0` with shipped scope before PR handoff: move unfinished issues out, ensure shipped issues are closed or correctly assigned, and close the completed milestone if release policy calls for it. Run `just update-schema`, `just check-schema`, `just check`, and `just ci`. Run the manual release performance gate against `v1.0.1` as required for a major release, record the artifact path, and distinguish expected major-contract functional changes from timing evidence by listing comparable command families and uncomparable v2-only workloads. Run a final multi-lane read-only review covering code behavior, schema compatibility, docs, and release process. Push `rc/2.0.0` and open a PR to `main` with explicit caveats, validation results, and a statement that it supersedes PR #43 without modifying `rc/1.1.0`.

### Concrete Steps

All commands run from repository root `/workspaces/wavepeek`.

For Milestone 0:

    git status --short --branch
    git add docs/tracker/wip/release-2.0-execplan.md
    git commit -m "docs(tracker): plan v2 release prep"

Expected result: branch `rc/2.0.0` is ahead by one planning commit, and `git status --short` is clean.

For Milestone 1, create schema artifacts before switching compile-time paths:

    cp schema/wavepeek_v1.json schema/wavepeek_v2.0.json
    cp schema/wavepeek-stream-v1.json schema/wavepeek-stream-v2.0.json

Then edit release metadata, schema constants, tooling, tests, and docs helpers. After edits, run:

    just check-schema
    cargo test --test schema_cli --test jsonl_cli
    just test-aux

Expected result: `wavepeek schema` bytes match `schema/wavepeek_v2.0.json`; JSON envelopes point at `https://kleverhq.github.io/wavepeek/wavepeek_v2.0.json`; schema patterns accept future same-major URLs such as `wavepeek_v2.1.json`; v2 schema validators accept representative records with extra object fields.

For Milestone 2, after CLI, engine, and generated-help contract edits, run:

    cargo test --test cli_contract --test change_cli --test property_cli
    cargo test --lib change
    cargo test --lib property

Expected result: missing `--on` errors are covered, edge-trigger commands default to pre-edge, help text and usage are current, and explicit wildcard native commands still work.

For Milestone 3, after broad test, benchmark, and docs edits, run:

    cargo test
    just docs-site-check
    just check-bench-e2e-fsdb-catalog
    just pre-commit

Expected result: all Rust tests pass, docs build/check passes, FSDB benchmark catalog remains aligned with the FST catalog, and the pre-commit benchmark smoke that reads `bench/e2e/tests_commit.json` passes.

For Milestone 4:

    just update-schema
    just check-schema
    just check
    just ci
    just bench-gate v1.0.1 HEAD

Expected result: schema and quality gates pass. The benchmark gate may fail functional comparison because this is a major release with intentional JSON and CLI-contract changes; if it fails, preserve the artifact path, list which command families remained comparable, list which `change`/`property` workloads were uncomparable because `v1.0.1` lacks v2 flags or semantics, and summarize comparable timing evidence accurately in the PR.

### Validation and Acceptance

The final branch is acceptable when all of the following are true:

- `cargo metadata --no-deps --format-version 1` reports package `wavepeek` version `2.0.0`.
- `wavepeek schema` prints exactly the checked-in `schema/wavepeek_v2.0.json` bytes.
- `wavepeek schema --stream` prints exactly the checked-in `schema/wavepeek-stream-v2.0.json` bytes.
- JSON output envelopes contain `$schema: https://kleverhq.github.io/wavepeek/wavepeek_v2.0.json`.
- JSONL begin records contain `$schema: https://kleverhq.github.io/wavepeek/wavepeek-stream-v2.0.json`.
- v2.0 schemas accept same-major future `$schema` URLs such as `wavepeek_v2.1.json` and extra object fields while preserving required fields and command/payload constraints.
- `wavepeek change` and `wavepeek property` without `--on` fail with argument errors.
- `wavepeek property --on 'posedge clk' ...` defaults to pre-edge sampling and emits `sample_time` for rows.
- `wavepeek change --on '*' --sample-mode native ...` preserves raw wildcard scan behavior.
- `just check` and `just ci` pass before PR creation.
- `just pre-commit` or an equivalent pre-commit benchmark smoke passes after benchmark catalog edits.
- GitHub Milestone state for `2.0.0` is reconciled with shipped scope before PR handoff.
- The final PR body explicitly says it supersedes PR #43 while leaving `rc/1.1.0` untouched.
- Required reviews have either no substantive findings or findings fixed in follow-up commits.

### Idempotence and Recovery

The plan favors small commits. If a milestone fails, inspect `git status --short`, keep user-owned files in `tmp/` intact, and either fix forward or revert only the current milestone commit. Copying v1 schema artifacts to v2.0 names is safe to repeat if the v2 files are still unmodified; after manual v2 edits, do not overwrite them without reviewing the diff. `just update-schema` should be run only after intentional schema edits and should not be used to mask a broken artifact.

If changing `src/schema_contract.rs` causes compilation to fail because a schema file is missing, restore the previous constants or create the missing `schema/wavepeek_v2.0.json` and `schema/wavepeek-stream-v2.0.json` files before rerunning cargo. Do not delete historical schema files.

### Artifacts and Notes

Read-only exploration found these implementation anchors:

    Runtime schema embedding:
      src/schema_contract.rs
      src/engine/schema.rs
      src/output.rs

    Schema tooling and tests:
      justfile
      tools/schema/check_schema_contract.py
      tests/schema_cli.rs
      tests/jsonl_cli.rs
      tests/common/mod.rs
      tools/docs/check_deploy.py
      tools/docs/publish_docs.py

    CLI sampling behavior:
      src/cli/sampling.rs
      src/cli/change.rs
      src/cli/property.rs
      src/cli/mod.rs
      src/engine/change.rs
      src/engine/property.rs
      tests/cli_contract.rs
      tests/change_cli.rs
      tests/property_cli.rs
      tests/change_opt_equivalence.rs
      bench/e2e/tests.json
      bench/e2e/tests_commit.json

The current branch starts with release-prep commit:

    050cf81 chore(release): prepare v1.1.0

This commit will be superseded by later `2.0.0` release-prep edits on `rc/2.0.0`; the original `rc/1.1.0` branch and PR remain untouched.

### Interfaces and Dependencies

`src/schema_contract.rs` must provide these constants after Milestone 1:

    pub const SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/wavepeek_v2.0.json";
    pub const STREAM_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/wavepeek-stream-v2.0.json";
    pub const CANONICAL_SCHEMA_JSON: &str = include_str!(..."schema/wavepeek_v2.0.json");
    pub const CANONICAL_STREAM_SCHEMA_JSON: &str = include_str!(..."schema/wavepeek-stream-v2.0.json");

The exact implementation should still derive those strings from Cargo package version components rather than hard-coding `2.0`, so future `2.1.0` builds point at `wavepeek_v2.1.json`.

`src/cli/sampling.rs` must keep the public values `native` and `pre-edge` for `SampleMode`, with `PreEdge` as the default.

`src/cli/change.rs` and `src/cli/property.rs` must expose a required `--on <EXPR>` argument and a `--sample-mode <MODE>` argument whose default is `pre-edge`.

`src/engine/change.rs` and `src/engine/property.rs` must continue using `event_expr_is_edge_only` to reject pre-edge sampling for wildcard, plain signal, or mixed non-edge event expressions. Wildcard remains valid only when explicitly passed and paired with `--sample-mode native`.

Revision note: Initial ExecPlan created on 2026-06-24 to convert the release-candidate work from `1.1.0` to `2.0.0`, define schema-versioning policy, and stage the CLI sampling behavior change before implementation.

Revision note: Plan review findings incorporated on 2026-06-24. The plan now uses valid cargo commands, moves help-contract updates into Milestone 2, includes `bench/e2e/tests_commit.json` and `just pre-commit`, resolves root schema alias policy, adds milestone reconciliation, defines benchmark evidence handling for uncomparable v2 workloads, and requires the final PR to supersede PR #43 explicitly.

Revision note: Milestone 1 local implementation recorded on 2026-06-24. The plan now reflects completed release metadata/schema infrastructure work and the targeted checks that passed before Milestone 1 review.

Revision note: Milestone 1 review follow-up recorded on 2026-06-24. The plan now reflects the fixed JSONL runtime `$schema` smoke and stricter docs schema artifact alias validation.

Revision note: Milestone 2 local implementation recorded on 2026-06-24. The plan now reflects completed explicit-trigger/default-pre-edge CLI work and targeted validation results before Milestone 2 review.

Revision note: Milestone 2 validation expanded on 2026-06-24. The plan now records that full Rust and optional FSDB tests forced native/wildcard updates in optimizer, parity, JSONL, FSDB-disabled, FSDB-enabled, and expression-shadow suites before commit.

Revision note: Milestone 2 review follow-up recorded on 2026-06-24. The plan now reflects stronger default-pre-edge coverage and corrected parity test naming.
