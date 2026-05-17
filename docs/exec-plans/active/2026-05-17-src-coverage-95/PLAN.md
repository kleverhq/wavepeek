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
- [x] (2026-05-17) Added a parser/lexer/docs/change/expr-runtime helper batch in `src/expr/parser.rs`, `src/expr/lexer.rs`, `src/docs/mod.rs`, `src/engine/change.rs`, and `src/engine/expr_runtime.rs`, then remeasured coverage with `tmp/coverage-batch3.json`.
- [x] (2026-05-17) Added another sema/waveform-focused batch in `src/expr/sema.rs`, `src/waveform/mod.rs`, and `tests/expression_direct_semantics.rs`, covering more manual AST binding variants, direct waveform helper calls, and public expression semantics through integration tests.
- [x] (2026-05-17) Added a mixed helper batch in `src/docs/mod.rs`, `src/engine/expr_runtime.rs`, `src/expr/sema.rs`, `src/expr/parser.rs`, `src/waveform/mod.rs`, and `src/expr/eval.rs`, covering embedded docs parse/export helpers, more runtime wrapper/error paths, more semantic rejection branches, more parser negative forms, and extra evaluator/cache/arithmetic helper paths.
- [x] (2026-05-17) Remeasured coverage with `tmp/coverage-batch9.json`; total `src/**` coverage improved to line 90.59%, region 91.74%, function 88.54%.
- [x] (2026-05-17) Added another helper-heavy batch across `src/cli/mod.rs`, `src/docs/mod.rs`, `src/engine/change.rs`, `src/expr/eval.rs`, `src/expr/lexer.rs`, `src/expr/parser.rs`, `src/expr/sema.rs`, and `src/waveform/mod.rs`, then remeasured through `tmp/coverage-batch15.json`; total `src/**` coverage is now line 93.63%, region 93.48%, function 93.01%, average 93.37%.
- [ ] Continue expanding semantic-helper tests in `src/expr/sema.rs` and waveform/helper-error tests in `src/waveform/mod.rs`, then return again to the remaining docs/export and change-engine residue in `src/docs/mod.rs` and `src/engine/change.rs`; those files still dominate the last stretch to 95%.
- [ ] Continue autonomous test/measure/commit loops without pausing for status handoff until the 95% average target is actually reached.
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

- Observation: the parser/docs/change/runtime sweep paid off, but not enough to avoid returning to the unpleasant giants.
  Evidence: totals improved to 87.63/89.21/85.82 from `tmp/coverage-batch3.json`, with `src/expr/sema.rs` still stranded at 74.56/76.19/71.32 and `src/waveform/mod.rs` still only at 86.16/92.33/81.20.

- Observation: even fairly aggressive additional unit tests on `sema` and `waveform` now produce only incremental total gains. The remaining uncovered surface is dominated by awkward helper branches, error plumbing, and low-level internals rather than broad untested happy paths.
  Evidence: after the next two batches, totals reached only 88.24/89.81/86.41 from `tmp/coverage-batch6.json`, while `src/expr/sema.rs` is still only 79.45/80.35/75.00 and `src/waveform/mod.rs` only 87.07/93.12/82.96.

- Observation: parser negative tests are cheap and useful, but they are not the whole story. They move line coverage efficiently, while the overall target is still bottlenecked by the ugliest helper internals in `sema`, `waveform`, `docs`, and a smaller residue in `change`.
  Evidence: after the mixed docs/runtime/sema/parser/waveform/eval batch, totals improved to 90.59/91.74/88.54 from `tmp/coverage-batch9.json`, with `src/expr/parser.rs` up to 90.18/91.89/94.19, but `src/expr/sema.rs` still only 86.11/86.55/79.73 and `src/docs/mod.rs` still only 86.01/91.36/82.18.

- Observation: direct helper tables and in-file host/cache smoke tests still move the needle, but the easy wins are mostly gone now. The branch is no longer missing broad feature coverage; it is fighting a stubborn tail of export error plumbing, waveform edge cases, and semantic const-eval residue.
  Evidence: after the later helper-heavy closure batches, totals climbed again to 93.63/93.48/93.01 from `tmp/coverage-batch15.json`, while the worst remaining files are `src/waveform/mod.rs` (94.20/95.77/87.10), `src/docs/mod.rs` (90.34/93.74/87.27), `src/engine/change.rs` (93.09/93.72/88.24), and `src/expr/sema.rs` (92.72/90.79/92.73).

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

The helper-module batch, the first direct internals batch, the parser/docs/change/runtime sweep, and the later docs/runtime/sema/parser/waveform/eval helper batches are now on the branch. Coverage is materially better than where this plan started, but the target is still not met. The ugliest residue has shifted: `src/expr/sema.rs` is no longer the disaster area it was earlier, but `src/waveform/mod.rs`, `src/docs/mod.rs`, and `src/engine/change.rs` are still stubborn enough to block the finish line, and the remaining work is now mostly awkward edge-case plumbing rather than missing surface behavior.

Final percentages, remaining weak spots, discovered bugs, and lessons learned will be recorded here when the work is done.

Revision Note: 2026-05-17 / Grin - Initial active coverage-closure ExecPlan created from current repository docs, baseline coverage artifacts, source/test inspection, and the user’s explicit test-only-plus-frequent-commit requirement.
Revision Note: 2026-05-17 / Grin - Updated after the first test-only helper-module batch to record completed smaller-file work, the post-batch coverage totals from `tmp/coverage-after-m1.json`, and the fact that the remaining risk is now concentrated in `src/expr/sema.rs`, `src/expr/eval.rs`, and `src/waveform/mod.rs`.
Revision Note: 2026-05-17 / Grin - Updated after the first direct internals batch on `src/expr/sema.rs`, `src/expr/eval.rs`, and `src/waveform/mod.rs` to record the new total coverage state from `tmp/coverage-after-m2a.json` and the widened list of next high-yield targets (`docs`, `parser`, `lexer`, `change`, and `expr_runtime`) beyond the still-dominant `sema` and `waveform` gaps.
Revision Note: 2026-05-17 / Grin - Updated after the parser/docs/change/runtime sweep to record the `tmp/coverage-batch3.json` totals and the fact that the remaining path is now concentrated even more brutally in `src/expr/sema.rs` and `src/waveform/mod.rs`.
Revision Note: 2026-05-17 / Grin - Updated after the latest sema/waveform and direct-expression integration batches to record the `tmp/coverage-batch6.json` totals and the fact that the remaining problem is no longer “find big missing features,” but “exhaust a nasty residue of helper branches and function-level stragglers.”
Revision Note: 2026-05-17 / Grin - Updated after the mixed docs/runtime/sema/parser/waveform/eval helper batch to record the `tmp/coverage-batch9.json` totals, the parser/docs/runtime gains, and the still-annoying fact that `src/expr/sema.rs` remains the main obstacle to the 95% average target.
Revision Note: 2026-05-17 / Grin - Updated after the later helper-heavy closure batches to record the `tmp/coverage-batch15.json` totals, the autonomous continue-until-done working mode, and the fact that the final blockers have consolidated into waveform/docs/change edge residue plus a smaller remaining `sema` tail.