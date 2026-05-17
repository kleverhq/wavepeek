# Raise `src/**` coverage above 95%

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

## Purpose / Big Picture

After this work, contributors will be able to run the repository coverage workflow and see the implementation code under `src/**` covered at an average of at least 95% across the three headline metrics: line, region, and function coverage. The user-visible proof is straightforward: from the repository root, run `cargo llvm-cov --workspace --all-features --summary-only --ignore-filename-regex '(/tests/|/target/|/\.cargo/registry/|/rustc/)'` or `make coverage-src` in the devcontainer and confirm that the average of line, region, and function percentages for `src/**` is at least 95%.

This plan is intentionally test-first and test-only where possible. The repository already has strong CLI and integration coverage, but several implementation-heavy modules still have large untested branch surfaces. The goal is to close those gaps by adding tests only. If a real bug is discovered in implementation code and fixing it would require logic changes, the test must be added in an expected-failure form when possible, the failure must be recorded here, and logic repair is deferred to a later task.

## Non-Goals

This plan does not intentionally change production logic in `src/**` unless a tiny testability-only adjustment becomes unavoidable and is explicitly recorded here. It does not redesign command behavior, expression semantics, waveform semantics, schema content, or docs content. It does not relax the target by excluding hard files. It does not silently delete or weaken existing tests to improve percentages.

## Current Baseline

The starting point was measured on 2026-05-17 with `cargo llvm-cov --workspace --all-features --summary-only --json` saved under `tmp/coverage-initial.json` and `cargo llvm-cov --workspace --all-features --json --output-path tmp/coverage-full.json`.

Current `src/**` totals at plan start:

- line: 81.91%
- region: 84.77%
- function: 81.28%
- average of the three metrics: 82.65%

Largest initial coverage gaps by total missing items were concentrated in:

- `src/expr/sema.rs`
- `src/expr/eval.rs`
- `src/waveform/mod.rs`
- `src/docs/mod.rs`
- `src/engine/expr_runtime.rs`
- `src/engine/time.rs`
- `src/engine/property.rs`
- `src/engine/docs.rs`
- `src/waveform/expr_host.rs`
- `src/output.rs`

## Context and Orientation

This repository is a single Rust CLI crate. The production code lives under `src/`. Coverage already includes a large integration suite under `tests/`, but low-level helper branches and error paths inside `src/**` are still under-covered.

Several repository-specific testing patterns matter here.

`tests/common/expr_runtime.rs` provides an in-memory expression host and fixture types for expression tests. `tests/common/expr_cases.rs` provides the manifest loader and negative-diagnostic assertions used by expression suites. Existing integration suites under `tests/expression_*.rs` already prove public expression contracts. The missing coverage is mostly below those public seams, inside helper functions and rare branches.

`src/docs/mod.rs`, `src/engine/time.rs`, `src/engine/property.rs`, `src/engine/expr_runtime.rs`, `src/expr/sema.rs`, `src/expr/eval.rs`, `src/waveform/mod.rs`, `src/waveform/expr_host.rs`, `src/output.rs`, and `src/schema_contract.rs` already contain unit tests, so adding more tests inside those files is the least disruptive way to cover internal helpers without changing production visibility.

The repository workflow is container-first. Fast iteration may use direct `cargo test` and `cargo llvm-cov` commands, but final validation must still include the relevant repository-quality commands. Disposable artifacts belong under `tmp/`.

## Strategy

The work will proceed in layers so coverage rises quickly while keeping commits reviewable.

First, add tests for smaller, high-yield modules with concentrated error branches: docs export/search helpers, time parsing and validation, property helper logic, engine expression-runtime helpers, waveform expression-host helpers, output rendering helpers, schema helper tests, and command-name coverage. These should raise function coverage quickly and close many branch-only gaps.

Second, attack the large expression internals in `src/expr/sema.rs` and `src/expr/eval.rs` with targeted unit tests that call private helpers directly from in-file `#[cfg(test)]` modules. These files contain many semantic and runtime helper branches that integration tests do not hit exhaustively.

Third, expand `src/waveform/mod.rs` unit coverage for helper functions, alias mapping, time-window calculations, candidate collection branches, and error paths. This file has many private helpers and format-specific fallbacks that are best covered locally.

After each substantial batch, rerun focused tests, then rerun coverage. Commit after each meaningful milestone or after finishing a file-focused batch, as requested.

## Progress

- [x] (2026-05-17) Collected baseline coverage and identified the main gap files from `tmp/coverage-initial.json` and `tmp/coverage-full.json`.
- [x] (2026-05-17) Read repository workflow docs, test breadcrumbs, source breadcrumbs, and prior expression/runtime execution plans relevant to existing test patterns.
- [x] (2026-05-17) Added focused unit tests for smaller helper-heavy modules: `src/docs/mod.rs`, `src/engine/docs.rs`, `src/engine/time.rs`, `src/engine/property.rs`, `src/engine/expr_runtime.rs`, `src/waveform/expr_host.rs`, `src/output.rs`, `src/schema_contract.rs`, and `src/engine/mod.rs`.
- [x] (2026-05-17) Ran the smaller-module batch through `cargo test --lib` and `cargo llvm-cov --workspace --all-features --summary-only --json > tmp/coverage-after-m1.json`; total `src/**` coverage improved to line 84.02%, region 86.47%, function 84.05%.
- [x] (2026-05-17) Added the first targeted semantic/evaluator/waveform helper batch in `src/expr/sema.rs`, `src/expr/eval.rs`, and `src/waveform/mod.rs`, covering event binding errors, cast/type checks, literal/constant helpers, runtime coercions, selection/shift behavior, event/runtime mismatches, expression-backed waveform sampling, candidate collection, and time/path helper branches.
- [ ] Continue expanding semantic-helper tests in `src/expr/sema.rs`; this file is still the single largest remaining blocker by both line and function coverage.
- [ ] Continue expanding evaluator-helper tests in `src/expr/eval.rs` and the remaining waveform helper tests in `src/waveform/mod.rs`, then remeasure coverage and commit the next batch.
- [ ] Add targeted waveform-helper tests in `src/waveform/mod.rs` for the still-uncovered alias, scope/signal, and fallback branches that remain after the first waveform batch.
- [ ] Run focused waveform tests, measure coverage again, and commit the waveform batch.
- [ ] If overall `src/**` average is still below 95%, iterate on the remaining low-coverage files using the updated coverage report until the threshold is met.
- [ ] Run final validation coverage command(s), record the final percentages here, and commit the completion state.

## Surprises & Discoveries

- Observation: the repository has excellent integration coverage already; the missing coverage is disproportionately concentrated in low-level helper branches and error paths inside a handful of large source files.
  Evidence: initial totals were only 81.91/84.77/81.28 despite the full suite passing with 108 library tests plus the integration suites.

- Observation: `docs/exec-plans/references/plan-template.md` does not exist even though the generic `exec-plan` skill mentions it.
  Evidence: direct read attempt returned `ENOENT`, so this plan is authored from current repository conventions and completed plans instead.

- Observation: the smaller helper-module batch raised total `src/**` coverage by only about 1.4 average points, which is useful but not remotely sufficient by itself.
  Evidence: totals moved from 81.91/84.77/81.28 to 84.02/86.47/84.05, so the remaining climb still depends on the large internals in `src/expr/sema.rs`, `src/expr/eval.rs`, and `src/waveform/mod.rs`.

- Observation: the first direct internals batch on `sema`/`eval`/`waveform` helped, but not enough to make the remaining path obvious. After this batch, `src/expr/sema.rs` is still the dominant hotspot, while `src/docs/mod.rs`, `src/expr/parser.rs`, `src/expr/lexer.rs`, `src/engine/change.rs`, and `src/engine/expr_runtime.rs` now look like the next realistic high-yield files.
  Evidence: totals moved again to 86.18/88.14/85.25, with the biggest remaining misses still in `src/expr/sema.rs` (1273 combined misses), `src/expr/eval.rs` (452), `src/waveform/mod.rs` (424), `src/docs/mod.rs` (323), `src/expr/parser.rs` (335), and `src/expr/lexer.rs` (243).

## Decision Log

- Decision: keep the work test-only unless a genuine production bug blocks useful coverage.
  Rationale: the user explicitly asked for tests without logic changes when possible, and the current gap profile suggests that is realistic.
  Date/Author: 2026-05-17 / Grin

- Decision: prioritize file-local unit tests for private helpers over forcing coverage indirectly through bulky CLI scenarios.
  Rationale: the largest gaps are in private semantic, evaluator, docs, and waveform helpers that are already designed to be tested from in-file `#[cfg(test)]` modules.
  Date/Author: 2026-05-17 / Grin

- Decision: commit after each meaningful batch rather than after every tiny test.
  Rationale: the user asked for frequent commits, but single-test commits would turn the branch into confetti.
  Date/Author: 2026-05-17 / Grin

## Milestones

## Milestone 1: Close the smaller helper-module gaps

This milestone covers modules with compact but under-tested helper surfaces: docs export/search helpers, time parsing and validation, property helper logic, engine expression-runtime helpers, waveform expression-host helpers, output rendering helpers, schema helper tests, and command-name coverage. At the end of this milestone, targeted unit tests for those files should pass and the overall coverage should move materially upward without touching production logic.

Files expected in scope:

- `src/docs/mod.rs`
- `src/engine/docs.rs`
- `src/engine/time.rs`
- `src/engine/property.rs`
- `src/engine/expr_runtime.rs`
- `src/waveform/expr_host.rs`
- `src/output.rs`
- `src/schema_contract.rs`
- `src/engine/mod.rs`

Validation commands:

    cargo test docs::tests engine::time::tests engine::expr_runtime::tests engine::property::tests waveform::expr_host::tests output::tests schema_contract::tests
    cargo llvm-cov --workspace --all-features --summary-only --json > tmp/coverage-after-m1.json

Acceptance for this milestone is that the added tests pass, no production logic changed, and the updated coverage report shows clear gains in the targeted files.

## Milestone 2: Cover semantic and evaluator internals

This milestone attacks `src/expr/sema.rs` and `src/expr/eval.rs`, which together account for a large fraction of the remaining uncovered implementation. The work consists of narrow, behavior-rich unit tests for helper functions and error branches that are difficult or wasteful to reach through only public integration manifests.

Files expected in scope:

- `src/expr/sema.rs`
- `src/expr/eval.rs`

Validation commands:

    cargo test expr::sema::tests expr::eval::tests
    cargo llvm-cov --workspace --all-features --summary-only --json > tmp/coverage-after-m2.json

Acceptance for this milestone is that expression helper coverage rises sharply, especially function coverage, while the existing integration suites remain green.

## Milestone 3: Cover waveform helper internals and finish the climb to 95

This milestone closes the remaining large gap in `src/waveform/mod.rs` and then uses the refreshed coverage report to mop up any stubborn hotspots left elsewhere. The target is not “better than before”; it is at least 95 average across line, region, and function metrics for `src/**`.

Files expected in scope:

- `src/waveform/mod.rs`
- whichever remaining `src/**` files still materially block the 95 threshold after the earlier milestones

Validation commands:

    cargo test waveform::tests
    cargo llvm-cov --workspace --all-features --summary-only --json > tmp/coverage-final.json

Acceptance for this milestone is that the final coverage report proves the required threshold, and the plan records the exact final percentages.

## Outcomes & Retrospective

Current status: in progress.

The helper-module batch is already committed. The next commit should record the first direct internals pass across `src/expr/sema.rs`, `src/expr/eval.rs`, and `src/waveform/mod.rs`, which raised overall totals again but still left the branch well below the 95 target. The remaining work is now a broader campaign, not a single silver bullet: more `sema`/`waveform` depth plus targeted coverage work in docs/parser/lexer/change/runtime helpers.

Final percentages, remaining weak spots, discovered bugs, and lessons learned will be recorded here when the work is done.

Revision Note: 2026-05-17 / Grin - Initial active coverage-closure ExecPlan created from current repository docs, baseline coverage artifacts, source/test inspection, and the user’s explicit test-only-plus-frequent-commit requirement.
Revision Note: 2026-05-17 / Grin - Updated after the first test-only helper-module batch to record completed smaller-file work, the post-batch coverage totals from `tmp/coverage-after-m1.json`, and the fact that the remaining risk is now concentrated in `src/expr/sema.rs`, `src/expr/eval.rs`, and `src/waveform/mod.rs`.
Revision Note: 2026-05-17 / Grin - Updated after the first direct internals batch on `src/expr/sema.rs`, `src/expr/eval.rs`, and `src/waveform/mod.rs` to record the new total coverage state from `tmp/coverage-after-m2a.json` and the widened list of next high-yield targets (`docs`, `parser`, `lexer`, `change`, and `expr_runtime`) beyond the still-dominant `sema` and `waveform` gaps. 