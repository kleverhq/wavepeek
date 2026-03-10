# C1 Expression Engine Foundation: Spanned Parsing, Diagnostics, and Host Interfaces

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek` will have a real expression-engine foundation instead of the current mix of string wrappers and ad-hoc parsing. A contributor will be able to point at one parser architecture, one diagnostic format, one host interface for symbol/type/value lookup, one `insta`-based diagnostic snapshot workflow, and one `Criterion`-based benchmark flow for parser/tokenization work. This is the contract-level `C1` milestone from `docs/expression_roadmap.md`, so the goal is not to ship expression evaluation yet; the goal is to make later phases safe, testable, and deterministic.

The result is observable in two ways. First, dedicated expression-engine tests will load locked positive and negative manifests, compare exact `insta` snapshots, and prove that malformed event syntax is rejected deterministically without panics. Second, the repository will contain a reproducible `Criterion` microbenchmark flow with exported baseline and compare artifacts under `bench/expr/runs/`, while `hyperfine` remains scoped to CLI end-to-end work under `bench/e2e/`. Later expression phases can then reuse a ready-made snapshot and microbenchmark infrastructure instead of inventing one mid-stream.

For existing CLI users, successful `change --on ...` behavior must remain the same and `property` must remain unimplemented. This plan does not use command-surface rollout as its acceptance target; existing CLI suites are rerun only as regression guards while strict malformed-syntax handling is locked at the expression-layer boundary for later integration phases.

## Non-Goals

This plan does not implement event runtime evaluation semantics. This plan does not implement boolean operator semantics, logical-expression parsing beyond the minimal scaffolding needed to establish stable interfaces, or `property` runtime execution. This plan does not make `change --on "... iff ..."` work end to end. This plan does not change successful JSON schemas, add new public commands or flags, or expose internal expression diagnostics directly as a new CLI contract. This plan does not fold `C2` or later roadmap work into `C1`.

## Progress

- [x] (2026-03-10 11:51Z) Reviewed `docs/expression_roadmap.md`, `docs/expression_lang.md`, `docs/DESIGN.md`, `docs/BACKLOG.md`, current `src/expr/` implementation, and recent ExecPlan examples.
- [x] (2026-03-10 11:58Z) Drafted an active `C1` ExecPlan with explicit file paths, milestones, manifests, diagnostics snapshots, benchmark artifacts, and review gates.
- [x] (2026-03-10 12:01Z) Completed focused plan-review lanes and captured scope, breadcrumb, and benchmark reproducibility findings.
- [x] (2026-03-10 12:01Z) Revised the plan to keep `C1` phase-correct, make library extraction explicit, and tighten the benchmark artifact contract.
- [x] (2026-03-10 12:51Z) Completed fresh control-pass review on the revised plan; no substantive findings remain.
- [x] (2026-03-10 13:36Z) Reopened the plan before implementation, reviewed repository benchmark/snapshot conventions, and revised the `C1` infrastructure strategy to adopt `insta` for diagnostics and `Criterion` for parser microbenchmarks while keeping `hyperfine` E2E-only.
- [x] (2026-03-10 13:57Z) Completed focused docs/architecture/performance review on the revised `insta`/`Criterion` plan and fixed sequencing, breadcrumb, named-baseline export, and reproducibility-gate issues.
- [x] (2026-03-10 13:57Z) Completed a fresh control-pass review on the revised dependency/tooling pivot; no substantive findings remain.
- [ ] Implement Milestone 1: lock `C1` behavior with positive/negative manifests, `insta` diagnostic snapshots, and red-phase tests.
- [ ] Implement Milestone 2: add spanned lexer/parser, AST, semantic host interface, and deterministic parse/semantic/runtime diagnostics.
- [ ] Implement Milestone 3: stabilize compatibility wrappers and regression guards without turning `change` or `property` into new `C1` integration surfaces.
- [ ] Implement Milestone 4: add `Criterion` parser/tokenization benchmark harness, exported baseline/compare artifacts, repository microbench collateral, and close the required review/validation loop.

## Surprises & Discoveries

- Observation: the current event parser is a raw string splitter with no spans, and it tolerates malformed closing-parenthesis cases because it decrements nesting depth with `saturating_sub`.
  Evidence: `src/expr/parser.rs` uses `depth = depth.saturating_sub(1)` while splitting `or` and `,` unions.

- Observation: `src/expr/lexer.rs` already exists, but nothing in the runtime consumes it today.
  Evidence: the only current call site for event parsing is `src/engine/change.rs`, and it calls `crate::expr::parse_event_expr(...)`, which delegates directly to `src/expr/parser.rs`.

- Observation: `property` remains a pure runtime stub, which gives this phase a safe place to establish parser/evaluator interfaces without accidentally expanding product behavior.
  Evidence: `src/engine/property.rs` returns `WavepeekError::Unimplemented("`property` command execution is not implemented yet")` immediately.

- Observation: the repository has benchmark infrastructure for CLI end-to-end work, but not for parser-internal benchmarking.
  Evidence: `bench/e2e/` contains the existing `hyperfine`-based harness, while no `bench/expr/` directory or parser benchmark driver exists today.

- Observation: repository docs already scope `hyperfine` to CLI end-to-end benchmarking rather than parser-internal work.
  Evidence: `docs/DEVELOPMENT.md` documents `bench/e2e/perf.py` as the CLI E2E benchmark harness and does not define any parser microbenchmark workflow.

- Observation: `Criterion` exposes `raw.csv` as its stable machine-readable artifact, while its JSON analysis files are private implementation details.
  Evidence: Criterion's user guide explicitly documents `raw.csv` as the stable export format for external tooling and warns that the JSON files may change without notice.

- Observation: a single cross-commit `Criterion` compare threshold is not enough by itself; the plan also needs an explicit same-commit reproducibility guard before the exported baseline evidence is trustworthy.
  Evidence: review of the first `Criterion` rewrite found that a lone `15%` threshold could both miss smaller regressions and fail spuriously on noisy hosts without a tighter same-state rerun check.

- Observation: moving `insta` dependency setup later than the first manifest/snapshot test step makes the red-phase instructions uncompilable.
  Evidence: review of the first `insta` rewrite found that Step 1 required `insta` assertions while `Cargo.toml` dependency wiring was still deferred to Step 4.

- Observation: the repository is currently a binary-only crate, so new manifest-driven expression integration tests and a `Criterion` bench target cannot call shared modules until a library entrypoint exists.
  Evidence: `src/main.rs` declares the module tree directly and there is no `src/lib.rs` today.

## Decision Log

- Decision: keep `C1` strictly phase-scoped and explicitly preserve the roadmap boundary that runtime evaluation and `property` execution remain unimplemented.
  Rationale: `docs/expression_roadmap.md` requires one phase boundary per plan, and blending `C2` or `C5` work into this plan would make acceptance ambiguous.
  Date/Author: 2026-03-10 / OpenCode

- Decision: introduce a new internal expression diagnostic type with parse, semantic, and runtime layers, but keep existing CLI-facing `WavepeekError` wrapping at public command boundaries until later integration phases.
  Rationale: `C1` needs a stable internal diagnostics contract now, while `docs/DESIGN.md` and current integration tests still lock the public `error: args:` and `error: unimplemented:` surfaces.
  Date/Author: 2026-03-10 / OpenCode

- Decision: use repository-tracked JSON manifests plus named `insta` file snapshots under `tests/snapshots/` for deterministic diagnostic rendering.
  Rationale: `C1` is infrastructure work, and adopting the repository's long-lived diagnostic snapshot workflow now gives later expression phases a ready-made review/update path without sacrificing readable committed inputs under `tests/fixtures/expr/`.
  Date/Author: 2026-03-10 / OpenCode

- Decision: adopt `Criterion` as the repository-standard harness for parser microbenchmarks, place benchmark code under `benches/`, and keep parser benchmarks separate from the existing `hyperfine`-backed CLI E2E harness in `bench/e2e/`.
  Rationale: `C1` regression gating is about lexer/parser internals rather than command latency, and `Criterion` integrates naturally with `cargo bench` while preserving the existing repository rule that `hyperfine` is for end-to-end CLI measurements.
  Date/Author: 2026-03-10 / OpenCode

- Decision: export committed microbenchmark artifacts from `Criterion`'s stable `raw.csv` output into repository-tracked run directories under `bench/expr/runs/` instead of depending on `target/criterion` or Criterion's private JSON files.
  Rationale: `target/` is ignored, later phases need durable baseline evidence, and a repo-local export/compare layer lets `C1` keep deterministic committed artifacts without reintroducing `hyperfine` for parser internals.
  Date/Author: 2026-03-10 / OpenCode

- Decision: prefer deterministic corpus-based no-panic testing over introducing external fuzz infrastructure in this phase.
  Rationale: `C1` requires fuzz/no-panic coverage, but a repo-local deterministic corpus can satisfy that requirement with lower setup cost and clearer CI behavior.
  Date/Author: 2026-03-10 / OpenCode

- Decision: add `src/lib.rs` and move `src/main.rs` to a thin wrapper before the new `C1` tests and `Criterion` bench target are introduced.
  Rationale: both `tests/expression_c1.rs` and `benches/expr_c1.rs` need a reusable crate surface; without `src/lib.rs`, a novice implementer would be blocked immediately.
  Date/Author: 2026-03-10 / OpenCode

- Decision: keep `bind_event_expr(...)` and the new host interface internal-only in `C1`; `src/engine/change.rs` keeps ownership of runtime name resolution and trigger execution in this phase.
  Rationale: this preserves the roadmap boundary that command integration is deferred while still establishing the semantic-layer architecture that later phases will consume.
  Date/Author: 2026-03-10 / OpenCode

- Decision: keep the new strict typed `wavepeek::expr` entry points decoupled from the legacy `parse_event_expr(...)` adapter in `C1`.
  Rationale: `change` currently calls the legacy adapter, and routing it through the strict parser in this phase would create an unintended command-surface rollout.
  Date/Author: 2026-03-10 / OpenCode

- Decision: use three benchmark directories in `C1`: `c1-foundation-candidate` for a dedicated clean pre-review commit after the first fully green implementation, `c1-foundation-baseline` for the final accepted post-review state that future phases inherit, and `c1-foundation-verify` for a second run from that same final state to prove reproducibility.
  Rationale: the parser benchmark harness does not exist before `C1`, so the plan needs one changed-state comparison during `C1` itself plus one carried-forward final golden for later phases, and the candidate state must be reconstructable by a stateless contributor even though the underlying `Criterion` working data in `target/criterion` is not committed.
  Date/Author: 2026-03-10 / OpenCode

- Decision: keep semantic and runtime diagnostic snapshot locking in internal unit tests under `src/expr/` instead of exposing new public APIs just for integration-test snapshot coverage.
  Rationale: `bind_logical_expr(...)` and `eval_logical_expr(...)` remain internal in `C1`, and widening the public expression API solely for snapshot tests would create unnecessary surface-area churn before the roadmap reaches command integration.
  Date/Author: 2026-03-10 / OpenCode

- Decision: require the exported `Criterion` verify run to pass a tighter `5%` same-commit reproducibility gate before treating the `15%` candidate-versus-baseline comparison as actionable.
  Rationale: the roadmap default `15%` gate is an acceptable coarse cross-commit regression screen, but review showed that single-run exported mean/median pairs still need a stricter same-state noise check to avoid misleading benchmark evidence.
  Date/Author: 2026-03-10 / OpenCode

## Outcomes & Retrospective

Current status: planning complete; implementation has not started yet.

This plan now gives a stateless contributor one place to find the `C1` scope, architecture targets, exact artifact paths, `insta` snapshot workflow, `Criterion` microbenchmark/export flow, commit boundaries, and review policy. The remaining work is implementation, validation, and closure against the locked `C1` gates.

The main risk to watch during implementation is accidental phase creep. The expression roadmap intentionally separates parser architecture from runtime semantics and from command-surface rollout. If a change starts enabling real logical evaluation, `property` execution, or new `change`/`property` public behavior, it belongs in a later plan and should be rejected here.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI. Expression-related code currently lives in `src/expr/mod.rs`, `src/expr/parser.rs`, `src/expr/lexer.rs`, and `src/expr/eval.rs`. Today, `src/expr/mod.rs` exposes a placeholder `Expression` that stores only source text, simple event-expression structs (`EventExpr`, `EventTerm`, `EventKind`), and thin wrappers around the parser. `src/expr/parser.rs` contains the real event parsing logic, but that logic is string-oriented and does not produce byte spans or structured diagnostics. `src/expr/lexer.rs` tokenizes a few event-expression words and punctuation, but it is not wired into the parser. `src/expr/eval.rs` always returns an unimplemented error.

The only production caller of the current event parser is `src/engine/change.rs`. That file parses `--on`, rejects runtime `iff` execution with `error: args: iff logical expressions are not implemented yet`, resolves event signal names relative to scope, and evaluates triggers over waveform samples. `src/engine/property.rs` is still an unimplemented stub and must remain so in this phase.

The expression language contract is split across two docs. `docs/expression_lang.md` defines the full intended syntax and semantics, including event unions, `iff` binding, and future boolean-expression behavior. `docs/expression_roadmap.md` defines the phased rollout. In that roadmap, `C1` means architecture and strict parser/diagnostic foundations only. It explicitly includes strict rejection of unmatched parentheses, empty `iff`, and broken union segmentation. It explicitly excludes runtime expression evaluation and `property` runtime execution.

Several terms in this plan have repository-specific meanings. A "span" is a byte-range inside an expression string, with inclusive `start` and exclusive `end`, so diagnostics can point at exact text without ambiguity. A "manifest" is a repository-tracked list of test cases that a Rust test loads at runtime; here it means JSON files under `tests/fixtures/expr/` that lock parse successes and failures. An "Insta snapshot" is a committed `.snap` file under `tests/snapshots/` produced by named `insta` assertions in `tests/expression_c1.rs`; in this plan it stores the exact rendered form of one internal expression diagnostic. "Criterion" is the Rust microbenchmark library integrated through `cargo bench`; in this plan it measures parser/tokenization scenarios from `benches/expr_c1.rs`, writes temporary results under `target/criterion`, and a repo-local export step copies the stable `raw.csv` measurements into committed run directories under `bench/expr/runs/`. A "host" is a trait that lets the expression layer ask the rest of the program to resolve signal names, query recovered type metadata, and fetch sampled values without importing waveform/engine logic directly. A "binder" is the semantic pass that converts parsed names into resolved handles and rejects invalid references before evaluation.

The current gaps that `C1` must close are concrete. There is no reusable library crate surface, no spanned token stream, no dedicated AST module, no semantic binding entry point, no internal parse/semantic/runtime diagnostic format, no locked manifest-based parser matrix, no deterministic no-panic corpus, and no parser benchmark harness. `docs/BACKLOG.md` already tracks two of these gaps directly: temporary `iff` capture rules in the event parser and unused lexer scaffolding.

## Open Questions

There are no blocking product questions left for `C1`. The plan resolves the two implementation-shape questions that matter here: use `insta` file snapshots for diagnostics while keeping manifests as repository fixtures, and use `Criterion` benchmarks under `benches/` plus repo-local export/compare helpers under `bench/expr/` instead of extending `bench/e2e`.

No alternate microbenchmark harness path is planned in `C1`. Use `cargo bench --bench expr_c1 ...` and export Criterion's stable `raw.csv` output into committed `bench/expr/runs/` artifacts; do not add `src/bin/*` parser benchmark drivers or `hyperfine`-based parser harnesses.

## Plan of Work

Milestone 1 locks the `C1` contract with tests before structural refactors begin. Add manifest-driven integration tests and named `insta` snapshots first, then run them in red state so there is branch-history proof that the current implementation does not satisfy `C1`. This milestone must define three categories of evidence: positive parse cases that should succeed, negative cases that should fail deterministically, and a no-panic corpus that intentionally exercises malformed token/parenthesis combinations.

Milestone 2 introduces the actual foundation modules. Add dedicated files for AST, diagnostics, semantic host interfaces, and binding entry points, then rewrite the lexer and parser around spanned tokens. This milestone is complete when `src/expr/` has one clear architecture: tokenization produces spanned tokens, parsing produces AST, binding has a stable host-facing entry point, and evaluation has a stable API that still returns a deterministic runtime-not-implemented diagnostic. The implementation must stay phase-correct: runtime semantics are not delivered here.

Milestone 3 stabilizes the compatibility boundary without broadening command integration. The new parser/diagnostic stack must be reachable through `src/expr/mod.rs` compatibility wrappers and direct expression tests, but `src/engine/change.rs` and `src/engine/property.rs` should remain command-level regression guards rather than active `C1` feature targets. The semantic binder and host interface are introduced in this milestone as internal-only architecture that later phases can consume.

Milestone 4 adds the benchmark and closure artifacts. Create a `Criterion` parser benchmark target under `benches/`, repo-local export/compare helpers under `bench/expr/`, and committed run directories under `bench/expr/runs/` that store exported `raw.csv` inputs plus derived summaries. The compare helper must enforce the roadmap default gate of no worse than 15% mean and median regression on the fixed `C1` scenario set. In this milestone, `c1-foundation-candidate` is captured from the first fully green implementation before review, `c1-foundation-baseline` is captured from the final accepted `C1` state after the last review-fix commit, and `c1-foundation-verify` is a second run from that same final state used to prove reproducibility. Then update collateral: establish `Criterion` as the documented repository-standard microbenchmark tool for parser-internal work, keep `hyperfine` documented as E2E-only, close the relevant `docs/BACKLOG.md` items when they are actually resolved, keep `CHANGELOG.md` unchanged because `C1` does not intentionally roll out new public behavior, and keep the active plan updated with red/green/review evidence.

Milestone 5 is validation and review closure. Run the dedicated expression tests, affected CLI tests, and the parser benchmark compare. Then run the mandatory review workflow using focused lanes and one fresh independent control pass. Any fixes from review must be committed separately, and the final validation run must happen after the last review-fix commit.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

0. Extract a minimal library entrypoint and typed API scaffold so the new tests and `Criterion` bench target have a stable crate surface.

   Add `src/lib.rs` and move ownership of the existing module tree there (`cli`, `engine`, `error`, `expr`, `output`, `schema_contract`, `waveform`). `src/main.rs` must stop declaring modules and become a thin binary wrapper around the library entrypoint. In the same step, add the minimal public `wavepeek::expr` type/function stubs needed for `tests/expression_c1.rs` and `benches/expr_c1.rs` to compile against the future `C1` surface. Also add the `insta` dev-dependency and the `profile.dev.package` optimization entries required for Step 1 snapshot tests to compile and stay ergonomic; reserve the `criterion` dependency and bench-target wiring for Step 4. Placeholder bodies are acceptable here if they return deterministic scaffold diagnostics; Step 2 will replace them with the full implementation.

   This is prerequisite scaffolding required to make the `C1` manifest tests and `Criterion` bench target feasible. Keep it minimal and structural; do not mix real parser semantics into this step.

1. Lock `C1` behavior with failing tests and fixtures first.

   Create these new manifest and snapshot files. The preferred authoring flow is to let `insta` create `.snap.new` files and then accept them into committed `.snap` files after review, but the repository state at milestone exit must contain the committed `.snap` files with the names below:

   - `tests/fixtures/expr/c1_positive_manifest.json`
   - `tests/fixtures/expr/c1_negative_manifest.json`
   - `tests/snapshots/expression_c1__parse_unmatched_open.snap`
   - `tests/snapshots/expression_c1__parse_unmatched_close.snap`
   - `tests/snapshots/expression_c1__parse_empty_iff.snap`
   - `tests/snapshots/expression_c1__parse_broken_union.snap`

   Add a new integration test file `tests/expression_c1.rs` that loads the manifests and snapshots. The positive manifest must at minimum cover `*`, named events, `posedge`/`negedge`/`edge`, `or` and `,` unions, and `iff` binding to only the immediately preceding event term. The negative manifest must at minimum cover empty input, unmatched `(`, unmatched `)`, empty `iff`, leading/trailing union separators, duplicated separators, and missing names after edge keywords. In the negative cases, assert the structured diagnostic layer/code/span fields directly and then use named `insta::assert_snapshot!` calls on `ExprDiagnostic::render(...)` so later changes get an ergonomic review flow without weakening the exactness of the contract.

   Also add a deterministic no-panic corpus test in `tests/expression_c1.rs`. Build the corpus from all positive and negative manifest inputs plus a small generated mutation set (for example, prefixes/suffixes containing extra parentheses, commas, and `or`/`iff` tokens). The test must call the public parser entry point inside `std::panic::catch_unwind` and fail if any input panics.

   Red-phase commands:

        cargo test --test expression_c1 c1_positive_manifest_parses -- --exact
        INSTA_UPDATE=no cargo test --test expression_c1 c1_negative_manifest_matches_snapshots -- --exact
        cargo test --test expression_c1 c1_no_panic_corpus_holds -- --exact

   Preferred local snapshot authoring workflow after the failing contract is in place:

        cargo insta test --test expression_c1 c1_negative_manifest_matches_snapshots --review

   If `cargo-insta` is not installed, use this fallback that requires no extra tool:

        INSTA_UPDATE=new cargo test --test expression_c1 c1_negative_manifest_matches_snapshots -- --exact

   Then review the generated `tests/snapshots/*.snap.new` files manually, rename each approved file to the matching `.snap` name, and rerun with `INSTA_UPDATE=no`.

   Expected red evidence before implementation is complete: at least one of the first two tests fails because the current parser has no spans/snapshots and still accepts or mis-classifies malformed parenthesis/union cases. Validation later in this plan must run with `INSTA_UPDATE=no` so no snapshot files are rewritten silently.

2. Introduce the `C1` expression-engine foundation modules.

   Add these source files:

   - `src/expr/ast.rs`
   - `src/expr/diagnostic.rs`
   - `src/expr/host.rs`
   - `src/expr/sema.rs`

   Update these existing source files:

   - `src/expr/mod.rs`
   - `src/expr/lexer.rs`
   - `src/expr/parser.rs`
   - `src/expr/eval.rs`

   The new architecture must work like this. `src/expr/lexer.rs` produces spanned tokens for event-expression parsing. `src/expr/parser.rs` consumes those tokens and produces AST from `src/expr/ast.rs`. `src/expr/diagnostic.rs` owns parse/semantic/runtime diagnostics and their deterministic renderer. `src/expr/host.rs` defines the host trait and the handle/type/value structs the expression layer depends on. `src/expr/sema.rs` owns binding entry points from AST to bound forms. `src/expr/eval.rs` exposes the evaluator API but still returns a deterministic runtime-layer unimplemented diagnostic.

   Keep a compatibility facade in `src/expr/mod.rs` so existing callers do not need to understand the full internal reorganization yet. In `C1`, that facade should expose the new typed public lex/parse API while keeping the legacy `parse(...)` and `parse_event_expr(...)` adapters compatibility-preserving for current command code. Do not route the legacy adapter through the strict parser in this phase.

   Targeted build/test commands after these changes:

        cargo test expr::lexer::tests
        cargo test expr::parser::tests
        INSTA_UPDATE=no cargo test --test expression_c1

3. Stabilize the compatibility wrappers and keep command paths as regression guards only.

   Implement the new parser/diagnostic stack behind `src/expr/mod.rs` compatibility wrappers. Those wrappers may translate internal diagnostics into the current `WavepeekError` categories and help-hint wording, but `C1` acceptance must be defined at the expression-layer tests rather than by shipping new command-path behavior.

   Keep the ownership boundary explicit. `src/expr/sema.rs` and `src/expr/host.rs` are internal `C1` architecture. `src/engine/change.rs` continues owning runtime signal resolution and trigger execution in this phase. `src/engine/property.rs` remains an unimplemented regression guard and must not become a new parser/runtime surface.

   Add direct expression-layer tests for the public typed `wavepeek::expr` lex/parse surface:

   - malformed event syntax must fail through the new expression-layer diagnostics,
   - syntactically valid `iff` forms must still be representable in the AST.

   Add internal unit tests in `src/expr/sema.rs` and `src/expr/eval.rs` for the internal-only architecture introduced in `C1`:

    - `bind_logical_expr(...)` may still return the deterministic semantic not-implemented diagnostic,
    - `eval_logical_expr(...)` must still return the deterministic runtime not-implemented diagnostic.

   Because those entry points remain internal in `C1`, lock their rendered diagnostic text in the same unit-test modules rather than in `tests/expression_c1.rs`. Named `insta` assertions in `src/expr/sema.rs` and `src/expr/eval.rs` are acceptable here and should produce committed snapshots under `src/expr/snapshots/`. Treat `src/expr/snapshots/` as an intentionally skipped snapshot leaf under the breadcrumb policy: it is a generated-style artifact directory discovered from adjacent test modules, so no extra local `AGENTS.md` file is required there.

   The compatibility adapters that still return `WavepeekError` exist only for current command code. Do not use those adapters as the new public typed test/benchmark surface in `C1`, and do not tighten `change --on` behavior through them in this phase.

   Add explicit regression tests in `tests/change_cli.rs` for the current legacy boundary on malformed `--on` inputs so command behavior cannot drift accidentally while the strict typed parser is added elsewhere. At minimum, lock these current outcomes:

   - `posedge (` -> `error: args: invalid --on expression 'posedge ('. See 'wavepeek change --help'.`
   - `posedge clk)` -> `error: args: invalid --on expression 'posedge clk)'. See 'wavepeek change --help'.`
   - `posedge clk or , clk` -> `error: args: invalid --on expression 'posedge clk or , clk'. See 'wavepeek change --help'.`
   - `posedge clk iff` -> `error: args: iff logical expressions are not implemented yet`

   Then rerun the command suites as drift guards:

       cargo test --test change_cli change_rejects_unmatched_open_parenthesis_with_legacy_error -- --exact
       cargo test --test change_cli change_rejects_unmatched_close_parenthesis_with_legacy_error -- --exact
       cargo test --test change_cli change_rejects_broken_union_with_legacy_error -- --exact
       cargo test --test change_cli change_empty_iff_stays_deferred_runtime_error -- --exact
       cargo test --test change_cli
       cargo test --test property_cli

4. Add parser/tokenization microbenchmark tooling and artifacts.

   Create new durable directories `bench/expr/`, `bench/expr/runs/`, and `benches/` and satisfy the breadcrumb policy in the same change:

    - update `AGENTS.md` to add `benches/AGENTS.md` under `Child Maps`
    - update `bench/AGENTS.md` to add `bench/expr/AGENTS.md` under `Child Maps`
    - add `benches/AGENTS.md`
    - add `benches/expr_c1.rs`
    - add `bench/expr/AGENTS.md`
    - add `bench/expr/runs/AGENTS.md`
    - add `bench/expr/capture.py`
    - add `bench/expr/compare.py`
    - add `bench/expr/test_capture.py`
    - add `bench/expr/test_compare.py`
    - add `bench/expr/runs/c1-foundation-candidate/README.md`
    - add `bench/expr/runs/c1-foundation-baseline/README.md`
    - add `bench/expr/runs/c1-foundation-verify/README.md`

   Update repository collateral in the same milestone so the new tooling becomes the default microbenchmark path for later phases:

    - update `Cargo.toml` to add the `criterion` dev-dependency, add the explicit `[[bench]]` entry for `expr_c1`, and disable stray bench harnesses on the library/main binary if needed so Criterion arguments do not get intercepted by `libtest`
    - update `docs/DEVELOPMENT.md` to document `Criterion` for parser/internal microbenchmarks and keep `hyperfine` scoped to `bench/e2e`
    - update `Makefile` to add a dedicated microbenchmark-helper unit-test target (for example `test-bench-expr`) and include it in `make ci`

   Add a `Criterion` benchmark target at `benches/expr_c1.rs`. It must benchmark only the public expression entry points, without touching waveform files or command runtimes. Use `criterion::black_box` inside the measured closures, configure a fixed profile in code (`sample_size = 100`, `warm_up_time = 3s`, `measurement_time = 5s`, `significance_level = 0.05`, `noise_threshold = 0.01`), and call `configure_from_args()` so named baselines and `--noplot` remain available from the command line. Define exactly these benchmark IDs:

   - `tokenize_union_iff`
   - `parse_event_union_iff`
   - `parse_event_malformed`

   `bench/expr/capture.py` must read only Criterion's stable `raw.csv` outputs from `target/criterion`, take a required named-baseline selector, validate exact scenario-set equality, compute per-iteration mean and median from the raw samples, copy only the stable `raw.csv` files associated with the requested baseline into the requested committed run directory, write a deterministic `summary.json`, and render a human-readable `README.md`. Do not depend on Criterion's JSON analysis files because their layout is not stable. The script must fail if the requested baseline is absent, if files from another baseline would be mixed in, or if scenarios are missing/extra/duplicated. `bench/expr/compare.py` must validate exact scenario-set equality before comparing metrics from the exported summaries. Cover both helpers with unit tests in `bench/expr/test_capture.py` and `bench/expr/test_compare.py`, including a case where multiple saved baselines coexist and the wrong named baseline would otherwise be exported.

   Use this fixed microbenchmark capture flow exactly. Record the exact `cargo bench` command, `cargo -V`, `rustc -V`, the resolved `criterion` crate version from `Cargo.lock`, the `source commit` from `git rev-parse HEAD`, whether the worktree was clean or dirty when the run was captured, and a short devcontainer/host note inside each run-directory `README.md`.

   Candidate run (from a dedicated clean pre-review commit after the first fully green implementation):

       cargo bench --bench expr_c1 -- --save-baseline c1-foundation-candidate --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c1-foundation-candidate --output bench/expr/runs/c1-foundation-candidate --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Baseline run (final accepted state, after the last review-fix commit):

       cargo bench --bench expr_c1 -- --save-baseline c1-foundation-baseline --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c1-foundation-baseline --output bench/expr/runs/c1-foundation-baseline --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Verify run (same final accepted state, second capture for reproducibility):

       cargo bench --bench expr_c1 -- --save-baseline c1-foundation-verify --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c1-foundation-verify --output bench/expr/runs/c1-foundation-verify --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Then run:

       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-baseline --golden bench/expr/runs/c1-foundation-candidate --max-negative-delta-pct 15
       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-verify --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 5

5. Commit atomic units.

   Suggested commit split:

        git add src/lib.rs src/main.rs src/expr/mod.rs
        git commit -m "refactor(crate): add library entrypoint and c1 api stubs"

        git add Cargo.toml tests/fixtures/expr tests/snapshots tests/expression_c1.rs tests/change_cli.rs
        git commit -m "test(expr): lock c1 parser manifests and insta diagnostics"

        git add src/expr/mod.rs src/expr/ast.rs src/expr/diagnostic.rs src/expr/host.rs src/expr/lexer.rs src/expr/parser.rs src/expr/sema.rs src/expr/eval.rs src/expr/snapshots
        git commit -m "refactor(expr): add c1 spanned parser foundation"

         git add Cargo.toml AGENTS.md bench/AGENTS.md bench/expr/AGENTS.md bench/expr/runs/AGENTS.md bench/expr/capture.py bench/expr/compare.py bench/expr/test_capture.py bench/expr/test_compare.py benches/AGENTS.md benches/expr_c1.rs docs/DEVELOPMENT.md Makefile
         git commit -m "bench(expr): add c1 criterion microbench harness"

         git add bench/expr/runs/c1-foundation-candidate
         git commit -m "bench(expr): capture c1 candidate criterion baseline"

         # after review fixes and final acceptance-state capture
         git add bench/expr/runs/c1-foundation-baseline bench/expr/runs/c1-foundation-verify docs/BACKLOG.md
         git commit -m "bench(expr): capture final c1 criterion baselines"

   This split assumes the `Cargo.toml` `insta` hunk lands during Steps 0-1 and the later `criterion`/bench-target hunk is added only during Step 4, so `git add Cargo.toml` in each commit stages the changes introduced in that milestone without interactive partial staging.

   If review finds issues, fix them in separate follow-up commits. Do not amend history.

6. Validation gates.

   Run the focused gates first:

        INSTA_UPDATE=no cargo test --test expression_c1
        cargo test --test change_cli
        cargo test --test property_cli
        cargo test --bench expr_c1
        python3 -m unittest discover -s bench/expr -p 'test_*.py'
        python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-baseline --golden bench/expr/runs/c1-foundation-candidate --max-negative-delta-pct 15
        python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-verify --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 5

   Then run repository gates:

       make check
         make ci

   Expected success signature after implementation is complete:

        INSTA_UPDATE=no cargo test --test expression_c1
        ...
        test result: ok.
        python3 bench/expr/compare.py --revised ... --golden ... --max-negative-delta-pct 15
        ok: no matched scenario exceeded 15.00% negative delta in mean or median
       make ci
       ...
       test result: ok.

7. Mandatory review workflow.

   Load `ask-review` skill and use focused lanes, because this phase touches parser correctness, architecture boundaries, and benchmark discipline.

   Run these lanes in parallel after the implementation and validation commands above:

   - Code lane: parser correctness, negative-case coverage, CLI regression risk in `change`, and missing tests.
   - Architecture lane: layering between `src/expr/`, `src/engine/change.rs`, and the new host/binder interfaces.
    - Performance lane: lexer/parser allocation risk, `Criterion` harness design, export/compare policy, and artifact reproducibility.

   Then run one fresh independent control pass on the consolidated diff. If any lane or the control pass finds real defects, fix them, commit them, rerun affected tests, and rerun the impacted lane plus a fresh control pass. Stop only when there are no unresolved substantive findings.

### Validation and Acceptance

Acceptance is complete only when all of the conditions below are true together:

- `tests/expression_c1.rs` loads `c1_positive_manifest.json` and every listed case parses successfully into the expected normalized AST shape.
- `tests/expression_c1.rs` loads `c1_negative_manifest.json` and every listed case fails with the expected diagnostic layer, code, span, and rendered `insta` snapshot under `tests/snapshots/`.
- The deterministic no-panic corpus test passes.
- The new parser rejects unmatched `(`, unmatched `)`, empty `iff`, and broken union segmentation deterministically.
- `src/lib.rs` exists and the new test/benchmark entry points use that shared crate surface rather than duplicating the module tree.
- `tests/expression_c1.rs` imports only the public `wavepeek::expr` lex/parse/AST/diagnostic paths, proving the typed `C1` API is usable by an external consumer.
- `tests/expression_c1.rs` uses named `insta` file snapshots for rendered diagnostics and validation runs with `INSTA_UPDATE=no`, so snapshot drift is explicit and reviewable.
- Existing `change_cli` and `property_cli` suites pass unchanged, proving that `C1` did not accidentally turn `change` or `property` into new command-surface rollout work.
- Targeted `change_cli` regression tests preserve the current legacy outcomes for unmatched `(`, unmatched `)`, broken union segmentation, and empty `iff`.
- A direct expression-layer test proves syntactically valid `iff` input is preserved in the AST while semantic/runtime work remains deferred.
- Internal unit tests in `src/expr/sema.rs` prove that the new binder/host interface can emit deterministic semantic diagnostics, even though full type semantics remain deferred.
- Internal unit tests in `src/expr/eval.rs` prove that the evaluator API still returns a deterministic runtime unimplemented diagnostic at `C1`.
- The internal semantic/runtime unit tests lock their rendered diagnostics in committed snapshots under `src/expr/snapshots/`, because those entry points remain internal in `C1`.
- The legacy `parse_event_expr(...)` adapter and the real `change` CLI boundary remain compatibility-preserving in `C1`; any strict malformed-input rollout is deferred to a later integration phase.
- `AGENTS.md` links to the new `benches/AGENTS.md` child map, `bench/AGENTS.md` links to the new `bench/expr/AGENTS.md` child map, and `bench/expr/AGENTS.md` links to `bench/expr/runs/AGENTS.md`.
- `bench/expr/runs/c1-foundation-candidate/`, `bench/expr/runs/c1-foundation-baseline/`, and `bench/expr/runs/c1-foundation-verify/` all exist with committed `*.raw.csv` exports, deterministic `summary.json`, and readable `README.md` summaries.
- Each benchmark `README.md` records the fixed `cargo bench` command, `cargo -V`, `rustc -V`, the resolved `criterion` crate version, the source commit used for measurement, whether the worktree was clean or dirty, and the basic environment note used to generate the artifacts. The candidate run must come from a clean dedicated commit so the source state is reconstructable.
- `cargo test --bench expr_c1` passes, proving the microbenchmark harness compiles and runs in Criterion test mode.
- `python3 -m unittest discover -s bench/expr -p 'test_*.py'` passes, including missing/extra-scenario hard-failure coverage and wrong-baseline export rejection when multiple saved baselines coexist.
- `python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-baseline --golden bench/expr/runs/c1-foundation-candidate --max-negative-delta-pct 15` passes, proving no unacceptable regression from the first-green state to the final accepted state.
- `python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-verify --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 5` passes, proving same-commit reproducibility stays within a tighter noise guard before cross-commit regression conclusions are trusted.
- `docs/DEVELOPMENT.md` documents `Criterion` as the microbenchmark workflow for parser/internal work and keeps `hyperfine` scoped to `bench/e2e` CLI runs.
- `Makefile` exposes the new microbenchmark-helper unit-test target and `make ci` runs it.
- `docs/BACKLOG.md` closes the parser-strictness debt and the unused-lexer debt only if the implementation actually resolves them; later-phase debts must remain open.
- `CHANGELOG.md` remains unchanged in `C1`; a changelog delta is a scope warning because this plan does not intentionally roll out new public behavior.
- `make check` and `make ci` pass after the final review-fix commit.
- Review lanes and the fresh independent control pass are clean, or all findings were fixed and rechecked.

TDD acceptance requirement: at least one manifest-driven parser test must fail before the implementation changes and pass after them in the same branch history.

### Idempotence and Recovery

All planned edits are additive or in-place text changes plus committed snapshot and benchmark artifacts. Re-running tests is always safe. Re-running the benchmark commands is safe if the existing `bench/expr/runs/c1-foundation-candidate/`, `bench/expr/runs/c1-foundation-baseline/`, and `bench/expr/runs/c1-foundation-verify/` directories are either removed first or intentionally overwritten with the same scenario names.

If the parser refactor temporarily breaks `change`, recover in this order: first restore the compatibility facade in `src/expr/mod.rs`, then restore the `src/engine/change.rs` wrapper mapping, then rerun `cargo test --test change_cli` before continuing. If an `insta` run leaves `.snap.new` files behind, review them, either accept them into the named `.snap` files or delete them, and rerun validation with `INSTA_UPDATE=no` so the committed snapshot state is explicit.

If benchmark capture fails because `target/criterion` contains stale or partial data, remove the affected `target/criterion/expr_c1*` directories and rerun the same `cargo bench --bench expr_c1 ...` command before touching code. If the same-commit `baseline` versus `verify` compare exceeds the tighter `5%` reproducibility guard, rerun the capture pair before drawing any cross-commit conclusion. Only treat the `15%` candidate-versus-baseline gate as actionable after the same-state reproducibility check is stable.

If review finds issues, apply follow-up commits instead of rewriting history. If the new durable directories `bench/expr/` and `benches/` are kept, ensure their local `AGENTS.md` files and the matching breadcrumb updates in `AGENTS.md` and `bench/AGENTS.md` are committed in the same review-fix sequence.

### Artifacts and Notes

Expected modified or added files for `C1` implementation:

- `Cargo.toml`
- `Makefile`
- `AGENTS.md`
- `src/expr/mod.rs`
- `src/expr/ast.rs`
- `src/expr/diagnostic.rs`
- `src/expr/host.rs`
- `src/expr/lexer.rs`
- `src/expr/parser.rs`
- `src/expr/sema.rs`
- `src/expr/eval.rs`
- `src/lib.rs`
- `src/main.rs`
- `tests/expression_c1.rs`
- `tests/change_cli.rs`
- `tests/fixtures/expr/c1_positive_manifest.json`
- `tests/fixtures/expr/c1_negative_manifest.json`
- `tests/snapshots/expression_c1__*.snap`
- `src/expr/snapshots/*.snap`
- `bench/AGENTS.md`
- `bench/expr/AGENTS.md`
- `bench/expr/runs/AGENTS.md`
- `bench/expr/capture.py`
- `bench/expr/compare.py`
- `bench/expr/test_capture.py`
- `bench/expr/test_compare.py`
- `bench/expr/runs/c1-foundation-candidate/`
- `bench/expr/runs/c1-foundation-baseline/`
- `bench/expr/runs/c1-foundation-verify/`
- `benches/AGENTS.md`
- `benches/expr_c1.rs`
- `docs/DEVELOPMENT.md`
- `docs/BACKLOG.md`

Before closing this plan, record these excerpts here:

- one red-phase failure from `tests/expression_c1.rs` before the parser foundation is implemented,
- one green-phase pass for the same test after implementation,
- one short typed expression-diagnostic excerpt proving malformed input is rejected deterministically,
- one short benchmark-compare success excerpt,
- one short summary each from the focused review lanes and the fresh control pass.

### Interfaces and Dependencies

The `C1` implementation must define these stable internal interfaces.

In `Cargo.toml`, add the new development-only tooling and benchmark target wiring:

    [dev-dependencies]
    assert_cmd = "~2"
    predicates = "~3"
    tempfile = "~3"
    insta = "~1"

    # added in Step 4, after snapshot tests already exist
    criterion = "~0.8"

    [profile.dev.package]
    insta.opt-level = 3
    similar.opt-level = 3

    [lib]
    bench = false

    [[bin]]
    name = "wavepeek"
    path = "src/main.rs"
    bench = false

    [[bench]]
    name = "expr_c1"
    harness = false

`insta` is required for the new diagnostic snapshot workflow and must be added before Step 1 so the red-phase tests compile. `criterion` is required for parser microbenchmarks and is introduced in Step 4 together with the bench-target wiring. The `profile.dev.package` entries keep snapshot review ergonomic by making diff generation fast. The explicit `bench = false` settings prevent default `libtest` harnesses from intercepting Criterion-only arguments such as `--save-baseline`.

In `src/lib.rs`, define a thin reusable crate surface for tests and benchmark targets:

    mod cli;
    mod engine;
    mod error;
    mod output;
    mod schema_contract;
    mod waveform;

    pub fn run_cli() -> Result<(), crate::error::WavepeekError>;

    pub mod expr;
    pub use crate::error::WavepeekError;

`src/main.rs` should become a tiny binary wrapper that calls `wavepeek::run_cli()` and keeps exit-code/error-print behavior unchanged. The public library surface must be sufficient for `tests/expression_c1.rs` and `benches/expr_c1.rs` to use expression entry points without duplicating private module wiring.

In `src/expr/diagnostic.rs`, define:

    pub struct Span {
        pub start: usize,
        pub end: usize,
    }

    pub enum DiagnosticLayer {
        Parse,
        Semantic,
        Runtime,
    }

    pub struct ExprDiagnostic {
        pub layer: DiagnosticLayer,
        pub code: &'static str,
        pub message: String,
        pub primary_span: Span,
        pub notes: Vec<String>,
    }

    impl ExprDiagnostic {
        pub fn render(&self, source: &str) -> String;
    }

`render(...)` must be deterministic and snapshot-friendly. It must not include ANSI color, timestamps, filesystem paths, or random identifiers.

In `src/expr/ast.rs`, define event-parse structures that carry spans explicitly:

    pub struct EventExprAst {
        pub terms: Vec<EventTermAst>,
        pub span: Span,
    }

    pub struct EventTermAst {
        pub event: BasicEventAst,
        pub iff: Option<DeferredLogicalExpr>,
        pub span: Span,
    }

    pub enum BasicEventAst {
        AnyTracked { span: Span },
        Named { name: String, span: Span },
        Posedge { name: String, span: Span },
        Negedge { name: String, span: Span },
        Edge { name: String, span: Span },
    }

    pub struct DeferredLogicalExpr {
        pub source: String,
        pub span: Span,
    }

`DeferredLogicalExpr` is intentional in `C1`: it records the `iff` payload without claiming full logical-expression parsing or semantics yet.

In `src/expr/lexer.rs`, define a spanned token stream for event parsing:

    pub enum TokenKind {
        Identifier,
        KeywordOr,
        KeywordIff,
        KeywordPosedge,
        KeywordNegedge,
        KeywordEdge,
        Star,
        Comma,
        LeftParen,
        RightParen,
    }

    pub struct Token {
        pub kind: TokenKind,
        pub span: Span,
        pub lexeme: String,
    }

    pub fn lex_event_expr(source: &str) -> Result<Vec<Token>, ExprDiagnostic>;

In `src/expr/host.rs`, define the layer boundary that later phases will depend on:

    pub struct SignalHandle(pub u32);

    pub struct ExprType {
        pub width: u32,
        pub is_four_state: bool,
        pub is_signed: bool,
    }

    pub struct SampledValue {
        pub bits: String,
    }

    pub trait ExpressionHost {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic>;
        fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic>;
        fn sample_value(&self, handle: SignalHandle, timestamp: u64) -> Result<SampledValue, ExprDiagnostic>;
    }

In `src/expr/sema.rs`, define explicit semantic entry points even though full semantics are not implemented yet:

    pub struct BoundEventExpr {
        pub terms: Vec<BoundEventTerm>,
    }

    pub struct BoundEventTerm {
        pub event: BoundEventKind,
        pub iff: Option<DeferredLogicalExpr>,
    }

    pub enum BoundEventKind {
        AnyTracked,
        Named(SignalHandle),
        Posedge(SignalHandle),
        Negedge(SignalHandle),
        Edge(SignalHandle),
    }

    pub(crate) fn bind_event_expr(ast: &EventExprAst, host: &dyn ExpressionHost) -> Result<BoundEventExpr, ExprDiagnostic>;

    pub(crate) fn bind_logical_expr(expr: &DeferredLogicalExpr, host: &dyn ExpressionHost) -> Result<(), ExprDiagnostic>;

`bind_logical_expr(...)` may still return a deterministic semantic-layer not-implemented diagnostic in `C1`; the important part is that the entry point and host boundary now exist and are tested.

`bind_event_expr(...)` is internal scaffolding in `C1`. Do not require `src/engine/change.rs` to consume it yet; later phases can move command integration onto this boundary once runtime semantics are ready.

In `src/expr/eval.rs`, define the runtime API that later phases will fill in:

    pub enum TruthValue {
        Zero,
        One,
        Unknown,
    }

    pub(crate) fn eval_logical_expr(
        expr: &DeferredLogicalExpr,
        host: &dyn ExpressionHost,
        timestamp: u64,
    ) -> Result<TruthValue, ExprDiagnostic>;

In `src/expr/mod.rs`, keep the public typed lex/parse surface separate from transitional command adapters:

    pub use crate::expr::ast::{BasicEventAst, DeferredLogicalExpr, EventExprAst, EventTermAst};
    pub use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
    pub use crate::expr::lexer::{Token, TokenKind};

    pub fn lex_event_expr(source: &str) -> Result<Vec<Token>, ExprDiagnostic>;
    pub fn parse_event_expr_ast(source: &str) -> Result<EventExprAst, ExprDiagnostic>;
    pub(crate) fn parse(source: &str) -> Result<Expression, WavepeekError>;
    pub(crate) fn parse_event_expr(source: &str) -> Result<EventExpr, WavepeekError>;

`tests/expression_c1.rs` must call the typed public lex/parse entry points directly and use named `insta` file snapshots for rendered parse diagnostics. Prefer explicit snapshot names `parse_unmatched_open`, `parse_unmatched_close`, `parse_empty_iff`, and `parse_broken_union` so the committed filenames stay deterministic under `tests/snapshots/`.

The internal unit tests in `src/expr/sema.rs` and `src/expr/eval.rs` should use their own named `insta` assertions for the semantic and runtime diagnostic layers, because those entry points remain `pub(crate)` in `C1` and are not part of the integration-test public surface.

In `benches/expr_c1.rs`, define one `Criterion` benchmark group that uses only `lex_event_expr(...)` and `parse_event_expr_ast(...)`, not private modules. The benchmark IDs must be exactly `tokenize_union_iff`, `parse_event_union_iff`, and `parse_event_malformed`. The benchmark bodies must panic if a supposedly-successful scenario fails or if the malformed scenario unexpectedly parses, so broken parser behavior is caught during measurement runs instead of silently skewing numbers.

In `bench/expr/capture.py`, define a CLI with this shape:

    python3 bench/expr/capture.py \
      --criterion-root target/criterion \
      --baseline-name <criterion-baseline> \
      --output bench/expr/runs/<run-name> \
      --source-commit <git-sha> \
      --worktree-state <clean|dirty> \
      --environment-note <text>

The script must export one `<scenario>.raw.csv` per benchmark scenario plus a deterministic `summary.json` in the run root. `summary.json` must include, for each scenario, the scenario name, the requested Criterion baseline name, sample count, mean nanoseconds per iteration, median nanoseconds per iteration, and the relative path to the exported `raw.csv` file. The script must fail if the requested baseline is absent or if any required scenario is missing, duplicated, unexpectedly present, or sourced from another saved baseline.

In `bench/expr/compare.py`, consume only the exported run directories and `summary.json` files, not `target/criterion`. Validate exact scenario-set equality, then compare both mean and median nanoseconds-per-iteration deltas against `--max-negative-delta-pct`. Support the two policy modes used by this plan: `15%` for cross-commit candidate-versus-baseline regression screening and `5%` for same-commit baseline-versus-verify reproducibility screening. Print one concise success line when all scenarios pass.

Treat `parse(...)` and `parse_event_expr(...)` as transitional adapters for existing command code, not as the new typed `C1` library contract. Those adapters must remain compatibility-preserving in this phase. Temporary parsing duplication between the strict typed path and the legacy compatibility adapter is acceptable in `C1` if it is required to preserve current command behavior; remove that duplication only in a later integration phase.

`C1` may add one new error variant to `src/error.rs` only if it is required for future integration and does not leak into current CLI behavior unexpectedly. If no such need appears, keep `WavepeekError` unchanged in this phase.

Revision Note: 2026-03-10 / OpenCode - Created the initial active ExecPlan for roadmap phase `C1`, then tightened it after review to keep command integration out of scope, make `src/lib.rs` extraction explicit, and turn the parser benchmark into a reproducible golden-plus-verify harness with exact scenario-set checks.

Revision Note: 2026-03-10 / OpenCode - Reopened the plan before implementation to adopt `insta` for diagnostic snapshots, switch parser microbenchmarks from a `hyperfine`-driven `src/bin` helper to a `Criterion` benchmark target under `benches/`, export committed `raw.csv`-based run artifacts under `bench/expr/runs/`, and document `hyperfine` as E2E-only collateral.
