# Schema Command and Versioned JSON Schema Contract

## Summary
- This plan simplifies `wavepeek schema` to a single deterministic behavior: no flags, no input file, and always one JSON Schema document written to stdout.
- The schema source of truth moves to a tracked artifact at `schema/wavepeek.json`, and quality gates enforce that runtime output and repository schema stay in sync.
- JSON envelope metadata migrates from numeric `schema_version` to a versioned `$schema` URL that points to the schema file under the tool's own tag (`blob/vX.Y.Z/...`).
- This keeps machine contracts explicit and self-describing while minimizing release-process overhead for the current release.

## Goals
- Make `schema` command argument-free and deterministic.
- Store canonical schema at `schema/wavepeek.json` in-repo.
- Replace envelope `schema_version` with `$schema` URL in all JSON outputs.
- Enforce schema freshness in pre-commit and CI.
- Add a developer-friendly `make update-schema` workflow for regenerating canonical schema.
- Document contract and release expectations in PRD/changelog/release docs.

## Non-Goals
- Publishing schema as a separate binary release asset in this release.
- Supporting multiple schema variants from `schema` command.
- Adding schema negotiation, discovery, or compatibility layers beyond versioned URL.

## Background / Context
- Current PRD describes `schema` with `--json` mode and human/JSON dual behavior, which conflicts with the requested simplification.
- CLI currently defines `schema --json`, but command execution is still unimplemented.
- JSON envelopes currently include `schema_version: 2` from a constant in `src/output.rs`.
- Pre-commit/CI already run through `make` targets, so schema validation can be added once in Makefile and inherited by both local and CI gates.

## Problem Statement
- The current `schema` surface is over-designed for the immediate need and does not provide a stable single-output contract.
- Numeric `schema_version` requires out-of-band mapping to an actual schema document.
- There is no repository-level guard to ensure schema artifact freshness and consistency with command output.

## Requirements
- **Functional:**
- `SC-R1`: `wavepeek schema` accepts no command-specific flags/arguments and writes exactly one JSON schema document to stdout.
- `SC-R2`: Canonical schema file exists at `schema/wavepeek.json`; command output is produced from an embedded/package-safe copy of this artifact (no dependency on current working directory).
- `SC-R3`: All JSON envelopes replace `schema_version` with `$schema` key.
- `SC-R4`: `$schema` value format is `https://github.com/kleverhq/wavepeek/blob/v<tool-version>/schema/wavepeek.json`.
- `SC-R5`: `schema` command output is deterministic and byte-stable: canonical schema formatting is fixed to pretty JSON in `schema/wavepeek.json` with trailing newline, and command output matches it exactly.
- `SC-R6`: CLI rejects removed schema flags/args (`--json`, `--human`, `--waves`, positional args) with stable `error: args:` diagnostics and help hints.
- **Non-functional / constraints:**
- `SC-R7`: Pre-commit and CI fail if schema artifact is missing, invalid JSON, mismatched with command output, or `$schema` URL string is inconsistent with package version and required URL pattern (string validation only; no network reachability check).
- `SC-R8`: Contract docs (`.agents/PRD.md`, `CHANGELOG.md`) are updated before/with implementation.
- `SC-R9`: Release process documents that tagging `vX.Y.Z` implicitly publishes schema at the tagged blob URL (no extra asset upload required).
- `SC-R10`: Root Makefile defines `update-schema` target that regenerates `schema/wavepeek.json`, and `fix` includes `update-schema`.
- `SC-R11`: `check-schema` failure output explicitly suggests running `make update-schema` when command/file mismatch is detected.

## Proposed Solution
- Canonicalize schema artifact in `schema/wavepeek.json`, embed/package it into the binary, and make engine `schema` path print that exact canonical content to stdout.
- Remove `--json` (and any other command-local formatting flags) from `schema` CLI args; command behavior is singular and always JSON schema text.
- Replace envelope field in `src/output.rs` from `schema_version` to `$schema` (serialized as `$schema`) and generate URL from `env!("CARGO_PKG_VERSION")`.
- Add `update-schema` target in root Makefile to regenerate `schema/wavepeek.json`, and include it in `fix` to keep developer workflow one-command.
- Add schema guard target(s) in Makefile (for example `check-schema`) and wire into both `check` and `ci`; make mismatch errors explicitly recommend `make update-schema`; add pre-commit hook entry if needed for explicit fast feedback.
- Add integration tests for `schema` command success path, removed-flag rejection, and envelope `$schema` field assertions across representative commands.
- Update PRD and changelog to reflect simplified command and new envelope contract.

## Alternatives Considered
- **GitHub release asset URL (`releases/download/...`) as `$schema`:** rejected for now because release workflow does not yet upload schema assets and this adds release coupling/complexity.
- **Keep numeric `schema_version` plus optional `$schema`:** rejected to avoid dual-source contract ambiguity.
- **Generate schema only at runtime without tracked file:** rejected because it weakens reviewability and makes freshness checks harder to reason about.

## Risks and Mitigations
- **Risk:** Breaking JSON consumers expecting `schema_version`. **Mitigation:** document as breaking contract in changelog and lock new contract with integration tests.
- **Risk:** URL/version mismatch during local unreleased development. **Mitigation:** derive URL from `CARGO_PKG_VERSION`; validate string shape and version interpolation in gates without requiring live URL resolution.
- **Risk:** Schema file drifts from runtime output. **Mitigation:** add deterministic output comparison in `check-schema` and run in pre-commit + CI.
- **Risk:** Overly strict byte-compare fails due formatting drift. **Mitigation:** lock canonical pretty format and trailing newline policy; test exact byte equality between command output and schema artifact.

## Rollout / Migration Plan
- Phase 1: Update docs/contracts (PRD + changelog) and record breaking change.
- Phase 2: Implement static schema artifact and simplified `schema` command.
- Phase 3: Migrate envelope metadata key/value to `$schema` URL.
- Phase 4: Add schema consistency checks to Makefile/pre-commit/CI and finalize tests.
- Rollback strategy: revert schema contract migration and command simplification as a single unit to avoid mixed envelope semantics.

## Observability
- Add integration assertions for `$schema` presence/value in `--json` outputs.
- Add command contract tests for `wavepeek schema` deterministic stdout and error behavior for removed flags.
- Keep quality-gate signal explicit with dedicated `check-schema` target failure messages.

## Open Questions
- None blocking for this release scope.

## Assumptions
- Current release accepts a breaking JSON envelope key change (`schema_version` -> `$schema`).
- Tag naming remains `v<version>` (currently stable releases use `vX.Y.Z`), matching current release process and URL format.
- Single-schema contract is sufficient for all current commands in this release.

## Definition of Done
- [x] `D1`: `wavepeek schema` runs without flags and prints one valid JSON schema document to stdout.
- [x] `D2`: `wavepeek schema --json`, `wavepeek schema --human`, `wavepeek schema --waves <file>`, and positional-arg variants fail with stable `error: args:` diagnostics.
- [x] `D3`: `schema/wavepeek.json` exists and validates as JSON.
- [x] `D4`: `wavepeek schema` output matches canonical `schema/wavepeek.json` byte-for-byte.
- [x] `D5`: JSON envelope uses `$schema` key (serialized literally as `$schema`) and no longer includes `schema_version`.
- [x] `D6`: `$schema` value follows `https://github.com/kleverhq/wavepeek/blob/v<version>/schema/wavepeek.json`, where `<version>` is exactly `CARGO_PKG_VERSION`.
- [x] `D7`: Integration tests for `info`, `scope`, `signal` (or equivalent covered commands) assert updated envelope contract.
- [x] `D8`: `make check` and `make ci` include schema consistency verification and pass.
- [x] `D9`: pre-commit executes schema consistency verification (direct hook or via `make` target path) and fails on mismatch.
- [x] `D10`: `.agents/PRD.md`, `CHANGELOG.md`, and `.agents/RELEASE.md` reflect new schema contract and URL publication model.
- [x] `D11`: root `Makefile` exposes `update-schema`, and `make fix` runs it.
- [x] `D12`: `check-schema` mismatch diagnostics explicitly suggest `make update-schema`.

## Implementation Plan (Task Breakdown)

### Task 1: Lock contract/documentation updates (~2-3h)
- Goal: Align source-of-truth docs with the simplified schema command and `$schema` envelope contract.
- Inputs: `.agents/PRD.md`, `CHANGELOG.md`, `.agents/RELEASE.md`, this plan.
- Known-unknowns: final changelog phrasing for breaking machine contract.
- Steps:
1. Update PRD `schema` command section to no-args single-schema stdout behavior.
2. Replace PRD envelope examples/field descriptions from `schema_version` to `$schema` URL.
3. Add changelog entry under `## [Unreleased]` describing breaking envelope key migration.
4. Update release runbook to state that tagged blob URL is the publication endpoint for schema.
- Outputs: Consistent docs baseline for implementation.

### Task 2: Add canonical schema artifact and schema command behavior (~3-4h)
- Goal: Implement deterministic single-output schema command backed by repository artifact.
- Inputs: `src/cli/schema.rs`, `src/engine/schema.rs`, `schema/wavepeek.json`, command integration tests.
- Known-unknowns: embedding path/mechanism choice (`include_str!` vs generated artifact) that best fits current module layout.
- Steps:
1. Create `schema/wavepeek.json` as canonical schema file in locked pretty format with trailing newline.
2. Remove command-local flags from schema CLI args and parser surface.
3. Implement engine schema execution to emit embedded/package-safe canonical schema content to stdout.
4. Add/adjust tests for success path and removed-flag rejection behavior.
- Outputs: Working argument-free `schema` command with deterministic output.

### Task 3: Migrate JSON envelope metadata to `$schema` URL (~2-3h)
- Goal: Make all machine outputs self-describing via versioned schema URL.
- Inputs: `src/output.rs`, JSON envelope tests, command integration tests.
- Known-unknowns: none blocking (version interpolation policy is exact `CARGO_PKG_VERSION` string passthrough).
- Steps:
1. Replace envelope field definition and serialization from `schema_version` to `$schema`.
2. Derive URL from package version and encode expected tagged blob path.
3. Update unit/integration tests to assert `$schema` presence/value and absence of `schema_version`.
4. Verify deterministic output ordering remains stable.
- Outputs: Updated envelope contract across commands/tests.

### Task 4: Enforce schema freshness in local/CI quality gates (~2-3h)
- Goal: Prevent schema drift and version reference regressions.
- Inputs: `Makefile`, `.pre-commit-config.yaml`, `.github/workflows/ci.yml`, schema tests/helpers.
- Known-unknowns: none blocking (checks must execute through both `make check` and `make ci`; hook wiring can be direct or inherited via `make`).
- Steps:
1. Add `update-schema` target that regenerates canonical `schema/wavepeek.json` from command/runtime source of truth.
2. Add `check-schema` target validating file existence, JSON validity, command/file sync, and URL/version consistency.
3. Wire schema checks into `make check` and `make ci`, and include `update-schema` under `make fix`.
4. Ensure pre-commit path executes schema checks (via existing hooks or explicit new hook).
5. Add clear failure messages for mismatch scenarios, including explicit remediation hint: `run make update-schema`.
- Outputs: Automated enforcement in pre-commit and CI.

### Task 5: Final regression sweep and release-readiness validation (~2h)
- Goal: Prove contract migration is complete and safe for current release branch.
- Inputs: all prior tasks, `make` quality gates, updated docs/tests.
- Known-unknowns: none blocking after prior tasks.
- Steps:
1. Run focused tests for schema command and envelope JSON contract.
2. Run `make check` and `make ci` in containerized environment.
3. Confirm DoD checklist closure and collect command evidence in PR/release notes.
- Outputs: Release-ready schema contract migration with verifiable gates.

## Execution Notes (Living)

### 2026-02-19 - Task 1 (docs/contracts) - completed
- Updated `.agents/PRD.md` to define the new canonical contract: `wavepeek schema` is argument-free and always emits a single schema document, while JSON envelopes now use versioned `$schema` URL instead of `schema_version`.
- Updated `CHANGELOG.md` with explicit breaking-change note for `schema_version` -> `$schema`, plus schema command simplification and gate enforcement notes.
- Updated `.agents/RELEASE.md` to document schema publication model via tagged blob URL and to include `make update-schema` in release prep.
- Decision: keep release-time schema publication as implicit tagged source blob (`blob/vX.Y.Z/...`) to avoid release-asset workflow coupling in this iteration.

### 2026-02-19 - Task 2 (schema artifact + command) - completed
- Added canonical schema artifact at `schema/wavepeek.json` in stable pretty-printed JSON format with trailing newline.
- Implemented `schema` command end-to-end: removed command flags, switched engine path from `Unimplemented` to deterministic output of embedded artifact bytes.
- Added `tests/schema_cli.rs` coverage for exact byte match with canonical artifact, JSON validity, and deterministic repeated output.
- Extended CLI contract tests to assert stable `error: args:` failures for removed `schema --json` and positional-argument variants.
- Decision: embed artifact with `include_str!` from `CARGO_MANIFEST_DIR` so runtime is package-safe and independent of current working directory.

### 2026-02-19 - Task 3 (envelope migration to `$schema`) - completed
- Replaced envelope metadata key from `schema_version` to literal `$schema` in `src/output.rs` serialization.
- `$schema` URL now derives from compile-time package version and tagged blob path format (`.../blob/v<version>/schema/wavepeek.json`).
- Updated JSON contract assertions in `tests/info_cli.rs`, `tests/modules_cli.rs`, `tests/signals_cli.rs`, and output unit tests to require `$schema` and ensure `schema_version` is absent.
- Surprise handled: schema command output path needed newline-preserving stdout behavior to avoid accidental double newline when schema text already includes trailing newline.

### 2026-02-19 - Task 4 (quality gates) - completed
- Added root Makefile targets: `update-schema` (regenerate canonical artifact from runtime command) and `check-schema` (artifact presence, JSON validity, trailing newline, runtime/file sync, URL/version contract checks).
- Wired `check-schema` into both `make check` and `make ci`; wired `update-schema` into `make fix`.
- Added `schema-contract` pre-commit hook that runs `make check-schema` for direct local feedback.
- Added helper script `scripts/check_schema_contract.py` to keep validation logic readable and deterministic, including explicit remediation hints (`run make update-schema`) for drift cases.

### 2026-02-19 - Task 5 (regression sweep) - completed
- Focused suites passed: `cargo test --test schema_cli --test cli_contract --test info_cli --test modules_cli --test signals_cli`.
- Full quality gates passed end-to-end: `make check` and `make ci`.
- Additional verification passed: `make check-schema` and `make update-schema`.
- Surprise handled: initial `make update-schema` implementation used direct output redirection and could truncate the source artifact before compile-time embedding; fixed by writing to temp file then moving atomically.

### 2026-02-19 - Reviewer loop - completed
- Reviewer pass 1 found two issues: stale changelog wording about `schema_version` and overly strict URL regex; both fixed.
- Control reviewer found one docs consistency issue (`master` vs `main` in release runbook); fixed in `.agents/RELEASE.md`.
- Final reviewer verdict after fixes: no new substantive findings.
