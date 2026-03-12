# C3 Core Integral Boolean Contract and Standalone Logical Engine

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek` will have a real standalone Boolean-expression engine for the `C3` phase from `docs/expression_roadmap.md`, not only the bounded `iff` subset delivered in `C2`. A contributor will be able to parse, bind, and evaluate integral-family expressions directly through public `wavepeek::expr` APIs, including integral literals, signal references, selection, concatenation, replication, logical, bitwise, reduction, exponentiation, arithmetic, shifts, comparisons, all integral equality families, conditional `?:`, `inside`, implicit integral common-type rules, and explicit casts to integral targets. The same engine will then power standalone `iff` evaluation for event terms without waiting for command integration.

The result is observable without changing default CLI behavior. New manifest-driven tests will exercise the public standalone logical API and the existing standalone event API, proving that `C3` expressions evaluate deterministically and that `iff` now accepts the full integral-core logical surface while `change` still stays on the legacy runtime path and `property` still stays unimplemented. A dedicated `Criterion` benchmark target will measure binder and evaluator throughput for the integral Boolean core, and committed baseline artifacts under `bench/expr/runs/` will let later phases compare against this exact state.

This plan targets exactly one roadmap boundary: `C3` from `docs/expression_roadmap.md`. `docs/expression_roadmap.md` is the authoritative phase contract if it is more specific than the version-oriented bullets in `docs/ROADMAP.md`. That matters here because `docs/ROADMAP.md` still mentions some integral operators under `v0.5.0`, while `docs/expression_roadmap.md` assigns the full core integral Boolean contract to `C3`. This plan follows the phase document and still keeps public command rollout deferred to `C5`.

For `iff`, this plan is intentionally narrower than the `C4` “full `iff logical_expr` surface” wording in the roadmap coverage map. `C3` enables `iff` reuse only for the integral-core logical forms owned by `C3`. Rich-type, metadata-driven, and event-member forms still remain deterministic `C4` failures even when they appear inside `iff`.

## Non-Goals

This plan does not implement `real` semantics, `string` semantics, dump-derived operand-type casts such as `type(operand_reference)'(expr)`, enum-label references such as `type(enum_operand_reference)::LABEL`, or event-member `.triggered` semantics. This plan does not switch default `change` execution to the typed engine, does not implement `property` runtime execution, does not change JSON schema or public CLI output contracts, and does not collapse `C4` or `C5` work into `C3`. Any rich-type or command-integration feature deferred by `docs/expression_roadmap.md` must continue to fail deterministically in this phase.

## Progress

- [x] (2026-03-12 06:19Z) Reviewed `docs/expression_roadmap.md`, `docs/expression_lang.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, current `src/expr/` modules, `tests/expression_c2.rs`, existing benchmark artifacts, and prior expression ExecPlans.
- [x] (2026-03-12 06:19Z) Drafted the active `C3` ExecPlan with explicit spec scope, file targets, manifest and snapshot strategy, benchmark artifacts, validation gates, and phase-correct command boundaries.
- [x] (2026-03-12 06:34Z) Completed focused plan-review lanes, revised the plan for dynamic bit-select scope, enum-type identity, benchmark provenance checks, roadmap collateral, and `EventEvalFrame` self-containment, then closed with a fresh clean control pass.
- [x] (2026-03-12 08:35Z) Implemented Milestone 1 with locked `C3` manifests, dedicated `tests/expression_c3.rs`, focused unknown-flow regressions, and committed deterministic snapshots for deferred `C4`/const-expression diagnostics.
- [x] (2026-03-12 08:35Z) Implemented Milestone 2 by replacing the bounded logical parser path with full `C3` logical AST/lexer/parser and public `parse_logical_expr_ast(...)` plus offset-aware `iff` parsing reuse.
- [x] (2026-03-12 08:35Z) Implemented Milestone 3 by extending public type metadata, adding standalone logical bind/eval APIs, and wiring event `iff` evaluation through the shared logical binder/evaluator.
- [ ] Implement Milestone 4 by adding `C3` benchmark scenarios and collateral updates while preserving deferred `change` and `property` command behavior.
- [ ] Run validation, capture candidate/baseline/verify benchmark artifacts, complete mandatory multi-lane review, apply any review-fix commits, and rerun final gates.

## Surprises & Discoveries

- Observation: the current public typed expression surface still has no standalone logical-expression API, only event parse/bind/evaluate entry points plus an unused placeholder `Expression` wrapper.
  Evidence: `src/expr/mod.rs` publicly exports `parse_event_expr_ast(...)`, `bind_event_expr_ast(...)`, and `event_matches_at(...)`, while `src/expr/eval.rs` still contains an unimplemented `eval(_expression: &Expression)` stub.

- Observation: the current `C2` logical AST and binder are intentionally narrow and cannot be incrementally stretched to `C3` without a real typed value model.
  Evidence: `src/expr/ast.rs` only models operand refs, integral literals, `!`, comparisons, and `&&` / `||`, and `src/expr/eval.rs` only knows truthiness and comparison on a small `RuntimeValue` helper.

- Observation: the current public `ExprType` contract is too small for `C3` because it does not distinguish bit-vectors, integer-like types, enum-core values, packed-versus-scalar selection eligibility, or same-enum-type identity.
  Evidence: `src/expr/host.rs` currently exposes only `width`, `is_four_state`, and `is_signed`.

- Observation: the command-boundary debt remains explicit and safe to preserve.
  Evidence: `docs/DESIGN.md` states that `src/engine/change.rs` still uses the crate-private legacy adapter and `src/engine/property.rs` still returns an unimplemented error.

- Observation: the repository already has the right artifact patterns for this phase, so `C3` should extend them rather than inventing new workflows.
  Evidence: `tests/fixtures/expr/c2_positive_manifest.json`, `tests/fixtures/expr/c2_negative_manifest.json`, `tests/snapshots/`, `bench/expr/expr_c2.rs`, and `bench/expr/scenarios/c2_event_runtime.json` already define the manifest, snapshot, and `Criterion` conventions.

- Observation: logical based literals needed an explicit four-state type-domain default even when digits contain only `0/1` to preserve unknown conditional merge behavior.
  Evidence: `tests/expression_c3.rs` case `c3_unknown_flow_regressions_hold` expected `10xx` for `x_cond ? 4'b1010 : 4'b1001`; binder literal typing originally collapsed to two-state `1000` until based literals were typed four-state.

- Observation: the shared `tests/common` module is compiled per integration test crate under clippy `-D warnings`, so new reusable fixtures require explicit dead-code allowances.
  Evidence: pre-commit `cargo clippy --all-targets --all-features -- -D warnings` failed with dead-code diagnostics in `tests/common/expr_runtime.rs` until file-level/item-level `#[allow(dead_code)]` was added.

## Decision Log

- Decision: expose a real standalone logical parse/bind/evaluate API in `wavepeek::expr` during `C3` instead of keeping Section `2` semantics reachable only through event `iff`.
  Rationale: `C3` is still a standalone-engine milestone, and direct public APIs are the cleanest way to prove the contract with tests and benchmarks without accidentally pulling command integration into scope.
  Date/Author: 2026-03-12 / OpenCode

- Decision: keep `docs/expression_roadmap.md` authoritative for `C3` scope and treat any mismatch in `docs/ROADMAP.md` as wording debt rather than a reason to shrink the phase.
  Rationale: the roadmap document itself requires phase plans to map to one exact `C0`-`C5` boundary, and `C3` explicitly owns the core integral Boolean contract there.
  Date/Author: 2026-03-12 / OpenCode

- Decision: enrich the internal type and value model before implementing more operators.
  Rationale: selection, casts, equality families, `inside`, and conditional merge all depend on stable width, sign, state-domain, storage, and operand-kind rules. Adding operators first would create avoidable rewrites.
  Date/Author: 2026-03-12 / OpenCode

- Decision: keep rich-type and metadata-heavy forms (`real`, `string`, `type(...)`, enum labels, `.triggered`) as deterministic `C3` failures rather than partially parsing or silently widening the binder.
  Rationale: those features belong to `C4`, and keeping them explicitly unsupported preserves the roadmap boundary and makes negative manifests meaningful.
  Date/Author: 2026-03-12 / OpenCode

- Decision: preserve opaque enum-type identity in `C3` type metadata, but not enum labels or richer metadata pipelines.
  Rationale: `C3` needs to distinguish “same enum type” from “different integral-like types” for conditional result typing, while label-preservation and label lookup remain `C4` work.
  Date/Author: 2026-03-12 / OpenCode

- Decision: benchmark both direct logical evaluation and `iff` reuse in `C3`, but keep the benchmark target on the typed library API rather than the CLI binary.
  Rationale: the exit criterion is evaluator throughput for the standalone engine, and command-path latency remains a later `C5` concern.
  Date/Author: 2026-03-12 / OpenCode

- Decision: treat a missing sampled logical operand value as an all-`x` value of the operand's declared width, while keeping event named/edge semantics on the existing “no prior sample means no match” rule.
  Rationale: this matches the current `C2` standalone logical behavior, keeps direct logical evaluation deterministic, and prevents host-level “missing sample” behavior from remaining implicit in `C3`.
  Date/Author: 2026-03-12 / OpenCode

## Outcomes & Retrospective

Current status: Milestones 1-3 implemented and committed; Milestones 4-5 (benchmark collateral, captured run artifacts, mandatory multi-lane review, and final closure) are in progress.

This plan now defines the intended `C3` contract, test manifests, public standalone API shape, benchmark artifacts, and review gates tightly enough for a stateless contributor to execute the phase without relying on prior context. The largest remaining delivery risk is the interaction between selection semantics, constant-expression validation, common-type rules, and unknown-value propagation; the plan therefore sequences type-model work before operator expansion.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI with a thin library entrypoint in `src/lib.rs`. Expression work lives in `src/expr/`. Today, the public typed surface in `src/expr/mod.rs` exposes a strict event parser (`parse_event_expr_ast(...)`), a standalone event binder (`bind_event_expr_ast(...)`), and a standalone event evaluator (`event_matches_at(...)`). Those APIs were enough for `C2`, which implemented Section `1` event semantics and a bounded `iff` logical subset.

The current `C2` logical implementation is deliberately incomplete. `src/expr/ast.rs` models only a narrow internal logical tree for operand references, integral literals, `!`, `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, and `||`. `src/expr/parser.rs` has an internal bounded logical parser used only while binding `DeferredLogicalExpr` payloads from event `iff`. `src/expr/sema.rs` resolves signal names and integral literals for that subset, and `src/expr/eval.rs` evaluates it with four-state truthiness and short-circuit behavior. None of those modules currently implement the full integral Boolean surface that `C3` requires.

The command boundary is intentionally still split. `src/engine/change.rs` continues to parse `--on` through the crate-private legacy adapter in `src/expr/legacy.rs` and still rejects runtime `iff` execution with `error: args: iff logical expressions are not implemented yet`. `src/engine/property.rs` still returns `error: unimplemented: \`property\` command execution is not implemented yet`. That split must remain intact in this phase: `C3` expands the standalone engine only.

The product contract for this phase comes from two documents. `docs/expression_lang.md` is the semantic source of truth for the language. `docs/expression_roadmap.md` defines rollout sequencing and says that `C3` completes Section `2` for the integral-family core while keeping `real`, `string`, enum-label introspection, and `.triggered` deferred to `C4`. `docs/ROADMAP.md` is version-oriented and currently less precise on the exact `C3` versus `v0.5.0` split, so this plan uses the phase document as the exact scope source.

Several terms in this plan have repository-specific meanings. The "integral family" means `bit-vector`, `integer-like`, and `enum` values participating through underlying bit-vector semantics. A "common type" is the deterministic width, signedness, state-domain shape, and when needed enum-type identity chosen before evaluating most integral binary operators. A "constant integer expression" in this repository means an expression evaluated during binding without reading waveform samples; `C3` needs that for part-select bounds, indexed part-select widths, and replication multipliers, but not for plain dynamic bit-select indices. An `EventEvalFrame` is the public event-evaluation input object in `wavepeek::expr`; it carries the current probe timestamp, the previous probe timestamp used by change and edge detection, and the tracked signal handles watched by wildcard `*`. A "manifest" is a committed JSON fixture file under `tests/fixtures/expr/` that drives a Rust test. An "Insta snapshot" is a committed `.snap` file under `tests/snapshots/` or `src/expr/snapshots/` that locks the rendered form of one `ExprDiagnostic`. A "Criterion run artifact" is an exported benchmark directory under `bench/expr/runs/` containing committed `*.raw.csv`, `summary.json`, and `README.md` files.

The repository already has patterns this plan must reuse. `tests/expression_c1.rs` and `tests/expression_c2.rs` show the expected integration-test style for expression phases. `bench/expr/expr_c1.rs`, `bench/expr/expr_c2.rs`, and the scenario manifests under `bench/expr/scenarios/` show the existing `Criterion` benchmark workflow. `docs/DESIGN.md`, `docs/DEVELOPMENT.md`, and `docs/BACKLOG.md` already record the current staged status and must be updated in the same implementation when `C3` changes that status.

## Open Questions

There are no blocking product questions left for `C3`. The implementation-shape choices that matter are resolved in this plan: expose a direct standalone logical API, keep command integration deferred, model enum values through their core integral representation only, and reject `C4` forms deterministically instead of half-implementing them.

If implementation discovers that the current `ExpressionHost` contract still cannot express required integral metadata cleanly, extend `src/expr/host.rs` in a typed, library-facing way and update this plan plus `docs/DESIGN.md` in the same change. Do not add hidden command-only escape hatches.

## Plan of Work

Milestone 1 locks the `C3` contract with failing tests before any parser or evaluator rewrite lands. Add one positive manifest and one negative manifest under `tests/fixtures/expr/`, then add a new integration-style test file that uses only public `wavepeek::expr` APIs plus a deterministic in-memory host. The positive manifest must prove actual standalone logical evaluation and actual standalone event `iff` reuse. The negative manifest must prove that every deferred `C4` feature and every `C3` constant-expression or type-rule violation fails deterministically with stable spans and snapshots.

Milestone 2 expands the parser and public API surface. Extend the lexer so the typed engine can tokenize the `C3` integral operator set, casts, selections, concatenation, replication, and `inside`. Replace the bounded internal logical AST with a real `C3` logical AST, add a public `parse_logical_expr_ast(...)` entry point, and keep `parse_event_expr_ast(...)` strict and phase-correct. `iff` parsing must reuse the same logical parser through an offset-aware internal helper so that diagnostics still point into the full event source.

Milestone 3 adds the real `C3` binder and evaluator. Enrich type metadata, add operator applicability and common-type rules, implement explicit integral casts and constant-expression validation, and evaluate the full `C3` integral surface with deterministic `x` / `z` behavior. The event engine must then reuse the same logical binder and evaluator for `iff`, so standalone event tests prove that `C2` event semantics now compose with `C3` logical semantics.

Milestone 4 updates collateral and benchmarks while preserving the command boundary. Add a new `Criterion` target and scenario manifest for `C3`, capture candidate, baseline, and verify artifacts, and update status documents. `docs/DESIGN.md` must record that the standalone engine now supports the core integral Boolean contract, while `docs/BACKLOG.md` must keep command integration debt open because default `change` and `property` behavior are still deferred. `docs/DEVELOPMENT.md` and `bench/expr/AGENTS.md` must describe the new `expr_c3` benchmark target.

Milestone 5 is validation and review closure. Run the new standalone logical suite, the earlier expression regression suites, the command-boundary regression suites, the `C3` benchmark compare, and repository gates. Then run mandatory focused review lanes for code, architecture, performance, and docs, followed by one fresh independent control pass on the consolidated diff. Any follow-up fixes must be committed separately and validated again.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

0. Keep the phase boundary explicit before implementation begins.

   Re-read `docs/expression_roadmap.md` Section `C3`, `docs/expression_lang.md` Sections `2.1` through `2.6` plus the `C3`-relevant grammar subset in `2.7`, and the current `docs/DESIGN.md` implementation-status block. If implementation discoveries force a scope change, update this plan and the relevant status docs in the same commit. Do not broaden scope by silently routing `change` or `property` through the typed engine.

1. Lock `C3` behavior with failing tests and fixtures first.

   Add these new files:

   - `tests/fixtures/expr/c3_positive_manifest.json`
   - `tests/fixtures/expr/c3_negative_manifest.json`
   - `tests/expression_c3.rs`
   - `tests/snapshots/expression_c3__*.snap`

   The positive manifest must be one file but may contain separate arrays for direct logical cases and standalone event cases. Use this shape so the file stays phase-local while still covering both direct and `iff`-embedded execution paths:

       {
         "logical_cases": [ ... ],
         "event_cases": [ ... ]
       }

   Each logical case must specify, at minimum, a case name, expression source, in-memory signal fixtures, one evaluation timestamp, the expected normalized result bits, and the expected public type metadata. Each event case must specify, at minimum, a case name, full event expression source, tracked-signal names, signal fixtures, probe timestamps, and the timestamps that must match.

   Cover at least these positive buckets:

   - precedence and associativity across unary operators, exponentiation, arithmetic, shifts, comparisons, equality, bitwise operators, logical operators, and `?:`;
   - explicit integral casts using `signed`, `unsigned`, `bit`, `logic`, `bit[N]`, `logic[N]`, `signed bit[N]`, `signed logic[N]`, and integer-like targets;
   - selection forms `expr[idx]`, `expr[msb:lsb]`, `expr[base +: width]`, and `expr[base -: width]`, where plain `expr[idx]` uses a dynamic integral index and the tests cover valid, out-of-range, and `x` / `z` index behavior;
   - concatenation and replication, including the rule that unsized constants are rejected in concatenation;
   - logical, bitwise, reduction, arithmetic, shift, comparison, equality, wildcard-equality, case-equality, conditional, and `inside` behavior over integral operands;
   - enum operands participating through their underlying integral value while preserving only opaque enum-type identity when both `?:` arms share the same enum type;
   - unknown-flow regressions where `x` / `z` either propagate, merge, or collapse according to the `C3` contract;
   - direct logical evaluation when `SampledValue.bits == None`, which must behave as an all-`x` value at the operand's declared width before normal operator semantics apply;
   - standalone event cases whose `iff` payloads use arithmetic, shifts, selection, casts, equality families, `?:`, `inside`, concatenation, or replication.

   The negative manifest should keep one flat `cases` array and include an `entrypoint` field set to either `logical` or `event`. Each case must specify source text, expected diagnostic layer, code, span, and optional snapshot name. Include at least one locked negative case for each bucket below:

   - deferred `C4` forms: `real` literals or casts, `string` literals or casts, `type(operand_reference)'(...)`, `type(enum_operand_reference)::LABEL`, and `.triggered`;
   - invalid constant-expression use in part-select bounds, indexed part-select widths, and replication multipliers;
   - invalid selection base types, especially scalar `integer-like` and enum-core values;
   - invalid integral cast targets or syntactically malformed cast forms;
   - unresolved signal names in direct logical expressions and inside event `iff` payloads;
   - malformed `inside` sets and malformed concatenation or replication syntax;
   - event-level grouping or command-integration behavior that must still stay outside `C3`.

   Add targeted tests alongside the manifests for tricky rules that are easier to read as code than as fixture rows. At minimum include dedicated tests for:

   - short-circuit preservation after new operators are added around existing logical nodes;
   - conditional merge with unknown condition and both integral arms;
   - wildcard equality versus case equality semantics;
   - selection behavior when the base expression already comes from concatenation or replication;
   - dynamic bit-select evaluation with sampled index signals and missing sampled operands.

   If the in-memory host helper becomes large, move it into `tests/common/expr_runtime.rs` and re-export it from `tests/common/mod.rs`. Keep the helper deterministic and library-facing; it must not depend on CLI argument parsing or waveform file loading. `tests/expression_c3.rs` should show the minimum `EventEvalFrame` construction pattern directly so a novice can see how `timestamp`, `previous_timestamp`, and `tracked_signals` interact.

   Red-phase commands:

       cargo test --test expression_c3 c3_positive_manifest_matches -- --exact
       INSTA_UPDATE=no cargo test --test expression_c3 c3_negative_manifest_matches_snapshots -- --exact
       cargo test --test expression_c3 c3_unknown_flow_regressions_hold -- --exact

   Expected red evidence before implementation exists: at least one test fails because `parse_logical_expr_ast(...)`, `bind_logical_expr_ast(...)`, or `eval_logical_expr_at(...)` is missing or because the current bounded parser rejects valid `C3` syntax.

2. Extend the typed parser and public standalone API.

   Update these source files:

   - `src/expr/mod.rs`
   - `src/expr/ast.rs`
   - `src/expr/lexer.rs`
   - `src/expr/parser.rs`
   - `src/expr/diagnostic.rs` (only as needed for new `C3` diagnostics)

   Keep `src/expr/legacy.rs` crate-private and compatibility-preserving. It may need compile-fix adjustments if shared types move, but it must not become the implementation target for `C3` semantics.

   `src/expr/lexer.rs` must be widened so it can tokenize the full `C3` integral surface: based integer literal forms, brackets, braces, `?:`, all equality variants, bitwise and reduction operator tokens, shift tokens, cast tick syntax, and the `inside` keyword. Event-expression tokenization and error paths from `C1` must stay deterministic.

   `src/expr/ast.rs` must replace the bounded logical tree with a real `C3` logical AST. The public AST needs enough structure for tests and benchmarks to call the parser directly. The tree must model, at minimum, operand references, integral literals, parenthesized expressions, explicit cast nodes, postfix selection nodes, unary prefix operators, binary operator nodes with precedence-correct structure, conditional nodes, `inside` items and ranges, concatenation, and replication. Event AST nodes remain public because `parse_event_expr_ast(...)` already exposes them.

   `src/expr/parser.rs` must expose a new public `parse_logical_expr_ast(source: &str) -> Result<LogicalExprAst, ExprDiagnostic>` entry point and an internal offset-aware helper used by event `iff` binding. The parser should cover exactly the `C3` grammar surface and reject `C4`-only forms deterministically with `C3` parse diagnostics. Keep `parse_event_expr_ast(...)` strict on event-level grouping and union segmentation; only the `iff` payload parser becomes richer.

   `src/expr/mod.rs` must re-export the new public logical surface and retire the unused placeholder `Expression` / `eval(...)` path so the library API reflects the real standalone-engine contract instead of an obsolete stub. After this step, the intended public entry points are `parse_logical_expr_ast(...)`, `bind_logical_expr_ast(...)`, and `eval_logical_expr_at(...)` for direct logical work, alongside the existing event parse/bind/evaluate functions.

   Targeted commands after the parser and API surface land:

       cargo test expr::lexer::tests
       cargo test expr::parser::tests
       INSTA_UPDATE=no cargo test --test expression_c3 c3_negative_manifest_matches_snapshots -- --exact

3. Add the `C3` type model, binder, and evaluator.

   Update these source files:

   - `src/expr/host.rs`
   - `src/expr/sema.rs`
   - `src/expr/eval.rs`
   - `src/expr/mod.rs`
   - `src/expr/diagnostic.rs` (if binder/runtime-specific `C3` codes improve determinism)

   `src/expr/host.rs` must grow richer public type metadata so the binder can distinguish bit-vectors, integer-like types, enum-core values, and same-enum-type identity and can reject invalid selection bases deterministically. Keep `SampledValue` as the host-returned waveform sample container, but define richer type-kind, storage, and opaque enum identity metadata on `ExprType` so tests and benches can assert actual result types instead of inferring everything from width and sign alone.

   `src/expr/sema.rs` must bind both direct logical ASTs and event `iff` payloads through the same logical binder. That binder must resolve signal names, compute operator result types, apply integral common-type rules, validate cast targets, validate constant integer expressions, and preserve enough bound information for the evaluator to implement width, signedness, and state-domain semantics without reparsing source text. `BoundLogicalExpr` should become a public opaque handle so tests and benches can pass it into the evaluator without inspecting internals.

   `src/expr/eval.rs` must evaluate direct logical expressions to a public result type that exposes normalized result bits plus public type metadata. It must implement the `C3` operator families and edge semantics described in `docs/expression_lang.md`: logical, bitwise, reduction, exponentiation, arithmetic, shifts, comparisons, equality families, conditional, `inside`, concatenation, replication, selection, and explicit integral casts. `x` / `z` propagation and merge behavior must follow the phase contract, not a simplified two-state shortcut. Only after those direct semantics are correct should `event_matches_at(...)` reuse them for `iff`, with the final event gate still reducing the logical result to two-state match behavior.

   Add low-level unit coverage in `src/expr/sema.rs` and `src/expr/eval.rs` for the rules that are hardest to express through one manifest row, especially common-type resolution, constant-expression rejection, cast resize/state-domain behavior, unknown comparisons, wildcard equality, conditional merge, `inside` range evaluation, and selection from derived packed values. Use committed `insta` snapshots under `src/expr/snapshots/` for binder or evaluator diagnostics that remain internal-only.

   Keep `C4` features deterministic and explicit. `real`, `string`, `type(operand_reference)'(...)`, enum-label references, and `.triggered` must remain parse or semantic failures with stable spans and error codes. Do not widen the host contract to carry enum label metadata yet.

   Targeted commands after the binder and evaluator exist:

       cargo test --test expression_c3
       cargo test expr::sema::tests
       cargo test expr::eval::tests
       INSTA_UPDATE=no cargo test --test expression_c2

4. Preserve the command boundary and add `C3` benchmark plus collateral updates.

   Keep `src/engine/change.rs` and `src/engine/property.rs` phase-correct. Do not route default command execution through the typed standalone logical engine. Instead, add or update regression tests so the boundary remains explicit:

   - `tests/change_cli.rs` must still prove that `change --on "... iff ..."` is recognized but runtime execution is deferred, even when the `iff` payload itself is valid `C3` syntax such as arithmetic or `inside`.
   - `tests/property_cli.rs` must still prove that `property` returns the unimplemented runtime error.

   Add these new benchmark and collateral files:

   - `bench/expr/expr_c3.rs`
   - `bench/expr/scenarios/c3_integral_boolean.json`
   - `bench/expr/runs/c3-integral-boolean-candidate/README.md`
   - `bench/expr/runs/c3-integral-boolean-baseline/README.md`
   - `bench/expr/runs/c3-integral-boolean-verify/README.md`

   Update these collateral files:

   - `Cargo.toml`
   - `bench/expr/compare.py`
   - `bench/expr/test_compare.py`
   - `docs/DESIGN.md`
   - `docs/DEVELOPMENT.md`
   - `docs/ROADMAP.md`
   - `docs/BACKLOG.md`
   - `bench/expr/AGENTS.md`

   `bench/expr/expr_c3.rs` must benchmark only the typed library surface, not the CLI binary, and must reuse the existing fixed Criterion profile (`sample_size = 100`, `warm_up_time = 3s`, `measurement_time = 5s`, `significance_level = 0.05`, `noise_threshold = 0.01`, `configure_from_args()`). The `bind_*` scenario must measure bind-only work. Each `eval_*` scenario must parse and bind once before `b.iter(...)` and must call only `eval_logical_expr_at(...)` or `event_matches_at(...)` inside the timed closure so evaluator regressions stay visible. Define exactly these benchmark IDs in both the scenario manifest and the bench target:

   - `bind_logical_core_integral`
   - `eval_logical_core_integral_true`
   - `eval_logical_core_integral_unknown`
   - `eval_event_iff_core_integral`

   `eval_logical_core_integral_true` must exercise a wide packed-value path that includes concatenation or replication followed by selection, cast, or comparison so allocator-heavy derived-value behavior is measured. `eval_logical_core_integral_unknown` must keep unknown-propagation visible on a non-trivial expression rather than on a single-bit toy case. The fourth scenario should benchmark event matching where the base event fires and the `iff` payload exercises selection, arithmetic, equality, or `inside`, proving that the event path reuses the logical engine.

   Update `bench/expr/compare.py` and `bench/expr/test_compare.py` so compare-time gates can require matching metadata keys when the plan depends on provenance, especially for the same-commit verify comparison. At minimum the verify comparison must reject mismatched `source_commit`, `worktree_state`, `cargo_version`, `rustc_version`, and `criterion_version`.

   Candidate run (first fully green pre-review implementation commit):

       cargo bench --bench expr_c3 -- --save-baseline c3-integral-boolean-candidate --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c3-integral-boolean-candidate --bench-target expr_c3 --scenario-set bench/expr/scenarios/c3_integral_boolean.json --output bench/expr/runs/c3-integral-boolean-candidate --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Final accepted baseline run (after the last review-fix commit):

       cargo bench --bench expr_c3 -- --save-baseline c3-integral-boolean-baseline --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c3-integral-boolean-baseline --bench-target expr_c3 --scenario-set bench/expr/scenarios/c3_integral_boolean.json --output bench/expr/runs/c3-integral-boolean-baseline --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Same-commit verify run:

       cargo bench --bench expr_c3 -- --save-baseline c3-integral-boolean-verify --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c3-integral-boolean-verify --bench-target expr_c3 --scenario-set bench/expr/scenarios/c3_integral_boolean.json --output bench/expr/runs/c3-integral-boolean-verify --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Then compare:

       python3 bench/expr/compare.py --revised bench/expr/runs/c3-integral-boolean-baseline --golden bench/expr/runs/c3-integral-boolean-candidate --max-negative-delta-pct 15 --require-matching-metadata worktree_state cargo_version rustc_version criterion_version
       python3 bench/expr/compare.py --revised bench/expr/runs/c3-integral-boolean-verify --golden bench/expr/runs/c3-integral-boolean-baseline --max-negative-delta-pct 5 --require-matching-metadata source_commit worktree_state cargo_version rustc_version criterion_version

5. Commit atomic units and close with review.

   Suggested commit split:

       git add tests/fixtures/expr/c3_positive_manifest.json tests/fixtures/expr/c3_negative_manifest.json tests/expression_c3.rs tests/snapshots
       git commit -m "test(expr): lock c3 integral manifests"

       git add src/expr/mod.rs src/expr/ast.rs src/expr/lexer.rs src/expr/parser.rs src/expr/diagnostic.rs
       git commit -m "feat(expr): add c3 logical parser surface"

       git add src/expr/host.rs src/expr/sema.rs src/expr/eval.rs src/expr/snapshots tests/expression_c2.rs tests/change_cli.rs tests/property_cli.rs
       git commit -m "feat(expr): implement c3 integral evaluator"

       git add Cargo.toml bench/expr/expr_c3.rs bench/expr/scenarios/c3_integral_boolean.json bench/expr/compare.py bench/expr/test_compare.py bench/expr/AGENTS.md docs/DESIGN.md docs/DEVELOPMENT.md docs/ROADMAP.md docs/BACKLOG.md
       git commit -m "bench(expr): add c3 integral benchmark flow"

       git add bench/expr/runs/c3-integral-boolean-candidate
       git commit -m "bench(expr): capture c3 candidate baseline"

       git add bench/expr/runs/c3-integral-boolean-baseline bench/expr/runs/c3-integral-boolean-verify
       git commit -m "bench(expr): capture final c3 baselines"

   If review finds issues, fix them in separate follow-up commits. Do not amend history.

6. Run validation gates.

   Focused gates first:

       INSTA_UPDATE=no cargo test --test expression_c1
       INSTA_UPDATE=no cargo test --test expression_c2
       INSTA_UPDATE=no cargo test --test expression_c3
       cargo test expr::parser::tests
       cargo test expr::sema::tests
       cargo test expr::eval::tests
       cargo test --test change_cli
       cargo test --test property_cli
       cargo test --bench expr_c3
       python3 bench/expr/compare.py --revised bench/expr/runs/c3-integral-boolean-baseline --golden bench/expr/runs/c3-integral-boolean-candidate --max-negative-delta-pct 15 --require-matching-metadata worktree_state cargo_version rustc_version criterion_version
       python3 bench/expr/compare.py --revised bench/expr/runs/c3-integral-boolean-verify --golden bench/expr/runs/c3-integral-boolean-baseline --max-negative-delta-pct 5 --require-matching-metadata source_commit worktree_state cargo_version rustc_version criterion_version

   Then repository gates:

       make check
       make ci

   Expected success signature after implementation is complete:

       INSTA_UPDATE=no cargo test --test expression_c3
       ...
       test result: ok.
       python3 bench/expr/compare.py --revised ... --golden ... --max-negative-delta-pct 15
       ok: no matched scenario exceeded 15.00% negative delta in mean or median
       make ci
       ...
       test result: ok.

7. Run the mandatory review workflow.

   Load `ask-review` skill and run these focused lanes in parallel after the green validation above:

   - Code lane: operator semantics, constant-expression enforcement, unknown-flow behavior, `iff` reuse, negative-coverage completeness, and missing tests.
   - Architecture lane: public standalone logical API shape, layering between `src/expr/` and command runtimes, and enum-core versus rich-type boundary discipline.
   - Performance lane: evaluator allocation patterns, selection and concat hot-path behavior, Criterion scenario quality, and benchmark-artifact reproducibility.
   - Docs lane: `docs/DESIGN.md`, `docs/DEVELOPMENT.md`, `docs/BACKLOG.md`, and any roadmap wording touched during implementation.

   Then run one fresh independent control pass on the consolidated diff. Any real findings must be fixed, committed, and revalidated before the phase is considered complete.

### Validation and Acceptance

Acceptance is complete only when all of the conditions below are true together:

- `tests/fixtures/expr/c3_positive_manifest.json` and `tests/fixtures/expr/c3_negative_manifest.json` exist and are the locked manifests for this phase.
- `tests/expression_c3.rs` uses only public `wavepeek::expr` parser, binder, evaluator, host, and result types to prove the standalone logical engine is externally usable.
- The positive manifest proves the `C3` integral-family core from `docs/expression_roadmap.md`: integral operand types and forms, integral casts, implicit common-type rules, dynamic bit-select plus constant-constrained part-select forms, concatenation, replication, operator precedence, and operator-family semantics in scope.
- Event cases in the same positive manifest prove that `iff` now accepts the `C3` integral-core logical surface while event semantics from `C2` remain correct and rich-type `C4` forms still fail deterministically.
- Unknown-flow regression tests pass for the new operators, selections, wildcard equality, conditional merge, and `inside` behavior.
- The negative manifest and snapshots prove deterministic failures for every deferred `C4` bucket and every important `C3` constant-expression or type-rule violation.
- `parse_logical_expr_ast(...)`, `bind_logical_expr_ast(...)`, and `eval_logical_expr_at(...)` exist as public `wavepeek::expr` entry points and replace the obsolete placeholder standalone `Expression` / `eval(...)` path.
- Public result values expose enough type metadata for tests to distinguish bit-vector, integer-like, and enum-core outcomes and to tell when same-enum-type identity is preserved by `?:`, without depending on internal bound-node structures.
- `change` still rejects runtime `iff` execution with the current CLI contract and does not silently switch to the typed runtime.
- `property` still reports the unimplemented runtime error.
- `bench/expr/expr_c3.rs` exists and benchmarks the typed library engine rather than the CLI binary.
- `bench/expr/scenarios/c3_integral_boolean.json` exists and matches the `expr_c3` benchmark IDs exactly.
- `bench/expr/runs/c3-integral-boolean-candidate/`, `bench/expr/runs/c3-integral-boolean-baseline/`, and `bench/expr/runs/c3-integral-boolean-verify/` all exist with committed exported `*.raw.csv`, deterministic `summary.json`, and human-readable `README.md` summaries.
- `docs/DESIGN.md` accurately reports `C3` standalone engine status, `docs/DEVELOPMENT.md` documents the `expr_c3` benchmark workflow, `docs/ROADMAP.md` no longer leaves the `C3` versus `v0.5.0` split ambiguous, and `docs/BACKLOG.md` keeps only the remaining command-integration debt open.
- `make check` and `make ci` pass after the last review-fix commit.
- Review lanes and the fresh independent control pass are clean, or all findings were fixed and rechecked.

TDD acceptance requirement: at least one `tests/expression_c3.rs` test must fail before the implementation changes and pass afterward in the same branch history.

### Idempotence and Recovery

All planned edits are additive or in-place source, test, benchmark, and documentation changes plus committed snapshot and benchmark artifacts. Re-running tests is safe. Re-running the in-memory logical and event tests is deterministic as long as the manifests are unchanged. Re-running the benchmark capture commands is safe if the destination `bench/expr/runs/c3-integral-boolean-*` directories are removed first or intentionally overwritten from the same scenario set.

If parser work destabilizes `C1` or `C2` event behavior, recover in this order: first restore the strict event parser path in `src/expr/parser.rs`, then restore offset-aware `iff` parsing through the last known green logical-parser helper, then rerun `INSTA_UPDATE=no cargo test --test expression_c1` and `INSTA_UPDATE=no cargo test --test expression_c2` before continuing. If an `insta` run leaves `.snap.new` files behind, review them, either accept them into the named `.snap` files or delete them, and rerun validation with `INSTA_UPDATE=no`.

If binder or evaluator work breaks the command boundary, the first repair step is to confirm that `src/engine/change.rs` still uses the legacy adapter and that no default command path was rerouted through the typed logical engine. Restore that split before touching standalone semantics again. If benchmark capture fails because `target/criterion` contains stale data, remove the affected `target/criterion/expr_c3*` directories and rerun the same `cargo bench --bench expr_c3 ...` command before modifying code.

If the same-commit `baseline` versus `verify` compare exceeds the tighter `5%` reproducibility guard, rerun both captures before treating the broader `15%` candidate-versus-baseline comparison as meaningful.

### Artifacts and Notes

Expected modified or added files for `C3` implementation:

- `Cargo.toml`
- `src/expr/mod.rs`
- `src/expr/ast.rs`
- `src/expr/lexer.rs`
- `src/expr/parser.rs`
- `src/expr/host.rs`
- `src/expr/sema.rs`
- `src/expr/eval.rs`
- `src/expr/diagnostic.rs`
- `src/expr/snapshots/*.snap`
- `tests/expression_c3.rs`
- `tests/expression_c2.rs`
- `tests/change_cli.rs`
- `tests/property_cli.rs`
- `tests/common/mod.rs` and possibly `tests/common/expr_runtime.rs`
- `tests/fixtures/expr/c3_positive_manifest.json`
- `tests/fixtures/expr/c3_negative_manifest.json`
- `tests/snapshots/expression_c3__*.snap`
- `bench/expr/expr_c3.rs`
- `bench/expr/scenarios/c3_integral_boolean.json`
- `bench/expr/compare.py`
- `bench/expr/test_compare.py`
- `bench/expr/runs/c3-integral-boolean-candidate/`
- `bench/expr/runs/c3-integral-boolean-baseline/`
- `bench/expr/runs/c3-integral-boolean-verify/`
- `bench/expr/AGENTS.md`
- `docs/DESIGN.md`
- `docs/DEVELOPMENT.md`
- `docs/ROADMAP.md`
- `docs/BACKLOG.md`

Before closing this plan, record these evidence excerpts here:

- Red phase (`cargo test --test expression_c3 c3_positive_manifest_matches -- --exact` before implementation): a direct logical or event `iff` case fails because the current bounded engine rejects valid `C3` syntax or lacks the public standalone API.
- Green phase (`INSTA_UPDATE=no cargo test --test expression_c3`): `test result: ok.` with all `C3` manifest and focused edge-case tests passing.
- Representative deterministic snapshot: one `tests/snapshots/expression_c3__*.snap` file showing a deferred `C4` feature or invalid constant-expression diagnostic.
- Benchmark compare gates:
  - `ok: no matched scenario exceeded 15.00% negative delta in mean or median`
  - `ok: no matched scenario exceeded 5.00% negative delta in mean or median`
- Review outcome:
  - Multi-lane review ran for code, architecture, performance, and docs.
  - Findings were fixed in follow-up commits if needed.
  - Fresh control review reported no substantive remaining issues.

### Interfaces and Dependencies

The `C3` implementation must leave these stable library-facing and internal interfaces in place.

In `src/expr/host.rs`, extend the public type model so the binder and tests can distinguish operand categories and storage shape:

    pub enum IntegerLikeKind {
        Byte,
        Shortint,
        Int,
        Longint,
        Integer,
        Time,
    }

    pub enum ExprTypeKind {
        BitVector,
        IntegerLike(IntegerLikeKind),
        EnumCore,
    }

    pub enum ExprStorage {
        PackedVector,
        Scalar,
    }

    pub struct ExprType {
        pub kind: ExprTypeKind,
        pub storage: ExprStorage,
        pub width: u32,
        pub is_four_state: bool,
        pub is_signed: bool,
        pub enum_type_id: Option<String>,
    }

Keep `SampledValue` and `ExpressionHost` library-facing. `SampledValue.bits == None` must continue to mean that no sampled state exists at or before that timestamp.

For non-enum values, `enum_type_id` must be `None`. For enum-core values, it must be a stable opaque identity string that is sufficient to distinguish “same enum type” from “different enum type” without exposing labels or richer metadata.

In `src/expr/eval.rs`, define a public standalone result type for direct logical evaluation:

    pub struct ExprValue {
        pub ty: ExprType,
        pub bits: String,
    }

`ExprValue.bits` must be normalized MSB-first text using only `0`, `1`, `x`, and `z`, so tests and benchmarks can assert exact outcomes without depending on private helper types.

In `src/expr/mod.rs`, expose the standalone logical API alongside the existing event API:

    pub use crate::expr::ast::{EventExprAst, LogicalExprAst};
    pub use crate::expr::eval::ExprValue;
    pub use crate::expr::sema::{BoundEventExpr, BoundLogicalExpr};

    pub fn parse_logical_expr_ast(source: &str) -> Result<LogicalExprAst, ExprDiagnostic>;

    pub fn bind_logical_expr_ast(
        ast: &LogicalExprAst,
        host: &dyn ExpressionHost,
    ) -> Result<BoundLogicalExpr, ExprDiagnostic>;

    pub fn eval_logical_expr_at(
        expr: &BoundLogicalExpr,
        host: &dyn ExpressionHost,
        timestamp: u64,
    ) -> Result<ExprValue, ExprDiagnostic>;

Keep these event entry points public and phase-correct:

    pub fn bind_event_expr_ast(
        ast: &EventExprAst,
        host: &dyn ExpressionHost,
    ) -> Result<BoundEventExpr, ExprDiagnostic>;

    pub fn event_matches_at(
        expr: &BoundEventExpr,
        host: &dyn ExpressionHost,
        frame: &EventEvalFrame<'_>,
    ) -> Result<bool, ExprDiagnostic>;

`EventEvalFrame` must remain the public event-evaluation input type from `wavepeek::expr`. Tests should construct it directly with the current probe timestamp, the previous probe timestamp, and the tracked signal handles slice used for wildcard matching.

`BoundLogicalExpr` should be a public opaque handle. The public API should not expose internal bound-node enums or internal temporary value helpers.

In `src/expr/parser.rs`, keep an internal helper for event `iff` reuse so diagnostics point at the original event source spans:

    pub(crate) fn parse_logical_expr_with_offset(
        source: &str,
        source_offset: usize,
    ) -> Result<LogicalExprAst, ExprDiagnostic>;

In `Cargo.toml`, add the new benchmark target while preserving the current Criterion workflow:

    [[bench]]
    name = "expr_c3"
    harness = false

Revision note (2026-03-12 / OpenCode): initial `C3` ExecPlan authored from `docs/expression_roadmap.md`, current `C2` engine state, and existing expression test and benchmark conventions; revised after plan review to clarify dynamic bit-select scope, enum-type identity, `EventEvalFrame` usage, benchmark provenance checks, and roadmap collateral updates.
