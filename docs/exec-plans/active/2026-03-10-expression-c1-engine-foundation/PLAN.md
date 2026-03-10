# C1 Expression Engine Foundation: Spanned Parsing, Diagnostics, and Host Interfaces

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek` will have a real expression-engine foundation instead of the current mix of string wrappers and ad-hoc parsing. A contributor will be able to point at one parser architecture, one diagnostic format, one host interface for symbol/type/value lookup, and one repeatable benchmark flow for parser/tokenization work. This is the contract-level `C1` milestone from `docs/expression_roadmap.md`, so the goal is not to ship expression evaluation yet; the goal is to make later phases safe, testable, and deterministic.

The result is observable in two ways. First, dedicated expression-engine tests will load locked positive and negative manifests, compare exact diagnostic snapshots, and prove that malformed event syntax is rejected deterministically without panics. Second, the repository will contain a reproducible parser/tokenization benchmark flow with committed baseline and compare artifacts, so later expression work can measure regression against a fixed `C1` starting point.

For existing CLI users, successful `change --on ...` behavior must remain the same. `property` must remain unimplemented. The only allowed externally visible tightening in this phase is stricter rejection of malformed event syntax that is already invalid by contract, with stable `error: args:` wrapping preserved at the CLI boundary.

## Non-Goals

This plan does not implement event runtime evaluation semantics. This plan does not implement boolean operator semantics, logical-expression parsing beyond the minimal scaffolding needed to establish stable interfaces, or `property` runtime execution. This plan does not make `change --on "... iff ..."` work end to end. This plan does not change successful JSON schemas, add new public commands or flags, or expose internal expression diagnostics directly as a new CLI contract. This plan does not fold `C2` or later roadmap work into `C1`.

## Progress

- [x] (2026-03-10 11:51Z) Reviewed `docs/expression_roadmap.md`, `docs/expression_lang.md`, `docs/DESIGN.md`, `docs/BACKLOG.md`, current `src/expr/` implementation, and recent ExecPlan examples.
- [x] (2026-03-10 12:05Z) Drafted an active `C1` ExecPlan with explicit file paths, milestones, manifests, diagnostics snapshots, benchmark artifacts, and review gates.
- [ ] Implement Milestone 1: lock `C1` behavior with positive/negative manifests, diagnostic snapshots, and red-phase tests.
- [ ] Implement Milestone 2: add spanned lexer/parser, AST, semantic host interface, and deterministic parse/semantic/runtime diagnostics.
- [ ] Implement Milestone 3: route current event-syntax parsing through the `C1` facade without enabling runtime evaluation or changing successful command behavior.
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

## Outcomes & Retrospective

Current status: planning complete; implementation has not started yet.

This plan now gives a stateless contributor one place to find the `C1` scope, architecture targets, exact artifact paths, test/benchmark commands, commit boundaries, and review policy. The remaining work is implementation, validation, and closure against the locked `C1` gates.

The main risk to watch during implementation is accidental phase creep. The expression roadmap intentionally separates parser architecture from runtime semantics. If a change starts enabling real logical evaluation or `property` execution, it belongs in a later plan and should be rejected here.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI. Expression-related code currently lives in `src/expr/mod.rs`, `src/expr/parser.rs`, `src/expr/lexer.rs`, and `src/expr/eval.rs`. Today, `src/expr/mod.rs` exposes a placeholder `Expression` that stores only source text, simple event-expression structs (`EventExpr`, `EventTerm`, `EventKind`), and thin wrappers around the parser. `src/expr/parser.rs` contains the real event parsing logic, but that logic is string-oriented and does not produce byte spans or structured diagnostics. `src/expr/lexer.rs` tokenizes a few event-expression words and punctuation, but it is not wired into the parser. `src/expr/eval.rs` always returns an unimplemented error.

The only production caller of the current event parser is `src/engine/change.rs`. That file parses `--on`, rejects runtime `iff` execution with `error: args: iff logical expressions are not implemented yet`, resolves event signal names relative to scope, and evaluates triggers over waveform samples. `src/engine/property.rs` is still an unimplemented stub and must remain so in this phase.

The expression language contract is split across two docs. `docs/expression_lang.md` defines the full intended syntax and semantics, including event unions, `iff` binding, and future boolean-expression behavior. `docs/expression_roadmap.md` defines the phased rollout. In that roadmap, `C1` means architecture and strict parser/diagnostic foundations only. It explicitly includes strict rejection of unmatched parentheses, empty `iff`, and broken union segmentation. It explicitly excludes runtime expression evaluation and `property` runtime execution.

Several terms in this plan have repository-specific meanings. A "span" is a byte-range inside an expression string, with inclusive `start` and exclusive `end`, so diagnostics can point at exact text without ambiguity. A "manifest" is a repository-tracked list of test cases that a Rust test loads at runtime; here it means JSON files under `tests/fixtures/expr/` that lock parse successes and failures. A "diagnostic snapshot" is a plain-text file containing the exact rendered form of one internal expression diagnostic. A "host" is a trait that lets the expression layer ask the rest of the program to resolve signal names, query recovered type metadata, and fetch sampled values without importing waveform/engine logic directly. A "binder" is the semantic pass that converts parsed names into resolved handles and rejects invalid references before evaluation.

The current gaps that `C1` must close are concrete. There is no spanned token stream, no dedicated AST module, no semantic binding entry point, no internal parse/semantic/runtime diagnostic format, no locked manifest-based parser matrix, no deterministic no-panic corpus, and no parser benchmark harness. `docs/BACKLOG.md` already tracks two of these gaps directly: temporary `iff` capture rules in the event parser and unused lexer scaffolding.

## Open Questions

There are no blocking product questions left for `C1`. The plan resolves the two implementation-shape questions that matter here: use plain repository fixtures for manifests/snapshots, and use a dedicated `bench/expr/` harness instead of extending `bench/e2e`.

If implementation discovers that a dedicated benchmark binary under `src/bin/` is materially noisier or harder to maintain than expected, it may move the same driver logic into `bench/expr/driver.rs` plus a small `cargo run --example ...` style wrapper. The committed artifact paths and compare policy under `bench/expr/runs/` must stay the same.

## Plan of Work

Milestone 1 locks the `C1` contract with tests before structural refactors begin. Add manifest-driven integration tests and snapshot fixtures first, then run them in red state so there is branch-history proof that the current implementation does not satisfy `C1`. This milestone must define three categories of evidence: positive parse cases that should succeed, negative cases that should fail deterministically, and a no-panic corpus that intentionally exercises malformed token/parenthesis combinations.

Milestone 2 introduces the actual foundation modules. Add dedicated files for AST, diagnostics, semantic host interfaces, and binding entry points, then rewrite the lexer and parser around spanned tokens. This milestone is complete when `src/expr/` has one clear architecture: tokenization produces spanned tokens, parsing produces AST, binding has a stable host-facing entry point, and evaluation has a stable API that still returns a deterministic runtime-not-implemented diagnostic. The implementation must stay phase-correct: runtime semantics are not delivered here.

Milestone 3 connects the new parser facade to current callers carefully. `change` should consume the new syntax parser for event expressions so malformed-input debt is actually resolved, but successful command behavior must remain the same. Existing public error wrapping must remain stable at the CLI boundary. `property` must stay unimplemented, and this milestone must explicitly preserve the current behavior where `property` does not become a new parser-validation surface yet.

Milestone 4 adds the benchmark and closure artifacts. Create a small parser benchmark driver, committed run directories under `bench/expr/runs/`, and a compare helper that enforces the roadmap default gate of no worse than 15% mean and median regression on the fixed `C1` scenario set. Then update collateral: close the relevant `docs/BACKLOG.md` items when they are actually resolved, add a focused `CHANGELOG.md` note only if CLI-visible malformed-input behavior changed, and keep the active plan updated with red/green/review evidence.

Milestone 5 is validation and review closure. Run the dedicated expression tests, affected CLI tests, and the parser benchmark compare. Then run the mandatory review workflow using focused lanes and one fresh independent control pass. Any fixes from review must be committed separately, and the final validation run must happen after the last review-fix commit.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

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

   Keep a compatibility facade in `src/expr/mod.rs` so existing callers do not need to understand the full internal reorganization yet. That facade may continue exporting `EventExpr`, `EventTerm`, `EventKind`, and `Expression`, but it must be backed by the new parser/diagnostic pipeline rather than ad-hoc string logic.

   Targeted build/test commands after these changes:

       cargo test expr::lexer::tests
       cargo test expr::parser::tests
       cargo test --test expression_c1

3. Wire the strict parser facade into current command paths without widening runtime behavior.

   Update `src/engine/change.rs` so `parse_event_expr(...)` goes through the new `C1` syntax parser. Successful behavior must stay unchanged. Runtime `iff` behavior must stay unchanged: the parser may capture `iff` structure, but `change` must still return `error: args: iff logical expressions are not implemented yet` when runtime execution would require logical evaluation.

   Preserve CLI-facing wrapping. For malformed `--on` inputs, keep `WavepeekError::Args` formatting and the existing help hints that point to `wavepeek change --help`. The internal diagnostic object can be richer, but it must be translated to the current CLI surface so existing contract tests do not drift accidentally.

   Do not make `src/engine/property.rs` a new integration point for expression semantics. Re-run the existing property regression that proves malformed `--on` still does not escape the command's unimplemented boundary.

   Add focused CLI coverage in `tests/change_cli.rs` for the malformed cases that `C1` explicitly closes:

   - unmatched opening parenthesis in `--on`
   - unmatched closing parenthesis in `--on`
   - empty `iff` clause
   - broken union segmentation

   Run:

       cargo test --test change_cli change_rejects_unmatched_on_parenthesis -- --exact
       cargo test --test change_cli change_rejects_empty_iff_clause -- --exact
       cargo test --test change_cli change_rejects_broken_union_segmentation -- --exact
       cargo test --test property_cli property_invalid_on_expression_still_fails_as_unimplemented -- --exact

4. Add parser/tokenization benchmark tooling and artifacts.

   Create a new durable benchmark directory `bench/expr/` and satisfy the breadcrumb policy in the same change:

   - `bench/expr/AGENTS.md`
   - `bench/expr/compare.py`
   - `bench/expr/runs/c1-foundation-baseline/`
   - `bench/expr/runs/c1-foundation-final/`

   Add a small benchmark driver binary at `src/bin/expr_c1_bench.rs`. That binary must accept `--scenario <name>` and `--iterations <n>`. It must execute the public expression entry points only, without touching waveform files, and exit non-zero if the selected scenario does not produce the expected success or failure result. Define exactly these baseline scenarios:

   - `tokenize_union_iff`
   - `parse_event_union_iff`
   - `parse_event_malformed`

   Generate baseline artifacts with `hyperfine` JSON export, then a final run after implementation. Use `bench/expr/compare.py` to compare both directories and fail if any matched scenario regresses by more than 15% in either mean or median.

   Example commands:

       cargo build --release --bin expr_c1_bench
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-baseline/tokenize_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario tokenize_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-baseline/parse_event_union_iff.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_union_iff --iterations 20000'
       hyperfine --warmup 3 --runs 10 --export-json bench/expr/runs/c1-foundation-baseline/parse_event_malformed.hyperfine.json './target/release/expr_c1_bench --scenario parse_event_malformed --iterations 20000'

   Repeat the same three commands with `bench/expr/runs/c1-foundation-final/` after implementation, then run:

       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-final --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 15

5. Commit atomic units.

   Suggested commit split:

       git add tests/fixtures/expr tests/expression_c1.rs tests/change_cli.rs
       git commit -m "test(expr): lock c1 parser manifests and diagnostics"

       git add src/expr/mod.rs src/expr/ast.rs src/expr/diagnostic.rs src/expr/host.rs src/expr/lexer.rs src/expr/parser.rs src/expr/sema.rs src/expr/eval.rs
       git commit -m "refactor(expr): add c1 spanned parser foundation"

       git add src/engine/change.rs tests/property_cli.rs
       git commit -m "refactor(change): route event syntax through c1 parser facade"

       git add bench/AGENTS.md bench/expr/AGENTS.md bench/expr/compare.py bench/expr/runs src/bin/expr_c1_bench.rs docs/BACKLOG.md CHANGELOG.md
       git commit -m "bench(expr): add c1 parser regression harness and collateral"

   If review finds issues, fix them in separate follow-up commits. Do not amend history.

6. Validation gates.

   Run the focused gates first:

       cargo test --test expression_c1
       cargo test --test change_cli
       cargo test --test property_cli
       python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-final --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 15

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
- Existing successful `change --on ...` behavior remains unchanged in the relevant integration suites.
- `wavepeek change --on 'posedge ('` fails with `error: args:` and still points the user to `wavepeek change --help`.
- `wavepeek change --on 'posedge clk iff'` fails deterministically and does not panic.
- `wavepeek property --waves <fixture> --on 'posedge (' --eval '1'` still fails as `error: unimplemented:` rather than becoming a new parser/runtime surface in this phase.
- A semantic-layer test proves that the new binder/host interface can emit a deterministic semantic diagnostic (for example unresolved signal), even though full type semantics remain deferred.
- A runtime-layer test proves that the evaluator API still returns a deterministic runtime unimplemented diagnostic at `C1`.
- `bench/expr/runs/c1-foundation-baseline/` and `bench/expr/runs/c1-foundation-final/` exist with committed JSON artifacts and a readable `README.md` summary in each run directory.
- `python3 bench/expr/compare.py --revised bench/expr/runs/c1-foundation-final --golden bench/expr/runs/c1-foundation-baseline --max-negative-delta-pct 15` passes.
- `docs/BACKLOG.md` closes the parser-strictness debt and the unused-lexer debt only if the implementation actually resolves them; later-phase debts must remain open.
- `CHANGELOG.md` is updated only if there is a real user-visible malformed-input contract change to report.
- `make check` and `make ci` pass after the final review-fix commit.
- Review lanes and the fresh independent control pass are clean, or all findings were fixed and rechecked.

TDD acceptance requirement: at least one manifest-driven parser test must fail before the implementation changes and pass after them in the same branch history.

### Idempotence and Recovery

All planned edits are additive or in-place text changes plus committed benchmark artifacts. Re-running tests is always safe. Re-running the benchmark commands is safe if the existing `bench/expr/runs/c1-foundation-baseline/` and `bench/expr/runs/c1-foundation-final/` directories are either removed first or intentionally overwritten with the same scenario names.

If the parser refactor temporarily breaks `change`, recover in this order: first restore the compatibility facade in `src/expr/mod.rs`, then restore the `src/engine/change.rs` wrapper mapping, then rerun `cargo test --test change_cli` before continuing. If benchmark noise causes borderline compare failures, rerun the same `hyperfine` commands once more before changing code; only revise code after confirming the regression is reproducible.

If review finds issues, apply follow-up commits instead of rewriting history. If a newly added durable directory (`bench/expr/`) is kept, ensure its breadcrumb file and any parent breadcrumb updates are committed in the same review-fix sequence.

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
- `src/engine/change.rs`
- `tests/expression_c1.rs`
- `tests/change_cli.rs`
- `tests/property_cli.rs`
- `tests/fixtures/expr/c1_positive_manifest.json`
- `tests/fixtures/expr/c1_negative_manifest.json`
- `tests/fixtures/expr/diagnostics/*.txt`
- `src/bin/expr_c1_bench.rs`
- `bench/AGENTS.md`
- `bench/expr/AGENTS.md`
- `bench/expr/compare.py`
- `bench/expr/runs/c1-foundation-baseline/`
- `bench/expr/runs/c1-foundation-final/`
- `docs/BACKLOG.md`
- `CHANGELOG.md` (only if warranted by user-visible malformed-input tightening)

Before closing this plan, record these excerpts here:

- one red-phase failure from `tests/expression_c1.rs` before the parser foundation is implemented,
- one green-phase pass for the same test after implementation,
- one short CLI failure excerpt proving malformed `change --on` input is rejected deterministically,
- one short benchmark-compare success excerpt,
- one short summary each from the focused review lanes and the fresh control pass.

### Interfaces and Dependencies

The `C1` implementation must define these stable internal interfaces.

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

    pub fn bind_event_expr(ast: &EventExprAst, host: &dyn ExpressionHost) -> Result<BoundEventExpr, ExprDiagnostic>;

    pub fn bind_logical_expr(expr: &DeferredLogicalExpr, host: &dyn ExpressionHost) -> Result<(), ExprDiagnostic>;

`bind_logical_expr(...)` may still return a deterministic semantic-layer not-implemented diagnostic in `C1`; the important part is that the entry point and host boundary now exist and are tested.

In `src/expr/eval.rs`, define the runtime API that later phases will fill in:

    pub enum TruthValue {
        Zero,
        One,
        Unknown,
    }

    pub fn eval_logical_expr(
        expr: &DeferredLogicalExpr,
        host: &dyn ExpressionHost,
        timestamp: u64,
    ) -> Result<TruthValue, ExprDiagnostic>;

In `src/expr/mod.rs`, keep compatibility wrappers for current callers and tests:

    pub fn parse(source: &str) -> Result<Expression, WavepeekError>;
    pub fn parse_event_expr(source: &str) -> Result<EventExpr, WavepeekError>;

Those wrappers must use the new internal pipeline and translate internal diagnostics back to the currently expected CLI-facing `WavepeekError` categories/messages. Do not duplicate parsing logic in the wrappers.

`C1` may add one new error variant to `src/error.rs` only if it is required for future integration and does not leak into current CLI behavior unexpectedly. If no such need appears, keep `WavepeekError` unchanged in this phase.

Revision Note: 2026-03-10 / OpenCode - Created the initial active ExecPlan for roadmap phase `C1`, choosing manifest/snapshot fixtures plus a dedicated `bench/expr/` regression harness so the phase is self-contained and executable by a stateless contributor.
