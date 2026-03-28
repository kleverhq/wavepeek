# C5 Command Integration and Expression Runtime Hardening

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek` will finally reach the `C5` boundary from `docs/expression_roadmap.md`: the typed expression engine in `src/expr/` will stop being a standalone-only capability and will become the real runtime path for waveform commands. Users will be able to run `wavepeek change --on "posedge clk iff valid && ready" ...` and get real snapshots instead of the current hard failure, and they will be able to run `wavepeek property --eval "valid && ready"` end to end in both human and `--json` modes with deterministic capture results across VCD and FST.

The change must be observable from the CLI, not only from library tests. A contributor following this plan will add end-to-end integration coverage for `change` and `property`, update the JSON schema and human rendering for `property`, extend command-level parity and performance coverage, and then prove the finished result with `make ci`, VCD/FST parity tests, and `bench/e2e` compare gates. This plan targets exactly one roadmap boundary: `C5` from `docs/expression_roadmap.md`.

This plan also resolves one stale contract mismatch before implementation starts. `docs/expression_roadmap.md` and the opening sentence of `docs/expression_lang.md` still mention a `change --eval` surface, but the actual CLI and command design define only `change --on` plus `property --eval`. `C5` will resolve that mismatch by explicit contract alignment, not by inventing a new undocumented `change --eval` flag in the same phase.

## Non-Goals

This plan does not add any language feature beyond `docs/expression_lang.md`. It does not add temporal property operators, implication, coverage syntax, streaming JSON, or any new command surface outside the already documented `change` and `property` flags. It does not keep the legacy `change --on` parser or runtime as a supported fallback once `C5` is complete, and it does not introduce a public `change --eval` flag. It does not redesign the `bench/e2e` harness, and it does not require expression microbenchmark workflow changes outside small collateral updates that may be needed if waveform-host hot paths move.

Implementation naming rule: the roadmap label `C5` is temporary planning terminology only. Do not introduce `C5` into new or updated files under `tests/`, `src/`, `schema/`, `bench/`, or live docs such as `docs/DESIGN.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, `docs/DEVELOPMENT.md`, or `CHANGELOG.md`. It is acceptable for this ExecPlan and the temporary roadmap documents to use the label for phase mapping, but shipped artifacts must use stable descriptive names instead.

## Progress

- [x] (2026-03-28 19:20Z) Reviewed `docs/expression_roadmap.md`, `docs/expression_lang.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, the completed `C1` through `C4` expression ExecPlans, current `src/engine/`, `src/expr/`, `src/output.rs`, `schema/wavepeek.json`, and the current `tests/` plus `bench/e2e/` coverage.
- [x] (2026-03-28 19:20Z) Drafted the active `C5` ExecPlan with explicit scope, command-surface decisions, integration test strategy, schema/output contract, parity requirements, and command-level performance gates.
- [x] (2026-03-28 19:20Z) Completed focused plan review lanes and revised the plan to define scope-aware typed binding, wildcard tracking for any `*` term including raw-event references, phase-local CLI contract artifacts, truthful doc-update sequencing, shared waveform ownership, and explicit tune-mode validation.
- [x] (2026-03-29 00:57Z) Added red-first command integration coverage and contract-alignment docs, captured failing evidence for `change --on ... iff ...`, `property`, help text, and schema gaps, then drove the implementation green on the same branch.
- [x] (2026-03-29 00:57Z) Routed `change` through typed event binding/evaluation, added shared command expression-runtime helpers, preserved deterministic payload/warning behavior, and kept tune-mode equivalence coverage green.
- [x] (2026-03-29 00:57Z) Implemented `property` end to end with capture modes, wildcard tracking from `--eval`, human/json rendering, schema support, manifest-backed command fixtures, and `error: expr:` command diagnostics.
- [x] (2026-03-29 00:57Z) Extended parity, schema, benchmark, and collateral coverage; added `property` entries to `bench/e2e/tests.json` and median-aware compare gating, but baseline refresh remains pending until after review completion.
- [ ] Run mandatory review lanes plus a fresh control pass, apply any follow-up fixes in separate commits, and close the plan with final validation evidence.

## Surprises & Discoveries

- Observation: the roadmap and expression-language docs still mention `change --eval`, but the shipped CLI and design contract do not expose that flag.
  Evidence: `docs/expression_roadmap.md` assigns `change` surfaces `--on, --eval` to `C5`, while `src/cli/change.rs` defines no `eval` field and `docs/DESIGN.md` documents only `change --on`.

- Observation: the shared typed engine is already capable of waveform-backed parse, bind, and evaluation, so `C5` is mostly command wiring, output, and validation work rather than a new standalone semantic phase.
  Evidence: `src/expr/mod.rs` already exports public parse/bind/evaluate entry points, and `src/waveform/expr_host.rs` already provides a crate-private `WaveformExprHost` that implements `ExpressionHost`.

- Observation: `property` currently exits before any expression parsing or waveform work happens, so `C5` must define new error priority and output contracts from scratch at the command layer.
  Evidence: `src/engine/property.rs` is still a single `WavepeekError::Unimplemented(...)` return, and `tests/property_cli.rs` currently asserts that even invalid `--on` expressions still fail as `error: unimplemented:`.

- Observation: the JSON output and schema layers are not merely missing `property` data examples; they structurally omit the command today.
  Evidence: `src/engine/mod.rs` omits `Property` from both `CommandName` and `CommandData`, `src/output.rs` has no property rendering branch, and `schema/wavepeek.json` enumerates only `info`, `scope`, `signal`, `value`, and `change`.

- Observation: the command-level performance harness already covers large `change` workloads, but it has no `property` scenarios yet.
  Evidence: `bench/e2e/tests.json` contains many `category: "change"` cases and no `category: "property"` entries.

- Observation: the old committed `bench/e2e` baseline appears to have broad environment drift relative to the current container image, including unchanged `info`/`value` scenarios, so pre-refresh compare failures are not isolated to the new command-integration code.
  Evidence: repeated `python3 bench/e2e/perf.py compare --revised ... --golden bench/e2e/runs/baseline --max-negative-delta-pct 15 --verbose` runs reported mean/median regressions for unchanged commands such as `info_scr1`, `signal_scr1_top_recursive_depth2_json`, and `value_chipyard_clusteredrocketconfig_dhrystone_signals_10` in addition to some `change` scenarios.

## Decision Log

- Decision: treat `docs/expression_roadmap.md` as the exact `C5` phase contract, but resolve the stale `change --eval` wording by documentation alignment rather than by adding a new flag.
  Rationale: `docs/DESIGN.md`, `src/cli/change.rs`, and all existing CLI tests define `change` as a trigger-and-snapshot command, not as a second property-style evaluator. Inventing `change --eval` would expand product scope without a stable command contract.
  Date/Author: 2026-03-28 / OpenCode

- Decision: keep `change` candidate-discovery engines (`baseline`, `fused`, `edge-fast`) only as performance prefilters; the final truth of whether an event term matches at a timestamp must come from the shared typed event binder and evaluator.
  Rationale: `C5` requires one shared expression engine for command consumers, but the repository already depends on change-specific candidate collection and dispatch heuristics for performance. Reusing those heuristics is acceptable only if they no longer define event semantics themselves.
  Date/Author: 2026-03-28 / OpenCode

- Decision: define `property` JSON output as an ordered array of rows with fields `time` and `kind`, where `kind` is one of `match`, `assert`, or `deassert`.
  Rationale: human output already targets lines such as `@123ns assert` and `@123ns match`. `switch` is a capture-mode selector, not an emitted event kind, so serializing `switch` would hide the actual runtime transition that occurred.
  Date/Author: 2026-03-28 / OpenCode

- Decision: initialize `property` switch-style baseline state by evaluating `--eval` once at the inclusive range start (`--from` or dump start) and never emit a row for that baseline probe itself.
  Rationale: this mirrors the existing `change` command rule that baseline state is initialized at the range start, keeps bounded queries self-contained, and avoids depending on out-of-range candidate timestamps to decide whether the first in-range event is an `assert` or `deassert`.
  Date/Author: 2026-03-28 / OpenCode

- Decision: add command-facing `error: expr:` support to `WavepeekError` and use it for expression parse, semantic, and runtime failures that originate from `--on` or `--eval`.
  Rationale: `docs/DESIGN.md` already defines `error: expr:` as the intended category for invalid expressions, and `C4` already standardized deterministic `ExprDiagnostic` layers and codes. Continuing to collapse typed expression failures into generic `args` or `unimplemented` output would keep command behavior behind the documented contract.
  Date/Author: 2026-03-28 / OpenCode

- Decision: implement command scoping through a scope-aware expression-host wrapper instead of rewriting expression source text ad hoc, and preserve the current short-name-only rule under `--scope`.
  Rationale: current command scope behavior is defined by token-to-canonical-path resolution, while the typed binder resolves names through `ExpressionHost`. A thin wrapper that prepends `scope.` only for undotted short names preserves the command contract for both `--on` and `--eval` without teaching the parser about CLI-only scope rules, and it avoids accidentally making dotted canonical names legal under `--scope`.
  Date/Author: 2026-03-28 / OpenCode

- Decision: whenever a bound event expression contains any wildcard term, the tracked set comes from every signal handle referenced by `--eval`, including raw-event handles used only through `.triggered()`.
  Rationale: the expression-language contract defines `*` against the command-defined tracked set, not only against the literal default `--on "*"` spelling. Raw event operands are still referenced signals for `property`, so default or unioned wildcard behavior must observe their occurrence timestamps as part of the tracked set.
  Date/Author: 2026-03-28 / OpenCode

- Decision: refactor waveform-backed command integration around one shared command-owned `Waveform` handle wrapped by `Rc<RefCell<...>>`, and let `WaveformExprHost` borrow that shared state instead of opening the dump a second time.
  Rationale: both `change` and `property` need direct waveform access for candidate collection and typed host access for expression binding/evaluation. Making both paths share one opened dump avoids hidden double-open cost and keeps the benchmark story honest.
  Date/Author: 2026-03-28 / OpenCode

- Decision: use the existing full-surface expression manifests `tests/fixtures/expr/rich_types_positive_manifest.json` and `tests/fixtures/expr/rich_types_negative_manifest.json` as the locked phase manifests for language semantics, and add phase-local command behaviors through code-level integration tests.
  Rationale: `C5` does not expand the standalone language contract; it wires the already locked `C4` surface into commands. Reusing the existing manifests avoids inventing a second fixture schema for semantics that are already covered, while command output and capture behavior remain clearer as CLI tests.
  Date/Author: 2026-03-28 / OpenCode

- Decision: satisfy the phase-gate artifact rule with small phase-local CLI manifests and snapshots in addition to the carried-forward rich-type manifests.
  Rationale: `C5` changes command behavior rather than standalone language semantics, so the plan needs one phase-local positive/negative artifact set for CLI contracts and deterministic `error: expr:` text even though the standalone language manifests are still reused. Those artifact names must stay descriptive and must not encode the temporary roadmap label.
  Date/Author: 2026-03-28 / OpenCode

- Decision: extend `bench/e2e/tests.json` with `property` scenarios and use the existing `bench/e2e/perf.py` compare flow as the C5 performance gate.
  Rationale: `C5` exit criteria are command-level, not library-microbenchmark-level. The repository already has a maintained CLI benchmark harness that captures timing plus JSON `data` parity, while dedicated Rust tests already cover `warnings` parity more precisely. To stay aligned with the roadmap default benchmark gate, `C5` must extend that harness so compare-time failure checks both mean and median regression at the same threshold.
  Date/Author: 2026-03-28 / OpenCode

- Decision: `property --eval` final truth reduction follows the same boolean-context rules as the typed expression engine: any integral payload containing `1` is true, non-zero `real` is true, and string/unknown final values reduce to false for capture output.
  Rationale: the shared typed engine already defines boolean-context truth reduction for `iff`, and command integration must not invent a narrower rule such as `bits == "1"` for `property`.
  Date/Author: 2026-03-29 / OpenCode

## Outcomes & Retrospective

Current status: implementation complete; review, final benchmark baseline refresh, and closeout remain in progress.

The branch now routes `change` and `property` through the shared typed expression runtime, ships `property` JSON/human output plus schema support, and locks the new command behavior with CLI fixtures, parity tests, and help/schema contract coverage. Repository gates (`cargo test`, `python3 scripts/check_schema_contract.py`, `make check`, `make ci`) are green. The remaining work is mandatory review, any follow-up fixes, a post-review `bench/e2e` baseline refresh, and final closeout notes.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI. The typed expression engine lives in `src/expr/`, the waveform adapter layer lives in `src/waveform/`, command runtimes live in `src/engine/`, and stdout rendering lives in `src/output.rs`. After `C4`, the standalone engine is already functionally complete for the language described in `docs/expression_lang.md`: `src/expr/mod.rs` exports `parse_logical_expr_ast(...)`, `bind_logical_expr_ast(...)`, `eval_logical_expr_at(...)`, `parse_event_expr_ast(...)`, `bind_event_expr_ast(...)`, and `event_matches_at(...)`, while `src/waveform/expr_host.rs` bridges real waveform dumps into that API through `WaveformExprHost`.

The command layer has not caught up yet. `src/engine/change.rs` still calls the crate-private compatibility parser through `parse_event_expr(...)` from `src/expr/legacy.rs`, resolves simple event terms manually, and hard-fails any `iff` clause with `error: args: iff logical expressions are not implemented yet`. `src/engine/property.rs` still returns `error: unimplemented: \`property\` command execution is not implemented yet` before it parses or validates `--on`, `--eval`, capture mode behavior, or output format. `src/engine/mod.rs`, `src/output.rs`, and `schema/wavepeek.json` all still treat `property` as nonexistent runtime output.

Several repository-specific terms matter in this plan. A "typed engine" means the public parse, bind, and evaluation APIs from `src/expr/`, not the legacy compatibility adapter in `src/expr/legacy.rs`. A "candidate timestamp" means a waveform timestamp that is worth checking because some trigger-relevant signal changed or some raw event in the tracked set occurred; candidate collection can be optimized, but final trigger truth still has to come from the bound typed event expression. A "tracked set" is the signal-handle set supplied through `EventEvalFrame`; for `change` it comes from requested `--signals`, and for `property` it comes from all signal handles referenced by `--eval` whenever the bound event expression contains any wildcard term, including raw-event `.triggered()` references. A "capture result" is the user-visible outcome emitted by `property`: `match`, `assert`, or `deassert`. A "capture mode" is the CLI filter that decides which results are emitted: `match`, `switch`, `assert`, or `deassert`. In this repository, `switch` is a filter that may emit either `assert` or `deassert`, not a row kind of its own. "Red-first" means adding tests that must fail before implementation changes land. A "control pass" means one fresh independent review after lane-specific reviews are complete. A "tune-mode" means the hidden `change` debug flags `--tune-engine`, `--tune-candidates`, and `--tune-edge-fast-force` that choose internal candidate/evaluation strategies without changing payload semantics.

The design and roadmap documents split responsibility. `docs/expression_lang.md` is the semantic source of truth for event and logical expressions. `docs/expression_roadmap.md` defines staged rollout and says `C5` owns command integration, schema parity, and command-level benchmark gates. `docs/DESIGN.md` defines command behavior and output contracts, including the intended `property` semantics and the repository-wide `error: <category>:` convention. `docs/ROADMAP.md` maps `C5` to the version-oriented roadmap, and `docs/BACKLOG.md` records the still-open debt for `change --on` `iff`, legacy parser split, and unimplemented `property` runtime.

The most important existing implementation pieces are these. `src/waveform/mod.rs` already offers `resolve_expr_signal(...)`, `sample_expr_value(...)`, `expr_event_occurred(...)`, and `collect_change_times_with_mode(...)`. `src/waveform/expr_host.rs` already wraps those capabilities behind `ExpressionHost`, but it currently owns its own `Waveform`; `C5` must refactor that so command candidate collection and typed evaluation share one opened dump. `src/engine/change.rs` already contains the time-window parsing, requested-signal resolution, candidate scheduling, warning behavior, output row construction, and auto-dispatch heuristics that must remain deterministic after the runtime swap. `src/cli/property.rs` already defines the final flag surface, including `CaptureMode`, while `src/cli/mod.rs` and `tests/cli_contract.rs` still describe it as unimplemented. `src/error.rs` currently lacks the documented `Expr` category, so command integration must add that mapping explicitly instead of improvising error text at call sites.

## Open Questions

There are no blocking product questions left for `C5`. This plan resolves the only material ambiguities up front: `change --eval` is removed from the command contract rather than implemented, expression-originated command failures use `error: expr:`, property JSON rows use `time` plus emitted `kind`, and switch-style capture state is initialized from the inclusive range start.

If implementation discovers a purely performance-motivated need for additional typed-engine caching inside `WaveformExprHost` or `src/engine/change.rs`, add that caching behind the same typed bind/evaluate path and record it in `Surprises & Discoveries`. Do not revive the legacy parser or ad hoc event semantics as an optimization shortcut.

## Plan of Work

Milestone 1 locks the contract with failing command tests and truth-preserving docs before any runtime swap lands. Update only the stale cross-document contract wording early, especially the `change --eval` mismatch, then convert the existing deferred-runtime tests into red tests that assert the intended `C5` behavior instead. Leave status docs truthful about the current branch state until the runtime, schema, and help changes are actually implemented and green.

Milestone 2 moves `change` onto the typed event path while preserving the current snapshot payload contract. The implementation should bind `--on` through `parse_event_expr_ast(...)` and `bind_event_expr_ast(...)`, construct a scope-aware waveform-backed host over the same command-owned waveform used for candidate collection, reuse existing candidate-discovery heuristics where they are still safe, and call `event_matches_at(...)` as the final gate for every emitted timestamp. Legacy `parse_event_expr(...)` and legacy manual event-term evaluation must disappear from production `change` execution.

Milestone 3 implements `property` end to end on top of the same typed host and evaluation flow. The command should parse and bind `--on` and `--eval`, derive the tracked set for wildcard terms from the bound logical expression, evaluate `--eval` at candidate timestamps through `eval_logical_expr_at(...)`, reduce the result to the documented two-state decision (`1` true, `0/x` false), apply capture-mode filtering, and emit deterministic human or JSON rows. This milestone also adds the missing `property` command data model, human renderer branch, JSON schema branch, and command help text.

Milestone 4 hardens parity, diagnostics, and collateral. Add VCD/FST parity coverage for both commands, add schema and output tests for `property`, add docs and backlog updates that describe the shipped command-integration behavior in stable user-facing terms rather than deferred debt, and add `bench/e2e` `property` scenarios so command-level performance plus JSON `data` parity stay guarded after the runtime swap. Keep `warnings` parity enforced by the Rust integration tests rather than by the benchmark harness.

Milestone 5 closes validation and review. Run the carried-forward standalone expression manifests to prove the language engine still behaves the same, then run the new command integration tests, schema tests, parity tests, and benchmark compare gates. After the diff is green, run the mandatory review lanes for code semantics, architecture/performance, and docs/schema, apply any fixes in separate commits, rerun the impacted gates, refresh the maintained `bench/e2e` baseline, and only then close the plan.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property`.

0. Reconfirm the phase boundary and make only truth-preserving early doc edits.

   Re-read `docs/expression_roadmap.md` Section `C5`, the opening paragraphs plus Section `1` of `docs/expression_lang.md`, the `change` and `property` command sections in `docs/DESIGN.md`, and the `v0.4.0` / `v0.5.0` bullets in `docs/ROADMAP.md`. Update these files in the same commit that locks the final scope:

   - `docs/expression_lang.md`
   - `docs/expression_roadmap.md`
   - `docs/DESIGN.md`
   - `docs/ROADMAP.md`
   - `docs/BACKLOG.md`

   Required alignment rules for this early pass:

   - remove or rewrite stale mentions of `change --eval` so the command contract is `change --on` plus `property --eval`;
   - keep the roadmap wording explicit that command integration and hardening is the remaining work, not new language design, without propagating the temporary phase label into live docs;
   - do not yet claim that `property` runtime is implemented or that backlog debt is closed; those shipped-status updates belong to the later green collateral milestone.

1. Add red-first command integration tests before changing runtime code.

   Update or add these test files first:

    - `tests/change_cli.rs`
    - `tests/property_cli.rs`
    - `tests/change_vcd_fst_parity.rs`
    - `tests/property_vcd_fst_parity.rs`
    - `tests/change_opt_equivalence.rs`
    - `tests/cli_contract.rs`
    - `tests/common/command_cases.rs` and `tests/common/mod.rs` for shared CLI manifest loading
    - `tests/schema_cli.rs`
    - `tests/command_fixture_contract.rs`
    - `tests/AGENTS.md` so the new `tests/fixtures/cli/` breadcrumb is discoverable from the parent map
    - `tests/fixtures/cli/AGENTS.md`
    - `tests/fixtures/cli/command_runtime_positive_manifest.json`
    - `tests/fixtures/cli/command_runtime_negative_manifest.json`
    - `tests/snapshots/property_cli__expr_runtime_*.snap` and `tests/snapshots/change_cli__expr_runtime_*.snap` as needed for deterministic command-facing `error: expr:` text
    - `src/output.rs` unit tests as needed for renderer coverage

   Keep the carried-forward semantic manifests locked and green throughout the phase:

   - `tests/fixtures/expr/rich_types_positive_manifest.json`
   - `tests/fixtures/expr/rich_types_negative_manifest.json`

    Convert the current deferred-runtime assertions into the intended `C5` outcomes. At minimum add or rewrite tests for these buckets:

    - scoped short-name expressions under `--scope` work through the typed path in both `change` and `property`, for example `--scope top --on "posedge clk iff rstn"` and `--scope top --eval "data == 8'h03"`, while dotted canonical names under `--scope` continue to fail as they do today;
    - `change --on "... iff ..."` succeeds in human and JSON modes and yields the same snapshot rows on VCD and FST for a shared fixture pair;
    - `change` accepts rich `iff` payloads that were already valid in standalone `C4`, including `type(...)`, `real`, `string`, and `.triggered()` forms where the fixture supports them;
    - wildcard unions such as `--on "* or posedge clk"` use the same tracked-set rules as bare `*`;
    - `property --capture match` emits one row per candidate timestamp where the final decision is true;
    - `property --capture switch` emits `assert` and `deassert` rows only on state transitions after the internal range-start baseline probe;
    - `property --capture assert` emits only `0 -> 1` transitions, and `--capture deassert` emits only `1 -> 0` transitions;
    - omitted `--on` behaves as wildcard `*` over every signal referenced by `--eval`, including raw-event handles used through `.triggered()`;
    - wildcard `*` with an `--eval` that references no signals fails deterministically with an `args` error instructing the user to pass `--on` explicitly;
    - invalid `--on` or `--eval` syntax, unknown expression names, missing metadata, and runtime sample failures now report `error: expr:` rather than `error: unimplemented:`;
    - `property --help` and top-level help no longer advertise the command as unimplemented;
    - `wavepeek schema` output now includes a `property` command branch.

   The new command-runtime manifests must be phase-local artifacts, not ad hoc helper files. If `tests/fixtures/cli/` does not already exist, add `tests/fixtures/cli/AGENTS.md` that links back to `tests/AGENTS.md` and names the new command-manifest contract, then update `tests/AGENTS.md` so the new child map is discoverable from the parent breadcrumb. Keep the manifests small and explicit. Add one shared loader under `tests/common/command_cases.rs` so `tests/command_fixture_contract.rs` and the command-specific suites validate the same rows through the same code path. Treat `tests/command_fixture_contract.rs` as an independent required gate, not as an ordering guarantee. A positive row should identify the command (`change` or `property`), the exact argument vector, the output mode (`human` or `json`), the expected exit code, and either the expected JSON `data` plus `warnings` payload or a snapshot name for human stdout. A negative row should identify the command, exact argument vector, expected exit code, expected stderr prefix (`error: expr:` or `error: args:`), and optional snapshot name when the full rendered message text is part of the contract. Use descriptive names such as `command_runtime_*` and `expr_runtime_*`, not roadmap labels.

   Red-phase commands:

       cargo test --test change_cli change_iff_executes_end_to_end -- --exact
       cargo test --test property_cli property_switch_capture_reports_transitions -- --exact
       cargo test --test property_cli property_invalid_eval_reports_expr_error -- --exact
       cargo test --test cli_contract unimplemented_subcommands_disclose_status_in_help -- --exact

   Expected red evidence before runtime work lands: at least one `change` test still fails with `iff logical expressions are not implemented yet`, at least one `property` test still fails with `error: unimplemented:`, and at least one schema/help assertion still fails because `property` output is absent from the runtime contract.

2. Add command-facing expression error mapping and shared command-runtime helpers.

   Update these files:

   - `src/error.rs`
   - `src/engine/mod.rs`
   - add `src/engine/expr_runtime.rs` or another crate-private helper module if shared logic becomes non-trivial

   `src/error.rs` must gain a real `Expr(String)` variant that formats as `error: expr: ...` and exits with code `1`. Add a narrow conversion helper from `ExprDiagnostic` to user-facing command text; keep the mapping deterministic and avoid dumping internal debug structs.

   In the shared engine area, create one crate-private place for command integration helpers that both `change` and `property` can use. That helper should own only the logic that is truly shared: constructing a scope-aware host wrapper, binding `WaveformExprHost`, normalizing expression diagnostics into `WavepeekError::Expr`, collecting referenced signal handles for wildcard tracking, converting those handles back into waveform candidate sources, and formatting raw timestamps into command-facing strings. Keep command-specific payload assembly in `src/engine/change.rs` and `src/engine/property.rs`.

   Make the handle-to-waveform bridge explicit in this phase. The implementation must add one crate-private accessor path from typed `SignalHandle` values back to waveform-level metadata (`ExprResolvedSignal` or equivalent canonical-path plus kind data) so command runtimes can derive both `EventEvalFrame.tracked_signals` and candidate timestamp sources from the same bound expressions. Treat this as required `C5` scope, not an optional helper.

   Add unit coverage for the new error category and any shared helper that computes tracked handles or converts expression diagnostics.

3. Route `change` through the typed event engine and remove the legacy runtime dependency.

   Update these files:

   - `src/engine/change.rs`
   - `src/expr/mod.rs` only if removing the legacy wrapper from production paths requires visibility cleanup
   - `src/expr/legacy.rs` only for compile-fix cleanup; do not keep it in the runtime path
   - `src/waveform/expr_host.rs` and `src/waveform/mod.rs` only if additional caching or signal-resolution helpers are needed

    Implementation rules for `change`:

    - Parse `--on` with `parse_event_expr_ast(...)`, bind once with `bind_event_expr_ast(...)`, and create one scope-aware `WaveformExprHost` view over a command-owned shared `Waveform` per run.
    - Add one shared conversion helper that can turn requested canonical signal paths and bound logical-expression handles into both `SignalHandle` slices for `EventEvalFrame` and waveform candidate sources for change-time collection.
    - Keep existing requested-signal resolution, range parsing, warnings, JSON envelope shape, human row rendering, and max-row truncation behavior unchanged.
   - Keep candidate-discovery and auto-dispatch heuristics if they still help, but treat them only as a way to choose which timestamps to probe. For every candidate timestamp that may be emitted, construct `EventEvalFrame` and call `event_matches_at(...)` as the final semantic gate.
   - If the bound event expression contains any wildcard term, make sure the `EventEvalFrame.tracked_signals` slice still reflects the command contract even for unioned forms such as `* or posedge clk`.
   - If a specialized fast path cannot preserve typed semantics for a trigger shape, fall back to the baseline path instead of keeping legacy event evaluation.
    - Remove the production-path early return that rejects any `iff` clause.

    Borrow-discipline rule: when the command and `WaveformExprHost` share one `Rc<RefCell<Waveform>>`, keep each borrow scoped to the smallest possible block. Do not hold a mutable waveform borrow across calls that may sample through the host, or the implementation will risk nested-borrow panics.

    Add or update tests that prove:

    - `change` still emits the same payloads for previously supported non-`iff` triggers;
    - `change` now succeeds for `iff` cases and rich `C4` payloads;
    - scoped short names still resolve correctly through the typed path;
    - wildcard unions containing `*` still honor the tracked-set contract;
    - VCD and FST continue to match for equivalent `change` requests, including at least one typed `iff` case;
    - tune-mode overrides in debug mode do not change payload semantics even after the runtime swap, as proven by `tests/change_opt_equivalence.rs` and explicit `DEBUG=1` runs.

    Targeted commands after the `change` runtime swap lands:

        cargo test --test change_cli
        cargo test --test change_vcd_fst_parity
        cargo test --test change_opt_equivalence
        cargo test change::tests

4. Implement `property` end to end on the same typed host.

   Update these files:

   - `src/engine/property.rs`
   - `src/cli/mod.rs`
    - `src/cli/property.rs`
   - `src/engine/mod.rs`
   - `src/output.rs`
   - `schema/wavepeek.json`
   - `src/schema_contract.rs` only if schema-loading tests require compile-time adjustments

   `src/engine/property.rs` must stop returning `Unimplemented` and instead perform the full command flow:

   - open the waveform and validate the time window using the same inclusive-range rules already used by `change`;
   - parse and bind `--on` through the typed event API and `--eval` through the typed logical API;
   - determine the tracked set from the bound `--eval` signal references whenever the bound event expression contains any wildcard term, including raw-event `.triggered()` references;
   - use the shared handle-to-waveform conversion helper to turn those bound references into both `EventEvalFrame.tracked_signals` and waveform candidate sources;
   - collect candidate timestamps from the trigger-relevant signals, combining ordinary value-change timestamps with raw-event occurrence timestamps when the tracked set contains raw events;
   - evaluate `--eval` at the range start once to seed rolling property state for `switch`, `assert`, and `deassert` modes;
   - evaluate `--eval` at each candidate timestamp, reduce the result to the documented final decision (`1` true, `0/x` false), apply capture-mode filtering, and emit rows in deterministic time order.

   Define these runtime output types in `src/engine/property.rs` and wire them through `src/engine/mod.rs`, `src/output.rs`, and `schema/wavepeek.json`:

       pub enum PropertyResultKind {
           Match,
           Assert,
           Deassert,
       }

       pub struct PropertyCaptureRow {
           pub time: String,
           pub kind: PropertyResultKind,
       }

   Serialization requirements:

   - `PropertyResultKind` serializes as `match`, `assert`, or `deassert`.
   - Human output is one line per row in the form `@<time> <kind>`.
   - JSON output uses the standard envelope with `command: "property"` and `data` equal to the ordered array of rows.
   - `switch` never appears in output rows; it only selects whether both `assert` and `deassert` transitions are emitted.

   Error-priority rules:

   - clap/flag parsing errors remain `error: args:`;
   - expression parse, semantic, metadata, and runtime evaluation failures become `error: expr:`;
   - file open/format failures remain `error: file:`;
   - direct non-expression waveform lookup failures outside the expression pipeline stay in their existing categories.

   Update help text in both `src/cli/mod.rs` and `src/cli/property.rs` so `property` no longer says execution is unimplemented. The long help and flag help must describe the actual capture semantics, including that `switch` emits `assert` and `deassert` rows, and they must stay truthful to the branch state until the runtime and schema changes are already in place.

   Targeted commands after `property` lands:

       cargo test --test property_cli
       cargo test --test cli_contract property_accepts_capture_flag_in_cli_then_runs -- --exact
       cargo test output::tests
       make update-schema
       python3 scripts/check_schema_contract.py

5. Add parity, schema, and benchmark hardening.

   Update or add these files:

    - `tests/property_vcd_fst_parity.rs`
    - `tests/schema_cli.rs`
    - `tests/cli_contract.rs`
    - `tests/command_fixture_contract.rs`
    - `tests/AGENTS.md`
    - `bench/e2e/tests.json`
    - `bench/e2e/perf.py`
    - `bench/e2e/test_perf.py`
    - `docs/DESIGN.md`
    - `docs/DEVELOPMENT.md`
    - `docs/ROADMAP.md`
    - `docs/BACKLOG.md`
    - `CHANGELOG.md`

    Parity rules:

    - `tests/change_vcd_fst_parity.rs` must cover at least one typed `iff` request on the checked-in `m2_core.vcd` / `m2_core.fst` pair.
    - `tests/property_vcd_fst_parity.rs` must compare JSON `data` and `warnings` for at least one `match` request and one transition-style request on the same VCD/FST pair.
    - the command-runtime manifests and snapshots must stay in sync through `tests/command_fixture_contract.rs` so the phase has explicit locked positive and negative artifacts.
    - keep output ordering deterministic; if no rows match, the command should emit an empty `data` array and a warning only when the existing command contract calls for one.

   Only after runtime, schema, and help updates are green should this milestone update shipped-status docs and release collateral. That includes removing unimplemented wording from `docs/DESIGN.md`, closing the relevant backlog debt in `docs/BACKLOG.md`, updating `docs/ROADMAP.md` milestone wording, and adding the user-visible result to `CHANGELOG.md`.

    Benchmark rules:

    - extend `bench/e2e/tests.json` with at least two `property` scenarios that exercise real runtime work rather than trivial constant expressions;
    - update `bench/e2e/perf.py` and `bench/e2e/test_perf.py` so the compare gate fails when either mean or median regression for a matched test exceeds the configured threshold;
    - keep existing `change` scenarios intact so regressions on the legacy command workload remain visible;
    - run the compare gate against the maintained baseline before review, then refresh `bench/e2e/runs/baseline/` only after the last accepted review-fix commit.

   Suggested property benchmark buckets:

   - one explicit-edge trigger with a small logical predicate and `--capture match`;
   - one default-wildcard trigger that derives its tracked set from `--eval` and uses `--capture switch`.

   Pre-review benchmark gate:

       WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/command-integration-candidate --compare bench/e2e/runs/baseline
       python3 bench/e2e/perf.py compare --revised bench/e2e/runs/command-integration-candidate --golden bench/e2e/runs/baseline --max-negative-delta-pct 15

   Final baseline refresh after review:

       make bench-e2e-update-baseline

   Note that newly added `property` tests will appear as unmatched warnings in the first compare against the old baseline. That is acceptable before the baseline refresh; existing matched `change` and other command scenarios must still stay within the regression gate.

6. Commit atomic units and keep review fixes separate.

   Suggested commit split:

       git add docs/expression_lang.md docs/expression_roadmap.md docs/DESIGN.md docs/ROADMAP.md docs/BACKLOG.md tests/change_cli.rs tests/property_cli.rs tests/change_vcd_fst_parity.rs tests/property_vcd_fst_parity.rs tests/change_opt_equivalence.rs tests/cli_contract.rs tests/schema_cli.rs tests/common/command_cases.rs tests/common/mod.rs tests/command_fixture_contract.rs tests/AGENTS.md tests/fixtures/cli/AGENTS.md tests/fixtures/cli tests/snapshots/change_cli__expr_runtime_*.snap tests/snapshots/property_cli__expr_runtime_*.snap
       git commit -m "test(expr): lock command integration contract"

       git add src/error.rs src/engine/mod.rs src/engine/expr_runtime.rs
       git commit -m "refactor(engine): add typed expression command helpers"

       git add src/engine/change.rs src/expr/mod.rs src/expr/legacy.rs src/waveform/expr_host.rs src/waveform/mod.rs tests/change_cli.rs tests/change_vcd_fst_parity.rs
       git commit -m "feat(change): run triggers through typed engine"

       git add src/engine/property.rs src/cli/mod.rs src/cli/property.rs src/output.rs schema/wavepeek.json src/schema_contract.rs tests/property_cli.rs tests/property_vcd_fst_parity.rs tests/schema_cli.rs tests/cli_contract.rs
       git commit -m "feat(property): implement typed runtime and schema"

       git add bench/e2e/tests.json bench/e2e/perf.py bench/e2e/test_perf.py docs/DESIGN.md docs/DEVELOPMENT.md docs/ROADMAP.md docs/BACKLOG.md CHANGELOG.md
       git commit -m "bench(e2e): add property integration coverage"

       git add bench/e2e/runs/baseline
       git commit -m "bench(e2e): refresh baseline after command integration"

   If review finds issues, fix them in separate follow-up commits. Do not amend history.

7. Run validation gates.

   Focused expression carry-forward first:

       INSTA_UPDATE=no cargo test --test expression_parse
       INSTA_UPDATE=no cargo test --test expression_event_runtime
       INSTA_UPDATE=no cargo test --test expression_integral_boolean
       INSTA_UPDATE=no cargo test --test expression_rich_types
       cargo test --test expression_fixture_contract

   Then command integration and parity gates:

        cargo test --test change_cli
        cargo test --test property_cli
        cargo test --test change_vcd_fst_parity
        cargo test --test property_vcd_fst_parity
        cargo test --test change_opt_equivalence
        cargo test --test cli_contract
        cargo test --test schema_cli
        cargo test --test command_fixture_contract
        python3 scripts/check_schema_contract.py

    Then repository gates and performance:

        make check
        make ci
        WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/command-integration-candidate --compare bench/e2e/runs/baseline
        WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py compare --revised bench/e2e/runs/command-integration-candidate --golden bench/e2e/runs/baseline --max-negative-delta-pct 15
        make bench-e2e-run

   Expected success signature after implementation is complete:

       cargo test --test property_cli
       ...
       test result: ok.
       cargo test --test property_vcd_fst_parity
       ...
       test result: ok.
        python3 bench/e2e/perf.py compare --revised ... --golden ... --max-negative-delta-pct 15
        ok: compare: all checks passed (use --verbose for detailed logs)
       make ci
       ...
       test result: ok.

8. Run the mandatory review workflow.

   Load `ask-review` skill after the full validation suite is green. Use one short context packet that contains the scope summary, the exact commit range, the validation commands already run, and the known high-risk areas: command error mapping, `change` runtime swap, `property` capture semantics, schema/output shape, and benchmark drift.

   Run these focused lanes in parallel:

   - Code-semantics lane: verify `change` and `property` both use the typed expression engine end to end, check `iff`, wildcard tracking, capture-mode correctness, and negative coverage completeness.
   - Architecture/performance lane: verify the legacy parser is gone from production command paths, confirm fast paths are only prefilters, inspect waveform-host caching or repeated sampling costs, and review `bench/e2e` additions.
   - Docs/schema lane: verify `docs/DESIGN.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, `CHANGELOG.md`, `src/cli/mod.rs`, `src/output.rs`, and `schema/wavepeek.json` all describe the same shipped behavior.

   After fixing any findings, rerun the affected tests and benchmark commands, then run one fresh independent control pass on the consolidated diff. Treat review as complete only when each focused lane plus the fresh control pass reports no substantive remaining issues or every reported issue has been fixed, committed, and revalidated.

### Validation and Acceptance

Acceptance is complete only when all of the conditions below are true together:

- `change` no longer rejects `iff` with `error: args: iff logical expressions are not implemented yet`; instead, `change --on` uses the shared typed parser, binder, and evaluator for runtime trigger truth.
- The production `change` command no longer depends on `src/expr/legacy.rs` for parsing or event semantics.
- `property` executes end to end for all four capture modes and no longer returns the unimplemented runtime error.
- `property` human output is one deterministic line per row in the form `@<time> <kind>`, where `kind` is `match`, `assert`, or `deassert`.
- `property` JSON output uses the standard envelope with `command: "property"` and `data` equal to an ordered array of `{ "time": <string>, "kind": <enum> }` rows.
- `schema/wavepeek.json` includes the `property` command and validates its `data` payload shape.
- Expression-originated command failures now print `error: expr:` and preserve stable deterministic messages and exit code `1`.
- The carried-forward rich-type manifests still pass, proving that command integration did not regress the already delivered `C4` standalone engine.
- `tests/change_vcd_fst_parity.rs` and `tests/property_vcd_fst_parity.rs` prove equivalent VCD/FST payloads for at least one typed expression-driven scenario each.
- `tests/property_cli.rs` proves default wildcard tracking from `--eval`, all capture modes, and the range-start baseline rule for `switch`/`assert`/`deassert`.
- `tests/change_cli.rs` and `tests/property_cli.rs` both prove scoped short-name expressions under `--scope` and wildcard unions containing `*`.
- `tests/cli_contract.rs`, `src/cli/mod.rs`, and `src/cli/property.rs` no longer describe `property` as unimplemented.
- `tests/fixtures/cli/command_runtime_positive_manifest.json`, `tests/fixtures/cli/command_runtime_negative_manifest.json`, and `tests/command_fixture_contract.rs` exist and lock the command integration contract, including representative `error: expr:` snapshots.
- New or updated files under `tests/`, `src/`, `schema/`, `bench/`, and live docs use descriptive names and prose and do not introduce the temporary roadmap label `C5`.
- `bench/e2e/tests.json` contains property scenarios, `make bench-e2e-run` passes, and the compare gate reports no matched mean or median regression worse than the 15% phase default.
- `make check` and `make ci` pass after the last review-fix commit.
- Mandatory review lanes and the fresh control pass are clean, or all findings were fixed and rechecked.

TDD acceptance requirement: at least one `change_cli` test and at least one `property_cli` test must fail before the runtime changes land and pass afterward in the same branch history.

### Idempotence and Recovery

All planned changes are additive or in-place edits to source, docs, tests, schema artifacts, and benchmark catalogs. Re-running the CLI tests and standalone expression suites is safe. Re-running `make update-schema` is safe and should produce the same canonical artifact bytes when the code has not changed.

If the `change` runtime swap breaks payload parity or performance, recover in this order: first restore typed binding plus final `event_matches_at(...)` gating in the baseline path, then temporarily disable any typed fast path that does not preserve semantics, then rerun `cargo test --test change_cli`, `cargo test --test change_vcd_fst_parity`, and the benchmark compare before touching `property`.

If `property` capture behavior becomes ambiguous during implementation, keep the plan's documented baseline rule: evaluate once at range start, emit nothing for that baseline probe, and compare later candidate decisions against that rolling state. Do not silently switch to an out-of-range previous-candidate model without updating this plan and the design docs in the same commit.

If schema regeneration or contract checks fail, rerun `make update-schema` and `python3 scripts/check_schema_contract.py` before debugging runtime logic. Schema drift is expected while `property` output is being introduced; it is not a reason to postpone the artifact update.

If the pre-review `bench/e2e` compare reports a real regression on an existing matched test, fix or disable the regressing optimization before refreshing the baseline. Newly added `property` tests must not be used to explain away regressions on previously matched command cases.

### Artifacts and Notes

Expected modified or added files for `C5` implementation:

- `docs/expression_lang.md`
- `docs/expression_roadmap.md`
- `docs/DESIGN.md`
- `docs/DEVELOPMENT.md`
- `docs/ROADMAP.md`
- `docs/BACKLOG.md`
- `CHANGELOG.md`
- `src/error.rs`
- `src/engine/mod.rs`
- `src/engine/change.rs`
- `src/engine/property.rs`
- `src/engine/expr_runtime.rs` if shared helpers are introduced
- `src/cli/mod.rs`
- `src/cli/property.rs`
- `src/output.rs`
- `src/schema_contract.rs`
- `schema/wavepeek.json`
- `src/waveform/expr_host.rs` and possibly `src/waveform/mod.rs`
- `src/expr/mod.rs` and possibly `src/expr/legacy.rs` for cleanup only
- `tests/change_cli.rs`
- `tests/property_cli.rs`
- `tests/change_vcd_fst_parity.rs`
- `tests/property_vcd_fst_parity.rs`
- `tests/change_opt_equivalence.rs`
- `tests/AGENTS.md`
- `tests/cli_contract.rs`
- `tests/common/command_cases.rs` and possibly `tests/common/mod.rs`
- `tests/command_fixture_contract.rs`
- `tests/schema_cli.rs`
- `tests/fixtures/cli/AGENTS.md`
- `tests/fixtures/cli/command_runtime_positive_manifest.json`
- `tests/fixtures/cli/command_runtime_negative_manifest.json`
- `tests/snapshots/change_cli__expr_runtime_*.snap` and `tests/snapshots/property_cli__expr_runtime_*.snap`
- `bench/e2e/tests.json`
- `bench/e2e/perf.py`
- `bench/e2e/test_perf.py`
- `bench/e2e/runs/baseline/`

Before closing this plan, record these evidence excerpts here:

- Red phase from `cargo test --test change_cli change_iff_executes_end_to_end -- --exact` before the runtime swap:
      test change_iff_executes_end_to_end ... FAILED
      stderr: error: args: iff logical expressions are not implemented yet

- Red phase from `cargo test --test property_cli property_switch_capture_reports_transitions -- --exact` before `property` implementation:
      test property_switch_capture_reports_transitions ... FAILED
      stderr: error: unimplemented: `property` command execution is not implemented yet

- Green phase from `cargo test --test property_cli` after implementation:
      running ... tests
      test result: ok.

- Green parity evidence from `cargo test --test property_vcd_fst_parity`:
      test result: ok.

- Benchmark compare evidence:
      ok: compare: all checks passed (use --verbose for detailed logs)

- Review outcome:
      focused review lanes ran for code semantics, architecture/performance, and docs/schema
      fresh control pass reported no substantive remaining issues

### Interfaces and Dependencies

The final implementation must leave these stable interfaces and dependencies in place.

In `src/error.rs`, define a real expression-facing command error category:

    pub enum WavepeekError {
        Args(String),
        File(String),
        Scope(String),
        Signal(String),
        Expr(String),
        Internal(String),
        Unimplemented(&'static str),
    }

`WavepeekError::Expr` must format as `error: expr: ...` and use exit code `1`.

In `src/engine/mod.rs`, add `property` as a real runtime command:

    pub enum CommandName {
        Schema,
        Info,
        Scope,
        Signal,
        Value,
        Change,
        Property,
    }

    pub enum CommandData {
        Schema(String),
        Info(info::InfoData),
        Scope(Vec<scope::ScopeEntry>),
        Signal(Vec<signal::SignalEntry>),
        Value(value::ValueData),
        Change(Vec<change::ChangeSnapshot>),
        Property(Vec<property::PropertyCaptureRow>),
    }

In `src/engine/property.rs`, keep the runtime payload explicit and serialization-friendly:

    pub enum PropertyResultKind {
        Match,
        Assert,
        Deassert,
    }

    pub struct PropertyCaptureRow {
        pub time: String,
        pub kind: PropertyResultKind,
    }

In `src/output.rs`, add a human rendering branch equivalent to:

    CommandData::Property(rows) => rows
        .iter()
        .map(|row| format!("@{} {}", row.time, row.kind))
        .collect::<Vec<_>>()
        .join("\n")

In `schema/wavepeek.json`, add a `propertyData` definition and command-specific routing so `command: "property"` requires that data shape.

In the shared command-runtime helper module, whether it is a new `src/engine/expr_runtime.rs` or a tight helper inside existing files, provide crate-private functions with behavior equivalent to:

    pub(crate) struct ScopedExprHost<'a> {
        inner: &'a dyn crate::expr::ExpressionHost,
        scope: Option<&'a str>,
    }

    fn bind_waveform_event_expr(
        waveform: std::rc::Rc<std::cell::RefCell<crate::waveform::Waveform>>,
        scope: Option<&str>,
        source: &str,
    ) -> Result<(crate::waveform::expr_host::WaveformExprHost, crate::expr::BoundEventExpr), WavepeekError>;

    fn bind_waveform_logical_expr(
        host: &crate::waveform::expr_host::WaveformExprHost,
        source: &str,
    ) -> Result<crate::expr::BoundLogicalExpr, WavepeekError>;

    fn referenced_signal_handles(expr: &crate::expr::BoundLogicalExpr) -> Vec<crate::expr::SignalHandle>;

    fn event_expr_contains_wildcard(expr: &crate::expr::BoundEventExpr) -> bool;

    fn candidate_sources_for_handles(
        host: &crate::waveform::expr_host::WaveformExprHost,
        handles: &[crate::expr::SignalHandle],
    ) -> Result<Vec<crate::waveform::ExprResolvedSignal>, WavepeekError>;

The exact signatures may vary, but the final code must expose one deterministic way for command runtimes to do all of the following without reparsing source text: bind once through a scope-aware host view, collect wildcard tracking handles from `--eval`, detect whether any wildcard term is present anywhere in the bound event expression, convert handles back into waveform-level candidate sources, and map `ExprDiagnostic` values into `WavepeekError::Expr`.

When `scope` is active, the scope-aware host must preserve the current CLI rule that only short names are accepted. A dotted token such as `top.clk` under `--scope top` must stay a deterministic failure; the wrapper must prepend `scope.` only for undotted names.

`src/engine/change.rs` must continue to own snapshot payload assembly, warning behavior, max-row truncation, and tune-mode dispatch. `src/engine/property.rs` must own capture filtering and row emission. `src/waveform/expr_host.rs` remains the waveform-backed `ExpressionHost` implementation, but in `C5` it must support a command-owned shared `Waveform` handle so typed evaluation and candidate collection observe the same opened dump state. It also needs one crate-private accessor that lets command helpers recover waveform candidate metadata from typed `SignalHandle` values. Keep the borrow discipline explicit: host methods and command helpers must not hold overlapping mutable borrows of the shared `RefCell<Waveform>` across nested calls.

Revision note (2026-03-28 / OpenCode): initial `C5` ExecPlan authored from `docs/expression_roadmap.md`, current `C4` standalone-engine state, current command/runtime/schema gaps, and existing CLI benchmark conventions; revised after focused plan review to define scope-aware binding, wildcard tracking for any `*` term, shared waveform ownership, truthful doc-update sequencing, explicit tune-mode validation, and phase-local C5 CLI artifacts.
