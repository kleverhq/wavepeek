# C4 Rich Types and Full Standalone Surface Closure

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek` will reach the `C4` standalone-expression boundary from `docs/expression_roadmap.md`, which means the public `wavepeek::expr` APIs will no longer stop at the integral-only `C3` surface. A contributor will be able to parse, bind, and evaluate the remaining Section `2` language forms directly through the standalone engine: `real` literals and casts, `string` literals and equality, operand-type casts written as `type(operand_reference)'(expr)`, enum-label references written as `type(enum_operand_reference)::LABEL`, enum label preservation rules, and raw-event `.triggered` primaries. Event terms with `iff` will reuse that same full logical surface in standalone evaluation.

The result must be observable without changing the default CLI runtime path. New manifest-driven tests will exercise the public standalone logical and event APIs, proving that rich-type expressions evaluate deterministically and that full `iff logical_expr` support now exists in the standalone engine. Additional waveform-backed tests will prove that VCD and FST metadata is either consumed correctly or rejected through deterministic `error: expr:` diagnostics when the dump cannot supply the needed rich metadata. A dedicated `Criterion` benchmark target and committed run artifacts under `bench/expr/runs/` will lock the performance baseline for this richer standalone state.

This plan targets exactly one roadmap boundary: `C4` from `docs/expression_roadmap.md`. It maps to the remaining standalone-engine scope in `docs/ROADMAP.md` under the `v0.4.0` query-engine milestone. Command integration remains intentionally deferred to `C5`: default `change` runtime behavior must stay compatibility-preserving, and `property` runtime execution must stay unimplemented until the later integration phase.

## Non-Goals

This plan does not route `change` or `property` through the typed standalone engine, does not change default CLI output, exit codes, or schema artifacts, and does not implement any language beyond `docs/expression_lang.md`. It does not add temporal/property operators, does not collapse `C5` command wiring into `C4`, and does not turn optional validation hooks into production-facing behavior. If a test-only shadow path is temporarily useful for validation, it must remain test-only and must not change the observable behavior of default commands.

## Progress

- [x] (2026-03-12 14:12Z) Reviewed `docs/expression_roadmap.md`, `docs/expression_lang.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, the completed `C1` through `C3` expression ExecPlans, and the current `src/expr/`, `src/waveform/`, `tests/`, and `bench/expr/` implementation state.
- [x] (2026-03-12 14:12Z) Drafted the active `C4` ExecPlan with explicit spec ownership, deterministic boundary rules, manifest and snapshot strategy, waveform-metadata fallback policy, benchmark artifacts, and validation gates.
- [x] (2026-03-12 14:37Z) Completed focused review lanes plus a fresh clean control pass, then revised the plan for recovered-type casts on `real` and `string`, real boolean and exponentiation coverage, string-result `?:`, waveform-adapter layering, explicit CLI boundary regressions, waveform benchmark lifecycle, and C3 carry-forward performance control.
- [x] (2026-03-12 16:46Z) Extended the public standalone host/value contract, parser, binder, and evaluator to cover `real`, `string`, recovered operand-type casts, enum-label references, enum-label preservation/reacquisition, and raw-event `.triggered`, then updated the `C2` and `C3` carry-forward suites so they continue validating earlier delivered behavior without freezing now-supported `C4` forms as permanent failures.
- [x] (2026-03-12 16:46Z) Added manifest-driven `tests/expression_c4.rs` coverage plus committed snapshots for deterministic rich-type runtime/semantic failures, and extended CLI boundary tests so `change` and `property` still stay on the `C5` side of command integration even when their inputs contain `C4`-valid standalone expressions.
- [x] (2026-03-12 16:46Z) Added the crate-private waveform expression adapter in `src/waveform/expr_host.rs`, added waveform-backed availability/fallback tests for VCD/FST metadata paths, and added the dedicated `expr_c4` benchmark target plus scenario manifest.
- [x] (2026-03-12 16:46Z) Ran the focused green validation suite for `expression_c1` through `expression_c4`, parser/sema/eval unit tests, waveform adapter tests, `change_cli`, `property_cli`, and `cargo test --bench expr_c4` before benchmark capture and review.

## Surprises & Discoveries

- Observation: the current public host contract in `src/expr/host.rs` still only models integral-family types and sampled bit strings.
  Evidence: `ExprTypeKind` currently exposes only `BitVector`, `IntegerLike`, and `EnumCore`, while `SampledValue` is still a single `bits: Option<String>` container.

- Observation: the waveform layer already sees backend-level `event`, `real`, `string`, and `enum` kind information, but the shared sampling path intentionally rejects or erases most of it.
  Evidence: `src/waveform/mod.rs` maps `VarType::Event`, `VarType::Real`, `VarType::String`, and `VarType::Enum` to listing strings, while `decode_signal_bits(...)` still rejects `SignalValue::String` and `SignalValue::Real` and collapses `SignalValue::Event` to an empty bit string.

- Observation: the standalone engine still has no waveform-backed `ExpressionHost` implementation.
  Evidence: current `ExpressionHost` implementations exist only in tests, benches, and private unit-test stubs; no production adapter bridges `src/waveform/mod.rs` into `src/expr/` yet.

- Observation: the C3 phase already locked every major C4 feature behind deterministic failures, which is good for red-first work because the missing surface is already enumerated.
  Evidence: `tests/fixtures/expr/c3_negative_manifest.json` explicitly rejects `real`, `string`, `type(...)`, enum-label references, and `.triggered` with stable diagnostics.

- Observation: plain VCD `enum` variables can advertise enum kind without carrying enum-table metadata, which makes them ideal deterministic fallback fixtures for `type(enum_operand_reference)::LABEL` failures.
  Evidence: the temporary rich VCD used in `src/waveform/expr_host.rs` binds `type(top.state)::BUSY` far enough to recover enum kind, but binding still fails with `C4-SEMANTIC-METADATA` because no label table is available.

- Observation: the historical `C2` and `C3` negative manifests needed carry-forward pruning once `C4` landed.
  Evidence: before updating those manifests, `expression_c2` and `expression_c3` failed because they still asserted that `real`, `string`, `type(...)`, enum-label references, and `.triggered` must remain deferred forever.

## Decision Log

- Decision: keep `docs/expression_roadmap.md` authoritative for exact `C4` scope and treat `docs/ROADMAP.md` as milestone alignment only.
  Rationale: the roadmap document defines the phase contract precisely and already states that `C4` is still standalone-only while `C5` owns command integration.
  Date/Author: 2026-03-12 / OpenCode

- Decision: preserve the existing public `ExprType` shape as much as possible and widen it instead of replacing it with a separate rich-type system.
  Rationale: `C3` tests and benchmarks already rely on `ExprType`, so the lowest-risk `C4` path is to extend the current model with new kind and metadata fields rather than inventing a second parallel contract.
  Date/Author: 2026-03-12 / OpenCode

- Decision: model raw `event` as a distinct operand kind that is never returned through ordinary value sampling and is only consumable through `.triggered`.
  Rationale: this matches `docs/expression_lang.md`, keeps raw event operands out of generic arithmetic and cast paths, and avoids overloading the integral sampling contract with event-only behavior.
  Date/Author: 2026-03-12 / OpenCode

- Decision: keep the concrete waveform-backed host outside `src/expr/` and outside the public `wavepeek::expr` facade.
  Rationale: `src/expr/` should stay the standalone engine contract, while the concrete bridge from waveform files into that contract belongs to the adapter side. This preserves the existing architecture and avoids public API growth for internal test convenience.
  Date/Author: 2026-03-12 / OpenCode

- Decision: prove positive enum-label, `real`, `string`, and `.triggered` semantics first with the deterministic in-memory test host, then add waveform-backed tests for metadata availability and fallback without changing production command routing.
  Rationale: the standalone engine contract should not depend on dump-format quirks, while the roadmap still requires explicit VCD/FST metadata behavior. Separating semantic conformance from dump-adapter fallback keeps failures diagnosable and preserves the `C4` versus `C5` boundary.
  Date/Author: 2026-03-12 / OpenCode

- Decision: if the waveform backend cannot supply enum label tables for a signal kind or file format, keep label-dependent forms as deterministic semantic failures on waveform-backed hosts instead of guessing labels from raw bits.
  Rationale: guessed labels would be non-deterministic and would violate the roadmap requirement for explicit fallback behavior when dump metadata is missing or unsupported.
  Date/Author: 2026-03-12 / OpenCode

- Decision: revise the initial plan after review to make the performance and boundary gates stricter than the first draft.
  Rationale: review found that recovered-type casts for `real` and `string`, real boolean/exponentiation semantics, C4-specific CLI boundary regressions, waveform adapter placement, and carry-forward performance controls needed to be explicit so a stateless implementer could not satisfy the plan with a narrower or phase-unsafe implementation.
  Date/Author: 2026-03-12 / OpenCode

- Decision: keep the waveform positive path lightweight by benchmarking and testing adapter-backed recovered bit-vector casts on checked-in `m2_core` fixtures, while using a temporary rich VCD fixture for positive `real`/`string`/`event` adapter semantics and enum-metadata fallback.
  Rationale: the repository already has stable VCD/FST hand fixtures for ordinary signal metadata, but it does not yet ship a compact checked-in rich-type FST pair; combining the existing fixtures with a temporary rich VCD preserves deterministic coverage without widening the public API or inventing fake enum labels.
  Date/Author: 2026-03-12 / OpenCode

## Outcomes & Retrospective

Current status: standalone engine, carry-forward suites, waveform adapter, CLI boundary regressions, and benchmark target are implemented; benchmark artifact capture, repository-wide gates, and review/cleanup remain.

The implementation reached the intended `C4` standalone boundary without changing default command routing. Public `wavepeek::expr` calls now parse, bind, and evaluate rich standalone expressions across `real`, `string`, enum labels, recovered operand-type casts, and raw-event `.triggered`, while the crate-private waveform adapter proves both availability and deterministic metadata fallback on dump-backed hosts. The remaining work is mechanical but important: capture the committed benchmark baselines, run the full repository gates, complete the mandatory review workflow, and then record the final evidence excerpts back into this plan.

## Context and Orientation

`wavepeek` is a single-crate Rust project. The public library surface for expression work lives in `src/expr/mod.rs`, and the CLI runtime still lives in `src/engine/`. After `C3`, the public standalone expression entry points are `parse_logical_expr_ast(...)`, `bind_logical_expr_ast(...)`, `eval_logical_expr_at(...)`, `parse_event_expr_ast(...)`, `bind_event_expr_ast(...)`, and `event_matches_at(...)`. Those APIs currently cover integral-family Section `2` semantics plus event `iff` reuse for the integral-only `C3` subset.

The current type and value model is intentionally narrower than the target `C4` contract. `src/expr/host.rs` still exposes only `bit-vector`, `integer-like`, and enum-core operand kinds, and sampled values are still represented as optional bit strings. `src/expr/parser.rs` and `src/expr/lexer.rs` still reject or defer `real`, `string`, `type(operand_reference)'(...)`, `type(enum_operand_reference)::LABEL`, and `.triggered`. `src/expr/sema.rs` still rejects non-integral operands in logical expressions, and `src/expr/eval.rs` still evaluates only integral-family runtime values.

The command boundary must remain explicit in this phase. `src/engine/change.rs` still parses `--on` through the crate-private compatibility adapter and still rejects any runtime `iff` evaluation with `error: args: iff logical expressions are not implemented yet`. `src/engine/property.rs` still returns the unimplemented runtime error. Those behaviors must remain true after `C4` lands, even though the standalone engine will support the full `iff logical_expr` surface through the public `wavepeek::expr` APIs.

The waveform layer matters in `C4` because some rich-type features depend on dump metadata. `src/waveform/mod.rs` already exposes time metadata, signal listing kinds, and backend `SignalValue` branches, but it does not yet provide a typed expression-host adapter. Existing CLI-facing sampling paths are intentionally conservative and reject non-bit-vector encodings. `C4` must add expression-specific metadata plumbing without accidentally changing the CLI-facing `value`, `change`, or `signal` command behavior.

Several terms in this plan have repository-specific meanings. A "rich type" means any remaining Section `2` operand family not completed by `C3`: `real`, `string`, raw `event` used only through `.triggered`, and the label-aware parts of `enum`. An "operand-type cast" means the `type(operand_reference)'(expr)` form that reuses the type recovered from a specific referenced signal, including recovered `real` and `string` targets when those types are available. An "enum label reference" means `type(enum_operand_reference)::LABEL`, which produces an enum value by label lookup in that operand's recovered enum type. A "metadata fallback" is the deterministic `error: expr:` path taken when a waveform-backed host can identify a signal but cannot provide the rich metadata a C4-only form requires. A "canonical signal path" is the dot-separated full hierarchy path already used by `src/waveform/mod.rs` for signal lookup and CLI output. "Interior mutability" in this repository means wrapping a value such as `Waveform` in `RefCell` so read-only trait methods can still lazily load backend signal data without changing the trait signature. A "manifest" is a committed JSON fixture under `tests/fixtures/expr/` that drives one or more tests. An "Insta snapshot" is a committed `.snap` file under `tests/snapshots/` or `src/expr/snapshots/` that locks the rendered output of a diagnostic. A "Criterion run artifact" is an exported benchmark directory under `bench/expr/runs/` containing committed `*.raw.csv`, `summary.json`, and `README.md` files.

The semantic source of truth remains `docs/expression_lang.md`, especially Sections `2.1` through `2.7`. The rollout contract remains `docs/expression_roadmap.md`, which assigns advanced enum semantics, operand-type casts, enum-label references, `real`, `string`, `.triggered`, full grammar closure, full `iff`, and metadata fallback to `C4`. `docs/DESIGN.md` must describe the staged architecture accurately as the standalone engine expands, and `docs/BACKLOG.md` must continue to show that command integration debt remains open until `C5`.

## Open Questions

There are no blocking product questions left for `C4`; the contract is clear enough to implement.

Two implementation-time capability checks are already resolved by policy in this plan. First, if the waveform backend cannot expose enum label tables for a signal, label-dependent forms on waveform-backed hosts must fail deterministically instead of inventing metadata. Second, if no stable checked-in VCD/FST pair can exercise a positive `real`, `string`, or `event` case across both formats, positive semantic coverage may remain on the in-memory host while waveform-backed tests focus on availability and fallback behavior. If either fallback is used, record it in `Surprises & Discoveries`, update `docs/DESIGN.md`, and keep the plan self-consistent.

## Plan of Work

Milestone 1 locks the `C4` contract with failing tests before parser or evaluator changes land. Add one positive manifest and one negative manifest under `tests/fixtures/expr/`, add a new integration-style test file under `tests/`, and extend the in-memory test host so it can model integral, enum-with-label metadata, `real`, `string`, and raw-event inputs. The new positive coverage must prove the full remaining Section `2` surface through public `wavepeek::expr` APIs, while the negative coverage must prove deterministic failures for invalid rich-type combinations, invalid `.triggered` uses, missing metadata, and the still-deferred `C5` command boundary.

Milestone 2 extends the typed parser so the standalone engine truly accepts the full Section `2.7` grammar. `src/expr/lexer.rs`, `src/expr/ast.rs`, and `src/expr/parser.rs` must stop treating C4 forms as deferred placeholders and instead model them explicitly in the public logical AST. The parser must recognize real literals, string literals, operand-type casts, enum-label primaries, and `.triggered` postfixes while keeping event-level syntax strict and leaving command-routing behavior untouched.

Milestone 3 widens the public host, binder, and evaluator contracts for rich types. `src/expr/host.rs` must grow enough type and value metadata to describe `real`, `string`, raw `event`, and enum label tables. `src/expr/sema.rs` must implement the remaining semantic rules from `docs/expression_lang.md`: enum-label lookup, enum label preservation in `?:` and operand-type casts, `real` and mixed numeric conversions, string restrictions, raw-event `.triggered` validation, and deterministic failures for disallowed type combinations or missing metadata. `src/expr/eval.rs` must then evaluate those bound forms with deterministic runtime behavior.

Milestone 4 adds waveform-backed metadata plumbing without changing CLI behavior. The expression engine needs a crate-private adapter that can query `src/waveform/mod.rs` for raw signal kind and sampled value information while keeping existing CLI-facing `sample_signals_at_time(...)` and related methods behaviorally unchanged. That adapter must support the rich information that is actually present and must raise stable `error: expr:` diagnostics when a dump lacks the metadata needed for enum labels, operand-type casts, or non-bit-vector value decoding.

Milestone 5 adds the dedicated `C4` benchmark target plus collateral updates. The benchmark target must measure binder and evaluator behavior on the new rich-type surface rather than on CLI commands. `docs/DESIGN.md`, `docs/DEVELOPMENT.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, and `bench/expr/AGENTS.md` must all describe the new standalone status accurately. Benchmark candidate, final baseline, and same-commit verify artifacts must be committed under `bench/expr/runs/`.

Milestone 6 closes validation and review. Run the focused expression suites, the earlier `C1` through `C3` regression suites, waveform-metadata tests, command-boundary regressions, benchmark compare gates, and repository gates. Then run the mandatory focused review lanes plus one fresh independent control pass on the consolidated diff. Any follow-up fix must land in a separate commit and be revalidated.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

0. Reconfirm the phase boundary before implementation begins.

   Re-read `docs/expression_roadmap.md` Section `C4`, `docs/expression_lang.md` Sections `2.1` through `2.7`, and the implementation-status block in `docs/DESIGN.md`. If discoveries require a scope clarification, update this ExecPlan and the affected status documents in the same commit. Do not silently route default `change` or `property` execution through the typed engine.

1. Lock the `C4` contract with red tests and richer fixture support.

   Add these files:

   - `tests/fixtures/expr/c4_positive_manifest.json`
   - `tests/fixtures/expr/c4_negative_manifest.json`
   - `tests/expression_c4.rs`
   - `tests/snapshots/expression_c4__*.snap`

   Extend these shared helpers as needed:

   - `tests/common/expr_runtime.rs`
   - `tests/common/mod.rs`

   Keep the positive manifest phase-local but expressive. Use one JSON file with at least `logical_cases` and `event_cases` arrays. Each logical case must specify the case name, expression source, input signals, one evaluation timestamp, expected public type metadata, and an expected result payload. The result payload must distinguish integral results from real and string results. For integral enum results, it must also allow an expected label or explicit `null` label. Event cases must specify the full event expression source, tracked-signal names, signal fixtures, probe timestamps, and the timestamps that must match.

   Extend the fixture schema in `tests/common/expr_runtime.rs` so a test signal can carry one of three ordinary sampled-value shapes plus one raw-event shape. Ordinary samples must support integral bits plus optional enum label, `real` numeric values, or `string` values. Raw-event fixtures should be modeled as an explicit list of timestamps where the event occurs, because raw event operands are not plain sampled values and should only become usable through `.triggered`.

   Cover at least these positive buckets:

   - `real` literals, `real'(expr)`, mixed integral-and-real arithmetic, mixed `**` exponentiation, mixed comparisons, and mixed equality where allowed;
   - `real` in boolean context, including `!real_expr`, `real_a && real_b`, `real_a || real_b`, and `real` as the condition operand of `?:`;
   - integral-target casts from `real`, including truncation toward zero, resize, signedness, and 2-state versus 4-state target behavior;
   - string literals, `string'(expr)` identity casts, exact `==` / `!=` string equality with no numeric coercion, and `?:` where both result arms are `string`;
   - operand-type casts through `type(operand_reference)'(expr)` for bit-vector, integer-like, enum, `real`, and `string` targets when those target types are recoverable from the referenced operand, with raw `event` reserved as a negative-only target;
   - enum-label references through `type(enum_operand_reference)::LABEL`, enum casts that regain a label when the resulting bit pattern matches a declared label, and conditional `?:` cases that preserve label-aware enum type when both arms share the same enum type;
   - raw event operand references used only as `.triggered`, including standalone logical evaluation and `iff` payloads that combine `.triggered` with arithmetic, equality, or string/real subexpressions;
   - precedence and associativity across the remaining rich forms, especially combinations of casts, enum-label primaries, selections, `.triggered`, equality, `inside`, and `?:`.

   The negative manifest should keep one flat `cases` array with an `entrypoint` field set to `logical` or `event`. Each case must specify the source text, expected diagnostic layer, code, span, and optional snapshot name. Include at least one locked negative case for each bucket below:

   - invalid mixed-type operations involving `string`, `real`, `event`, or enum labels;
   - invalid use of `string` in boolean context or as a `?:` condition;
   - invalid casts to or from `string`, invalid `real` conversions from integral values that contain `x` or `z`, and invalid casts involving raw `event`;
   - `type(operand_reference)'(expr)` when the referenced operand is unresolved or its rich metadata is unavailable;
   - `type(enum_operand_reference)::LABEL` when the referenced operand is not enum-typed, when the label is missing, or when the dump-backed host cannot supply enum labels;
   - `.triggered` on non-event operands, chained `.triggered`, or raw event operands used without `.triggered`;
   - any still-deferred `C5` command-integration behavior that must remain unchanged.

   Add focused code-style tests alongside the manifests for rules that are easier to understand in Rust than in JSON rows. At minimum include dedicated tests for real-promotion edge cases, real truthiness in logical operators, string equality with no coercion, enum-label preservation on unknown conditional merge, and `.triggered` evaluation at an exact timestamp versus a timestamp where the raw event did not occur.

   Red-phase commands:

       cargo test --test expression_c4 c4_positive_manifest_matches -- --exact
       INSTA_UPDATE=no cargo test --test expression_c4 c4_negative_manifest_matches_snapshots -- --exact
       cargo test --test expression_c4 c4_triggered_and_rich_type_regressions_hold -- --exact

   Expected red evidence before implementation exists: at least one test fails because the parser still rejects a valid C4 form, because the public host/value API cannot represent the needed type, or because `.triggered`, `real`, `string`, or enum-label semantics are still unimplemented.

2. Extend the lexer, AST, and parser to the full `C4` grammar surface.

   Update these source files:

   - `src/expr/mod.rs`
   - `src/expr/ast.rs`
   - `src/expr/lexer.rs`
   - `src/expr/parser.rs`
   - `src/expr/diagnostic.rs` as needed for new parse diagnostics

   `src/expr/lexer.rs` must tokenize the remaining C4 surface: real literals, string literals, the `type(...)` cast target form, enum-label `::`, and `.triggered` as a reserved postfix/member token sequence rather than as part of an operand-reference identifier. Keep existing `C1` and `C2` event-tokenization strictness unchanged.

   `src/expr/ast.rs` must grow explicit public nodes for real literals, string literals, operand-type cast targets, enum-label primaries, and `.triggered`. Keep the public AST readable enough that parser-focused tests and future standalone users can inspect it without depending on binder internals.

   `src/expr/parser.rs` must parse the full Section `2.7` grammar and keep `parse_logical_expr_ast(...)` as the public entry point. The internal offset-aware helper used for event `iff` parsing must continue to map diagnostics back into full event-source spans. Keep event-level syntax phase-correct: event unions, grouping rules, wildcard `*`, and legacy CLI behavior must not be broadened here.

   Add targeted parser unit coverage for `type(operand_reference)'(...)`, `type(enum_operand_reference)::LABEL`, string and real literal lexing, `.triggered` postfix parsing, and precedence when rich primaries mix with existing C3 operators.

   Targeted commands after parser work lands:

       cargo test expr::lexer::tests
       cargo test expr::parser::tests
       INSTA_UPDATE=no cargo test --test expression_c4 c4_negative_manifest_matches_snapshots -- --exact

3. Widen the public host, binder, and evaluator contracts for rich types.

   Update these source files:

   - `src/expr/host.rs`
   - `src/expr/sema.rs`
   - `src/expr/eval.rs`
   - `src/expr/mod.rs`
   - `src/expr/diagnostic.rs` as needed for semantic and runtime codes

   `src/expr/host.rs` must extend the public type model to represent `real`, `string`, and raw `event`, and it must carry enum label metadata when available. Keep the existing shape recognizable so current C3 tests can be updated rather than rewritten. Add a host method dedicated to raw-event occurrence at a specific timestamp instead of pretending that raw events are ordinary sampled scalar values.

   `src/expr/sema.rs` must stop treating non-integral operands as a blanket C4 deferral. It must instead implement the remaining Section `2` semantic rules: boolean-context rules for `real`, string restrictions including string/string `?:`, numeric promotion from integral to `real`, real-specific operator follow-through including exponentiation, real-to-integral cast rules, operand-type cast resolution for every recoverable non-event target kind, enum label lookup, enum label preservation in `?:`, validation of `.triggered` on raw events only, and deterministic diagnostics for missing metadata or disallowed operator/type combinations.

   `src/expr/eval.rs` must represent and evaluate rich runtime values explicitly. Integrals must continue to use normalized `0` / `1` / `x` / `z` text for public assertions, but real and string results need their own public payload forms. Runtime evaluation must reject invalid integral-to-real conversions when the integral operand contains `x` or `z`, must keep string equality exact and known-valued, must preserve enum labels only when the semantic rules require it, and must implement `.triggered` from the host's exact-timestamp event-occurrence query.

   Add unit coverage in `src/expr/sema.rs` and `src/expr/eval.rs` for metadata-dependent binding, enum label preservation, string restriction diagnostics, mixed real/integral promotion, real-cast failures on unknown bits, and `.triggered` validation. Use committed `insta` snapshots under `src/expr/snapshots/` for binder or evaluator diagnostics that are easier to lock at the unit-test layer.

   Targeted commands after binder and evaluator work lands:

       cargo test --test expression_c4
       cargo test expr::sema::tests
       cargo test expr::eval::tests
       INSTA_UPDATE=no cargo test --test expression_c3

4. Add waveform-backed metadata plumbing without changing CLI behavior.

   Add or update these files:

   - `src/waveform/expr_host.rs`
   - `src/waveform/mod.rs` only through crate-private helpers that preserve CLI-facing behavior

   The waveform adapter must be expression-specific but must not live inside the public standalone engine module. Do not change the meaning of `Waveform::sample_signals_at_time(...)`, `Waveform::sample_resolved_optional(...)`, or any human/json CLI output path just to make `C4` easier. Instead, add crate-private helpers that expose the raw signal kind and raw backend value needed by the expression adapter while leaving existing CLI-facing decode paths unchanged.

   Implement a crate-private `WaveformExprHost` in `src/waveform/expr_host.rs` that wraps a `Waveform` with interior mutability, resolves canonical signal paths to `SignalHandle`, maps waveform signal metadata into `ExprType`, and produces `SampledValue` or raw event-occurrence answers for the expression engine. If the waveform backend reports a signal kind but cannot provide the metadata needed for label-aware enum operations or operand-type casts, the adapter must surface deterministic semantic diagnostics such as "metadata for enum labels is unavailable for this signal" instead of leaking backend-specific errors.

   Add focused tests for waveform-backed availability and fallback behavior. Reuse existing VCD/FST fixtures where possible. If rich-type-positive dump fixtures are needed and can be checked in cleanly, add the smallest stable pair possible under `tests/fixtures/hand/`. If only fallback coverage is practical for a given kind, lock that fallback behavior explicitly and document why in `Surprises & Discoveries`.

   In the same milestone, extend `tests/change_cli.rs` and `tests/property_cli.rs` with C4-specific boundary regressions. At minimum add cases where the standalone engine would accept a richer `iff` payload or `--eval` expression, but the default CLI path must still reject runtime `iff` execution or remain unimplemented. Cover at least one recovered-type cast or string/real example and at least one `.triggered` example so the C4 versus C5 boundary stays explicit.

   Targeted commands after adapter work lands:

       cargo test waveform::expr_host::tests
       INSTA_UPDATE=no cargo test --test expression_c4 c4_negative_manifest_matches_snapshots -- --exact

5. Add the `C4` benchmark target and update collateral.

   Add these new benchmark and artifact files:

   - `bench/expr/expr_c4.rs`
   - `bench/expr/scenarios/c4_rich_types.json`
   - `bench/expr/runs/c4-rich-types-candidate/README.md`
   - `bench/expr/runs/c4-rich-types-baseline/README.md`
   - `bench/expr/runs/c4-rich-types-verify/README.md`

   Update these collateral files:

   - `Cargo.toml`
   - `bench/expr/AGENTS.md`
   - `docs/DESIGN.md`
   - `docs/DEVELOPMENT.md`
   - `docs/ROADMAP.md`
   - `docs/BACKLOG.md`

   `bench/expr/expr_c4.rs` must benchmark only the typed library API. Reuse the fixed `Criterion` profile already established for expression microbenchmarks. Parse and bind once outside timed evaluation loops. Include exactly these benchmark IDs in both the bench target and the scenario manifest so the exported run artifacts stay stable:

   - `bind_logical_rich_types`
   - `bind_waveform_host_metadata_path`
   - `eval_logical_real_mixed_numeric`
   - `eval_logical_string_equality`
   - `eval_logical_enum_label_preservation`
   - `eval_event_iff_triggered_rich`
   - `eval_waveform_host_metadata_path`

   The bind scenario must include at least one `type(...)` cast or enum-label primary so metadata-dependent binding cost is measured. The dedicated waveform bind/setup scenario must load the waveform fixture once outside Criterion timing and must then measure only `WaveformExprHost` construction, signal resolution, metadata lookup, and bind-time adapter overhead. The real-evaluation scenario must include integral-to-real promotion, not just a pure-real toy case. The enum scenario must exercise label preservation or label reacquisition through a cast. The event scenario must prove that `.triggered` composes with a richer `iff` payload. The waveform evaluation scenario must keep parse and bind outside the timed loop and must measure repeated adapter-backed sampling and evaluation through `WaveformExprHost`.

   If no stable checked-in positive rich-type waveform fixture is available, the waveform bind/setup benchmark may time a deterministic fallback bind path, and the waveform evaluation benchmark may use any stable adapter-backed expression that still exercises `WaveformExprHost` through real waveform metadata, such as a recovered bit-vector operand-type cast or another non-label-dependent path. The benchmark must still go through the real waveform adapter; only the exact expression may narrow under this fallback policy.

   Candidate run after the first fully green pre-review implementation commit:

       cargo bench --bench expr_c4 -- --save-baseline c4-rich-types-candidate --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c4-rich-types-candidate --bench-target expr_c4 --scenario-set bench/expr/scenarios/c4_rich_types.json --output bench/expr/runs/c4-rich-types-candidate --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Final accepted baseline run after the last review-fix commit:

       cargo bench --bench expr_c4 -- --save-baseline c4-rich-types-baseline --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c4-rich-types-baseline --bench-target expr_c4 --scenario-set bench/expr/scenarios/c4_rich_types.json --output bench/expr/runs/c4-rich-types-baseline --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Same-commit verify run:

       cargo bench --bench expr_c4 -- --save-baseline c4-rich-types-verify --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c4-rich-types-verify --bench-target expr_c4 --scenario-set bench/expr/scenarios/c4_rich_types.json --output bench/expr/runs/c4-rich-types-verify --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"

   Then compare:

       python3 bench/expr/compare.py --revised bench/expr/runs/c4-rich-types-baseline --golden bench/expr/runs/c4-rich-types-candidate --max-negative-delta-pct 15 --require-matching-metadata worktree_state cargo_version rustc_version criterion_version
       python3 bench/expr/compare.py --revised bench/expr/runs/c4-rich-types-verify --golden bench/expr/runs/c4-rich-types-baseline --max-negative-delta-pct 5 --require-matching-metadata source_commit worktree_state cargo_version rustc_version criterion_version

   As a carry-forward control for already-shipped `C3` workloads, rerun the existing `expr_c3` benchmark target after the `C4` changes and compare it against the committed `C3` baseline artifacts:

       cargo bench --bench expr_c3 -- --save-baseline c4-c3-carry-forward --noplot
       python3 bench/expr/capture.py --criterion-root target/criterion --baseline-name c4-c3-carry-forward --bench-target expr_c3 --scenario-set bench/expr/scenarios/c3_integral_boolean.json --output bench/expr/runs/c4-c3-carry-forward --source-commit "$(git rev-parse HEAD)" --worktree-state clean --environment-note "wavepeek devcontainer/CI image"
       python3 bench/expr/compare.py --revised bench/expr/runs/c4-c3-carry-forward --golden bench/expr/runs/c3-integral-boolean-baseline --max-negative-delta-pct 15 --require-matching-metadata worktree_state cargo_version rustc_version criterion_version

6. Commit atomic units and keep review fixes separate.

   Suggested commit split:

       git add tests/fixtures/expr/c4_positive_manifest.json tests/fixtures/expr/c4_negative_manifest.json tests/expression_c4.rs tests/common tests/snapshots
       git commit -m "test(expr): lock c4 rich-type manifests"

       git add src/expr/mod.rs src/expr/ast.rs src/expr/lexer.rs src/expr/parser.rs src/expr/diagnostic.rs
       git commit -m "feat(expr): parse c4 rich expression forms"

       git add src/expr/host.rs src/expr/sema.rs src/expr/eval.rs src/expr/snapshots
       git commit -m "feat(expr): implement c4 rich-type evaluator"

       git add src/waveform/expr_host.rs src/waveform/mod.rs tests/change_cli.rs tests/property_cli.rs
       git commit -m "feat(expr): add waveform metadata host"

       git add Cargo.toml bench/expr/expr_c4.rs bench/expr/scenarios/c4_rich_types.json bench/expr/AGENTS.md docs/DESIGN.md docs/DEVELOPMENT.md docs/ROADMAP.md docs/BACKLOG.md
       git commit -m "bench(expr): add c4 rich-type benchmark flow"

       git add bench/expr/runs/c4-rich-types-candidate
       git commit -m "bench(expr): capture c4 candidate baseline"

       git add bench/expr/runs/c4-rich-types-baseline bench/expr/runs/c4-rich-types-verify bench/expr/runs/c4-c3-carry-forward
       git commit -m "bench(expr): capture final c4 baselines"

   If review finds issues, fix them in separate follow-up commits. Do not amend history.

7. Run validation gates.

   Focused gates first:

       INSTA_UPDATE=no cargo test --test expression_c1
       INSTA_UPDATE=no cargo test --test expression_c2
       INSTA_UPDATE=no cargo test --test expression_c3
       INSTA_UPDATE=no cargo test --test expression_c4
       cargo test expr::parser::tests
       cargo test expr::sema::tests
       cargo test expr::eval::tests
       cargo test waveform::expr_host::tests
       cargo test --test change_cli
       cargo test --test property_cli
       cargo bench --bench expr_c4 -- --noplot
       python3 bench/expr/compare.py --revised bench/expr/runs/c4-rich-types-baseline --golden bench/expr/runs/c4-rich-types-candidate --max-negative-delta-pct 15 --require-matching-metadata worktree_state cargo_version rustc_version criterion_version
       python3 bench/expr/compare.py --revised bench/expr/runs/c4-rich-types-verify --golden bench/expr/runs/c4-rich-types-baseline --max-negative-delta-pct 5 --require-matching-metadata source_commit worktree_state cargo_version rustc_version criterion_version
       python3 bench/expr/compare.py --revised bench/expr/runs/c4-c3-carry-forward --golden bench/expr/runs/c3-integral-boolean-baseline --max-negative-delta-pct 15 --require-matching-metadata worktree_state cargo_version rustc_version criterion_version

   Then repository gates:

       make check
       make ci

   Expected success signature after implementation is complete:

       INSTA_UPDATE=no cargo test --test expression_c4
       ...
       test result: ok.
        cargo bench --bench expr_c4 -- --noplot
        ...
        python3 bench/expr/compare.py --revised ... --golden ... --max-negative-delta-pct 15
        ok: no matched scenario exceeded 15.00% negative delta in mean or median
       make ci
       ...
       test result: ok.

8. Run the mandatory review workflow.

   Load `ask-review` skill and prepare one short context packet that includes: a two-to-four sentence scope summary, the exact diff or commit range, the focus files for each lane, the validation commands already run, and the known risks still worth checking. Then run these focused lanes in parallel after the green validation above:

   - Code lane: `real`, `string`, enum-label, operand-type-cast, and `.triggered` semantics; negative-coverage completeness; and missing rich-type edge cases.
   - Architecture lane: public host/value API shape, crate-private waveform adapter isolation, and command-boundary discipline.
   - Performance lane: allocator pressure or repeated metadata lookups in rich-type evaluation, benchmark scenario quality, and reproducibility of the new `expr_c4` run artifacts.
   - Docs lane: `docs/DESIGN.md`, `docs/DEVELOPMENT.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, and `bench/expr/AGENTS.md` updates.

   For each lane, require the reviewer to return prioritized findings only and to say explicitly when no substantive issues remain in that lane. Save or summarize those lane outputs in the implementation notes for this plan. After lane fixes are committed and revalidated, run one fresh independent control pass on the consolidated diff with the same context packet but no lane-specific narrowing. Treat review as complete only when each focused lane plus the control pass either reports no substantive findings or all reported findings have been fixed, committed, and rechecked. If the performance lane produces a real finding, rerun the affected benchmark capture and compare commands, not just the generic test suite.

### Validation and Acceptance

Acceptance is complete only when all of the conditions below are true together:

- `INSTA_UPDATE=no cargo test --test expression_c4` passes and proves that the public `wavepeek::expr` APIs now accept and evaluate the full `C4` standalone surface, including advanced enum semantics, operand-type casts, enum-label references, `real`, `string`, `.triggered`, and the full `iff logical_expr` surface.
- The locked `C4` positive cases prove `type(operand_reference)'(expr)` for every recoverable non-event target kind, including recovered `real` and `string` targets when available.
- The locked `C4` positive cases prove `real` truthiness in `!`, `&&`, `||`, and `?:` condition positions, and they prove integral-target casts from `real` with truncation, resize, signedness, and state-domain behavior.
- The locked `C4` negative cases prove deterministic failures for invalid rich-type combinations, invalid `.triggered` use, invalid string boolean-context use, missing metadata, and still-deferred `C5` command behavior.
- Public host and result types can represent integral, `real`, and `string` outcomes and can preserve enum labels when the semantic rules require it.
- Raw `event` operands are only usable through `.triggered`, and `.triggered` produces the integral `bit` result defined by `docs/expression_lang.md`.
- Waveform-backed tests prove metadata availability or deterministic fallback for VCD and FST fixtures without changing default CLI output or exit codes.
- The concrete waveform-backed host remains crate-private outside the public `wavepeek::expr` facade, and `docs/DESIGN.md` still describes `expr` as the standalone engine with the waveform bridge living in the adapter layer.
- `tests/change_cli.rs` and `tests/property_cli.rs` include C4-valid standalone expressions that still stay on the current CLI boundary: `change` does not silently adopt typed rich `iff` runtime execution, and `property` remains unimplemented.
- `change` still rejects runtime `iff` execution on the default CLI path.
- `property` still reports the unimplemented runtime error.
- The new `expr_c4` benchmark compare passes, and the carry-forward `expr_c3` compare also passes, proving that richer shared types did not regress the already-shipped integral workloads beyond the phase gate.
- `make check` and `make ci` pass after the last review-fix commit.
- Review lanes and the fresh independent control pass are clean, or all findings were fixed and rechecked.
- The required manifests, snapshots, benchmark files, run artifacts, and updated docs listed in `Artifacts and Notes` exist and match the behavior proven by the commands above.

TDD acceptance requirement: at least one `tests/expression_c4.rs` test must fail before implementation changes and pass afterward in the same branch history.

### Idempotence and Recovery

All planned edits are additive or in-place source, test, benchmark, and documentation changes plus committed snapshot and benchmark artifacts. Re-running the tests is safe. Re-running the in-memory standalone tests is deterministic as long as the manifests are unchanged. Re-running the benchmark capture commands is safe if the target `bench/expr/runs/c4-rich-types-*` directory is removed first or intentionally overwritten from the same scenario set.

If parser work destabilizes `C1`, `C2`, or `C3` behavior, recover in this order: first restore the strict event parser boundary in `src/expr/parser.rs`, then restore the last known green logical rich-type parser helper, then rerun `INSTA_UPDATE=no cargo test --test expression_c1`, `INSTA_UPDATE=no cargo test --test expression_c2`, and `INSTA_UPDATE=no cargo test --test expression_c3` before continuing.

If rich-type binder or evaluator work breaks the public integral-only `C3` expectations, restore the previous public `ExprType` and `ExprValue` behavior for integral results before debugging richer cases. The integral surface is already shipped and must stay stable while C4 expands around it.

If waveform-adapter work starts changing existing CLI-facing sampling behavior, stop and move the new decode logic behind crate-private helpers instead. `C4` must not broaden `value`, `signal`, or `change` user-facing behavior as an accidental side effect of expression work.

If benchmark capture fails because `target/criterion` contains stale data, remove the affected `target/criterion/expr_c4*` directories and rerun the same `cargo bench --bench expr_c4 ...` command before changing code. If the same-commit `baseline` versus `verify` comparison exceeds the tighter `5%` reproducibility guard, rerun both captures before treating the broader `15%` candidate-versus-baseline comparison as meaningful.

### Artifacts and Notes

Expected modified or added files for `C4` implementation:

- `Cargo.toml`
- `src/expr/mod.rs`
- `src/expr/ast.rs`
- `src/expr/lexer.rs`
- `src/expr/parser.rs`
- `src/expr/host.rs`
- `src/expr/sema.rs`
- `src/expr/eval.rs`
- `src/expr/diagnostic.rs`
- `src/waveform/expr_host.rs`
- `src/expr/snapshots/*.snap`
- `src/waveform/mod.rs`
- `tests/expression_c4.rs`
- `tests/expression_c3.rs` only if public helper expectations or compatibility coverage need adjustment
- `tests/change_cli.rs`
- `tests/property_cli.rs`
- `tests/common/mod.rs`
- `tests/common/expr_runtime.rs`
- `tests/fixtures/expr/c4_positive_manifest.json`
- `tests/fixtures/expr/c4_negative_manifest.json`
- `tests/snapshots/expression_c4__*.snap`
- `bench/expr/expr_c4.rs`
- `bench/expr/scenarios/c4_rich_types.json`
- `bench/expr/runs/c4-rich-types-candidate/`
- `bench/expr/runs/c4-rich-types-baseline/`
- `bench/expr/runs/c4-rich-types-verify/`
- `bench/expr/runs/c4-c3-carry-forward/`
- `bench/expr/AGENTS.md`
- `docs/DESIGN.md`
- `docs/DEVELOPMENT.md`
- `docs/ROADMAP.md`
- `docs/BACKLOG.md`

Before closing this plan, record these evidence excerpts here:

- Red phase from `cargo test --test expression_c4 c4_positive_manifest_matches -- --exact` showing that a valid C4 expression still fails before implementation.
- Green phase from `INSTA_UPDATE=no cargo test --test expression_c4` with `test result: ok.`.
- One representative deterministic snapshot showing a missing-metadata or invalid-rich-type diagnostic.
- One waveform-backed test excerpt showing availability or deterministic fallback on both VCD and FST fixtures.
- Benchmark compare gates:
  - `ok: no matched scenario exceeded 15.00% negative delta in mean or median`
  - `ok: no matched scenario exceeded 5.00% negative delta in mean or median`
  - `ok: no matched scenario exceeded 15.00% negative delta in mean or median` for the `expr_c3` carry-forward control run versus `c3-integral-boolean-baseline`
- Review outcome:
  - multi-lane review ran for code, architecture, performance, and docs;
  - findings were fixed in follow-up commits if needed;
  - fresh control review reported no substantive remaining issues.

### Interfaces and Dependencies

The `C4` implementation must leave these library-facing and internal interfaces in place.

In `src/expr/host.rs`, widen the public type model rather than replacing it outright:

    pub enum ExprTypeKind {
        BitVector,
        IntegerLike(IntegerLikeKind),
        EnumCore,
        Real,
        String,
        Event,
    }

    pub struct EnumLabelInfo {
        pub name: String,
        pub bits: String,
    }

    pub struct ExprType {
        pub kind: ExprTypeKind,
        pub storage: ExprStorage,
        pub width: u32,
        pub is_four_state: bool,
        pub is_signed: bool,
        pub enum_type_id: Option<String>,
        pub enum_labels: Option<Vec<EnumLabelInfo>>,
    }

Keep `width` in place to minimize churn. Use `64` for `real`, `0` for `string` and raw `event`, and the existing integral meaning everywhere else. Keep `storage` meaningful for integrals and use `Scalar` for `real`, `string`, and raw `event`.

Replace the current one-shape sampled-value struct with an explicit public enum plus a raw-event occurrence query:

    pub enum SampledValue {
        Integral {
            bits: Option<String>,
            label: Option<String>,
        },
        Real {
            value: Option<f64>,
        },
        String {
            value: Option<String>,
        },
    }

    pub trait ExpressionHost {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic>;
        fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic>;
        fn sample_value(
            &self,
            handle: SignalHandle,
            timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic>;
        fn event_occurred(
            &self,
            handle: SignalHandle,
            timestamp: u64,
        ) -> Result<bool, ExprDiagnostic>;
    }

`sample_value(...)` must never be used for raw `event`, and `event_occurred(...)` must never be used for non-event kinds. Both misuses should become deterministic internal or host diagnostics during testing.

In `src/expr/eval.rs`, make the public standalone result type rich enough for all C4 outcomes:

    pub enum ExprValuePayload {
        Integral {
            bits: String,
            label: Option<String>,
        },
        Real {
            value: f64,
        },
        String {
            value: String,
        },
    }

    pub struct ExprValue {
        pub ty: ExprType,
        pub payload: ExprValuePayload,
    }

Integral public results must continue to use normalized MSB-first text with only `0`, `1`, `x`, and `z`. Real and string results must preserve their natural value forms so tests can assert exact semantics without relying on internal helpers.

In `src/expr/mod.rs`, continue to expose the public standalone logical and event APIs from `C3`. The only changes should be the richer public type/value exports that genuinely belong to the standalone contract. Do not publicly re-export the concrete waveform-backed host or any test-only helpers.

In `src/waveform/expr_host.rs`, define a crate-private adapter with interior mutability around `crate::waveform::Waveform` so the host can lazily load signal data even though `ExpressionHost` methods take `&self`:

    pub(crate) struct WaveformExprHost {
        waveform: std::cell::RefCell<crate::waveform::Waveform>,
        ...
    }

Do not expose the full waveform module publicly just to support this phase. Keep the adapter narrowly scoped to expression use and future `C5` reuse.

Revision note (2026-03-12 / OpenCode): initial `C4` ExecPlan authored from `docs/expression_roadmap.md`, the `C3` standalone implementation state, current waveform metadata limits, and existing expression/benchmark conventions. Revised after multi-lane review and a clean control pass to clarify recovered-type casts for `real` and `string`, real boolean/exponentiation coverage, string-result `?:`, waveform adapter placement outside `src/expr`, explicit CLI boundary regressions, waveform benchmark setup versus eval timing, and the committed `expr_c3` carry-forward performance gate.
