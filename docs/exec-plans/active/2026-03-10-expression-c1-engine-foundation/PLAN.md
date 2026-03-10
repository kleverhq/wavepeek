# C1 Expression Engine Foundation: Spanned Parsing, Diagnostics, and Host Interfaces

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek` will have a real expression-engine foundation instead of the current mix of string wrappers and ad-hoc parsing. A contributor will be able to point at one parser architecture, one diagnostic format, one host interface for symbol/type/value lookup, and one repeatable benchmark flow for parser/tokenization work. This is the contract-level `C1` milestone from `docs/expression_roadmap.md`, so the goal is not to ship expression evaluation yet; the goal is to make later phases safe, testable, and deterministic.

The result is observable in two ways. First, dedicated expression-engine tests will load locked positive and negative manifests, compare exact diagnostic snapshots, and prove that malformed event syntax is rejected deterministically without panics. Second, the repository will contain a reproducible parser/tokenization benchmark flow with committed baseline and compare artifacts, so later expression work can measure regression against a fixed `C1` starting point.

For existing CLI users, successful `change --on ...` behavior must remain the same and `property` must remain unimplemented. This plan does not use command-surface rollout as its acceptance target; existing CLI suites are rerun only as regression guards while strict malformed-syntax handling is locked at the expression-layer boundary for later integration phases.

## Non-Goals

This plan does not implement event runtime evaluation semantics. This plan does not implement boolean operator semantics, logical-expression parsing beyond the minimal scaffolding needed to establish stable interfaces, or `property` runtime execution. This plan does not make `change --on "... iff ..."` work end to end. This plan does not change successful JSON schemas, add new public commands or flags, or expose internal expression diagnostics directly as a new CLI contract. This plan does not fold `C2` or later roadmap work into `C1`.

## Progress

- [x] (2026-03-10 11:51Z) Reviewed `docs/expression_roadmap.md`, `docs/expression_lang.md`, `docs/DESIGN.md`, `docs/BACKLOG.md`, current `src/expr/` implementation, and recent ExecPlan examples.
- [x] (2026-03-10 11:58Z) Drafted an active `C1` ExecPlan with explicit file paths, milestones, manifests, diagnostics snapshots, benchmark artifacts, and review gates.
- [x] (2026-03-10 12:01Z) Completed focused plan-review lanes and captured scope, breadcrumb, and benchmark reproducibility findings.
- [x] (2026-03-10 12:01Z) Revised the plan to keep `C1` phase-correct, make library extraction explicit, and tighten the benchmark artifact contract.
- [x] (2026-03-10 12:51Z) Completed fresh control-pass review on the revised plan; no substantive findings remain.
- [ ] Implement Milestone 1: lock `C1` behavior with positive/negative manifests, diagnostic snapshots, and red-phase tests.
- [ ] Implement Milestone 2: add spanned lexer/parser, AST, semantic host interface, and deterministic parse/semantic/runtime diagnostics.
- [ ] Implement Milestone 3: stabilize compatibility wrappers and regression guards without turning `change` or `property` into new `C1` integration surfaces.
- [ ] Implement Milestone 4: add parser/tokenization benchmark harness and committed baseline/compare artifacts, update collateral, and close the required review/validation loop.

## Surprises & Discoveries

- Observation: the current event parser is a raw string splitter with no spans, and it tolerates malformed closing-parenthesis cases because it decrements nesting depth with `saturating_sub`.
  Evidence: `src/expr/parser.rs` uses `depth = depth.saturating_sub(1)` while splitting `or` and `,` unions.

- Observation: `src/expr/lexer.rs` already exists, but nothing in the runtime consumes it today.
  Evidence: the only current call site for event parsing is `src/engine/change.rs`, and it calls `crate::expr::parse_event_expr(...)`, which delegates directly to `src/expr/parser.rs`.

- Observation: `property` remains a pure runtime stub, which gives this phase a safe place to establish parser/evaluator interfaces without accidentally expanding product behavior.
  Evidence: `src/engine/property.rs` returns `WavepeekError::Unimplemented("`property` command execution is not implemented yet")` immediately.

- Observation: the repository has benchmark infrastructure for CLI end-to-end work, but not for parser-internal benchmarking.
  Evidence: `bench/e2e/` contains the existing `hyperfine`-based harness, while no `bench/expr/` directory or parser benchmark driver exists today.

- Observation: the repository is currently a binary-only crate, so new manifest-driven expression integration tests and a benchmark helper binary cannot call shared modules until a library entrypoint exists.
  Evidence: `src/main.rs` declares the module tree directly and there is no `src/lib.rs` today.

## Decision Log

- Decision: keep `C1` strictly phase-scoped and explicitly preserve the roadmap boundary that runtime evaluation and `property` execution remain unimplemented.
  Rationale: `docs/expression_roadmap.md` requires one phase boundary per plan, and blending `C2` or `C5` work into this plan would make acceptance ambiguous.
  Date/Author: 2026-03-10 / OpenCode

- Decision: introduce a new internal expression diagnostic type with parse, semantic, and runtime layers, but keep existing CLI-facing `WavepeekError` wrapping at public command boundaries until later integration phases.
  Rationale: `C1` needs a stable internal diagnostics contract now, while `docs/DESIGN.md` and current integration tests still lock the public `error: args:` and `error: unimplemented:` surfaces.
  Date/Author: 2026-03-10 / OpenCode

- Decision: use repository-tracked JSON manifests plus plain-text snapshot files under `tests/fixtures/expr/` instead of introducing a snapshot-testing crate.
  Rationale: the repo does not currently depend on `insta` or similar tooling, and plain fixtures keep the `C1` contract readable, diffable, and self-contained.
  Date/Author: 2026-03-10 / OpenCode

- Decision: add a dedicated internal benchmark area under `bench/expr/` with its own breadcrumb file, and keep parser benchmarks separate from the existing CLI E2E harness.
  Rationale: `C1` regression gating is about lexer/parser internals rather than end-to-end command latency, so a separate harness avoids distorting `bench/e2e` semantics.
  Date/Author: 2026-03-10 / OpenCode

- Decision: prefer deterministic corpus-based no-panic testing over introducing external fuzz infrastructure in this phase.
  Rationale: `C1` requires fuzz/no-panic coverage, but a repo-local deterministic corpus can satisfy that requirement with lower setup cost and clearer CI behavior.
  Date/Author: 2026-03-10 / OpenCode

- Decision: add `src/lib.rs` and move `src/main.rs` to a thin wrapper before the new `C1` tests and benchmark binary are introduced.
  Rationale: both `tests/expression_c1.rs` and `src/bin/expr_c1_bench.rs` need a reusable crate surface; without `src/lib.rs`, a novice implementer would be blocked immediately.
  Date/Author: 2026-03-10 / OpenCode

- Decision: keep `bind_event_expr(...)` and the new host interface internal-only in `C1`; `src/engine/change.rs` keeps ownership of runtime name resolution and trigger execution in this phase.
  Rationale: this preserves the roadmap boundary that command integration is deferred while still establishing the semantic-layer architecture that later phases will consume.
  Date/Author: 2026-03-10 / OpenCode

- Decision: keep the new strict typed `wavepeek::expr` entry points decoupled from the legacy `parse_event_expr(...)` adapter in `C1`.
  Rationale: `change` currently calls the legacy adapter, and routing it through the strict parser in this phase would create an unintended command-surface rollout.
  Date/Author: 2026-03-10 / OpenCode

- Decision: use three benchmark directories in `C1`: `c1-foundation-candidate` for a dedicated clean pre-review commit after the first fully green implementation, `c1-foundation-baseline` for the final accepted post-review state that future phases inherit, and `c1-foundation-verify` for a second run from that same final state to prove reproducibility.
  Rationale: the parser benchmark harness does not exist before `C1`, so the plan needs one changed-state comparison during `C1` itself plus one carried-forward final golden for later phases, and the candidate state must be reconstructable by a stateless contributor.
  Date/Author: 2026-03-10 / OpenCode

## Outcomes & Retrospective

Current status: planning complete; implementation has not started yet.

This plan now gives a stateless contributor one place to find the `C1` scope, architecture targets, exact artifact paths, test/benchmark commands, commit boundaries, and review policy. The remaining work is implementation, validation, and closure against the locked `C1` gates.

The main risk to watch during implementation is accidental phase creep. The expression roadmap intentionally separates parser architecture from runtime semantics and from command-surface rollout. If a change starts enabling real logical evaluation, `property` execution, or new `change`/`property` public behavior, it belongs in a later plan and should be rejected here.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI. Expression-related code currently lives in `src/expr/mod.rs`, `src/expr/parser.rs`, `src/expr/lexer.rs`, and `src/expr/eval.rs`. Today, `src/expr/mod.rs` exposes a placeholder `Expression` that stores only source text, simple event-expression structs (`EventExpr`, `EventTerm`, `EventKind`), and thin wrappers around the parser. `src/expr/parser.rs` contains the real event parsing logic, but that logic is string-oriented and does not produce byte spans or structured diagnostics. `src/expr/lexer.rs` tokenizes a few event-expression words and punctuation, but it is not wired into the parser. `src/expr/eval.rs` always returns an unimplemented error.

The only production caller of the current event parser is `src/engine/change.rs`. That file parses `--on`, rejects runtime `iff` execution with `error: args: iff logical expressions are not implemented yet`, resolves event signal names relative to scope, and evaluates triggers over waveform samples. `src/engine/property.rs` is still an unimplemented stub and must remain so in this phase.

The expression language contract is split across two docs. `docs/expression_lang.md` defines the full intended syntax and semantics, including event unions, `iff` binding, and future boolean-expression behavior. `docs/expression_roadmap.md` defines the phased rollout. In that roadmap, `C1` means architecture and strict parser/diagnostic foundations only. It explicitly includes strict rejection of unmatched parentheses, empty `iff`, and broken union segmentation. It explicitly excludes runtime expression evaluation and `property` runtime execution.

Several terms in this plan have repository-specific meanings. A "span" is a byte-range inside an expression string, with inclusive `start` and exclusive `end`, so diagnostics can point at exact text without ambiguity. A "manifest" is a repository-tracked list of test cases that a Rust test loads at runtime; here it means JSON files under `tests/fixtures/expr/` that lock parse successes and failures. A "diagnostic snapshot" is a plain-text file containing the exact rendered form of one internal expression diagnostic. A "host" is a trait that lets the expression layer ask the rest of the program to resolve signal names, query recovered type metadata, and fetch sampled values without importing waveform/engine logic directly. A "binder" is the semantic pass that converts parsed names into resolved handles and rejects invalid references before evaluation.

The current gaps that `C1` must close are concrete. There is no reusable library crate surface, no spanned token stream, no dedicated AST module, no semantic binding entry point, no internal parse/semantic/runtime diagnostic format, no locked manifest-based parser matrix, no deterministic no-panic corpus, and no parser benchmark harness. `docs/BACKLOG.md` already tracks two of these gaps directly: temporary `iff` capture rules in the event parser and unused lexer scaffolding.

## Open Questions

There are no blocking product questions left for `C1`. The plan resolves the two implementation-shape questions that matter here: use plain repository fixtures for manifests/snapshots, and use a dedicated `bench/expr/` harness instead of extending `bench/e2e`.

No alternate benchmark harness path is planned in `C1`. Use `src/bin/expr_c1_bench.rs` so the benchmark commands, artifact names, and review expectations stay deterministic.

## Plan of Work

Milestone 1 locks the `C1` contract with tests before structural refactors begin. Add manifest-driven integration tests and snapshot fixtures first, then run them in red state so there is branch-history proof that the current implementation does not satisfy `C1`. This milestone must define three categories of evidence: positive parse cases that should succeed, negative cases that should fail deterministically, and a no-panic corpus that intentionally exercises malformed token/parenthesis combinations.

Milestone 2 introduces the actual foundation modules. Add dedicated files for AST, diagnostics, semantic host interfaces, and binding entry points, then rewrite the lexer and parser around spanned tokens. This milestone is complete when `src/expr/` has one clear architecture: tokenization produces spanned tokens, parsing produces AST, binding has a stable host-facing entry point, and evaluation has a stable API that still returns a deterministic runtime-not-implemented diagnostic. The implementation must stay phase-correct: runtime semantics are not delivered here.

Milestone 3 stabilizes the compatibility boundary without broadening command integration. The new parser/diagnostic stack must be reachable through `src/expr/mod.rs` compatibility wrappers and direct expression tests, but `src/engine/change.rs` and `src/engine/property.rs` should remain command-level regression guards rather than active `C1` feature targets. The semantic binder and host interface are introduced in this milestone as internal-only architecture that later phases can consume.

Milestone 4 adds the benchmark and closure artifacts. Create a small parser benchmark driver, committed run directories under `bench/expr/runs/`, and a compare helper that enforces the roadmap default gate of no worse than 15% mean and median regression on the fixed `C1` scenario set. In this milestone, `c1-foundation-candidate` is captured from the first fully green implementation before review, `c1-foundation-baseline` is captured from the final accepted `C1` state after the last review-fix commit, and `c1-foundation-verify` is a second run from that same final state used to prove reproducibility. Then update collateral: close the relevant `docs/BACKLOG.md` items when they are actually resolved, keep `CHANGELOG.md` unchanged because `C1` does not intentionally roll out new public behavior, and keep the active plan updated with red/green/review evidence.

Milestone 5 is validation and review closure. Run the dedicated expression tests, affected CLI tests, and the parser benchmark compare. Then run the mandatory review workflow using focused lanes and one fresh independent control pass. Any fixes from review must be committed separately, and the final validation run must happen after the last review-fix commit.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

0. Extract a minimal library entrypoint and typed API scaffold so the new tests and benchmark driver have a stable crate surface.

   Add `src/lib.rs` and move ownership of the existing module tree there (`cli`, `engine`, `error`, `expr`, `output`, `schema_contract`, `waveform`). `src/main.rs` must stop declaring modules and become a thin binary wrapper around the library entrypoint. In the same step, add the minimal public `wavepeek::expr` type/function stubs needed for `tests/expression_c1.rs` and `src/bin/expr_c1_bench.rs` to compile against the future `C1` surface. Placeholder bodies are acceptable here if they return deterministic scaffold diagnostics; Step 2 will replace them with the full implementation.

   This is prerequisite scaffolding required to make the `C1` manifest tests and benchmark driver feasible. Keep it minimal and structural; do not mix real parser semantics into this step.

1. Lock `C1` behavior with failing tests and fixtures first.

   Create these new fixture files:

   - `tests/fixtures/expr/c1_positive_manifest.json`
   - `tests/fixtures/expr/c1_negative_manifest.json`
   - `tests/fixtures/expr/diagnostics/parse-unmatched-open.txt`
   - `tests/fixtures/expr/diagnostics/parse-unmatched-close.txt`
   - `tests/fixtures/expr/diagnostics/parse-empty-iff.txt`
   - `tests/fixtures/expr/diagnostics/parse-broken-union.txt`
   - `tests/fixtures/expr/diagnostics/semantic-unresolved-signal.txt`
   - `tests/fixtures/expr/diagnostics/runtime-eval-unimplemented.txt`

   Add a new integration test file `tests/expression_c1.rs` that loads the manifests and snapshots. The positive manifest must at minimum cover `*`, named events, `posedge`/`negedge`/`edge`, `or` and `,` unions, and `iff` binding to only the immediately preceding event term. The negative manifest must at minimum cover empty input, unmatched `(`, unmatched `)`, empty `iff`, leading/trailing union separators, duplicated separators, and missing names after edge keywords.

   Also add a deterministic no-panic corpus test in `tests/expression_c1.rs`. Build the corpus from all positive and negative manifest inputs plus a small generated mutation set (for example, prefixes/suffixes containing extra parentheses, commas, and `or`/`iff` tokens). The test must call the public parser entry point inside `std::panic::catch_unwind` and fail if any input panics.

   Red-phase commands:

       cargo test --test expression_c1 c1_positive_manifest_parses -- --exact
       cargo test --test expression_c1 c1_negative_manifest_matches_snapshots -- --exact
       cargo test --test expression_c1 c1_no_panic_corpus_holds -- --exact

   Expected red evidence before implementation is complete: at least one of the first two tests fails because the current parser has no spans/snapshots and still accepts or mis-classifies malformed parenthesis/union cases.

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
       cargo test --test expression_c1

3. Stabilize the compatibility wrappers and keep command paths as regression guards only.

   Implement the new parser/diagnostic stack behind `src/expr/mod.rs` compatibility wrappers. Those wrappers may translate internal diagnostics into the current `WavepeekError` categories and help-hint wording, but `C1` acceptance must be defined at the expression-layer tests rather than by shipping new command-path behavior.

   Keep the ownership boundary explicit. `src/expr/sema.rs` and `src/expr/host.rs` are internal `C1` architecture. `src/engine/change.rs` continues owning runtime signal resolution and trigger execution in this phase. `src/engine/property.rs` remains an unimplemented regression guard and must not become a new parser/runtime surface.

   Add direct expression-layer tests for the public typed `wavepeek::expr` lex/parse surface:

   - malformed event syntax must fail through the new expression-layer diagnostics,
   - syntactically valid `iff` forms must still be representable in the AST.

   Add internal unit tests in `src/expr/sema.rs` and `src/expr/eval.rs` for the internal-only architecture introduced in `C1`:

   - `bind_logical_expr(...)` may still return the deterministic semantic not-implemented diagnostic,
   - `eval_logical_expr(...)` must still return the deterministic runtime not-implemented diagnostic.

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

4. Add parser/tokenization benchmark tooling and artifacts.

   Create a new durable benchmark directory `bench/expr/` and satisfy the breadcrumb policy in the same change:

    - update `bench/AGENTS.md` to add `bench/expr/AGENTS.md` under `Child Maps`
   - add `src/bin/AGENTS.md` and update `src/AGENTS.md` to link to it
    - `bench/expr/AGENTS.md`
    - `bench/expr/compare.py`
    - `bench/expr/test_compare.py`
   - `bench/expr/runs/c1-foundation-candidate/README.md`
   - `bench/expr/runs/c1-foundation-baseline/README.md`
   - `bench/expr/runs/c1-foundation-verify/README.md`

   Add a small benchmark driver binary at `src/bin/expr_c1_bench.rs`. That binary must accept `--scenario <name>` and `--iterations <n>`. It must execute the public expression entry points only, without touching waveform files, and exit non-zero if the selected scenario does not produce the expected success or failure result. Define exactly these baseline scenarios:

   - `tokenize_union_iff`
   - `parse_event_union_iff`
   - `parse_event_malformed`

   `bench/expr/compare.py` must validate exact scenario-set equality before comparing metrics. Missing, extra, or duplicate scenario artifacts are hard failures. Cover that behavior with unit tests in `bench/expr/test_compare.py`.

   Use this fixed benchmark profile exactly. Record the exact commands, `hyperfine --version`, `rustc -V`, the `source commit` from `git rev-parse HEAD`, whether the worktree was clean or dirty when the run was captured, and a short devcontainer/host note inside each run-directory `README.md`.

   Candidate run (from a dedicated clean pre-review commit after the first fully green implementation):

       cargo build --release --bin expr_c1_bench
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-candidate/tokenize_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario tokenize_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-candidate/parse_event_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-candidate/parse_event_malformed.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_malformed --iterations 20000'

   Baseline run (final accepted state, after the last review-fix commit):

       cargo build --release --bin expr_c1_bench
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-baseline/tokenize_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario tokenize_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-baseline/parse_event_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-baseline/parse_event_malformed.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_malformed --iterations 20000'

   Verify run (same final accepted state, second capture for reproducibility):

       cargo build --release --bin expr_c1_bench
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-verify/tokenize_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario tokenize_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-verify/parse_event_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-verify/parse_event_malformed.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_malformed --iterations 20000'

   Then run:

       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-baseline --golden bench/expr/runs/c1-foundation-candidate --max-negative-delta-pct 15
       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-verify --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 15

5. Commit atomic units.

   Suggested commit split:

       git add src/lib.rs src/main.rs src/expr/mod.rs
       git commit -m "refactor(crate): add library entrypoint and c1 api stubs"

       git add tests/fixtures/expr tests/expression_c1.rs tests/change_cli.rs
       git commit -m "test(expr): lock c1 parser manifests and diagnostics"

       git add src/expr/mod.rs src/expr/ast.rs src/expr/diagnostic.rs src/expr/host.rs src/expr/lexer.rs src/expr/parser.rs src/expr/sema.rs src/expr/eval.rs
       git commit -m "refactor(expr): add c1 spanned parser foundation"

        git add bench/AGENTS.md bench/expr/AGENTS.md bench/expr/compare.py bench/expr/test_compare.py src/AGENTS.md src/bin/AGENTS.md src/bin/expr_c1_bench.rs
        git commit -m "bench(expr): add c1 parser benchmark harness"

        git add bench/expr/runs/c1-foundation-candidate
        git commit -m "bench(expr): capture c1 candidate parser baseline"

        # after review fixes and final acceptance-state capture
        git add bench/expr/runs/c1-foundation-baseline bench/expr/runs/c1-foundation-verify docs/BACKLOG.md
        git commit -m "bench(expr): capture final c1 parser benchmark baselines"

   If review finds issues, fix them in separate follow-up commits. Do not amend history.

6. Validation gates.

   Run the focused gates first:

       cargo test --test expression_c1
       cargo test --test change_cli
       cargo test --test property_cli
       python3 -m unittest discover -s bench/expr -p 'test_compare.py'
       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-baseline --golden bench/expr/runs/c1-foundation-candidate --max-negative-delta-pct 15
       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-verify --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 15

   Then run repository gates:

       make check
       make ci

   Expected success signature after implementation is complete:

       cargo test --test expression_c1
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
   - Performance lane: lexer/parser allocation risk, benchmark-driver design, compare policy, and artifact reproducibility.

   Then run one fresh independent control pass on the consolidated diff. If any lane or the control pass finds real defects, fix them, commit them, rerun affected tests, and rerun the impacted lane plus a fresh control pass. Stop only when there are no unresolved substantive findings.

### Validation and Acceptance

Acceptance is complete only when all of the conditions below are true together:

- `tests/expression_c1.rs` loads `c1_positive_manifest.json` and every listed case parses successfully into the expected normalized AST shape.
- `tests/expression_c1.rs` loads `c1_negative_manifest.json` and every listed case fails with the expected diagnostic layer, code, span, and rendered snapshot file under `tests/fixtures/expr/diagnostics/`.
- The deterministic no-panic corpus test passes.
- The new parser rejects unmatched `(`, unmatched `)`, empty `iff`, and broken union segmentation deterministically.
- `src/lib.rs` exists and the new test/benchmark entry points use that shared crate surface rather than duplicating the module tree.
- `tests/expression_c1.rs` imports only the public `wavepeek::expr` lex/parse/AST/diagnostic paths, proving the typed `C1` API is usable by an external consumer.
- Existing `change_cli` and `property_cli` suites pass unchanged, proving that `C1` did not accidentally turn `change` or `property` into new command-surface rollout work.
- Targeted `change_cli` regression tests preserve the current legacy outcomes for unmatched `(`, unmatched `)`, broken union segmentation, and empty `iff`.
- A direct expression-layer test proves syntactically valid `iff` input is preserved in the AST while semantic/runtime work remains deferred.
- Internal unit tests in `src/expr/sema.rs` prove that the new binder/host interface can emit deterministic semantic diagnostics, even though full type semantics remain deferred.
- Internal unit tests in `src/expr/eval.rs` prove that the evaluator API still returns a deterministic runtime unimplemented diagnostic at `C1`.
- The legacy `parse_event_expr(...)` adapter and the real `change` CLI boundary remain compatibility-preserving in `C1`; any strict malformed-input rollout is deferred to a later integration phase.
- `bench/AGENTS.md` links to the new `bench/expr/AGENTS.md` child map.
- `bench/expr/runs/c1-foundation-candidate/`, `bench/expr/runs/c1-foundation-baseline/`, and `bench/expr/runs/c1-foundation-verify/` all exist with committed JSON artifacts and readable `README.md` summaries.
- Each benchmark `README.md` records the fixed command profile, `hyperfine --version`, `rustc -V`, the source commit used for measurement, whether the worktree was clean or dirty, and the basic environment note used to generate the artifacts. The candidate run must come from a clean dedicated commit so the source state is reconstructable.
- `python3 -m unittest discover -s bench/expr -p 'test_compare.py'` passes, including missing/extra-scenario hard-failure coverage.
- `python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-baseline --golden bench/expr/runs/c1-foundation-candidate --max-negative-delta-pct 15` passes, proving no unacceptable regression from the first-green state to the final accepted state.
- `python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-verify --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 15` passes.
- `docs/BACKLOG.md` closes the parser-strictness debt and the unused-lexer debt only if the implementation actually resolves them; later-phase debts must remain open.
- `CHANGELOG.md` remains unchanged in `C1`; a changelog delta is a scope warning because this plan does not intentionally roll out new public behavior.
- `make check` and `make ci` pass after the final review-fix commit.
- Review lanes and the fresh independent control pass are clean, or all findings were fixed and rechecked.

TDD acceptance requirement: at least one manifest-driven parser test must fail before the implementation changes and pass after them in the same branch history.

### Idempotence and Recovery

All planned edits are additive or in-place text changes plus committed benchmark artifacts. Re-running tests is always safe. Re-running the benchmark commands is safe if the existing `bench/expr/runs/c1-foundation-candidate/`, `bench/expr/runs/c1-foundation-baseline/`, and `bench/expr/runs/c1-foundation-verify/` directories are either removed first or intentionally overwritten with the same scenario names.

If the parser refactor temporarily breaks `change`, recover in this order: first restore the compatibility facade in `src/expr/mod.rs`, then restore the `src/engine/change.rs` wrapper mapping, then rerun `cargo test --test change_cli` before continuing. If benchmark noise causes borderline compare failures, rerun the same `hyperfine` commands once more before changing code; only revise code after confirming the regression is reproducible.

If review finds issues, apply follow-up commits instead of rewriting history. If the new durable directory `bench/expr/` is kept, ensure both `bench/expr/AGENTS.md` and the matching `bench/AGENTS.md` child-map update are committed in the same review-fix sequence.

### Artifacts and Notes

Expected modified or added files for `C1` implementation:

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
- `src/AGENTS.md`
- `src/bin/AGENTS.md`
- `tests/expression_c1.rs`
- `tests/change_cli.rs`
- `tests/fixtures/expr/c1_positive_manifest.json`
- `tests/fixtures/expr/c1_negative_manifest.json`
- `tests/fixtures/expr/diagnostics/*.txt`
- `src/bin/expr_c1_bench.rs`
- `bench/AGENTS.md`
- `bench/expr/AGENTS.md`
- `bench/expr/compare.py`
- `bench/expr/test_compare.py`
- `bench/expr/runs/c1-foundation-candidate/`
- `bench/expr/runs/c1-foundation-baseline/`
- `bench/expr/runs/c1-foundation-verify/`
- `docs/BACKLOG.md`

Before closing this plan, record these excerpts here:

- one red-phase failure from `tests/expression_c1.rs` before the parser foundation is implemented,
- one green-phase pass for the same test after implementation,
- one short typed expression-diagnostic excerpt proving malformed input is rejected deterministically,
- one short benchmark-compare success excerpt,
- one short summary each from the focused review lanes and the fresh control pass.

### Interfaces and Dependencies

The `C1` implementation must define these stable internal interfaces.

In `src/lib.rs`, define a thin reusable crate surface for tests and helper binaries:

    mod cli;
    mod engine;
    mod error;
    mod output;
    mod schema_contract;
    mod waveform;

    pub fn run_cli() -> Result<(), crate::error::WavepeekError>;

    pub mod expr;
    pub use crate::error::WavepeekError;

`src/main.rs` should become a tiny binary wrapper that calls `wavepeek::run_cli()` and keeps exit-code/error-print behavior unchanged. The public library surface must be sufficient for `tests/expression_c1.rs` and `src/bin/expr_c1_bench.rs` to use expression entry points without duplicating private module wiring.

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

The benchmark driver must use `lex_event_expr(...)` and/or `parse_event_expr_ast(...)`, not private modules. `tests/expression_c1.rs` may call the typed public lex/parse entry points directly. Treat `parse(...)` and `parse_event_expr(...)` as transitional adapters for existing command code, not as the new typed `C1` library contract. Those adapters must remain compatibility-preserving in this phase. Temporary parsing duplication between the strict typed path and the legacy compatibility adapter is acceptable in `C1` if it is required to preserve current command behavior; remove that duplication only in a later integration phase.

`C1` may add one new error variant to `src/error.rs` only if it is required for future integration and does not leak into current CLI behavior unexpectedly. If no such need appears, keep `WavepeekError` unchanged in this phase.

Revision Note: 2026-03-10 / OpenCode - Created the initial active ExecPlan for roadmap phase `C1`, then tightened it after review to keep command integration out of scope, make `src/lib.rs` extraction explicit, and turn the parser benchmark into a reproducible golden-plus-verify harness with exact scenario-set checks.
