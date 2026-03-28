# Require `triggered()` Call Syntax for Raw Events

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, the standalone expression engine will stop reserving bare `.triggered` as special syntax. Users will be able to write ordinary operand references whose canonical path ends in `.triggered`, such as `top.dut.triggered` or even `ev.triggered` when that is a real signal name, and those names will resolve like any other signal path. Raw event occurrence will remain available, but only through the explicit empty-call form `ev.triggered()`.

The result must be visible in two concrete ways. First, a host that exposes both raw event `ev` and ordinary signal `ev.triggered` must evaluate `ev.triggered` as the signal and `ev.triggered()` as raw-event occurrence in the same test run. Second, the waveform-backed host must prove the same collision with a real dump-backed pair such as `top.ev` and `top.ev.triggered`, showing that bare `top.ev.triggered` resolves as the ordinary signal while `top.ev.triggered()` still means raw-event occurrence. The old shorthand meaning disappears completely: if `ev.triggered` is not a real signal name, it must fail as an ordinary unknown-signal reference. The command-runtime boundary remains unchanged: `change --on "... iff ..."` still rejects runtime `iff` execution, and `property` still remains unimplemented.

Phase mapping: this is a `C4` standalone-expression contract correction only. It updates the already-delivered `C4` surface in `docs/expression_lang.md` Sections `2.1`, `2.2`, `2.3.13`, `2.6`, and `2.7` without pulling any `C5` command-runtime integration into scope.

## Non-Goals

This plan does not add general function-call or method-call syntax to the logical language. `f()`, `f(a)`, `top.fsm.next()`, and every non-`triggered()` call-like form must remain unsupported. This plan does not change the staged command-integration boundary from `docs/expression_roadmap.md`; no default `change` or `property` runtime wiring changes are allowed. This plan does not preserve a backward-compatible raw-event shorthand alias for `.triggered`, because that would keep the name collision alive. This plan also does not rewrite completed historical ExecPlans under `docs/exec-plans/completed/`; those remain historical records.

## Progress

- [x] (2026-03-28 13:13Z) Reviewed `docs/expression_lang.md`, `docs/expression_roadmap.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, the current `src/expr/` implementation, rich-type manifests, CLI boundary tests, waveform-host tests, and expression microbenchmarks to map every current `.triggered` dependency.
- [x] (2026-03-28 13:24Z) Drafted this active ExecPlan with the contract correction, red-first test strategy, parser/binder implementation path, collateral updates, benchmark handling, and mandatory review gates.
- [x] (2026-03-28 13:26Z) Ran focused review lanes on the plan and revised it for same-prefix collision coverage, waveform-host collision coverage, manifest-contract validation, phase mapping, and supported benchmark commands.
- [x] (2026-03-28 13:29Z) Ran a fresh control review pass and revised the benchmark workflow so committed baseline collateral is captured only after the review-clean final diff.
- [x] (2026-03-28 13:31Z) Completed the final control review pass with no substantive remaining issues.
- [x] (2026-03-28 17:07Z) Updated `docs/expression_lang.md`, `docs/expression_roadmap.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, and `CHANGELOG.md` so the standalone contract now names `triggered()` as the only raw-event form and records the breaking shorthand removal.
- [x] (2026-03-28 17:07Z) Added the red parser/lexer/manifest coverage first, captured expected failures (`EXPR-PARSE-LOGICAL-TRAILING` for `ev.triggered()` and suffix-stealing assertions for `top.dut.triggered`), then removed logical suffix reservation and whitelisted only `.triggered()` in the lexer/parser.
- [x] (2026-03-28 17:10Z) Updated semantic diagnostics, deferred-runtime CLI assertions, waveform-host collision coverage, and the `expr_event` benchmark source so every remaining standalone execution surface now uses `triggered()` and bare `.triggered` resolves through ordinary signal lookup.
- [ ] Run focused/full validation, complete the mandatory review workflow, refresh the maintained benchmark baseline on the accepted final diff, and record the final evidence in this plan.

## Surprises & Discoveries

- Observation: the collision is caused entirely by logical-expression lexing, not by the event-expression lexer.
  Evidence: `src/expr/lexer.rs` allows `.` inside logical identifiers, but its `peek_reserved_triggered_suffix()` helper breaks identifier scanning before `.triggered`; `lex_event_expr(...)` does not do that.

- Observation: the current parser already rejects generic call-like syntax, which makes a one-off `triggered()` exception much safer than adding general call support.
  Evidence: `tests/fixtures/expr/integral_boolean_negative_manifest.json` locks `f()`, `f(a)`, and `top.fsm.next()` as `EXPR-PARSE-LOGICAL-TRAILING` failures.

- Observation: most collateral is test and benchmark data, not production runtime wiring.
  Evidence: short-form `.triggered` appears in `tests/fixtures/expr/rich_types_positive_manifest.json`, `tests/fixtures/expr/rich_types_negative_manifest.json`, `tests/change_cli.rs`, `tests/property_cli.rs`, `src/waveform/expr_host.rs`, and `bench/expr/expr_event.rs`; the evaluator already models raw-event occurrence through `ExpressionHost::event_occurred(...)`.

- Observation: the shared fixture host already supports dotted signal names, so the collision fix does not require schema or helper redesign.
  Evidence: `tests/common/expr_runtime.rs` resolves signal fixtures by full `name: String` keys and does not special-case dot segments.

- Observation: the existing parser already gives the right malformed-call failure shape once the lexer stops stealing bare suffixes; the main parser work is consuming an empty `()` and improving member-suffix notes.
  Evidence: after the red test edit but before the parser change, `ev.triggered()` failed as `EXPR-PARSE-LOGICAL-TRAILING` at the opening `(`, which confirmed that generic call syntax stayed closed and only the postfix whitelist needed to move.

- Observation: the waveform-backed collision proof does not require any host redesign; a nested VCD scope named `ev` under `top` naturally yields the ordinary signal path `top.ev.triggered` alongside the raw event `top.ev`.
  Evidence: `src/waveform/expr_host.rs` now exercises a temp VCD where `top.ev` is declared as an event and `top.ev.triggered` is declared as a wire inside `$scope module ev`, and one logical expression distinguishes the two at timestamp `10`.

## Decision Log

- Decision: raw-event access will use `event_operand.triggered()` with an empty argument list, and bare `.triggered` will lose all special meaning.
  Rationale: the whole point of this change is to make `... .triggered` available as an ordinary signal-name suffix, so the shorthand cannot remain active in any compatibility mode.
  Date/Author: 2026-03-28 / OpenCode

- Decision: the parser will whitelist only the zero-argument member-like form `.triggered()` and will continue rejecting every other call-like syntax.
  Rationale: this is the smallest safe change that resolves the collision without widening the expression language beyond the documented contract.
  Date/Author: 2026-03-28 / OpenCode

- Decision: bare identifiers ending in `.triggered` will go through ordinary signal resolution first, even when their prefix is also a raw event signal.
  Rationale: signal-name collisions disappear only if full operand references win. If a user keeps writing the old shorthand, the expression must now behave like a normal signal lookup, not like a hidden event-method fallback.
  Date/Author: 2026-03-28 / OpenCode

- Decision: reuse the existing `LogicalExprNode::Triggered` and bound `Triggered` runtime semantics if possible, and limit code churn to syntax recognition, spans, and diagnostics.
  Rationale: the runtime meaning of raw-event occurrence does not change; only how the syntax reaches that existing semantic node changes.
  Date/Author: 2026-03-28 / OpenCode

- Decision: refresh the committed expression microbenchmark baseline after the final accepted implementation commit.
  Rationale: the `expr_event` benchmark source string changes from `ev.triggered` to `ev.triggered()`, so the maintained baseline should reflect the shipped workload rather than historical syntax.
  Date/Author: 2026-03-28 / OpenCode

## Outcomes & Retrospective

Current status: implementation largely complete; validation, review, and baseline refresh remain.

The contract docs are updated, the red phase is captured, the lexer/parser now preserve bare `.triggered` signal names while routing only `.triggered()` into raw-event syntax, and the remaining collateral now matches that contract. Remaining work is the final validation/review cycle and the benchmark-baseline refresh.

This plan still gives a stateless implementer a concrete route to finish the contract correction, preserve raw-event semantics through `triggered()`, unblock signal names ending in `.triggered`, and update every affected test, benchmark, and documentation surface. The draft itself already passed focused review lanes plus a fresh clean control pass, and the execution notes above now record the first completed implementation milestones.

## Context and Orientation

`wavepeek` is a single-crate Rust repository. The standalone expression engine lives in `src/expr/` and is already public through `wavepeek::expr::parse_logical_expr_ast(...)`, `bind_logical_expr_ast(...)`, `eval_logical_expr_at(...)`, `parse_event_expr_ast(...)`, `bind_event_expr_ast(...)`, and `event_matches_at(...)`. Command integration is intentionally incomplete: `change` still keeps a compatibility runtime path, and `property` still returns an unimplemented error. This plan must preserve that staged boundary.

The current bug is specific to logical-expression tokenization. Canonical signal paths in this repository are dot-separated hierarchical names such as `top.cpu.data`, so logical identifiers intentionally allow `.` as an interior character. In `src/expr/lexer.rs`, however, the `peek_reserved_triggered_suffix()` helper forces identifier scanning to stop before `.triggered`, which means `top.dut.triggered` cannot survive lexing as one operand reference. The parser in `src/expr/parser.rs` then treats that suffix as the special postfix form `.triggered`, and `src/expr/sema.rs` restricts it to raw `event` operands only. That design made the shorthand convenient, but it also made a real signal named `top.dut.triggered` impossible to address inside logical expressions or `iff` payloads.

Several repository terms matter here. A "raw event operand" is a signal whose recovered expression type is `ExprTypeKind::Event`; it is not sampled like a normal value and can only be queried as an occurrence at an exact timestamp. An "operand reference" is an identifier that resolves to a waveform signal path through `ExpressionHost::resolve_signal(...)`. A "collision" in this plan means that the parser steals a signal-name suffix and interprets it as syntax. "Collateral" means every non-core file that still locks or documents the old syntax: manifests, unit tests, CLI boundary tests, waveform-host tests, benchmark targets, roadmap/design docs, and the changelog.

The production code paths relevant to this change are concentrated and already separated by responsibility. `src/expr/lexer.rs` owns logical tokenization and currently performs the `.triggered` reservation. `src/expr/parser.rs` owns postfix parsing and currently accepts `.triggered` as the only member-like suffix. `src/expr/ast.rs` exposes `LogicalExprNode::Triggered`. `src/expr/sema.rs` validates that the triggered form applies only to raw-event operand references, and `src/expr/eval.rs` implements the runtime query through `ExpressionHost::event_occurred(...)`. The public host contract in `src/expr/host.rs` already has the correct runtime abstraction, so the expected semantic change is mostly syntax and diagnostics, not value-model redesign.

The regression surface is wider than the core parser files. `tests/expression_rich_types.rs` loads the manifest pair `tests/fixtures/expr/rich_types_positive_manifest.json` and `tests/fixtures/expr/rich_types_negative_manifest.json`; those files currently hard-code the short form in multiple logical and event cases. `tests/expression_fixture_contract.rs` is the schema guard for those manifests and must run whenever they change. `tests/change_cli.rs` and `tests/property_cli.rs` use `.triggered` inside deferred-runtime boundary assertions. `src/waveform/expr_host.rs` contains a waveform-backed rich-type test that uses the short form. `bench/expr/expr_event.rs` benchmarks `eval_event_iff_triggered_rich` with the short form, and `bench/expr/runs/baseline/` is the maintained benchmark collateral that must match the shipped source. The normative docs are `docs/expression_lang.md`, `docs/expression_roadmap.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, and `CHANGELOG.md`.

One important constraint is already helpful: generic call syntax is still intentionally unsupported. Existing negatives lock `f()`, `f(a)`, and `top.fsm.next()` as failures. That means the implementation should not add a general call AST or callable-expression subsystem. The safe path is to recognize exactly one method-like postfix, `.triggered()`, while leaving all other call-like text outside the whitelist.

## Open Questions

There are no blocking product questions left for this fix.

Implementation-time judgment is still required in one narrow area: whether to emit a new targeted parse diagnostic for malformed `.triggered(...)` forms with arguments, or to reuse existing `EXPR-PARSE-LOGICAL-EXPECTED` notes. If a new message is added, keep the existing error-code taxonomy stable unless a new code is clearly necessary. Do not add any fallback that revives the bare shorthand.

## Plan of Work

Milestone 1 updates the contract and locks the new behavior with red tests. Start by rewriting the normative language in `docs/expression_lang.md` so the contract says `event.triggered()` everywhere it currently says `.triggered` or `event.triggered`. Rename the member-like wording to method-like wording where helpful, update the grammar sketch to show `".triggered" "(" ")"`, and add an explicit note that bare `.triggered` remains part of an operand reference and is no longer reserved. Then align `docs/expression_roadmap.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, and `CHANGELOG.md` so every current-status and milestone summary names `triggered()` instead of the shorthand and records the breaking syntax change.

In the same milestone, add or rewrite the tests that prove the new contract before changing parser code. The manifest-driven rich-type suite is the main acceptance surface, so update the existing short-form positive cases to use `ev.triggered()`, add one same-prefix collision case where the host exposes both raw event `ev` and ordinary signal `ev.triggered`, and add one separate hierarchical case that uses a plain signal named `top.dut.triggered` together with a raw event `ev.triggered()` in the same expression. Add one negative migration case where the host provides only `ev` as a raw event and the source `ev.triggered` now fails as `EXPR-SEMANTIC-UNKNOWN-SIGNAL` over the full identifier span. Rewrite the existing trigger-specific negatives to use call syntax where they are still intended to be about raw-event semantics, for example `data.triggered()`, `(ev).triggered()`, `ev.triggered().triggered()`, `ev.triggered()[0]`, and `ev[0].triggered()`. Lock malformed call syntax in parser-focused coverage instead of the rich-type negative manifest: add unit regressions for `ev.triggered(1)` and `ev.triggered(` so only the empty `triggered()` call is whitelisted. Keep the generic `f()` and `top.fsm.next()` negatives unchanged so they continue guarding against accidental general call support.

Milestone 2 changes lexing and parsing with the smallest possible syntax delta. In `src/expr/lexer.rs`, replace the current reserved-suffix helper with a narrower lookahead that breaks identifier scanning only when the source tail is `.triggered` followed by an identifier boundary, optional whitespace, and `(`. That keeps `top.dut.triggered` as one identifier, still lets `ev.triggered()` tokenize as `Identifier("ev")`, `Dot`, `Identifier("triggered")`, `LeftParen`, `RightParen`, and allows the parser to own malformed-call diagnostics such as missing `)` or non-empty arguments. No new token kind is required because `Dot`, `LeftParen`, and `RightParen` already exist.

In `src/expr/parser.rs`, update `LogicalParser::parse_postfix_expr(...)` so `.triggered()` is the only supported member-like call. When the parser sees `.triggered`, it must require `(` and `)` immediately after the identifier token stream, allow only an empty argument list, and create the existing `LogicalExprNode::Triggered` node with a span that ends at the closing parenthesis. Any other dotted suffix that is tokenized as a member access must still fail with an `EXPR-PARSE-LOGICAL-EXPECTED` diagnostic whose note now names `.triggered()` instead of `.triggered`. Keep all ordinary identifiers that include `.triggered` segments untouched; they must parse as `LogicalExprNode::OperandRef`. Update or extend the lexer and parser unit tests so they explicitly cover `top.dut.triggered`, `top.dut.triggered[0]`, `ev.triggered()`, `ev.triggered ()`, and `ev.triggered(1)`.

Milestone 3 aligns semantic binding, runtime diagnostics, and the remaining collateral. The semantic runtime model already does the right thing, so keep the binding and evaluation rules but update them to describe the call form. In `src/expr/sema.rs`, keep validating that the triggered node applies only to a raw-event operand reference, but rewrite every diagnostic message and note to mention `.triggered()`. The important semantic change is indirect: because bare `.triggered` is no longer tokenized as syntax, `bind_logical_operand_ref(...)` must now see the full identifier and either resolve it as an ordinary signal or return the ordinary unknown-signal error. `src/expr/eval.rs` should require little or no logic change beyond span-sensitive tests, because the bound runtime kind is still the same exact-timestamp `Triggered` query. Update `src/expr/host.rs` comments if they still mention bare `.triggered`.

Finish this milestone by refreshing every collateral execution surface. Update `tests/change_cli.rs` and `tests/property_cli.rs` so their deferred-runtime expressions use `triggered()` while still proving that the command boundary is unchanged. Extend the inline waveform-backed test fixture in `src/waveform/expr_host.rs` so it exposes raw event `top.ev` and ordinary signal `top.ev.triggered` together, then prove in one dump-backed test that bare `top.ev.triggered` resolves as the signal while `top.ev.triggered()` still resolves as raw-event occurrence. Update `bench/expr/expr_event.rs` so `eval_event_iff_triggered_rich` benchmarks the new syntax. Do not change `src/engine/change.rs`, `src/engine/property.rs`, or `src/expr/legacy.rs`; those runtime boundaries are intentionally out of scope for this syntax correction.

Milestone 4 closes validation, review, and benchmark collateral. Run the focused expression suites first, then the repository gates, then the expression microbenchmark compare against the currently maintained baseline. Do not refresh `bench/expr/runs/baseline/` yet. Instead, use that pre-review compare only as a gate that the change is safe enough to review. Then load `ask-review` skill and run at least two focused review lanes in parallel: one for parser/semantic correctness and one for docs/tests/benchmark collateral. After fixing any findings in separate commits, run one fresh control pass on the consolidated diff. Only when the final post-review commit is accepted should `bench/expr/runs/baseline/` be refreshed to match the shipped syntax, and only then should the plan be closed.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

1. Reconfirm the current contract and staged boundary before editing code.

   Read `docs/expression_lang.md`, `docs/expression_roadmap.md`, and the expression-status block in `docs/DESIGN.md`. Confirm that the only intended behavior change is raw-event syntax, not command-runtime integration.

2. Update the contract docs and add red tests that demonstrate the collision and the new syntax.

   Edit these files first:

   - `docs/expression_lang.md`
   - `docs/expression_roadmap.md`
   - `docs/DESIGN.md`
   - `docs/ROADMAP.md`
   - `CHANGELOG.md`
   - `tests/fixtures/expr/rich_types_positive_manifest.json`
   - `tests/fixtures/expr/rich_types_negative_manifest.json`
   - `src/expr/lexer.rs` tests
   - `src/expr/parser.rs` tests

   Keep the red phase focused on four proofs:

   - `ev.triggered()` should be the new positive raw-event form.
   - `ev.triggered` should be a positive ordinary signal reference when that exact signal exists alongside raw event `ev`.
   - `top.dut.triggered` should be a positive ordinary signal reference.
   - `ev.triggered` should now be an unknown-signal negative when the host only exposes raw event `ev`.

   Red commands:

       cargo test expr::lexer::tests -- --nocapture
       cargo test expr::parser::tests -- --nocapture
       cargo test --test expression_fixture_contract
       INSTA_UPDATE=no cargo test --test expression_rich_types rich_types_positive_manifest_matches -- --exact
       INSTA_UPDATE=no cargo test --test expression_rich_types rich_types_negative_manifest_matches_snapshots -- --exact

   Expected red evidence before parser changes land is a parse failure for `ev.triggered()` and at least one failing assertion showing that `top.dut.triggered` is still being split into syntax instead of preserved as one operand reference. A short example is:

       thread 'logical_parser_accepts_rich_type_surface_sample' panicked at ...
       expression should parse: ExprDiagnostic { code: "EXPR-PARSE-LOGICAL-TRAILING", ... }

3. Implement the lexer and parser changes.

   Update these files:

   - `src/expr/lexer.rs`
   - `src/expr/parser.rs`
   - `src/expr/ast.rs` only if span handling or node comments need adjustment
   - `src/expr/diagnostic.rs` only if a new parse-note helper is truly needed

   After the changes, rerun:

       cargo test expr::lexer::tests
       cargo test expr::parser::tests
       INSTA_UPDATE=no cargo test --test expression_rich_types rich_types_negative_manifest_matches_snapshots -- --exact

   Success at this step means `ev.triggered()` parses, `top.dut.triggered` stays a single operand reference, `ev.triggered(1)` deterministically fails, and generic call-like negatives still remain failures.

4. Align semantics, runtime diagnostics, and the remaining collateral.

   Update these files:

   - `src/expr/sema.rs`
   - `src/expr/eval.rs` only if span-sensitive tests expose a necessary change
   - `src/expr/host.rs` comments if needed
   - `tests/change_cli.rs`
   - `tests/property_cli.rs`
   - `src/waveform/expr_host.rs`
   - `bench/expr/expr_event.rs`

   Keep `src/engine/change.rs`, `src/engine/property.rs`, and `src/expr/legacy.rs` untouched.

   Focused validation after this milestone:

        cargo test expr::sema::tests
        cargo test expr::eval::tests
        cargo test --test expression_fixture_contract
        INSTA_UPDATE=no cargo test --test expression_parse
        INSTA_UPDATE=no cargo test --test expression_event_runtime
        INSTA_UPDATE=no cargo test --test expression_integral_boolean
        INSTA_UPDATE=no cargo test --test expression_rich_types
        cargo test waveform::expr_host::tests
        cargo test --test change_cli
        cargo test --test property_cli

5. Run full validation and pre-review benchmark compare.

   Use the container-first benchmark workflow from `docs/DEVELOPMENT.md`:

       make bench-expr-run
       make check
       make ci

   Expected green signature after the implementation is complete is:

       INSTA_UPDATE=no cargo test --test expression_rich_types
       ...
       test result: ok.
       make bench-expr-run
       ...
       ok: no matched scenario exceeded 15.00% negative delta in mean or median
       make ci
        ...
        test result: ok.

6. Commit atomic units without rewriting history.

   Suggested commit split:

       git add docs/expression_lang.md docs/expression_roadmap.md docs/DESIGN.md docs/ROADMAP.md CHANGELOG.md
       git commit -m "docs(expr): require triggered() event call syntax"

       git add src/expr/lexer.rs src/expr/parser.rs src/expr/ast.rs tests/fixtures/expr/rich_types_positive_manifest.json tests/fixtures/expr/rich_types_negative_manifest.json
       git commit -m "feat(expr): parse triggered() without suffix reservation"

       git add src/expr/sema.rs src/expr/eval.rs src/expr/host.rs tests/change_cli.rs tests/property_cli.rs src/waveform/expr_host.rs bench/expr/expr_event.rs
       git commit -m "test(expr): align triggered() semantics and collateral"

   If review finds issues, fix them in one or more follow-up commits. Do not amend or squash.

7. Run the mandatory review workflow.

   Load `ask-review` skill and prepare one short context packet containing the scope summary, the exact commit range or diff, the validation commands already run, and the residual risks. Run these focused lanes in parallel:

   - Parser and semantics lane: lexer reservation removal, `.triggered()` parsing, bare `.triggered` signal resolution, semantic diagnostics, and generic-call regression risk.
   - Collateral lane: doc wording, manifest completeness, CLI boundary tests, waveform-host coverage, benchmark source and baseline refresh.

   After fixing any findings, run one fresh control pass on the consolidated diff. Treat review as complete only when both focused lanes and the control pass are clean, or all reported findings have been fixed, committed, and revalidated.

8. Refresh and commit the maintained benchmark baseline only after the review-clean final diff exists.

   Re-run the benchmark compare on the final post-review commit if any review fix touched parser, binder, evaluator, waveform-host, or benchmark files. Then refresh the maintained baseline:

       make bench-expr-run
       make bench-expr-update-baseline

   Commit that collateral last:

       git add bench/expr/runs/baseline
       git commit -m "bench(expr): refresh triggered() baseline"

### Validation and Acceptance

Acceptance is complete only when all of the conditions below are true together:

- `docs/expression_lang.md` names `event.triggered()` as the only raw-event value form, updates precedence/grammar wording accordingly, and states explicitly that bare `.triggered` remains part of operand references.
- `parse_logical_expr_ast("ev.triggered()")` succeeds, while `parse_logical_expr_ast("top.dut.triggered")` also succeeds as an ordinary operand reference rather than a special postfix.
- The rich-type positive manifest proves the same-prefix collision rule directly: when both raw event `ev` and ordinary signal `ev.triggered` exist, `ev.triggered` binds as the signal while `ev.triggered()` binds as raw-event occurrence.
- The rich-type positive manifest also proves a separate hierarchical signal whose path ends in `.triggered`, such as `top.dut.triggered`, and pairs it with a raw-event `triggered()` call in the same branch.
- The rich-type negative manifest proves that `ev.triggered` is no longer shorthand syntax and that `.triggered()` on non-events, chained triggered calls, and selected triggered calls still fail semantically.
- Parser-focused coverage in `src/expr/parser.rs` proves that malformed call syntax such as `ev.triggered(1)` and `ev.triggered(` is rejected, so only the empty `triggered()` call is whitelisted.
- Generic call-like negatives such as `f()` and `top.fsm.next()` still fail, proving that the parser surface did not silently widen beyond the single whitelisted method-like form.
- `cargo test --test expression_fixture_contract` passes after the manifest edits, proving the fixture schema and snapshot references remain valid.
- `tests/change_cli.rs` and `tests/property_cli.rs` still prove the same staged runtime boundary, only with `triggered()` syntax in their standalone-valid expressions.
- `src/waveform/expr_host.rs` proves the same collision on a dump-backed host: when both raw event `top.ev` and ordinary signal `top.ev.triggered` exist, bare `top.ev.triggered` resolves as the signal while `top.ev.triggered()` still resolves as raw-event occurrence.
- `bench/expr/expr_event.rs` uses the new syntax, `make bench-expr-run` passes, and the committed baseline is refreshed if the final implementation is accepted.
- `make check` and `make ci` pass after the last review-fix commit.
- The focused review lanes and the fresh control pass are clean, or every finding was fixed and rechecked.

TDD acceptance requirement: at least one newly edited lexer, parser, or rich-type test must fail before the code change and pass afterward in the same branch history.

### Idempotence and Recovery

All planned edits are source, test, benchmark, and documentation changes only. There are no schema migrations, no irreversible data transformations, and no runtime state changes outside committed benchmark artifacts. Re-running the test commands is safe.

If the lexer change accidentally broadens generic call syntax, recover by restoring the dot-in-identifier behavior first and rerunning `cargo test expr::lexer::tests`, `cargo test expr::parser::tests`, and `INSTA_UPDATE=no cargo test --test expression_integral_boolean` before touching semantic code. If `top.dut.triggered` still fails after parser work, inspect the lexer token stream first; the parser and binder cannot fix a name that was already split incorrectly.

If benchmark compare becomes noisy after the source-string change, capture a fresh revised run and compare it again before changing code. Only refresh `bench/expr/runs/baseline/` after the final reviewed diff is stable. If `make bench-expr-run` passes but `make ci` fails, treat `make ci` as the blocking gate and do not close the plan until it is green.

### Artifacts and Notes

Expected modified files for this plan:

- `docs/expression_lang.md`
- `docs/expression_roadmap.md`
- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `CHANGELOG.md`
- `src/expr/lexer.rs`
- `src/expr/parser.rs`
- `src/expr/ast.rs` only if needed for span clarity
- `src/expr/sema.rs`
- `src/expr/eval.rs` only if tests require a change
- `src/expr/host.rs` comments if needed
- `tests/fixtures/expr/rich_types_positive_manifest.json`
- `tests/fixtures/expr/rich_types_negative_manifest.json`
- `tests/expression_fixture_contract.rs` only if helper expectations need to change
- `tests/change_cli.rs`
- `tests/property_cli.rs`
- `src/waveform/expr_host.rs`
- `bench/expr/expr_event.rs`
- `bench/expr/runs/baseline/`

Before closing this plan, record these evidence excerpts here:

- Red phase from the first failing test after the new syntax is introduced but before parser changes are complete. A representative example should show `ev.triggered()` still failing to parse.
- Green phase from `INSTA_UPDATE=no cargo test --test expression_rich_types`.
- Green phase from `make bench-expr-run`.
- Green phase from `make ci`.
- Short review notes summarizing each focused lane and the fresh control pass.

Revision note: 2026-03-28 / Progress updated after docs, red tests, and lexer-parser implementation landed; remaining scope narrowed to semantic/collateral alignment, validation, review, and baseline refresh.
