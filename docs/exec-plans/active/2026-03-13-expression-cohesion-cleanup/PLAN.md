# Expression Cohesion Cleanup

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, the expression subsystem will read as one cohesive product surface instead of a fossil record of rollout phases. A contributor will be able to navigate `src/expr/`, `tests/`, `bench/expr/`, and the main status documents without running into `c1`, `c2`, `c3`, or `c4` prefixes, rollout-coded diagnostic identifiers, or benchmark collateral that advertises historical phase boundaries. The only places that will still speak in terms of `C1` through `C5` are the roadmap files that are explicitly about rollout history, plus completed execution-plan history: `docs/expression_roadmap.md`, `docs/ROADMAP.md`, and `docs/exec-plans/completed/`.

The observable outcome is concrete. A repository-wide search for expression rollout tags will stop finding live source, tests, benches, and status docs. The expression tests will run under feature-oriented names such as parser, event runtime, integral boolean, and rich types. Benchmark targets and exported run artifacts will use those same feature-oriented names. Expression diagnostics will use stable `EXPR-*` codes that describe the problem itself instead of the phase when that problem first appeared. The result will be visible through file names, command lines, diagnostic snapshots, benchmark manifests, and a hygiene guard that prevents phase tags from creeping back into non-roadmap areas.

## Non-Goals

This plan does not change expression language semantics from `docs/expression_lang.md`, does not integrate the standalone engine into default `change` or `property` execution, and does not remove the phase model from the documents that are explicitly about rollout sequencing. It does not add new expression features, does not change benchmark methodology beyond renaming and recapturing expression collateral, and does not preserve backwards-compatible aliases for old diagnostic codes or old benchmark target names. This is a deliberate cleanup, not a compatibility bridge.

This plan also does not leave behind temporary control-only artifacts. If implementation work creates a throwaway test, fixture, script, file, or helper only to inspect state, compare intermediate outputs, or drive a one-off migration, that artifact must be deleted before the milestone commit lands unless it is intentionally promoted into durable regression coverage, benchmark coverage, or a permanent hygiene guard.

## Progress

- [x] (2026-03-13 00:00Z) Audited `docs/expression_lang.md`, `docs/expression_roadmap.md`, `src/expr/`, `src/waveform/expr_host.rs`, `tests/`, `bench/expr/`, and the current benchmark/docs collateral to inventory every remaining expression phase tag outside roadmap/history surfaces.
- [x] (2026-03-13 00:00Z) Chose the end-state policy for this cleanup: rollout-phase language remains only in roadmap and execution-plan history, while all live expression code/tests/bench/docs use capability-oriented names plus stable `EXPR-*` diagnostics.
- [x] (2026-03-13 00:00Z) Drafted this active ExecPlan with explicit naming targets, migration order, validation gates, review lanes, and commit checkpoints.
- [x] (2026-03-13 00:00Z) Completed docs and architecture review plus a fresh control pass for the plan itself, then tightened the hygiene-guard policy to completed-history paths, added archive steps for stale active plans, and expanded the diagnostic-cleanup scope to include lexer-owned codes.
- [x] (2026-03-13 00:00Z) Incorporated the explicit repository rule that any temporary control-only tests, files, fixtures, scripts, or helpers created during execution must be removed before the milestone or final cleanup commit unless they are intentionally promoted into permanent coverage.
- [ ] Add a repository guard that fails when expression phase tags appear outside the allowed roadmap/history paths.
- [ ] Reorganize expression integration tests, fixture manifests, snapshots, and shared test helpers around feature-oriented names instead of `expression_c1` through `expression_c4`.
- [ ] Replace rollout-coded expression diagnostics and helper-only trap codes with stable `EXPR-*` and `TEST-*` identifiers, and rewrite messages so they describe current behavior instead of rollout history.
- [ ] Rename expression benchmark targets, scenario manifests, run artifacts, and benchmark-tool tests to the same capability-oriented taxonomy, then recapture committed run collateral under the new names.
- [ ] Rewrite live status and workflow documents, CLI boundary test names, waveform-host fixture names, and benchmark breadcrumb docs so they describe the current subsystem without phase language.
- [ ] Run the mandatory review workflow, fix findings in follow-up commits, move this plan into `docs/exec-plans/completed/`, rerun the final validation suite, and leave the worktree clean with only roadmap/history phase references remaining.

## Surprises & Discoveries

- Observation: phase tags are not confined to cosmetic file names; they currently leak into the asserted behavior of live tests.
  Evidence: `tests/fixtures/expr/c2_negative_manifest.json` expects `C3-SEMANTIC-UNKNOWN-SIGNAL`, and `tests/fixtures/expr/c4_negative_manifest.json` still expects `C3-SEMANTIC-UNKNOWN-SIGNAL` and `C3-SEMANTIC-EXPECTED-INTEGRAL` for current rich-type coverage.

- Observation: the expression test area already has one richer shared host helper, but the event-runtime suite still keeps a duplicate phase-local host and repeated manifest-loading utilities.
  Evidence: `tests/common/expr_runtime.rs` contains `InMemoryExprHost`, while `tests/expression_c2.rs` defines its own `InMemoryHost`, plus another copy of `fixture_expr_path(...)`, `load_positive_manifest(...)`, `load_negative_manifest(...)`, and `expected_layer(...)`.

- Observation: the benchmark layer mixes two generations of naming, so the cleanup must touch historical collateral, not only source filenames.
  Evidence: `bench/expr/scenarios/c1_parser.json` uses the newer scenario-set metadata shape, but `bench/expr/runs/c1-foundation-baseline/README.md` and `bench/expr/runs/c1-foundation-baseline/summary.json` still use older `foundation` naming and omit fields that the later `c2+` runs already record.

- Observation: rollout wording appears inside diagnostic messages and notes, not only in identifiers.
  Evidence: `tests/snapshots/expression_c2__c2_parse_unsupported_real_literal.snap` says `real literals are outside the C2 iff subset`, `tests/snapshots/expression_c3__c3_parse_deferred_real_cast.snap` says `real casts are deferred to C4`, and `src/expr/sema.rs` still has notes such as `replication with zero multiplier is not supported in C3`.

- Observation: live status docs still present current expression capabilities through phase labels, which makes the subsystem look mid-rollout even when the implementation surface is already cohesive.
  Evidence: `docs/DESIGN.md` says `standalone C4 tests/benchmarks`, and `docs/BACKLOG.md` says `standalone typed C4 event/logical runtime is now available`.

## Decision Log

- Decision: do a hard cleanup with no compatibility aliases for old phase-coded diagnostics, test file names, benchmark target names, or run-artifact directories.
  Rationale: the user explicitly wants phases to remain only in roadmap/history surfaces. Leaving aliases behind would preserve the old vocabulary in live code and would make future cleanup incomplete by design.
  Date/Author: 2026-03-13 / OpenCode

- Decision: standardize the live expression taxonomy on four capability buckets: parser, event runtime, integral boolean, and rich types.
  Rationale: those four buckets map cleanly onto the existing test and benchmark split, but they describe the subsystem by present-day behavior rather than by rollout phase. They also keep the current coverage partitioning small enough to review and validate independently.
  Date/Author: 2026-03-13 / OpenCode

- Decision: rename all rollout-coded expression diagnostics to stable `EXPR-*` identifiers, preserving the semantic suffix whenever possible but adding a capability noun when the old code was phase-scoped rather than meaning-scoped.
  Rationale: the diagnostic code is part of the subsystem contract. Stable problem-oriented names such as `EXPR-PARSE-EVENT-BROKEN-UNION` and `EXPR-RUNTIME-REAL-CAST` describe what happened without encoding historical rollout context.
  Date/Author: 2026-03-13 / OpenCode

- Decision: add a hygiene guard that scans repository text files for banned expression phase patterns outside an explicit allowlist.
  Rationale: without an automated guard, this cleanup would be one large rename with no long-term protection. A dedicated guard turns the cleanup into an enforceable repository rule.
  Date/Author: 2026-03-13 / OpenCode

- Decision: treat benchmark run artifacts as active collateral and recapture them under the new names instead of preserving old directories as an archive.
  Rationale: the benchmark directories live under `bench/expr/runs/` and are used as current performance baselines. Keeping old `c1` through `c4` directory names there would violate the cleanup goal even if the code changed everywhere else.
  Date/Author: 2026-03-13 / OpenCode

- Decision: use additive migration inside each milestone, then remove the old names before the milestone commit lands.
  Rationale: this keeps the work safe and reviewable. The implementer can create neutral files, move coverage over, prove parity, and only then delete the old phase-tagged files in the same atomic unit.
  Date/Author: 2026-03-13 / OpenCode

- Decision: temporary control-only artifacts must be cleaned up before final acceptance.
  Rationale: this cleanup is meant to make the repository feel cohesive, not to trade one kind of historical residue for another. Throwaway migration scripts, temporary fixtures, and one-off audit tests are acceptable only while they actively de-risk a milestone; they must either be deleted or explicitly promoted into permanent coverage before the milestone commit is finalized.
  Date/Author: 2026-03-13 / OpenCode

## Outcomes & Retrospective

Current status: planning only.

This plan captures the full cleanup scope needed to make expression work feel cohesive across code, diagnostics, tests, benchmarks, and status docs. The main risk is not semantic correctness but cross-cutting rename churn: diagnostic codes, snapshot names, benchmark artifacts, and file stems all move together. The plan therefore uses capability-oriented buckets, explicit commit checkpoints, and a final hygiene guard so the repository can end in a self-reinforcing clean state rather than in a one-time rename burst.

## Context and Orientation

`wavepeek` is a single-crate Rust repository. The expression engine itself lives in `src/expr/`, with public entry points such as `wavepeek::expr::parse_event_expr_ast(...)`, `wavepeek::expr::parse_logical_expr_ast(...)`, `wavepeek::expr::bind_event_expr_ast(...)`, `wavepeek::expr::bind_logical_expr_ast(...)`, `wavepeek::expr::event_matches_at(...)`, and `wavepeek::expr::eval_logical_expr_at(...)`. Those public APIs are already mostly phase-neutral. The cleanup is needed because the surrounding collateral still carries rollout names and rollout-coded diagnostics.

In this plan, a "phase tag" means any live expression artifact that encodes rollout history with `c1`, `c2`, `c3`, `c4`, `C1`, `C2`, `C3`, or `C4` in a name, diagnostic code, benchmark ID, or prose explanation. Examples include `tests/expression_c3.rs`, `bench/expr/expr_c4.rs`, `C3-SEMANTIC-CONST-REQUIRED`, and messages such as `outside the C2 iff subset`. A "capability-oriented name" is the replacement vocabulary that describes what the artifact covers today, such as parser, event runtime, integral boolean, or rich types.

The current live expression test layout is phase-oriented. `tests/expression_c1.rs` covers strict event parsing from JSON manifests. `tests/expression_c2.rs` covers standalone event runtime plus bounded `iff` behavior, but it duplicates its own in-memory host and helper utilities. `tests/expression_c3.rs` covers integral boolean semantics and some event reuse. `tests/expression_c4.rs` covers rich types, `.triggered`, and richer waveform-host behavior. Their fixtures live flat under `tests/fixtures/expr/` as `c1_positive_manifest.json` through `c4_negative_manifest.json`, and deterministic diagnostics are locked in `tests/snapshots/expression_c*__*.snap`.

The current live benchmark layout mirrors the same rollout vocabulary. `Cargo.toml` registers four bench targets: `expr_c1`, `expr_c2`, `expr_c3`, and `expr_c4`. Their implementations live in `bench/expr/expr_c1.rs` through `bench/expr/expr_c4.rs`, with scenario manifests such as `bench/expr/scenarios/c3_integral_boolean.json` and committed run artifacts such as `bench/expr/runs/c4-rich-types-baseline/`. Benchmark-tool tests under `bench/expr/test_capture.py` and `bench/expr/test_compare.py` also hard-code the old names in sample payloads.

The current diagnostic surface is also rollout-coded. `src/expr/parser.rs`, `src/expr/sema.rs`, and `src/expr/eval.rs` emit codes like `C1-PARSE-BROKEN-UNION`, `C3-SEMANTIC-CONST-REQUIRED`, and `C4-RUNTIME-REAL-CAST`. Helper-only test codes such as `C2-RUNTIME-UNEXPECTED-SAMPLE` and `C3-RUNTIME-UNEXPECTED-SAMPLE` appear in test hosts. Some messages and notes still describe features as deferred to a later phase rather than describing the actual present-day rule.

The status and workflow documents are split between the desired and undesired places for phase language. `docs/expression_roadmap.md`, `docs/ROADMAP.md`, and the completed plan folders under `docs/exec-plans/completed/` are allowed to keep phase terminology because they are explicitly about rollout sequencing and history. By contrast, `docs/DESIGN.md`, `docs/BACKLOG.md`, `docs/DEVELOPMENT.md`, active plans under `docs/exec-plans/active/`, and the benchmark breadcrumb docs under `bench/expr/` are live surfaces and must be rewritten or moved so they do not remain permanent exceptions.

At the time this plan was written, one older expression plan that already reports itself complete still lives under `docs/exec-plans/active/`: `docs/exec-plans/active/2026-03-12-expression-c4-rich-types-full-surface-closure/PLAN.md`. The cleanup cannot reach its final state unless that completed plan is archived into `docs/exec-plans/completed/` before the final hygiene pass.

Two more files matter even though they are not the core of the subsystem. `src/waveform/expr_host.rs` contains crate-private waveform-host tests with fixture strings like `wavepeek-c4` and temporary filenames like `rich-c4.vcd`; these must become neutral. `tests/change_cli.rs` and `tests/property_cli.rs` include boundary tests whose function names currently mention `c4`; these are live tests and must be renamed even though the underlying CLI behavior stays unchanged.

## Open Questions

There are no blocking product questions left for this cleanup. The naming policy is resolved: phases remain only in roadmap/history surfaces, and everything else must become capability-oriented and phase-neutral.

There are two implementation-time discipline questions that this plan resolves up front. First, if a rename can be done either by moving files or by creating new neutral files and deleting the old ones, choose the additive path so tests can be compared before deletion. Second, if benchmark run collateral has to be regenerated because directory names and metadata fields change, do not try to preserve the old run names in parallel; regenerate under the new names and update any compare tests or docs that mention the old ones.

## Plan of Work

Milestone 1 introduces the hygiene guard and reorganizes the live expression test surface so the repository stops teaching the old phase story through file names, manifests, and snapshots. At the end of this milestone, a contributor will run feature-oriented test files, read feature-oriented fixture names, and have an automated red/green signal for forbidden rollout tags.

Milestone 2 renames the expression diagnostic contract and helper-only trap codes. At the end of this milestone, every expression parse, semantic, and runtime diagnostic emitted by `src/expr/` will use `EXPR-*` codes and present-tense messages, and every affected unit test, integration test, and snapshot will be updated to the new names.

Milestone 3 renames the expression benchmark subsystem and recaptures its committed collateral. At the end of this milestone, `cargo bench` targets, scenario manifests, README breadcrumbs, run directories, summary metadata, and benchmark-tool tests will all use capability-oriented names.

Milestone 4 cleans the remaining live documentation and helper surfaces, archives any already-complete expression plans that are still sitting under `docs/exec-plans/active/`, and tightens the hygiene guard to one final temporary exemption for this in-flight plan. At the end of this milestone, every live surface except this active plan will be phase-neutral, and the guard will be one archive step away from its final roadmap/history-only allowlist.

Milestone 5 closes the work through validation, focused review lanes, follow-up fixes, archiving this plan into `docs/exec-plans/completed/`, a fresh control pass, and final commits. At the end of this milestone, the branch will contain a clean sequence of atomic commits whose messages describe the cleanup by subsystem area, and the hygiene guard will be strict with no active-plan exceptions left.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property` inside the repository devcontainer or the same CI image used by `make check` and `make ci`.

1. Lock the naming policy with a red hygiene guard and the target file layout.

   Add a new integration-style test file, `tests/expression_hygiene.rs`, that scans repository text files for banned expression phase patterns outside an explicit allowlist. The final allowlist must include only rollout-history paths:

   - `docs/expression_roadmap.md`
   - `docs/ROADMAP.md`
   - `docs/exec-plans/completed/**`

   During implementation, the guard may temporarily exempt the current active plan file `docs/exec-plans/active/2026-03-13-expression-cohesion-cleanup/PLAN.md` so the work can proceed. That temporary exemption must be removed during closeout after this plan is moved into `docs/exec-plans/completed/`.

   The banned-pattern scan must catch the live forms this cleanup is meant to remove, not every incidental `c1` string in the repository. At minimum, scan for:

   - expression file stems such as `expression_c1`, `expression_c2`, `expression_c3`, `expression_c4`;
   - benchmark target stems such as `expr_c1`, `expr_c2`, `expr_c3`, `expr_c4`;
   - rollout-coded engine diagnostics matching the generic family `C[1-5]-(PARSE|SEMANTIC|RUNTIME)-`;
   - manifest, scenario, run, or snapshot prefixes such as `c1_`, `c2_`, `c3_`, `c4_`, `c1-`, `c2-`, `c3-`, `c4-` when they occur in expression-related file names or snapshot IDs.

   Keep the guard implementation local to the repository; do not shell out to external tools during test execution. A Rust integration test that walks text files and uses a compiled regex set is sufficient.

   In the same step, define the target neutral names in code comments at the top of the new test so the migration has one canonical vocabulary:

   - `tests/expression_parse.rs`
   - `tests/expression_event_runtime.rs`
   - `tests/expression_integral_boolean.rs`
   - `tests/expression_rich_types.rs`
   - `bench/expr/expr_parser.rs`
   - `bench/expr/expr_event_runtime.rs`
   - `bench/expr/expr_integral_boolean.rs`
   - `bench/expr/expr_rich_types.rs`

   Red-phase command:

       cargo test --test expression_hygiene

   Expected red evidence before cleanup starts: the new guard fails and lists current offenders from `tests/`, `bench/expr/`, `src/expr/`, and `docs/DESIGN.md` or `docs/BACKLOG.md`.

2. Reorganize the integration tests, fixtures, snapshots, and shared helpers around the neutral taxonomy.

   Update or add these files:

   - `tests/expression_parse.rs` replacing `tests/expression_c1.rs`
   - `tests/expression_event_runtime.rs` replacing `tests/expression_c2.rs`
   - `tests/expression_integral_boolean.rs` replacing `tests/expression_c3.rs`
   - `tests/expression_rich_types.rs` replacing `tests/expression_c4.rs`
   - `tests/common/mod.rs`
   - `tests/common/expr_runtime.rs`
   - new shared manifest helper file `tests/common/expr_cases.rs`

   Rename the fixture manifests in `tests/fixtures/expr/` to match those four capability buckets:

   - `parse_positive_manifest.json`
   - `parse_negative_manifest.json`
   - `event_runtime_positive_manifest.json`
   - `event_runtime_negative_manifest.json`
   - `integral_boolean_positive_manifest.json`
   - `integral_boolean_negative_manifest.json`
   - `rich_types_positive_manifest.json`
   - `rich_types_negative_manifest.json`

   Rename snapshot files and snapshot IDs so they follow the same scheme. For example:

   - `tests/snapshots/expression_parse__parse_unmatched_open.snap`
   - `tests/snapshots/expression_event_runtime__semantic_unknown_signal.snap`
   - `tests/snapshots/expression_integral_boolean__semantic_const_required_part_select.snap`
   - `tests/snapshots/expression_rich_types__runtime_real_cast_unknown.snap`

   Do not preserve phase names in Rust test function names, case names, or manifest case names. Replace names like `event_grouping_stays_unsupported`, `event_level_grouping_stays_outside_c3`, `c2_short_circuit_subset_holds`, and `c4_triggered_and_rich_type_regressions_hold` with present-tense feature names such as `event_grouping_remains_invalid`, `event_runtime_short_circuit_holds`, and `rich_type_and_triggered_regressions_hold`.

   Consolidate repeated helper logic into `tests/common/expr_cases.rs`. That file must provide the repository-relative fixture loader, the `DiagnosticLayer` parser, and any small manifest record types that are duplicated today. Extend `tests/common/expr_runtime.rs` so the event-runtime suite can use the shared in-memory expression host instead of its current local `InMemoryHost`. Preserve the existing public test behavior; this is a structural cleanup, not a semantic rewrite.

   Keep the migration additive until parity is proven. Copy each old phase-named test file to its new neutral name, switch the helpers and fixtures over, run the target test file, and only then delete the old file in the same milestone. Apply the same approach to fixtures and snapshots.

   If a temporary comparator test, ad hoc fixture, or one-off migration helper is needed during this test move, keep it local to the working tree and delete it before the milestone commit unless it graduates into a permanent shared helper or durable regression test.

   Commands while moving each suite:

       cargo test --test expression_parse
       cargo test --test expression_event_runtime
       cargo test --test expression_integral_boolean
       cargo test --test expression_rich_types
       INSTA_UPDATE=no cargo test --test expression_parse -- --nocapture

   Acceptance for this milestone: the four neutral test files are the only live expression integration suites, the old `tests/expression_c*.rs` files and `tests/fixtures/expr/c*_*.json` files are gone, snapshot filenames are neutral, and the hygiene guard reports fewer offenders while the expression suites stay green.

   Commit this milestone as one atomic unit after the old phase-named test files, manifests, and snapshots are deleted. Recommended commit message:

       test(expr): reorganize suites by capability

3. Replace rollout-coded diagnostics and helper-only trap codes with stable present-day identifiers.

   Update these files at minimum:

   - `src/expr/lexer.rs`
   - `src/expr/parser.rs`
   - `src/expr/sema.rs`
   - `src/expr/eval.rs`
   - `src/expr/diagnostic.rs` only if helper routines need refactoring, not for a public API shape change
   - `tests/common/expr_runtime.rs`
   - `tests/expression_event_runtime.rs`
   - `tests/expression_integral_boolean.rs`
   - `tests/expression_rich_types.rs`
   - `src/waveform/expr_host.rs`
   - all affected snapshots under `tests/snapshots/` and `src/expr/snapshots/`

   Apply one systematic rename rule: all expression diagnostics emitted by the engine must start with `EXPR-<LAYER>-...`, where `<LAYER>` is `PARSE`, `SEMANTIC`, or `RUNTIME`, and the remainder names the actual problem. Keep helper-only test diagnostics separate as `TEST-*` so they cannot be confused with the engine contract.

   Use the old suffix meaning when it already names the real problem. Examples that must be applied literally:

   - `C1-PARSE-BROKEN-UNION` -> `EXPR-PARSE-EVENT-BROKEN-UNION`
   - `C1-PARSE-UNMATCHED-OPEN` -> `EXPR-PARSE-EVENT-UNMATCHED-OPEN`
   - `C3-PARSE-LOGICAL-EXPECTED` -> `EXPR-PARSE-LOGICAL-EXPECTED`
   - `C3-SEMANTIC-CONST-REQUIRED` -> `EXPR-SEMANTIC-CONST-REQUIRED`
   - `C3-SEMANTIC-EXPECTED-INTEGRAL` -> `EXPR-SEMANTIC-INTEGRAL-REQUIRED`
   - `C4-SEMANTIC-METADATA` -> `EXPR-SEMANTIC-METADATA`
   - `C4-RUNTIME-REAL-CAST` -> `EXPR-RUNTIME-REAL-CAST`
   - `C2-RUNTIME-UNEXPECTED-SAMPLE` and `C3-RUNTIME-UNEXPECTED-SAMPLE` -> one shared helper code such as `TEST-RUNTIME-UNEXPECTED-SAMPLE`

   In the same milestone, rewrite messages and notes so they describe present-day rules rather than rollout history. For example, replace `real literals are outside the C2 iff subset` with `real literals are not valid in this expression` only if the expression surface truly rejects them today; if the form is now valid, remove the negative case entirely. Replace `real casts are deferred to C4` with the real current rule or delete the obsolete test if the feature is now supported.

   Re-run all unit and integration tests that assert diagnostics. Update snapshots intentionally; do not use blanket snapshot acceptance without reviewing each diff.

   Commands for this milestone:

       cargo test expr::lexer::tests
       cargo test expr::parser::tests
       cargo test expr::sema::tests
       cargo test expr::eval::tests
       INSTA_UPDATE=always cargo test --test expression_parse
       INSTA_UPDATE=always cargo test --test expression_event_runtime
       INSTA_UPDATE=always cargo test --test expression_integral_boolean
       INSTA_UPDATE=always cargo test --test expression_rich_types

   Acceptance for this milestone: no `C1-`, `C2-`, `C3-`, or `C4-` diagnostic codes remain in the live expression engine or its test helpers, helper-only trap codes are neutral, and every changed snapshot reads like a current product rule instead of a rollout note.

   Commit this milestone as one atomic unit. Recommended commit message:

       refactor(expr): replace rollout-coded diagnostics

4. Rename the benchmark subsystem and recapture committed benchmark collateral.

   Update these files and directories:

   - `Cargo.toml`
   - `bench/expr/expr_parser.rs` replacing `bench/expr/expr_c1.rs`
   - `bench/expr/expr_event_runtime.rs` replacing `bench/expr/expr_c2.rs`
   - `bench/expr/expr_integral_boolean.rs` replacing `bench/expr/expr_c3.rs`
   - `bench/expr/expr_rich_types.rs` replacing `bench/expr/expr_c4.rs`
   - `bench/expr/scenarios/parser.json`
   - `bench/expr/scenarios/event_runtime.json`
   - `bench/expr/scenarios/integral_boolean.json`
   - `bench/expr/scenarios/rich_types.json`
   - `bench/expr/test_capture.py`
   - `bench/expr/test_compare.py`
   - every committed directory under `bench/expr/runs/`

   Rename helper constructors and local bench hosts inside the Rust bench files so they match the new bucket names. For example, `BenchHost::c3()` becomes `BenchHost::integral_boolean()`, and `RichBenchHost::c4()` becomes `RichBenchHost::rich_types()`.

   Replace benchmark run-directory names and metadata fields to remove phase prefixes. The committed artifact layout must end with these directory families:

   - `bench/expr/runs/parser-candidate`
   - `bench/expr/runs/parser-baseline`
   - `bench/expr/runs/parser-verify`
   - `bench/expr/runs/event-runtime-candidate`
   - `bench/expr/runs/event-runtime-baseline`
   - `bench/expr/runs/event-runtime-verify`
   - `bench/expr/runs/integral-boolean-candidate`
   - `bench/expr/runs/integral-boolean-baseline`
   - `bench/expr/runs/integral-boolean-verify`
   - `bench/expr/runs/rich-types-candidate`
   - `bench/expr/runs/rich-types-baseline`
   - `bench/expr/runs/rich-types-verify`
   - `bench/expr/runs/integral-boolean-carry-forward`

   Regenerate any run metadata whose schema shape still reflects the older C1 export format so every `summary.json` records `bench_target`, `scenario_set_id`, and `scenario_set_path`. Update the Python unit tests so their example payloads use the new names and still validate mismatch handling.

   Capture new artifacts only after the renamed bench files and scenario manifests are green in test mode.

   If you need a temporary export helper, rename shim, or analysis script to recapture the benchmark collateral safely, treat it as throwaway control infrastructure and delete it before the milestone commit unless it becomes part of the permanent benchmark workflow documented in `bench/expr/AGENTS.md` and `docs/DEVELOPMENT.md`.

   Commands for this milestone:

       cargo test --bench expr_parser
       cargo test --bench expr_event_runtime
       cargo test --bench expr_integral_boolean
       cargo test --bench expr_rich_types
       python3 -m unittest bench.expr.test_capture bench.expr.test_compare

       cargo bench --bench expr_parser -- --save-baseline parser-candidate --noplot
       cargo bench --bench expr_parser -- --save-baseline parser-baseline --noplot
       cargo bench --bench expr_parser -- --save-baseline parser-verify --noplot

       cargo bench --bench expr_event_runtime -- --save-baseline event-runtime-candidate --noplot
       cargo bench --bench expr_event_runtime -- --save-baseline event-runtime-baseline --noplot
       cargo bench --bench expr_event_runtime -- --save-baseline event-runtime-verify --noplot

       cargo bench --bench expr_integral_boolean -- --save-baseline integral-boolean-candidate --noplot
       cargo bench --bench expr_integral_boolean -- --save-baseline integral-boolean-baseline --noplot
       cargo bench --bench expr_integral_boolean -- --save-baseline integral-boolean-verify --noplot
       cargo bench --bench expr_integral_boolean -- --save-baseline integral-boolean-carry-forward --noplot

       cargo bench --bench expr_rich_types -- --save-baseline rich-types-candidate --noplot
       cargo bench --bench expr_rich_types -- --save-baseline rich-types-baseline --noplot
       cargo bench --bench expr_rich_types -- --save-baseline rich-types-verify --noplot

   Follow each `cargo bench` capture with `python3 bench/expr/capture.py ...` and `python3 bench/expr/compare.py ...` using the new bench targets, scenario manifests, and run-directory names. If `target/criterion` contains stale directories under the old names, remove only the affected expression bench subdirectories and rerun the same command before changing code.

   Acceptance for this milestone: `Cargo.toml` and the bench files expose only the neutral target names, benchmark-tool tests pass with the new IDs, committed run directories have neutral names and modern metadata, and no `expr_c*` or `c*-` run identifiers remain outside roadmap/history paths.

   Commit this milestone as one atomic unit. Recommended commit message:

       perf(expr): rename benchmark surfaces and artifacts

5. Rewrite live docs, helper names, and fixture strings, then make the hygiene guard pass.

   Update these files at minimum:

   - `docs/DESIGN.md`
   - `docs/BACKLOG.md`
   - `docs/DEVELOPMENT.md`
   - `bench/AGENTS.md`
   - `bench/expr/AGENTS.md`
   - `bench/expr/scenarios/AGENTS.md`
   - `bench/expr/runs/AGENTS.md`
   - `tests/change_cli.rs`
   - `tests/property_cli.rs`
   - `src/waveform/expr_host.rs`
   - `tests/expression_hygiene.rs`

   Rewrite prose in the live status docs so it describes current behavior rather than rollout stage. Examples that must be fixed:

   - `standalone C4 tests/benchmarks` -> `standalone expression tests/benchmarks` or `waveform-backed rich-type expression tests/benchmarks` depending on the sentence;
   - `This keeps C1 command behavior stable` -> describe the concrete compatibility boundary without naming `C1`;
   - benchmark workflow text must list the new bench target names rather than `expr_c1` through `expr_c4`.

   Rename the remaining helper and boundary-test identifiers that still carry phase tags. In particular:

   - `change_rich_c4_iff_payload_stays_deferred_runtime_error` must become a neutral boundary name;
   - `property_rich_c4_surface_stays_unimplemented` must become a neutral boundary name;
   - `wavepeek-c4` and `rich-c4.vcd` inside `src/waveform/expr_host.rs` must become neutral strings such as `wavepeek-expr-rich-types` and `rich-types.vcd`.

   Finish the milestone by tightening `tests/expression_hygiene.rs` as far as possible without blocking this still-active plan. Archive the already-complete expression plan `docs/exec-plans/active/2026-03-12-expression-c4-rich-types-full-surface-closure/PLAN.md` into `docs/exec-plans/completed/` so it no longer forces an exception for active plans, and reduce the temporary allowlist to only `docs/expression_roadmap.md`, `docs/ROADMAP.md`, `docs/exec-plans/completed/**`, and this current active plan path. If the implementation exposed any other false positives, fix the offending live files instead of widening the allowlist.

   Commands for this milestone:

       cargo test --test expression_hygiene
       cargo test --test change_cli
       cargo test --test property_cli
       cargo test --test expression_parse --test expression_event_runtime --test expression_integral_boolean --test expression_rich_types
       python3 -m unittest bench.expr.test_capture bench.expr.test_compare
       make check

   Acceptance for this milestone: the hygiene guard passes with only one temporary active-plan exemption left for this in-flight plan, the older completed expression plan has been archived out of `docs/exec-plans/active/`, the live docs and breadcrumb docs are phase-neutral, and helper/test identifiers no longer advertise rollout phases.

   Commit this milestone as one atomic unit. Recommended commit message:

       docs(expr): remove rollout language from live surfaces

6. Run the mandatory review and final validation workflow, then land follow-up fixes as separate commits and archive this plan into history.

   Use the `ask-review` skill for the implementation diff, not for this plan alone. This cleanup is multi-focus, so run parallel focused lanes after the previous milestones are green:

   - Code lane: correctness of renames, broken references, missed diagnostics, stale test loaders, snapshot mismatch risk.
   - Docs lane: wording in `docs/DESIGN.md`, `docs/BACKLOG.md`, `docs/DEVELOPMENT.md`, and benchmark breadcrumb docs.
   - Architecture lane: cohesion of new helper modules, whether the capability taxonomy is consistently applied, and whether the hygiene guard allowlist is appropriately narrow.
   - Performance lane: benchmark-target rename correctness, run-artifact compare integrity, and risk that recaptured collateral no longer matches scenario manifests.

   After applying lane findings, rerun only the impacted lanes, then run one fresh independent control pass on the consolidated diff. If a follow-up fix is needed, make it a new commit instead of amending the prior milestone commit.

   After the final review-fix commit, ensure no completed expression plans remain under `docs/exec-plans/active/`. At minimum, archive `docs/exec-plans/active/2026-03-12-expression-c4-rich-types-full-surface-closure/PLAN.md` if it is still present there. Then move this plan from `docs/exec-plans/active/2026-03-13-expression-cohesion-cleanup/PLAN.md` to `docs/exec-plans/completed/2026-03-13-expression-cohesion-cleanup/PLAN.md`, update its `Progress` and `Outcomes & Retrospective` sections to reflect completion, remove any temporary hygiene-guard exemption for the active-plan path, and then run the final validation commands.

   Final validation commands:

       cargo test expr::lexer::tests
       cargo test expr::parser::tests
       cargo test expr::sema::tests
       cargo test expr::eval::tests
       cargo test --test expression_parse --test expression_event_runtime --test expression_integral_boolean --test expression_rich_types --test expression_hygiene --test change_cli --test property_cli
       cargo test --bench expr_parser --bench expr_event_runtime --bench expr_integral_boolean --bench expr_rich_types
       python3 -m unittest bench.expr.test_capture bench.expr.test_compare
       make check
       make ci

   Expected final proof:

   - the hygiene guard passes and reports no forbidden expression phase tags outside the final allowlist;
   - all expression suites run under neutral file names and pass;
   - benchmark-tool tests pass and the renamed run-artifact compare gates succeed;
   - `make check` and `make ci` are green;
   - review lanes plus one fresh control pass report no substantive remaining issues;
   - this plan now lives under `docs/exec-plans/completed/`, and no completed expression plans remain under `docs/exec-plans/active/`, so active plans are no longer an exception to the hygiene rule.

   Commit strategy for review fixes:

       fix(expr): address review findings in <area>

   Keep each fix commit scoped to one review batch so the review trail is easy to follow.

### Validation and Acceptance

The cleanup is complete only when all four forms of evidence agree.

First, repository navigation must be visibly cleaner: the neutral file names, manifest names, snapshot names, benchmark targets, and run directories must exist, and the old `expression_c*`, `expr_c*`, `c*_manifest`, and `c*-baseline` names must be absent from live surfaces. Second, the engine contract must be visibly cleaner: diagnostic snapshots and assertion strings must use `EXPR-*` codes and present-tense rule descriptions. Third, the workflow must be visibly cleaner: `docs/DEVELOPMENT.md` and the benchmark breadcrumbs must instruct contributors to run the neutral target names. Fourth, the hygiene guard must prove the rule automatically by failing if a forbidden phase tag reappears outside the roadmap/history allowlist. Fifth, no temporary control-only tests, files, fixtures, or scripts created during the cleanup may remain unless they were explicitly promoted into permanent repository coverage.

The simplest human check after implementation is to run:

    cargo test --test expression_hygiene

and then search the repository manually for a spot-check:

    rg -n "expression_c[1-4]|expr_c[1-4]|C[1-5]-(PARSE|SEMANTIC|RUNTIME)|\bc[1-4](_|-)" docs src tests bench Cargo.toml

The expected result is either no output or matches only in `docs/expression_roadmap.md`, `docs/ROADMAP.md`, and `docs/exec-plans/completed/`.

### Idempotence and Recovery

This cleanup is safe to implement incrementally because each milestone is additive until the new neutral surface is green. If a test-file rename goes wrong, restore the neutral file from the old phase-named file, rerun the targeted `cargo test --test ...` command, and only delete the old file after parity is restored. If snapshot churn becomes hard to inspect, rerun a single test file with `INSTA_UPDATE=always` and review only the affected snapshot diff before moving on.

Benchmark collateral needs special care because `target/criterion` can retain stale directories. If capture or compare commands start mixing old and new names, delete only the affected expression bench directories under `target/criterion/` and rerun the same `cargo bench --bench ...` command. Do not delete committed run artifacts until their neutral replacements have been captured and verified.

The hygiene guard allowlist is the one place where being overly permissive would hide mistakes. If the guard flags a live path, treat that as a rename bug, not as a reason to widen the allowlist. Only roadmap/history paths may remain exempt. Apply the same discipline to temporary control artifacts: if a milestone needed a throwaway script or file to get unstuck, delete it before closing the milestone instead of normalizing it as accidental residue.

### Artifacts and Notes

Representative evidence snippets to capture during implementation:

    test expression_hygiene ... ok

    ok: no matched scenario exceeded 15.00% negative delta in mean or median

    parse:EXPR-PARSE-EVENT-UNMATCHED-OPEN: unmatched opening parenthesis
    --> span 0..1
    source: (posedge clk) or ready

    git log --oneline --decorate -5
    <commit for test reorg>
    <commit for diagnostics rename>
    <commit for benchmark rename>
    <commit for docs and guard>
    <commit for review fixes, if any>

Use short excerpts only. The proof should show that the cleanup changed both names and behavior-visible diagnostics, not just that files moved around.

### Interfaces and Dependencies

This plan intentionally avoids new public library interfaces. The expression public API in `src/expr/mod.rs` and the host trait in `src/expr/host.rs` should remain semantically the same except for diagnostic-code strings. The new structural interfaces are internal repository helpers:

In `tests/common/expr_cases.rs`, define shared manifest-loading helpers used by the four expression integration suites. At minimum, this helper file must expose a repository-relative `fixture_expr_path(...)` function, a string-to-`DiagnosticLayer` parser, and any small reusable record types that today live in multiple test files.

In `tests/expression_hygiene.rs`, define one test that walks repository text files, applies the allowlist, and fails with a readable offender list. Keep the implementation local and deterministic.

In `Cargo.toml`, the final bench registrations must be exactly these names and paths:

    [[bench]]
    name = "expr_parser"
    path = "bench/expr/expr_parser.rs"
    harness = false

    [[bench]]
    name = "expr_event_runtime"
    path = "bench/expr/expr_event_runtime.rs"
    harness = false

    [[bench]]
    name = "expr_integral_boolean"
    path = "bench/expr/expr_integral_boolean.rs"
    harness = false

    [[bench]]
    name = "expr_rich_types"
    path = "bench/expr/expr_rich_types.rs"
    harness = false

The benchmark scenario manifests and run summaries must record matching neutral identities. The four scenario-set IDs must be `parser`, `event_runtime`, `integral_boolean`, and `rich_types`.

Revision note (2026-03-13 / OpenCode): initial plan authored from the expression contract docs, current repository implementation state, targeted audits of tests/bench/docs/source, and the user requirement that rollout phases remain only in roadmap and execution-plan history. Revised after focused docs and architecture review plus a fresh control pass to narrow the final hygiene allowlist to roadmap plus completed-plan history, archive stale completed plans that still sit under `docs/exec-plans/active/`, and include lexer-owned diagnostics in the `EXPR-*` cleanup scope. Revised again after user feedback to make cleanup of temporary control-only tests, files, fixtures, and scripts an explicit success condition rather than an implied expectation.
