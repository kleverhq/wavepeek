# C2 Event Runtime Core and Bounded `iff` Logical Subset

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek` will have a real typed runtime for Section `1` event expressions and the bounded `iff` predicate subset required by `docs/expression_roadmap.md` `C2`. A contributor will be able to bind and evaluate `*`, named events, `posedge`/`negedge`/`edge`, unions via `or` and `,`, and `iff` predicates that use operand references, integral literals, parentheses, `!`, `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, and `||`, with deterministic four-state behavior and short-circuit semantics.

The result is observable without waiting for command rollout. New manifest-driven runtime tests will exercise the typed engine directly through the public `wavepeek::expr` event-runtime surface, using locked positive and negative fixtures plus `insta` diagnostics snapshots. A dedicated `Criterion` benchmark target will measure event binding and evaluation scenarios and export committed baseline artifacts under `bench/expr/runs/`. The `change` and `property` commands remain phase-correct: `change` stays on the legacy runtime path by default, and `property` remains unimplemented until later phases.

This plan targets exactly one roadmap boundary: `C2` from `docs/expression_roadmap.md`. It maps to the internal expression-engine portion of `docs/ROADMAP.md` `Query Engine (-> v0.4.0)`, but not to public command completion. Public `property` delivery and default command convergence remain `C5` work.

## Non-Goals

This plan does not implement full Section `2` Boolean Expression support. Anything outside the bounded `C2` subset remains intentionally unsupported, including bitwise, reduction, arithmetic, shift, wildcard or case equality, conditional, `inside`, casts, selections, concatenation, replication, `real`, `string`, enum-label introspection, and `.triggered`. This plan does not make `property` execute end to end, does not switch `change` to the typed runtime by default, does not change JSON schema or public command help beyond implementation-status wording, and does not collapse `C3`, `C4`, or `C5` scope into `C2`.

## Progress

- [x] (2026-03-11 09:12Z) Reviewed `docs/expression_roadmap.md`, `docs/expression_lang.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, current `src/expr/` and `src/engine/change.rs`, and prior ExecPlan conventions.
- [x] (2026-03-11 09:24Z) Drafted the active `C2` ExecPlan with explicit scope clauses, runtime/test strategy, benchmark artifact requirements, and roadmap alignment.
- [x] (2026-03-11 09:35Z) Completed focused docs and architecture plan review, revised roadmap wording, progress tracking, host-contract wording, parity strategy, and acceptance criteria, then closed with a fresh clean control pass.
- [x] (2026-03-11 12:05Z) Implemented Milestone 1: added locked `C2` runtime manifests, snapshots, and red-to-green runtime proof in `tests/expression_c2.rs`.
- [x] (2026-03-11 12:44Z) Implemented Milestone 2: shipped bounded logical binder/evaluator and public typed event-runtime surface in `src/expr/`.
- [x] (2026-03-11 13:02Z) Implemented Milestone 3: added short-circuit/unknown-flow coverage and non-`iff` parity guard while keeping default `change`/`property` behavior deferred.
- [x] (2026-03-11 13:28Z) Implemented Milestone 4: generalized benchmark workflow, added `expr_c2` scenarios, and captured `c2-event-runtime-*` run artifacts.
- [x] (2026-03-11 15:31Z) Implemented Milestone 5: completed validation (`make check`, `make ci`), multi-lane review, control pass, and committed follow-up fixes.

## Surprises & Discoveries

- Observation: the current semantic truth for event runtime behavior lives in `src/engine/change.rs`, not in the typed `src/expr/` engine.
  Evidence: `src/expr/sema.rs` resolves event names only, `src/expr/eval.rs` still returns `C1-RUNTIME-LOGICAL-NOT-IMPLEMENTED`, and `src/engine/change.rs` already implements named and edge event matching plus candidate scheduling.

- Observation: the current `ExpressionHost` abstraction cannot represent "no sampled value exists" because `SampledValue` always carries a concrete bit string.
  Evidence: `src/expr/host.rs` defines `pub struct SampledValue { pub bits: String }`, while `docs/expression_lang.md` requires "no previous sampled value means no edge" and practical named-event change detection also needs a distinguishable missing prior sample state.

- Observation: the expression benchmark helper flow is phase-specific today.
  Evidence: `bench/expr/capture.py` hard-codes the `expr_c1` scenario set (`tokenize_union_iff`, `parse_event_union_iff`, `parse_event_malformed`) and `docs/DEVELOPMENT.md` documents only `bench/expr/expr_c1.rs`.

- Observation: `property` remains a safe regression guard for this phase.
  Evidence: `src/engine/property.rs` still returns `error: unimplemented: \`property\` command execution is not implemented yet`, so `C2` can deliver the standalone engine without forcing command-output rollout.

## Decision Log

- Decision: keep `C2` standalone and observable through the typed expression API, tests, and benchmarks, but do not route the default `change` or `property` commands through that runtime yet.
  Rationale: `docs/expression_roadmap.md` makes command integration a `C5` responsibility, and changing default command paths in `C2` would blur the acceptance boundary.
  Date/Author: 2026-03-11 / OpenCode

- Decision: expose a narrow public event-runtime surface in `wavepeek::expr` (`bind_event_expr_ast(...)`, `event_matches_at(...)`, host/context types, and an opaque bound-event handle), while keeping the bounded logical parser and bound logical AST internal.
  Rationale: integration tests and `Criterion` benches need a reusable library entry point, but exposing a partial standalone Boolean-expression parser in `C2` would imply more Section `2` surface than the phase actually owns.
  Date/Author: 2026-03-11 / OpenCode

- Decision: preserve `EventTermAst.iff` as a source-plus-span capture at parse time and parse the bounded logical subset during binding, using span-offset-aware diagnostics.
  Rationale: this keeps `C1` event parsing strictness intact, avoids widening the public parser contract prematurely, and still lets `C2` produce deterministic parse or semantic failures for unsupported `iff` payloads.
  Date/Author: 2026-03-11 / OpenCode

- Decision: generalize the existing benchmark capture helpers to phase-specific scenario manifests instead of duplicating `c2`-only scripts.
  Rationale: `C1` and `C2` artifacts must coexist under `bench/expr/runs/`, and a shared capture/compare workflow keeps later expression phases from multiplying one-off benchmark tooling.
  Date/Author: 2026-03-11 / OpenCode

- Decision: update `docs/ROADMAP.md` in the same planning change to state that `C2` through `C4` are standalone engine milestones and that public command rollout remains `C5`.
  Rationale: `docs/expression_roadmap.md` requires each phase plan to state roadmap mapping, and the roadmap text is currently too easy to read as one lump delivery of engine work plus command integration.
  Date/Author: 2026-03-11 / OpenCode

## Outcomes & Retrospective

Current status: `C2` implementation complete and validated on this branch.

Delivered outcomes:

- Standalone typed `C2` event runtime with bounded `iff` subset is live in `src/expr/` through `bind_event_expr_ast(...)` and `event_matches_at(...)`.
- Default command behavior stayed phase-correct: `change` still rejects runtime `iff` and `property` remains unimplemented.
- Deterministic manifests/snapshots and benchmark artifacts are committed for `c2-event-runtime-candidate`, `c2-event-runtime-baseline`, and `c2-event-runtime-verify`.
- Post-review fixes addressed mixed-sign extension correctness, decimal overflow guardrails, evaluator sample dedupe, benchmark-host determinism, and capture output-dir safety.

Residual risk:

- Command integration remains deferred to later phases (`C5`), so semantic drift risk between legacy and typed runtime still exists; this is bounded by parity tests and explicit backlog tracking.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI with a thin library surface in `src/lib.rs`. Expression work is concentrated in `src/expr/`. The module now exposes typed parser plus standalone event-runtime surface (`lex_event_expr(...)`, `parse_event_expr_ast(...)`, `bind_event_expr_ast(...)`, `event_matches_at(...)`) with deterministic diagnostics. It still keeps crate-private legacy compatibility wrappers (`parse(...)`, `parse_event_expr(...)`) used by the current `change` runtime.

At plan start, the `C1` typed parser was intentionally incomplete for runtime work. `src/expr/ast.rs` stored `iff` payloads as `DeferredLogicalExpr { source, span }`, while `src/expr/sema.rs` and `src/expr/eval.rs` had no bounded logical binding/evaluation path. `C2` implementation closed this gap while preserving deferred command integration.

The production behavior that users see today still lives in command runtimes. `src/engine/change.rs` continues to parse `--on` through the legacy adapter in `src/expr/legacy.rs` and still hard-fails runtime `iff` terms with `error: args: iff logical expressions are not implemented yet`. `src/engine/property.rs` remains a runtime stub. This split remains intentional in `C2`; command integration is deferred to `C5`.

The contract for this phase comes from two documents. `docs/expression_lang.md` is the source of truth for syntax and semantics; for `C2`, the in-scope clauses are Section `1.1` through `1.6` event forms and runtime behavior plus a bounded subset of Section `2` used only inside `iff`: operand references, integral literals, parentheses, `!`, `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, and `||`. `docs/expression_roadmap.md` defines the rollout boundary: `C2` implements event runtime and the bounded `iff` subset, while any other Section `2` surface, rich types, and command integration must still fail deterministically.

Several terms in this plan have repository-specific meanings. A "tracked set" is the command-defined set of signals that wildcard `*` watches; in `change` it comes from resolved `--signals`, and in later `property` work it will come from `--eval` references. A "probe timestamp" is a concrete raw waveform time at which the typed event engine is asked whether an event matches; `C2` runtime tests drive the engine with explicit probe timestamps rather than with full command integration. A "four-state truth value" is the internal `0`/`1`/`x` logical result used by SystemVerilog-like matching rules; `C2` still reduces the final `iff` decision to a two-state event gate where only `1` passes and `0` or `x` suppresses the event. A "shadow parity" test is a test-only comparison between the new typed runtime and the current legacy `change` runtime on scenarios that both support, without changing the default CLI path.

The current repository already contains useful infrastructure that `C2` must reuse rather than replace. `tests/expression_c1.rs` plus `tests/fixtures/expr/c1_positive_manifest.json`, `tests/fixtures/expr/c1_negative_manifest.json`, `tests/snapshots/`, and `src/expr/snapshots/` define the manifest-plus-snapshot pattern for deterministic expression phases. `bench/expr/expr_c1.rs`, `bench/expr/capture.py`, `bench/expr/compare.py`, and the committed `bench/expr/runs/c1-foundation-*` directories define the repository-standard parser microbenchmark workflow. `C2` must extend those patterns to runtime evaluation instead of inventing parallel conventions.

## Open Questions

There are no blocking product questions left for `C2`. The implementation-shape questions that matter here are resolved by the `Decision Log`: keep the logical parser internal, expose only the event-runtime API publicly, preserve default command behavior, and generalize the benchmark tooling instead of forking it.

If an implementer later discovers that typed event matching cannot be expressed cleanly with the existing host interface, the fallback is to widen `src/expr/host.rs` in a typed, library-facing way and update the plan plus `docs/DESIGN.md` in the same change. Do not respond by wiring hidden CLI-only escape hatches or by silently routing default `change` behavior through the typed runtime.

## Plan of Work

Milestone 1 locks the `C2` boundary with failing tests before runtime code changes land. Add one locked positive manifest and one locked negative manifest under `tests/fixtures/expr/`, then add a new integration-style runtime test file that uses only public `wavepeek::expr` APIs plus a local in-memory host. The positive manifest must prove actual event-runtime behavior, not just parsing: wildcard `*`, named events, edge terms, union via `or` and `,`, same-timestamp deduplication, `iff` binding to the immediately preceding event term, bounded logical operators, integral literals, and the final two-state gating of four-state results. The negative manifest must lock deterministic failures for every still-unsupported `C2` bucket: out-of-scope operators, out-of-scope literal and type domains, out-of-scope primary forms, unsupported event-parenthesis grouping, and invalid name resolution inside `iff`.

Milestone 2 adds the typed runtime itself. Extend `src/expr/ast.rs`, `src/expr/sema.rs`, `src/expr/eval.rs`, `src/expr/host.rs`, `src/expr/parser.rs`, and `src/expr/mod.rs` so a caller can parse an event expression, bind it against an `ExpressionHost`, and evaluate whether it matches at a specific probe timestamp. Keep the event parser strict and phase-correct: `iff` remains source-plus-span data in the public AST, while binding invokes a bounded logical parser internally and either returns a bound predicate or produces a deterministic `ExprDiagnostic`. Unsupported syntax should fail at the earliest deterministic layer. Unsupported token or grammar forms in the bounded subset should become parse diagnostics with exact spans inside the full `--on` source. Supported syntax that resolves to unsupported type domains or impossible operations in `C2` should become semantic diagnostics.

Milestone 3 proves that the new runtime behaves like the contract and does not accidentally roll out command behavior. Add targeted short-circuit and unknown-flow tests in `tests/expression_c2.rs` or nearby internal unit modules, then add one small test-only parity guard against the legacy event-matching logic for non-`iff` event forms on an existing hand fixture such as `tests/fixtures/hand/change_edge_cases.vcd`. That parity guard must compare raw event-match timestamps before snapshot delta filtering or output formatting, because CLI snapshot emission in `change` has extra delta rules beyond event semantics. The parity guard is not the acceptance target for `C2`; it is only a drift guard while command convergence stays deferred. The default `change` CLI tests for deferred `iff` failure and the `property` unimplemented path must remain green.

Milestone 4 extends the benchmark and collateral workflow. Add a new `Criterion` target for `C2` runtime scenarios, generalize the benchmark capture helpers to support per-phase scenario manifests, export committed candidate, baseline, and verify run directories for `C2`, and update status documents. `docs/DESIGN.md` must say that the typed expression engine now owns standalone event runtime and bounded `iff` evaluation while command integration remains pending. `docs/BACKLOG.md` must keep the command-integration debts open and, if necessary, reword them so they no longer claim that the reusable evaluator itself is wholly absent. `docs/DEVELOPMENT.md` and `bench/expr/AGENTS.md` must describe the generalized runtime benchmark flow.

Milestone 5 is validation and review closure. Run the new runtime suites, existing regression suites, benchmark compares, and repository gates. Then run mandatory focused review lanes for code, architecture, performance, and docs, followed by one fresh independent control pass on the consolidated diff. Any follow-up fixes must be committed separately and revalidated after the last review-fix commit.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

0. Keep the roadmap mapping clear before implementation work begins.

   The roadmap note in `docs/ROADMAP.md` should already state that `C2` through `C4` are standalone expression-engine milestones and that public command rollout stays in `C5`. If implementation discoveries force a different milestone mapping, update `docs/ROADMAP.md` and the `Decision Log` in this plan in the same commit.

1. Lock `C2` behavior with failing tests and fixtures first.

   Add these new files:

   - `tests/fixtures/expr/c2_positive_manifest.json`
   - `tests/fixtures/expr/c2_negative_manifest.json`
   - `tests/expression_c2.rs`
   - `tests/snapshots/expression_c2__*.snap`

   If the in-memory runtime host grows beyond a few helper structs, move it into `tests/common/expr_runtime.rs` and update `tests/common/mod.rs` to re-export it. Keep the helper deterministic and local to tests; do not couple it to CLI argument parsing.

   The positive manifest should drive probe-based runtime checks through public typed APIs only. Each case should specify, at minimum, the event expression source, the tracked-set signal names, an in-memory signal timeline, the probe timestamps to check, and the timestamps that must match. Use at least these locked positive buckets:

   - wildcard `*` on tracked signals with same-timestamp deduplication;
   - named-event change detection with "no prior sample means no match" at the first probe;
   - `posedge`, `negedge`, and `edge` using least-significant-bit classification and `x`/`z` normalization;
   - `iff` binding to one preceding event term in a mixed union;
   - bounded `iff` predicates that use operand references, decimal and based integral literals, `!`, comparisons, `&&`, `||`, and parentheses;
   - four-state cases where an intermediate `x` result suppresses the final event even though the base event fires.

   The negative manifest should assert the deterministic failure path after parsing and before runtime matching. Each case should specify source text, expected diagnostic layer, code, span, and optional snapshot name. Include at least one locked negative case for each still-out-of-scope bucket:

   - arithmetic, bitwise, reduction, shift, wildcard equality, case equality, conditional, and `inside` operators;
   - `real` literals, `string` literals, enum-label forms, and `.triggered`;
   - casts, selections, concatenation, and replication;
   - unresolved signal names inside `iff`;
   - standalone event grouping such as `(posedge clk) or ready`.

   Add targeted short-circuit tests alongside the manifests. Binding still resolves signal names eagerly in `C2`, so use a host where the right-hand signal resolves successfully but `sample_value(...)` traps if evaluation touches it unexpectedly. At minimum prove these behaviors:

   - `0 && rhs_sig` yields `0` without sampling `rhs_sig`;
   - `1 || rhs_sig` yields `1` without sampling `rhs_sig`;
   - `x && 0` yields `0`;
   - `x || 1` yields `1`;
   - `x && 1` yields `x`, which suppresses the event match.

   Red-phase commands:

       cargo test --test expression_c2 c2_positive_manifest_matches -- --exact
       INSTA_UPDATE=no cargo test --test expression_c2 c2_negative_manifest_matches_snapshots -- --exact
       cargo test --test expression_c2 c2_short_circuit_subset_holds -- --exact

   Expected red evidence before the runtime implementation exists: at least one test fails because `bind_event_expr_ast(...)` or `event_matches_at(...)` is missing or still returns a not-implemented diagnostic.

2. Add the typed `C2` binding and evaluation surface.

   Update these source files:

   - `src/expr/mod.rs`
   - `src/expr/ast.rs`
   - `src/expr/parser.rs`
   - `src/expr/sema.rs`
   - `src/expr/eval.rs`
   - `src/expr/host.rs`
   - `src/expr/diagnostic.rs` (only if new `C2` codes or helper constructors make diagnostics clearer)

   Add a narrow public event-runtime surface in `src/expr/mod.rs`. The library-facing entry points must be enough for tests and benches to use the typed engine without exposing a partial standalone Boolean-expression API. The intended end state is:

       pub use crate::expr::host::{EventEvalFrame, ExprType, ExpressionHost, SampledValue, SignalHandle};
       pub use crate::expr::sema::BoundEventExpr;

       pub fn bind_event_expr_ast(
           ast: &EventExprAst,
           host: &dyn ExpressionHost,
       ) -> Result<BoundEventExpr, ExprDiagnostic>;

       pub fn event_matches_at(
           expr: &BoundEventExpr,
           host: &dyn ExpressionHost,
           frame: &EventEvalFrame<'_>,
       ) -> Result<bool, ExprDiagnostic>;

    Keep the logical parser itself crate-private. `src/expr/parser.rs` should gain an internal bounded-subset parser that consumes `DeferredLogicalExpr.source`, applies the original source-span offset, and supports exactly the `C2` subset. `src/expr/ast.rs` should add the internal logical AST and integral-literal representation needed to preserve operator precedence and spans. `src/expr/sema.rs` should keep `BoundEventExpr` public but opaque, move the internal `iff` storage from `Option<DeferredLogicalExpr>` to `Option<BoundLogicalExpr>`, and make `bind_event_expr(...)` invoke the new bounded logical binder.

   `src/expr/host.rs` must be widened just enough to model runtime sampling correctly. Introduce `EventEvalFrame<'a>` with `timestamp`, `previous_timestamp`, and `tracked_signals`. Change `SampledValue` so it can represent missing sampled state explicitly, for example `pub bits: Option<String>`, and document that `ExpressionHost::sample_value(handle, timestamp)` returns `Result<SampledValue, ExprDiagnostic>`, where `SampledValue.bits == None` means no sampled value exists at or before that timestamp. Keep `ExprType` width, sign, and four-state metadata because `C2` comparisons and truthiness need them.

   `src/expr/eval.rs` should implement both bounded logical evaluation and event-term matching. `TruthValue` stays the internal result type. `event_matches_at(...)` should evaluate union as OR over terms, with `iff` applying only to the immediately bound term, and should use `frame.previous_timestamp` when comparing named or edge events so no prior sample yields no match. Final event gating remains two-state: only `TruthValue::One` passes `iff`.

   Use deterministic `C2-*` diagnostic codes. Prefer parse-layer codes for unsupported tokens or malformed bounded-subset grammar, semantic-layer codes for unsupported-but-parseable domains or unresolved names, and runtime-layer codes only for sampling failures that a host reports at evaluation time.

   Targeted green-phase commands after the runtime exists:

       cargo test --test expression_c2
       cargo test expr::sema::tests
       cargo test expr::eval::tests

3. Add test-only parity guards without changing default command behavior.

   Keep `src/engine/change.rs` on the legacy runtime path in `C2`. Do not route normal `wavepeek change` execution through `bind_event_expr_ast(...)` or `event_matches_at(...)`. Instead, add a small parity guard that compares typed-engine raw event-match timestamps with a test-only legacy helper for event forms both paths already support without `iff`. Reuse an existing hand fixture such as `tests/fixtures/hand/change_edge_cases.vcd`, and compare the event-match layer before requested-signal delta emission, engine-mode dispatch, or CLI output formatting.

   Update or add regression tests so these contracts remain explicit:

   - `tests/change_cli.rs` still proves that `change --on "... iff ..."` fails with `error: args: iff logical expressions are not implemented yet`.
   - `tests/property_cli.rs` still proves that `property` returns the unimplemented error.
   - The new parity guard does not change public CLI output, exit codes, or schema artifacts.
   - If the legacy helper can observe multiple engine modes, it must prove the same raw event-match timestamps across those modes before comparing with the typed engine.

   Useful targeted commands:

       cargo test --test change_cli change_iff_is_recognized_but_runtime_is_deferred -- --exact
       cargo test --test property_cli
        cargo test --test expression_c2 c2_shadow_parity_matches_legacy_event_matches_for_non_iff_subset -- --exact

4. Extend the benchmark tooling and capture `C2` artifacts.

   Add these new files and updates:

   - `bench/expr/expr_c2.rs`
   - `bench/expr/scenarios/c1_parser.json`
   - `bench/expr/scenarios/c2_event_runtime.json`
   - `bench/expr/runs/c2-event-runtime-candidate/README.md`
   - `bench/expr/runs/c2-event-runtime-baseline/README.md`
   - `bench/expr/runs/c2-event-runtime-verify/README.md`
   - updates to `bench/expr/capture.py`
   - updates to `bench/expr/compare.py`
   - updates to `bench/expr/test_capture.py`
   - updates to `bench/expr/test_compare.py`
   - updates to `Cargo.toml`, `docs/DEVELOPMENT.md`, `bench/expr/AGENTS.md`, and `bench/expr/runs/AGENTS.md`

   Generalize `bench/expr/capture.py` so it accepts a committed scenario-manifest file and the bench target name instead of hard-coding `expr_c1`. Keep the current stability rules: read only Criterion `raw.csv`, validate exact scenario-set equality, export deterministic `summary.json`, and write a readable `README.md`. The exported summary should also record the bench target and scenario-manifest identity so `compare.py` can reject wrong-phase comparisons before it evaluates timing deltas. Update `bench/expr/compare.py` and its tests so it validates scenario-set equality from exported summaries and continues to reject missing, extra, duplicate, or mismatched scenarios.

   `bench/expr/expr_c2.rs` should benchmark only the typed library surface, not the CLI binary. Reuse the fixed Criterion profile from `C1` (`sample_size = 100`, `warm_up_time = 3s`, `measurement_time = 5s`, `significance_level = 0.05`, `noise_threshold = 0.01`, `configure_from_args()`). Define exactly these benchmark IDs in the `c2_event_runtime` scenario manifest and in the bench target:

   - `bind_event_union_iff`
   - `eval_event_union_iff_true`
   - `eval_event_union_iff_unknown`

   Candidate run (first fully green pre-review implementation commit):

       cargo bench --bench expr_c2 -- --save-baseline c2-event-runtime-candidate --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c2-event-runtime-candidate --bench-target expr_c2 --scenario-set bench/expr/scenarios/c2_event_runtime.json --output bench/expr/runs/c2-event-runtime-candidate --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Final accepted baseline run (after the last review-fix commit):

       cargo bench --bench expr_c2 -- --save-baseline c2-event-runtime-baseline --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c2-event-runtime-baseline --bench-target expr_c2 --scenario-set bench/expr/scenarios/c2_event_runtime.json --output bench/expr/runs/c2-event-runtime-baseline --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Same-commit verify run:

       cargo bench --bench expr_c2 -- --save-baseline c2-event-runtime-verify --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c2-event-runtime-verify --bench-target expr_c2 --scenario-set bench/expr/scenarios/c2_event_runtime.json --output bench/expr/runs/c2-event-runtime-verify --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Then compare:

       python3 bench/expr/compare.py --revised bench/expr/runs/c2-event-runtime-baseline --golden bench/expr/runs/c2-event-runtime-candidate --max-negative-delta-pct 15
       python3 bench/expr/compare.py --revised bench/expr/runs/c2-event-runtime-verify --golden bench/expr/runs/c2-event-runtime-baseline --max-negative-delta-pct 5

5. Commit atomic units and close with review.

   Suggested commit split:

       git add tests/fixtures/expr/c2_positive_manifest.json tests/fixtures/expr/c2_negative_manifest.json tests/expression_c2.rs tests/snapshots
       git commit -m "test(expr): lock c2 runtime manifests"

       git add src/expr/mod.rs src/expr/ast.rs src/expr/parser.rs src/expr/sema.rs src/expr/eval.rs src/expr/host.rs src/expr/diagnostic.rs src/expr/snapshots
       git commit -m "feat(expr): add c2 event runtime and iff subset"

       git add Cargo.toml bench/expr/expr_c2.rs bench/expr/scenarios bench/expr/capture.py bench/expr/compare.py bench/expr/test_capture.py bench/expr/test_compare.py docs/DEVELOPMENT.md bench/expr/AGENTS.md bench/expr/runs/AGENTS.md docs/DESIGN.md docs/BACKLOG.md
       git commit -m "bench(expr): add c2 runtime benchmark flow"

       git add bench/expr/runs/c2-event-runtime-candidate
       git commit -m "bench(expr): capture c2 candidate baseline"

       git add bench/expr/runs/c2-event-runtime-baseline bench/expr/runs/c2-event-runtime-verify
       git commit -m "bench(expr): capture final c2 baselines"

   If review finds problems, fix them in separate follow-up commits. Do not amend history.

6. Run validation gates.

   Focused gates first:

       INSTA_UPDATE=no cargo test --test expression_c2
       cargo test --test change_cli
       cargo test --test property_cli
       cargo test --bench expr_c2
       python3 -m unittest discover -s bench/expr -p 'test_*.py'
       python3 bench/expr/compare.py --revised bench/expr/runs/c2-event-runtime-baseline --golden bench/expr/runs/c2-event-runtime-candidate --max-negative-delta-pct 15
       python3 bench/expr/compare.py --revised bench/expr/runs/c2-event-runtime-verify --golden bench/expr/runs/c2-event-runtime-baseline --max-negative-delta-pct 5

   Then repository gates:

       make check
       make ci

   Expected success signature after implementation is complete:

       INSTA_UPDATE=no cargo test --test expression_c2
       ...
       test result: ok.
       python3 bench/expr/compare.py --revised ... --golden ... --max-negative-delta-pct 15
       ok: no matched scenario exceeded 15.00% negative delta in mean or median
       make ci
       ...
       test result: ok.

7. Run the mandatory review workflow.

   Load `ask-review` skill and run these focused lanes in parallel after the green validation above:

   - Code lane: event semantics, four-state logic, short-circuit correctness, unsupported-feature coverage, and missing tests.
   - Architecture lane: public typed runtime surface, layering between `src/expr/` and `src/engine/change.rs`, and command-integration boundary discipline.
   - Performance lane: event-evaluation complexity, host-sampling strategy, Criterion scenario quality, and capture/compare reproducibility.
   - Docs lane: `docs/DESIGN.md`, `docs/DEVELOPMENT.md`, `docs/BACKLOG.md`, and any roadmap wording touched during the implementation.

   Then run one fresh independent control pass on the consolidated diff. Any real findings must be fixed, committed, and revalidated before the phase is considered complete.

### Validation and Acceptance

Acceptance is complete only when all of the conditions below are true together:

- `tests/fixtures/expr/c2_positive_manifest.json` and `tests/fixtures/expr/c2_negative_manifest.json` exist and are the locked manifests for this phase.
- `tests/expression_c2.rs` uses only public `wavepeek::expr` parser, binder, event-runtime, and host/context types to prove the typed engine is externally usable as a standalone library surface.
- The positive manifest proves Section `1` runtime semantics for `*`, named, edge, union, `iff` binding, and same-timestamp union deduplication.
- The bounded `iff` subset works for operand references, integral literals, parentheses, `!`, `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, and `||`.
- Short-circuit tests prove that dead branches are not resolved or sampled.
- Four-state tests prove that `x` propagates through the bounded subset in the expected cases and only `TruthValue::One` allows an `iff`-gated event to match.
- No prior sampled value yields no named-event or edge-event match at that probe timestamp.
- The negative manifest and snapshots prove deterministic failures for every out-of-scope `C2` bucket listed in `docs/expression_roadmap.md`.
- Unsupported event grouping such as `(posedge clk) or ready` fails deterministically and does not silently become Boolean parentheses.
- Unresolved event-term names and unresolved names inside `iff` fail deterministically with semantic diagnostics and exact spans.
- `change` still rejects `iff` at runtime with the existing CLI error and does not silently switch to the typed runtime.
- `property` still reports the unimplemented runtime error.
- Any parity wiring added for drift detection is test-only and does not alter default CLI output, exit codes, or schema artifacts.
- `docs/ROADMAP.md` remains unambiguous that `C2` through `C4` are standalone engine milestones and that shared typed-engine convergence for `change` plus end-to-end `property` runtime remain gated on `C5`.
- `bench/expr/expr_c2.rs` exists and benchmarks the typed library runtime rather than the CLI binary.
- `bench/expr/capture.py` and `bench/expr/compare.py` support both `C1` and `C2` scenario manifests, and their unit tests cover wrong-scenario and wrong-baseline failures.
- `bench/expr/runs/c2-event-runtime-candidate/`, `bench/expr/runs/c2-event-runtime-baseline/`, and `bench/expr/runs/c2-event-runtime-verify/` all exist with committed exported `*.raw.csv`, deterministic `summary.json`, and human-readable `README.md` summaries.
- `docs/DESIGN.md` accurately reports standalone `C2` runtime status, and `docs/BACKLOG.md` keeps only the remaining command-integration debt open.
- `make check` and `make ci` pass after the last review-fix commit.
- Review lanes and the fresh independent control pass are clean, or all findings were fixed and rechecked.

TDD acceptance requirement: at least one `tests/expression_c2.rs` runtime test must fail before the implementation changes and pass afterward in the same branch history.

### Idempotence and Recovery

All planned edits are additive or in-place source and documentation changes plus committed snapshot and benchmark artifacts. Re-running tests is safe. Re-running the in-memory runtime tests is deterministic as long as the manifests are unchanged. Re-running the benchmark capture commands is safe if the destination `bench/expr/runs/c2-event-runtime-*` directories are removed first or intentionally overwritten from the same scenario set.

If a logical-parser change destabilizes existing `C1` event parsing, recover by reverting the logical-parser call sites inside `src/expr/sema.rs` first, then rerun `cargo test --test expression_c1` and `cargo test --test expression_c2` before continuing. If an `insta` run leaves `.snap.new` files behind, review them, either accept them into the named `.snap` files or delete them, and rerun validation with `INSTA_UPDATE=no`.

If command regression tests begin failing during `C2`, the first repair step is to confirm that `src/engine/change.rs` still calls the legacy adapter and that no default CLI path was rerouted through the typed runtime. Restore that boundary before touching the typed engine again. If benchmark export fails because the scenario-manifest generalization broke `C1`, fix the helper scripts so both `bench/expr/scenarios/c1_parser.json` and `bench/expr/scenarios/c2_event_runtime.json` pass their unit tests before capturing any new artifacts.

If the same-commit `baseline` versus `verify` compare exceeds the tighter `5%` reproducibility guard, rerun both captures before treating the broader `15%` candidate-versus-baseline comparison as meaningful.

### Artifacts and Notes

Expected modified or added files for `C2` implementation:

- `Cargo.toml`
- `src/expr/mod.rs`
- `src/expr/ast.rs`
- `src/expr/parser.rs`
- `src/expr/sema.rs`
- `src/expr/eval.rs`
- `src/expr/host.rs`
- `src/expr/diagnostic.rs`
- `src/expr/snapshots/*.snap`
- `tests/expression_c2.rs`
- `tests/common/mod.rs` and possibly `tests/common/expr_runtime.rs`
- `tests/fixtures/expr/c2_positive_manifest.json`
- `tests/fixtures/expr/c2_negative_manifest.json`
- `tests/snapshots/expression_c2__*.snap`
- `tests/change_cli.rs`
- `tests/property_cli.rs`
- `bench/expr/expr_c2.rs`
- `bench/expr/scenarios/c1_parser.json`
- `bench/expr/scenarios/c2_event_runtime.json`
- `bench/expr/capture.py`
- `bench/expr/compare.py`
- `bench/expr/test_capture.py`
- `bench/expr/test_compare.py`
- `bench/expr/runs/c2-event-runtime-candidate/`
- `bench/expr/runs/c2-event-runtime-baseline/`
- `bench/expr/runs/c2-event-runtime-verify/`
- `bench/expr/AGENTS.md`
- `bench/expr/runs/AGENTS.md`
- `docs/DESIGN.md`
- `docs/DEVELOPMENT.md`
- `docs/BACKLOG.md`

Evidence snippets recorded during execution:

- Green phase (`INSTA_UPDATE=no cargo test --test expression_c2`): `test result: ok. 4 passed; 0 failed`.
- Deferred command guards: `cargo test --test change_cli change_iff_is_recognized_but_runtime_is_deferred -- --exact` and `cargo test --test property_cli` both passed.
- Representative unsupported-feature diagnostic: `tests/snapshots/expression_c2__c2_parse_unsupported_inside.snap` (`C2-PARSE-LOGICAL-UNSUPPORTED`).
- Benchmark compare gates:
  - `ok: no matched scenario exceeded 15.00% negative delta in mean or median`
  - `ok: no matched scenario exceeded 5.00% negative delta in mean or median`
- Review outcome:
  - Multi-lane review ran for code, architecture, performance, and docs.
  - Findings were fixed in follow-up implementation/doc changes.
  - Fresh control re-review reported clean for code/performance, with docs clean after plan-status updates.

### Interfaces and Dependencies

The `C2` implementation must leave these stable library-facing and internal interfaces in place.

In `src/expr/host.rs`, define:

    pub struct SampledValue {
        pub bits: Option<String>,
    }

    pub struct EventEvalFrame<'a> {
        pub timestamp: u64,
        pub previous_timestamp: Option<u64>,
        pub tracked_signals: &'a [SignalHandle],
    }

    pub trait ExpressionHost {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic>;
        fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic>;
        fn sample_value(
            &self,
            handle: SignalHandle,
            timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic>;
    }

The meaning of `sample_value(...)` must be documented in code comments or nearby tests: it returns `Result<SampledValue, ExprDiagnostic>`, and `SampledValue.bits == None` means no sampled state exists at or before that timestamp.

In `src/expr/sema.rs`, define or keep these event-runtime types:

    pub struct BoundEventExpr {
        ...
    }

    pub(crate) struct BoundEventTerm {
        pub event: BoundEventKind,
        pub iff: Option<BoundLogicalExpr>,
    }

    pub(crate) enum BoundEventKind {
        AnyTracked,
        Named(SignalHandle),
        Posedge(SignalHandle),
        Negedge(SignalHandle),
        Edge(SignalHandle),
    }

    pub(crate) struct BoundLogicalExpr {
        ...
    }

    pub fn bind_event_expr_ast(
        ast: &EventExprAst,
        host: &dyn ExpressionHost,
    ) -> Result<BoundEventExpr, ExprDiagnostic>;

`BoundEventExpr` should be a public opaque handle that tests and benches can pass into `event_matches_at(...)` without inspecting the internal logical representation. `BoundLogicalExpr` should stay internal to avoid promising a partial standalone Boolean-expression API before `C4`.

In `src/expr/eval.rs`, keep:

    pub enum TruthValue {
        Zero,
        One,
        Unknown,
    }

    pub fn event_matches_at(
        expr: &BoundEventExpr,
        host: &dyn ExpressionHost,
        frame: &EventEvalFrame<'_>,
    ) -> Result<bool, ExprDiagnostic>;

    pub(crate) fn eval_logical_expr(
        expr: &BoundLogicalExpr,
        host: &dyn ExpressionHost,
        timestamp: u64,
    ) -> Result<TruthValue, ExprDiagnostic>;

In `Cargo.toml`, add the new benchmark target while preserving the existing Criterion workflow:

    [[bench]]
    name = "expr_c2"
    harness = false

Keep `criterion` as the benchmark dependency, keep `insta` as the snapshot dependency, and keep the repository benchmark policy aligned with `docs/DEVELOPMENT.md`.

Revision note (2026-03-11 / OpenCode): initial `C2` ExecPlan authored from `docs/expression_roadmap.md`, with roadmap mapping clarified in `docs/ROADMAP.md` so standalone engine phases (`C2` through `C4`) remain distinct from `C5` command integration.
