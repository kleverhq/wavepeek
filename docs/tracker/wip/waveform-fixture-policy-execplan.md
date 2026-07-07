# Waveform Fixture Source Policy ExecPlan

This plan implements GitHub issue #55 for the `wavepeek` repository. After the change, ordinary waveform test data is regenerated from checked-in HDL source instead of being stored as checked-in dumps, while hand-written VCD dumps remain only where the VCD syntax or metadata is part of the test. A contributor can run one documented command, `just prepare-waveform-fixtures`, to create the ignored generated VCD/FST files used by tests.

The worktree is `/workspaces/wavepeek/.worktrees/fix-waveform_fixture_policy` on branch `fix/waveform_fixture_policy`. Keep all edits in this worktree. Repository commands are run from the repository root. The repository is container-first; standard local pre-handoff validation is `just check`, and full CI-style validation is `just ci`.

## Repository orientation

`wavepeek` is a Rust CLI that inspects waveform dumps. VCD is a text waveform format. FST is a binary waveform format. FSDB is an optional proprietary binary waveform format used only when local Verdi tooling is available. The integration tests under `tests/` invoke the CLI and use waveform fixtures from `tests/fixtures/`.

Before this plan, all small reusable VCD and FST fixtures lived under `tests/fixtures/hand/`. That made it unclear which dumps were intentionally hand-written VCD corner cases and which were ordinary signal/value fixtures. The target layout is:

- `tests/fixtures/hand/` contains only checked-in VCD dumps whose raw VCD syntax, scope metadata, event variables, real values, or same-timestamp event ordering is part of the test.
- `tests/fixtures/source/` contains checked-in Verilog source for ordinary reusable waveform fixtures.
- `tests/fixtures/generated/` contains ignored VCD/FST dumps generated from source-backed fixtures or derived from hand VCDs when a binary FST copy is needed for parity tests.
- `tests/fixtures/waveform_policy.json` records why each checked-in hand dump exists and which generated outputs are expected.

The generator belongs under `tools/waveform/` because it is repository automation. `justfile` is the stable interface for contributors and CI. `docs/dev/testing.md` owns the human fixture policy. `docs/dev/fsdb.md` owns FSDB-specific fixture generation details.

## Progress

- [x] Read issue #55 and inspect current tracked waveform fixtures.
- [x] Create this ExecPlan in `docs/tracker/wip/waveform-fixture-policy-execplan.md`.
- [x] Add source-backed fixture manifest and Verilog sources.
- [x] Add deterministic waveform fixture generator and `just prepare-waveform-fixtures`.
- [x] Move ordinary reusable VCD/FST dumps out of the tracked hand directory.
- [x] Update fixture lookup helpers, schema validation, and FSDB preparation to use generated fixtures.
- [x] Document the policy in maintainer docs and test guidance.
- [x] Add policy tests that reject undocumented checked-in dumps and tracked FSDB fixtures.
- [x] Update the devcontainer/CI image and Codex setup dependencies so Icarus Verilog and VCD-to-FST conversion are available.
- [ ] Run focused tests, `just check`, and the highest feasible full gate.
- [ ] Run code review, address findings, commit, push the branch, and open a PR.

## Milestone 1: declare the policy in files that tools can enforce

Create `tests/fixtures/waveform_policy.json`. It must have three top-level arrays. `hand_dumps` lists each checked-in `.vcd` or `.fst` under `tests/fixtures/hand/` with a non-empty `reason`. `source_backed` lists each checked-in HDL source under `tests/fixtures/source/` and the ignored generated outputs under `tests/fixtures/generated/`. `derived_outputs` lists generated binary outputs derived from hand VCDs, such as an FST generated from `expr_triggered_collision.vcd` for cross-format tests.

Add `tests/waveform_fixture_policy.rs`. This integration test reads the manifest, checks that every checked-in hand dump is listed, checks that every source file is listed, checks that generated outputs exist after `just prepare-waveform-fixtures`, and checks that no `.fsdb` files exist under `tests/fixtures/`. This provides an observable failure if someone adds a dump without documenting why it is tracked.

At the end of this milestone, running `cargo test -q --test waveform_fixture_policy` after fixture generation should pass. If generated fixtures are missing, the test should fail with an explicit message naming `just prepare-waveform-fixtures`.

## Milestone 2: generate ordinary waveform dumps from HDL source

Add `tools/waveform/prepare_fixtures.py`. The helper reads `tests/fixtures/waveform_policy.json`, compiles each `source_backed` Verilog file with `iverilog -g2012`, runs the compiled simulation with `vvp`, canonicalizes the generated VCD header date/version so output is deterministic, and writes requested VCD outputs under `tests/fixtures/generated/`. For requested FST outputs, it runs `vcd2fst` from the generated VCD. For `derived_outputs`, it converts the listed hand VCD to FST under `tests/fixtures/generated/`.

The helper must be deterministic and retry-safe. It should write through temporary files in `tmp/waveform-fixtures/` and atomically replace final generated outputs. It should validate that `iverilog`, `vvp`, and `vcd2fst` are on `PATH` before doing work and produce clear error messages if they are missing.

Add `just prepare-waveform-fixtures` as the single documented command. Make `just test`, `just ci` paths that execute tests, and schema validation prepare these fixtures before they need them.

At the end of this milestone, running `just prepare-waveform-fixtures` should create the ignored `tests/fixtures/generated/` directory with the expected VCD/FST files.

## Milestone 3: migrate obvious ordinary fixtures

Move ordinary reusable dumps from `tests/fixtures/hand/` into source-backed generation. The first and most important fixture is `m2_core`, because many CLI tests use it for ordinary hierarchy, signal, value, change, property, JSON, and schema behavior. It must produce both `m2_core.vcd` and `m2_core.fst` in `tests/fixtures/generated/`.

Also migrate ordinary behavior fixtures that do not need raw VCD syntax: `change_edge_cases`, `change_from_boundary`, `change_many_events`, `change_property_core`, `change_property_offset_start`, `change_scope_ambiguous`, `signal_recursive_depth`, `value_delayed`, and `value_vectors`. These should be generated as VCD files from source.

Keep checked-in hand VCDs only for format-specific cases: `same_time_updates.vcd` for multiple same-timestamp value changes and event ordering, `change_property_events.vcd` and `expr_triggered_collision.vcd` for VCD event variables, `change_property_real_output.vcd` and `value_real.vcd` for VCD real-value syntax, and `scope_mixed_kinds.vcd` for non-module VCD scope kinds. Remove checked-in FST files unless a true FST-specific corner case is documented. The existing FST copies are derived parity fixtures, so they belong under `tests/fixtures/generated/`.

At the end of this milestone, `git ls-files '*.vcd' '*.fst' '*.fsdb'` should show no `.fsdb`, no checked-in `.fst` unless explicitly justified, and only hand VCDs with manifest reasons.

## Milestone 4: update consumers and documentation

Update `tests/common/mod.rs` so `fixture_path("name.vcd")` and `fixture_path("name.fst")` resolve generated fixtures first and hand fixtures second. This avoids changing every integration test call site while still moving the dump source of truth.

Update any direct literal paths that bypass `fixture_path`, including schema contract validation and JSON fixture manifests, to point at generated source-backed fixtures when they use `m2_core`. Update source unit-test helpers under `src/` that have their own test-only fixture path helper.

Update `tools/fsdb/prepare_fsdb_fixtures.sh` so optional FSDB test fixtures are derived from both hand VCDs and generated source-backed VCDs. Make the corresponding just recipes prepare source-backed fixtures first.

Update `docs/dev/testing.md`, `docs/dev/fsdb.md`, `docs/dev/environment.md`, and `docs/dev/automation.md` where needed. The docs must state that Icarus Verilog is the source-backed fixture generator, generated VCD/FST files live under ignored `tests/fixtures/generated/`, FSDB is never checked in, and `just prepare-waveform-fixtures` is the regeneration command.

At the end of this milestone, test code should not reference removed hand fixture paths, and maintainer docs should describe the durable policy rather than issue history.

## Milestone 5: validate, review, and publish

Run `just prepare-waveform-fixtures`, focused fixture policy tests, and focused CLI tests that use generated fixtures. Then run `just check`. If the environment allows the full test gate in time, run `just ci`; otherwise run the broadest feasible subset and record the exact reason any gate could not run.

Run a code review pass before finalizing. Use subagents or focused manual review for at least fixture policy/generator correctness and test-consumer compatibility. Address real findings before committing final changes.

Commit with a conventional commit message. Push `fix/waveform_fixture_policy` to `origin` and open a GitHub PR against the repository default branch. The PR body should summarize the policy, migration, generator command, and validation evidence.

## Surprises & Discoveries

- The current container did not have `iverilog` or `vvp` on `PATH`. A temporary user-local unpacked Ubuntu package was installed under `~/.local/opt/iverilog-deb` to allow development without rebuilding the container. The repository image still needs to install Icarus Verilog properly.
- `vcd2fst` is currently available in the development container, but it comes from waveform GUI tooling and must be available in CI if tests need generated FST fixtures.
- `m2_core` can be generated by Icarus while preserving the public signal set if the testbench dumps only selected wires, registers, and parameters instead of `$dumpvars(0, top)`. This avoids leaking driver registers and intermediate wires into CLI signal lists.
- Icarus emits repeated `$scope module top` blocks when dumping selected hierarchical signals. Wellen can read these VCDs, but Verdi `vcd2fsdb` preserves them as ambiguous FSDB canonical scope paths. The generator now merges duplicate VCD scope definitions before writing generated VCDs.

## Decision Log

- Decision: generated fixtures live under `tests/fixtures/generated/` and are ignored. Rationale: tests can use stable repository-relative paths while source files and manifest remain the checked-in source of truth.
- Decision: `fixture_path` searches generated fixtures before hand fixtures. Rationale: most tests care about behavior and should not need call-site churn when a fixture moves from checked-in dump to generated dump.
- Decision: keep hand VCDs for event variables, real values, missing initial values, same-time updates, and explicit VCD scope-kind metadata. Rationale: those tests depend on VCD syntax, metadata, or absence of a time-zero value that should remain visible and reviewed as text.
- Decision: remove checked-in FST parity fixtures and regenerate them. Rationale: no existing FST fixture has a documented FST-specific corner case; generated FST outputs satisfy cross-format tests without storing binary dumps.
- Decision: canonicalize generated VCD scope definitions by merging duplicate scope paths. Rationale: selected `$dumpvars` calls keep signal sets small but produce repeated scopes that break FSDB conversion; merging preserves the same signal paths while producing converter-safe VCD.

## Outcomes & Retrospective

Implementation is in progress. The manifest, HDL sources, generator, helper path changes, docs updates, image/Codex dependency changes, and focused tests have been added. Focused tests that have passed so far include `cargo test -q --test waveform_fixture_policy`, `cargo test -q --test schema_cli`, `cargo test -q --test info_cli --test signal_cli --test value_cli`, `cargo test -q --test change_cli`, selected expression-host unit tests, selected FSDB-disabled tests, `cargo test -q`, `just test-fsdb`, and selected fixture-contract tests. Full gates and review remain.

## Revision Notes

- Initial plan created after reading issue #55 and auditing the current fixture layout. The plan records the target source-backed fixture workflow, the likely migration set, validation strategy, and image dependency update.
- Updated after implementing the manifest, generator, fixture migration, docs, image dependency changes, and focused tests. `value_delayed.vcd` remains a hand fixture because the missing initial value is represented directly in VCD syntax.
