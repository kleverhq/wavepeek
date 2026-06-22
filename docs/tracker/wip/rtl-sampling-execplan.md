# Add native and pre-edge sampling modes for event-driven waveform commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. It is intentionally self-contained so a contributor can resume the work with only this file and the repository checkout.

## Purpose / Big Picture

Users debugging register-transfer level designs often compare `wavepeek property --on 'posedge clk'` output with simulator logs or SystemVerilog assertions. In this document, RTL means register-transfer level design behavior; SVA means SystemVerilog Assertions; NBA means nonblocking assignment, the common `<=` assignment form whose update is scheduled later in the same simulator time slot. Today `wavepeek` evaluates values at the dump timestamp where the clock edge appears. That is useful and must remain the default, but it differs from RTL and SVA-style sampled semantics where values are sampled before the clock edge and values driven on that edge are normally observed by a property on the next clock edge.

After this change, `change` and `property` will accept `--sample-mode native|pre-edge`. `native` keeps the existing dump-native behavior. `pre-edge` is an opt-in mode for edge-triggered `--on` expressions that evaluates selected values at a query point strictly before the triggering edge while reporting the original edge timestamp. That query point is the previous representable raw dump tick before the edge; value sampling then resolves the last known value at or before that query point. A user should be able to run the same `property` query once with `native` and once with `pre-edge` and observe a same-edge update move from the edge where it was dumped to the following edge, matching the common RTL/SVA debugging mental model.

## Non-Goals

This plan does not implement full SystemVerilog scheduler reconstruction. A scheduler is the simulator's ordered set of work phases within one timestamp. Waveform dumps do not preserve all Active, NBA, Observed, Reactive, or Postponed region ordering, so the implementation will be a dump-based approximation of pre-edge sampling.

This plan does not change the default behavior of existing commands. `native` remains the default so existing scripts and JSON consumers continue to see the same rows unless they opt in.

This plan does not add sampling modes to `value`, because `value` has no `--on` edge trigger. Documentation will explain that `value` is explicit timestamp sampling; users who need clock-edge pre-sampling should use `change` or `property`, or manually query an explicit timestamp before the edge if they know one.

This plan does not change JSON output shape. The new flag changes which rows are produced and which values appear in existing fields, but it does not add new JSON fields.

This plan does not optimize `pre-edge` through every existing fast path. Correctness is the first contract. `change --sample-mode pre-edge` may use a dedicated baseline path even when the native engine would select a fused or edge-fast path. The implementation adds paired native/pre-edge E2E benchmark cases so reviewers can inspect timings in normal benchmark reports.

This plan does not refresh or add committed benchmark baseline run artifacts. Benchmark catalogs may change, but `bench/e2e/runs/baseline_fst/` and `bench/e2e/runs/baseline_fsdb/` should remain untouched unless a maintainer explicitly requests a baseline refresh.

## Progress

- [x] (2026-06-19T20:04Z) Researched the relevant repository structure, command docs, expression runtime, event trigger binding, and existing `change`/`property` engine paths.
- [x] (2026-06-19T20:04Z) Confirmed GitHub issue #23 is titled "Clock-edge sampling semantics and RTL/SVA expectations" and asks for documentation and a decision on user-visible sampling modes.
- [x] (2026-06-19T20:04Z) Confirmed the chosen public mode names are `native` and `pre-edge`, with `native` as the compatibility default.
- [x] (2026-06-19T20:04Z) Drafted this ExecPlan with implementation, documentation, validation, and review steps.
- [x] (2026-06-19T20:09Z) Committed this ExecPlan as branch-local WIP artifact `43ef77d docs: add rtl sampling execution plan`.
- [x] (2026-06-19T20:20Z) Ran focused read-only review lanes for code/design feasibility, docs/user semantics, and ExecPlan completeness.
- [x] (2026-06-19T20:24Z) Updated this ExecPlan from review findings: boundary behavior, FSDB-neutral pre-edge contract, exact fixture expectations, reference docs requirements, neutral wording, valid VCD vector syntax, and corrected boundary expectations.
- [x] (2026-06-19T20:30Z) Ran a final post-fix control review; it reported no substantive findings.
- [x] (2026-06-19T21:02Z) Updated the plan for paired native/pre-edge E2E benchmark cases without new baseline artifacts.
- [x] (2026-06-19T21:16Z) Implemented the core `--sample-mode native|pre-edge` CLI and engine behavior for `change` and `property`, including boundary tests and public command/troubleshooting docs.
- [x] (2026-06-19T21:16Z) Ran targeted behavior and docs tests: `cargo test --test property_cli -- property_sample_mode_pre_edge --test-threads=1`, `cargo test --test change_cli -- change_sample_mode_pre_edge --test-threads=1`, and `cargo test --test docs_cli -- docs --test-threads=1`.
- [x] (2026-06-19T21:33Z) Implemented paired native/pre-edge benchmark catalog coverage, generated FSDB catalog refresh, and maintainer docs.
- [x] (2026-06-19T21:33Z) Ran benchmark validation: `python3 -m unittest bench/e2e/test_perf.py`, `python3 -B tools/fsdb/generate_bench_catalog.py --check`, and `just bench-e2e-smoke-commit`.
- [x] (2026-06-19T21:49Z) Ran focused review lanes for code correctness, docs/user semantics, and benchmark/performance. Code review found a boundary-evaluation bug; docs review found two wording gaps; benchmark review reported no substantive findings.
- [x] (2026-06-19T21:53Z) Fixed review findings by skipping pre-edge transition boundary rows before evaluating pre-window values, adding a regression test with an error-producing pre-window sample, and clarifying docs.
- [x] (2026-06-19T21:53Z) Reran `cargo test --test property_cli -- property_sample_mode_pre_edge --test-threads=1` and `cargo test -q` after review fixes.
- [x] (2026-06-19T22:11Z) Ran final `just check`; it passed, including FSDB lint/build smoke in this Verdi-equipped container.
- [x] (2026-06-19T22:27Z) Ran a fresh final control review on the consolidated diff; it reported no substantive findings.
- [x] (2026-06-20T00:18Z) Refactored benchmark changes after maintainer feedback: removed the automatic sampling-mode comparison command and justfile gate, kept the paired native/pre-edge benchmark cases for manual inspection in normal run reports, and regenerated the FSDB catalog.
- [x] (2026-06-20T00:22Z) Validated the benchmark refactor with `python3 -m unittest bench/e2e/test_perf.py`, `python3 -B tools/fsdb/generate_bench_catalog.py --check`, and `just bench-e2e-smoke-commit`.
- [x] (2026-06-20T00:27Z) Ran `just check` after the benchmark refactor; it passed.
- [x] (2026-06-20T00:32Z) Ran focused benchmark-refactor review. It found stale ExecPlan wording only; after fixing that, a follow-up recheck reported no substantive findings.
- [x] (2026-06-20T20:09Z) Analyzed PR review comment about `property --sample-mode pre-edge` with raw event `.triggered()` operands. Reproduced that using the previous recorded timestamp could replay an older event at a later clock edge.
- [x] (2026-06-20T20:13Z) Fixed property pre-edge evaluation to use the raw tick immediately before the trigger edge, preserving value floor-sampling while making raw event `.triggered()` exact to that pre-edge tick.
- [x] (2026-06-20T20:13Z) Added a regression test showing an event at 5ns is not replayed for a 10ns clock edge, while an event at 19ns still matches the 20ns clock edge in pre-edge mode.
- [x] (2026-06-20T20:14Z) Ran `cargo test --test property_cli -- property_sample_mode_pre_edge --test-threads=1`; it passed.
- [x] (2026-06-20T20:18Z) Ran `cargo test -q`; it passed.
- [x] (2026-06-20T20:22Z) Ran focused code review for the raw-event pre-edge fix; it reported no substantive findings.
- [x] (2026-06-20T20:27Z) Ran `just check`; it passed.
- [x] (2026-06-22T00:00Z) Merged `origin/main` into `feat/rtl-sampling`, resolved conflicts in `docs/dev/benchmarking.md` and `src/engine/change.rs`, adapted `change --sample-mode pre-edge` to the streaming sink path from main, and updated release benchmark catalog sampling pairs to the new 10-run/5-warmup minimum.
- [x] (2026-06-22T00:00Z) Ran post-merge validation: `python3 -m unittest bench/e2e/test_perf.py`, `just check-bench-e2e-fsdb-catalog`, targeted `change`/`property` pre-edge CLI tests, and `just check`; all passed.

## Surprises & Discoveries

- Observation: The expression runtime already has an `event_expr_is_edge_only` helper that accepts `posedge`, `negedge`, and `edge` terms and rejects wildcard and plain signal terms.
  Evidence: `src/engine/expr_runtime.rs` defines `event_expr_is_edge_only` over `BoundEventKind::Posedge`, `BoundEventKind::Negedge`, and `BoundEventKind::Edge`.

- Observation: `property` transition capture already documents that its baseline state is taken at `--from` and that moving `--from` can change visible `assert` and `deassert` rows.
  Evidence: `docs/public/commands/property.md` states, "Time bounds are inclusive. In transition modes, the baseline state is taken at `--from`."

- Observation: `change` has multiple optimized engines. A pre-edge implementation that only compares requested signal values at the raw edge timestamp would miss the main use case, because a register updated at edge T should first appear in pre-edge sampling at the next edge, where the register may not change at that exact timestamp.
  Evidence: `src/engine/change.rs` currently uses native fast paths selected by `select_engine_mode`; the planned pre-edge path must compare sampled values across selected edge firings rather than only immediate dump deltas at the edge timestamp.

- Observation: The Wellen-backed VCD/FST implementation and the FSDB implementation currently use different predecessor mechanics. Wellen can return the previous recorded timestamp, while FSDB returns `raw_time - 1` when that is within dump bounds.
  Evidence: `src/waveform/wellen_backend.rs` implements `previous_sample_time` with the indexed time table; `src/waveform/fsdb_backend.rs` implements it by checking dump start and returning `raw_time.checked_sub(1)`.

- Observation: Review found that boundary edges need explicit handling. A trigger exactly at `--from` can have a pre-edge sample before the selected window; transition baselines must not be overwritten by that pre-window sample. The implementation review later tightened this: pre-edge transition captures must skip the boundary row before evaluating the pre-window value, because that value may be invalid or unsupported even though the `--from` baseline is valid.
  Evidence: Code/design review of this plan flagged the existing `property` `timestamp == from_raw` branch and the planned `change` pre-edge path as boundary risks; implementation review found `src/engine/property.rs` evaluated the pre-window decision before the boundary skip, and `tests/property_cli.rs` now covers `real'(sig)` with `sig=x` before `--from`.

- Observation: Control review caught that the original inline VCD used invalid vector literals and that boundary tests expecting `@15ns` rows contradicted the `--from` baseline rule.
  Evidence: The fixture now uses binary VCD vector values such as `b10101010 #`, and boundary acceptance now expects empty-result diagnostics at `--from 5ns --to 15ns` plus companion longer-window checks for `@35ns` transitions.

- Observation: Raw event operands in `property --eval`, such as `ev.triggered()`, are exact event queries rather than floor-sampled values. Passing the previous recorded timestamp as the pre-edge evaluation timestamp can replay an older event at a later clock edge.
  Evidence: A local VCD with `ev` at 5ns and `posedge clk` at 10ns produced a false pre-edge match before the fix. The regression test `property_sample_mode_pre_edge_does_not_replay_previous_raw_event` now expects no match at 10ns and a match only when `ev` occurs at 19ns for a 20ns edge.

- Observation: The existing E2E benchmark compare path checks revised runs against committed golden directories, but new sampling-mode benchmark cases have no committed golden artifacts because this plan intentionally avoids refreshing baselines.
  Evidence: `bench/e2e/perf.py compare` compares matching artifact names between revised and golden directories and only warns about tests that exist only in the revised run. After maintainer feedback, the final design intentionally keeps only paired benchmark catalog entries and relies on normal run reports for human timing inspection.

- Observation: `bench/e2e/tests_fsdb.json` is generated from the FST benchmark catalog and is checked by repository gates.
  Evidence: `docs/dev/benchmarking.md` documents `just update-bench-e2e-fsdb-catalog` and `just check-bench-e2e-fsdb-catalog`; changing `bench/e2e/tests.json` without regenerating the FSDB catalog would make `just check` fail.

- Observation: The lightweight commit benchmark catalog is currently tested as a command subset of the full FST catalog, and all existing non-sampling commit cases use one run and no warmup.
  Evidence: `bench/e2e/test_perf.py` contains `test_tests_commit_catalog_commands_match_tests_json` and `test_tests_commit_catalog_exact_subset_and_distribution`; the sampling-mode smoke pairs need matching full-catalog commands and should explicitly update these tests when their names, categories, runs, or warmup values change.

## Decision Log

- Decision: Expose a shared CLI enum named `SampleMode` with values `native` and `pre-edge`.
  Rationale: `native` describes the dump-native timestamp behavior better than `current`, which can be misread as "the current release behavior" rather than "the value at the selected timestamp." `pre-edge` is short, explicit, and close to SystemVerilog's preponed or `#1step` mental model without promising exact simulator-region reconstruction.
  Date/Author: 2026-06-19 / Grin

- Decision: Keep `native` as the default for both `change` and `property`.
  Rationale: Existing scripts, tests, and workflows depend on current row timing and value sampling. Changing defaults would silently alter debug conclusions and break compatibility.
  Date/Author: 2026-06-19 / Grin

- Decision: Permit `pre-edge` only when the user explicitly passes `--on` and every event term is `posedge`, `negedge`, or `edge`, with optional `iff` guards.
  Rationale: `pre-edge` is meaningful for synchronous edge-driven sampling. Plain signal triggers and wildcard triggers do not have a single clock edge whose pre-edge value should govern the query. Requiring explicit `--on` also avoids treating omitted `--on` as wildcard by accident.
  Date/Author: 2026-06-19 / Grin

- Decision: Evaluate edge detection and `iff` guards at the native trigger timestamp, but evaluate `property --eval` and `change --signals` values at the pre-edge sample timestamp in `pre-edge` mode.
  Rationale: The clock edge must be detected at the dump timestamp where it occurs; otherwise there is no edge to trigger on. SystemVerilog also treats clocking-event expressions differently from sampled assertion values: concurrent assertion values are sampled, while the clock expression controls when the assertion triggers. This preserves useful gated clock expressions such as `posedge clk iff rst_n`.
  Date/Author: 2026-06-19 / Grin

- Decision: Keep output row timestamps equal to the triggering edge timestamp in both modes.
  Rationale: Users ask "which clock edge did this property report on?" The value sample point changes in `pre-edge`, but the selected event remains the clock edge. Reporting the pre-edge query timestamp would make rows harder to compare with logs and SVA reports.
  Date/Author: 2026-06-19 / Grin

- Decision: Define `pre-edge` as sampling at a query point strictly before the edge, not necessarily at a recorded timestamp.
  Rationale: This is backend-neutral. VCD/FST indexed sampling may use the previous recorded timestamp because floor sampling at any point before the edge resolves to that value. FSDB can sample at `raw_time - 1` even when that is not itself a recorded change timestamp. Both produce the value immediately before the edge within dump precision.
  Date/Author: 2026-06-19 / Grin

- Decision: For `pre-edge`, evaluate `property --eval` at the raw tick immediately before the trigger edge when that tick is within dump bounds.
  Rationale: Ordinary value sampling already floors a raw query time to the last recorded value, so `T-1` preserves the intended pre-edge value behavior. Raw event `.triggered()` operands are exact event queries, so using `T-1` prevents an event from an older recorded timestamp from being replayed at a later clock edge.
  Date/Author: 2026-06-20 / Grin

- Decision: For `pre-edge`, if an edge has no representable query timestamp before it within dump bounds, skip that trigger for value evaluation.
  Rationale: There is no recorded or queryable T- value to sample. Fabricating SystemVerilog default sampled values would require simulator-state knowledge that the dump does not contain.
  Date/Author: 2026-06-19 / Grin

- Decision: Preserve transition baselines at `--from` and do not let a boundary edge at exactly `--from` overwrite the baseline with a pre-edge sample from before the window.
  Rationale: Existing property docs state that transition baselines are taken at `--from`. A pre-edge sample for an edge at `--from` may be before the selected window and must not redefine that baseline.
  Date/Author: 2026-06-19 / Grin

- Decision: Implement `change --sample-mode pre-edge` with a correctness-first path that compares values across selected edge firings and uses the range start as the transition baseline.
  Rationale: This is the behavior users need for off-by-one-clock debugging. The native delta-oriented optimized paths are not equivalent to pre-edge event-stream sampling.
  Date/Author: 2026-06-19 / Grin

- Decision: Keep native/pre-edge E2E performance coverage as paired benchmark catalog entries, not a custom automatic comparison command.
  Rationale: Maintainer feedback was that the paired benchmark cases are enough for this feature. Normal `perf.py run` reports show the timings for the native and pre-edge variants so reviewers can inspect them directly; extra comparison logic in benchmark scripts and just recipes is unnecessary machinery.
  Date/Author: 2026-06-20 / Grin

## Outcomes & Retrospective

Implementation is complete on branch `feat/rtl-sampling`. `change` and `property` now accept `--sample-mode native|pre-edge`; `native` remains the default, and `pre-edge` is accepted only with explicit edge-only triggers while preserving native trigger and `iff` evaluation. Public docs explain the native/pre-edge distinction, RTL/SVA-style one-clock mismatches, boundary behavior, and first-edge/no-predecessor skips. The benchmark catalogs now have paired native/pre-edge E2E cases for normal report-based timing inspection without refreshing committed baseline artifacts or adding custom comparison automation.

Validation passed with targeted CLI/docs tests, `cargo test -q`, `python3 -m unittest bench/e2e/test_perf.py`, `python3 -B tools/fsdb/generate_bench_catalog.py --check`, `just bench-e2e-smoke-commit`, and final `just check`. Focused code, docs, and performance reviews were run; the code/docs findings were fixed, and a final control review reported no substantive findings. The follow-up benchmark refactor removed custom comparison automation, passed focused benchmark tests and smoke validation, passed `just check`, and passed a focused post-refactor review after stale plan wording was fixed. The PR-review raw-event fix passed targeted property tests, `cargo test -q`, `just check`, and a focused code review.

## Context and Orientation

`wavepeek` is a Rust command-line tool for deterministic waveform inspection. The root `justfile` defines the standard gates. `just check` is the local pre-handoff gate and `just ci` is the full standard quality gate.

The relevant command-line argument structs live under `src/cli/`. `src/cli/change.rs` defines `ChangeArgs`, and `src/cli/property.rs` defines `PropertyArgs`. These structs use `clap` derives, so adding a field with `#[arg(long, value_enum, default_value_t = ...)]` creates the corresponding command-line flag and generated help text. `src/cli/mod.rs` contains the long command descriptions shown by `wavepeek --help` and `wavepeek help <command>`.

The command execution code lives under `src/engine/`. `src/engine/change.rs` implements the `change` command. It resolves requested signal names, binds the `--on` event expression, collects candidate timestamps, chooses a native engine path, and builds `ChangeSnapshot` rows. `src/engine/property.rs` implements the `property` command. It resolves `--on`, resolves `--eval`, collects event candidate timestamps, evaluates the property at selected timestamps, and builds `PropertyCaptureRow` rows.

The expression language lives under `src/expr/` and the waveform-backed expression host lives in `src/waveform/expr_host.rs`. A `BoundEventExpr` is the bound form of an `--on` expression. Its event terms include wildcard `*`, plain named signal changes, `posedge`, `negedge`, and `edge`. `src/engine/expr_runtime.rs` already exposes these important helpers:

    event_expr_contains_wildcard(expr)
    event_expr_is_any_tracked_only(expr)
    event_expr_is_edge_only(expr)
    event_expr_matches(source, expr, host, frame)
    eval_bound_logical_truth(source, expr, host, timestamp)

A `Waveform` in `src/waveform/mod.rs` can ask for a pre-edge sample query point with `previous_sample_time(raw_time)`. The existing method name says "sample time," but the backend behavior is best understood as "a valid query time strictly before `raw_time`." For Wellen-backed VCD/FST files this resolves through the indexed time table to the previous recorded timestamp. For FSDB it currently resolves to `raw_time - 1` when that is within dump bounds. Value sampling then returns the last known value at or before the query point. The implementation must rely on this query-time contract rather than assuming every backend returns a recorded change timestamp.

The public documentation source lives in `docs/public/`. It is embedded into the binary by `src/docs/mod.rs` using `include_dir!("$CARGO_MANIFEST_DIR/docs/public")`, so adding a Markdown topic under `docs/public/troubleshooting/` automatically packages it if the front matter is valid. Public command topics explain behavior and edge cases but should not duplicate exact generated flag tables. Exact flag syntax belongs to generated help from `clap`.

The end-to-end benchmark harness lives under `bench/e2e/`. `bench/e2e/perf.py` is a Python standard-library script that loads benchmark cases from JSON catalogs, runs `wavepeek` through `hyperfine`, captures JSON output, and can compare timing and functional artifacts against committed baselines. The default full FST catalog is `bench/e2e/tests.json`; the lightweight pre-commit smoke catalog is `bench/e2e/tests_commit.json`; committed baseline run artifacts live under `bench/e2e/runs/baseline_fst/`. This feature expands the catalogs with paired native/pre-edge cases, but it does not add or refresh committed baseline artifacts and does not add custom sampling-mode comparison automation.

The relevant SystemVerilog concept is sampled value timing. In IEEE 1800-2023, the Preponed region is immediately before the current simulation time slot, and `#1step` sampling is equivalent to taking samples there. Clocking block inputs default to `#1step`, meaning they sample the value at the end of the previous time step before the clock event. Concurrent assertions use sampled values, generally from the Preponed region, and evaluate property expressions later in the Observed region. `wavepeek` cannot reconstruct the full scheduler from a dump, but `pre-edge` approximates this by querying the waveform at the previous representable raw time before the selected edge and using normal waveform floor sampling from that point.

## Open Questions

There are no open product questions before implementation. The user has chosen the public mode names `native` and `pre-edge`, confirmed that `edge clk` should be allowed, and confirmed that wildcard and plain signal `--on` terms should be rejected for `pre-edge`.

Implementation may uncover edge cases in optimized `change` paths or FSDB support. If that happens, update `Surprises & Discoveries` and the plan before changing course.

## Plan of Work

### Milestone 1: Add the public CLI surface and validation

At the end of this milestone, `wavepeek help change` and `wavepeek help property` show `--sample-mode <MODE>` with `native` as the default, and both commands reject invalid `pre-edge` trigger forms before doing value evaluation.

Create a shared enum in a new file `src/cli/sampling.rs`:

    use clap::ValueEnum;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
    #[value(rename_all = "kebab-case")]
    pub enum SampleMode {
        #[default]
        Native,
        PreEdge,
    }

Add `pub mod sampling;` to `src/cli/mod.rs`.

In `src/cli/change.rs`, import `crate::cli::sampling::SampleMode` and add this field to `ChangeArgs` near `--on`, because it controls event-selected value sampling:

    /// Value sampling mode for event-selected rows
    #[arg(
        long,
        value_enum,
        default_value_t = SampleMode::Native,
        value_name = "MODE",
        help_heading = "Selection options"
    )]
    pub sample_mode: SampleMode,

Make the equivalent addition to `PropertyArgs` in `src/cli/property.rs`.

In `src/engine/change.rs` and `src/engine/property.rs`, add a validation helper after binding `--on` and before collecting candidates. The helper must reject `SampleMode::PreEdge` unless `args.on.is_some()` and `event_expr_is_edge_only(&bound_event)` is true. Use a message close to:

    --sample-mode pre-edge requires explicit --on with only edge event terms (posedge, negedge, or edge); wildcard and plain signal triggers are not supported

This helper should allow `posedge clk`, `negedge rst_n`, `edge clk`, `posedge clk iff rst_n`, and unions such as `posedge a or negedge b`. It should reject omitted `--on`, `--on '*'`, `--on ready`, and `--on 'ready or posedge clk'`.

Update `src/cli/mod.rs` long help for `change` and `property` to mention that value sampling defaults to dump-native behavior and that `pre-edge` is opt-in for edge-only triggers. Keep the text concise; detailed examples belong in docs.

Acceptance for this milestone is manual and testable: running `wavepeek help property` shows `--sample-mode <MODE>`, and running `wavepeek property --sample-mode pre-edge --on '*' ...` fails with `fatal: args:` and the message above.

### Milestone 2: Implement `property --sample-mode pre-edge`

At the end of this milestone, `property` still detects `--on` events at the native edge timestamp, but evaluates `--eval` at the pre-edge query timestamp in `pre-edge` mode. Row timestamps remain the native edge timestamp.

In `src/engine/property.rs`, keep the existing native path for `SampleMode::Native`. Introduce a small helper:

    fn value_sample_time_for_mode(
        waveform: &Waveform,
        sample_mode: SampleMode,
        trigger_timestamp: u64,
    ) -> Option<u64>

For `Native`, return `Some(trigger_timestamp)`. For `PreEdge`, return `waveform.previous_sample_time(trigger_timestamp)`. Treat the returned value as a pre-edge query timestamp, not necessarily a recorded change timestamp. If `None` is returned, skip value evaluation for that trigger because no representable T- query point exists within dump bounds.

In the loop over `candidate_schedule`, keep the existing call to `event_expr_matches` with `frame.timestamp == trigger timestamp`. That preserves edge detection and `iff` guard behavior at the event timestamp. After the event matches, compute `decision_timestamp` with the helper above and pass that timestamp to `eval_bound_logical_truth`.

For `CaptureMode::Match`, push a row at `trigger_timestamp` when the decision is true. For transition captures, preserve the existing documented baseline rule: initial `previous_state` is evaluated at `from_raw` for both modes, and selected event rows compare their decision against that state. If `sample_mode == PreEdge` and `trigger_timestamp == from_raw`, suppress transition output and do not update `previous_state` from the pre-edge decision; the baseline remains the value sampled at the range start. If a trigger has no pre-edge sample timestamp, it does not update `previous_state`.

Add focused tests in `tests/property_cli.rs` using this exact inline VCD content, written through the existing `write_fixture` helper:

    $date
      today
    $end
    $version
      wavepeek-rtl-sampling
    $end
    $timescale 1ns $end
    $scope module top $end
    $var wire 1 ! clk $end
    $var wire 1 " valid $end
    $var wire 8 # data $end
    $upscope $end
    $enddefinitions $end
    #0
    0!
    0"
    b00000000 #
    #5
    1!
    1"
    b10101010 #
    #10
    0!
    #15
    1!
    #20
    0!
    #25
    1!
    0"
    b01010101 #
    #30
    0!
    #35
    1!

With this fixture, assert these exact property outcomes:

    property --from 0ns --to 20ns --on 'posedge clk' --eval valid --capture assert
    => @5ns assert

    property --from 0ns --to 20ns --on 'posedge clk' --eval valid --capture assert --sample-mode pre-edge
    => @15ns assert

    property --from 0ns --to 20ns --on 'edge clk' --eval valid --capture assert --sample-mode pre-edge
    => @10ns assert

Add a boundary test with `--from 5ns --to 20ns --on 'posedge clk' --eval valid --capture assert --sample-mode pre-edge`. It should emit no rows and should produce the standard empty-result warning, because the native baseline at `5ns` is already true and the `#5` edge's pre-edge value from before the range must not overwrite that baseline. Add a companion `--from 5ns --to 35ns --capture deassert` run if you want to prove the next genuine sampled transition still appears; it should emit `@35ns deassert`.

Add negative tests for `pre-edge` with omitted `--on`, wildcard `--on '*'`, plain signal `--on valid`, and mixed `--on 'valid or posedge clk'`. These may be direct CLI assertions rather than shared manifest cases, because they are command-level argument contract tests.

### Milestone 3: Implement `change --sample-mode pre-edge`

At the end of this milestone, `change --sample-mode pre-edge --on 'posedge clk'` reports selected signal values sampled before each clock edge, but prints rows at the edge timestamps. A register that changes at edge T should appear in `change` output at the next selected edge, not at T.

Keep the existing native engine dispatch untouched for `SampleMode::Native` to preserve compatibility. For `SampleMode::PreEdge`, route to a new correctness-first function in `src/engine/change.rs`, for example:

    fn run_pre_edge(...same core inputs as run_baseline...) -> Result<ChangeRunOutput, WavepeekError>

This function should collect candidate times from edge event sources only. The earlier validation ensures wildcard and named signal triggers are impossible, so `candidate_sources` should come from `event_candidate_handles(&bound_event)` rather than requested signal changes. It should iterate candidate timestamps, skip any `trigger_timestamp <= baseline_raw`, call `event_expr_matches` at the native trigger timestamp, and only then sample requested signals at `waveform.previous_sample_time(trigger_timestamp)`. Treat that value as a pre-edge query timestamp. If there is no previous sample time, skip that trigger.

Maintain a persistent vector of previous requested values initialized from `baseline_raw` using native sampling at the range start. For each matching trigger after the baseline with a valid pre-edge sample time, sample all requested signals at that pre-edge query timestamp, compare against the persistent vector, update the persistent vector to the current sampled values, and emit a row at the trigger timestamp only when at least one requested value changed. This comparison is intentionally across selected trigger firings, not merely across adjacent dump timestamps.

Do not pass `pre-edge` through `run_fused` or `run_edge_fast` in this first implementation. The optimized native paths compare offsets and candidate changes in ways that are not equivalent to pre-edge event-stream sampling. A future performance pass can optimize after behavior is locked by tests.

Add focused tests in `tests/change_cli.rs` using the exact inline VCD shown in Milestone 2. Assert these exact outcomes:

    change --from 0ns --to 15ns --signals data --on 'posedge clk'
    => one row, @5ns data=8'haa

    change --from 0ns --to 15ns --signals data --on 'posedge clk' --sample-mode pre-edge
    => one row, @15ns data=8'haa

Add a boundary test with `--from 5ns --to 15ns --signals data --on 'posedge clk' --sample-mode pre-edge`. It should emit no rows and should produce the standard empty-result warning, because the native baseline at `5ns` is already `8'haa` and the `#5` edge's pre-edge value from before the range must not overwrite that baseline. Add a companion `--from 5ns --to 35ns` run if you want to prove the next genuine sampled delta still appears; it should emit `@35ns data=8'h55`. Also add at least one negative CLI test for `change --sample-mode pre-edge --on data`, expecting a `fatal: args:` error.

### Milestone 4: Update public docs and troubleshooting

At the end of this milestone, installed docs explain why dump-native edge sampling differs from RTL/SVA-style sampled values, how to choose a mode, and why timestamps may look one clock off.

Update `docs/public/commands/property.md`. Add a short section near the mental model explaining `--sample-mode native|pre-edge`. It should say that `native` evaluates `--eval` at the selected dump timestamp, while `pre-edge` evaluates `--eval` at a query point strictly before an edge trigger and reports the edge timestamp. Say this is a dump-based approximation of RTL/SVA-style pre-edge sampling, not full simulator-region reconstruction. Mention that `pre-edge` requires explicit edge-only `--on` terms.

Update `docs/public/commands/change.md`. Add a section explaining that `native` samples displayed `--signals` at the trigger timestamp, and `pre-edge` samples displayed values at a query point strictly before the trigger edge. Make clear that `change` rows remain delta snapshots over selected trigger firings, and `pre-edge` is useful when a registered value is driven at one edge but should be observed by RTL-like checks at the next edge.

Update `docs/public/reference/expression-language.md` with a concise reference note that sampling mode does not change event detection. In `pre-edge`, `--on` edge detection and any `iff` guard remain evaluated at the native trigger timestamp; only `property --eval` values and displayed `change --signals` values use the pre-edge query point. Do not duplicate generated command help or flag tables.

Create a new troubleshooting topic, likely `docs/public/troubleshooting/clock-edge-sampling.md`, with front matter similar to existing topics. Use an ID such as `troubleshooting/clock-edge-sampling`, title `Clock-edge sampling`, and `see_also` links to `commands/change`, `commands/property`, and `reference/expression-language`.

The troubleshooting topic must explain the mismatch that issue #23 describes. Include concise ASCII diagrams. Avoid fenced code blocks inside this plan if quoting examples here, but the actual docs file can use normal Markdown fences if appropriate. The topic should include at least these concepts:

- A waveform dump records a value at a timestamp after the simulator has chosen what value is stored for that timestamp; `native` samples that stored value.
- RTL/SVA-style checks commonly sample before the clock edge. Values assigned by flip-flops or nonblocking assignments on that edge are therefore observed by the next edge.
- `pre-edge` approximates this by using a query point strictly before the edge and normal waveform floor sampling from that point.
- Row timestamps still name the edge that triggered the command.
- First edges can be skipped when there is no representable pre-edge query point within dump bounds.
- A pre-edge query point may be before `--from` for a trigger exactly at the range start; transition modes keep the `--from` baseline and do not use that pre-window sample to redefine it.
- `pre-edge` does not duplicate one-cycle pulses and cannot reconstruct simulator regions that were not recorded in the dump.

An ASCII diagram should be similar to:

    time:        0ns      5ns      10ns     15ns
    clk:         0   ____/----\____/----\____
    valid dump:  0        1-------------------

    native @5ns:   valid=1
    pre-edge @5ns: valid=0
    pre-edge @15ns: valid=1

Update `docs/public/intro.md` only if a new troubleshooting topic should be easier to find from the documentation map. Usually the topic list/search is enough, so this may not be necessary.

### Milestone 5: Extend E2E benchmark coverage for both sampling modes

At the end of this milestone, the benchmark catalogs contain paired native and pre-edge cases that can be inspected in normal benchmark run reports. This milestone intentionally does not write or refresh any committed baseline run artifacts, and it does not add custom sampling-mode comparison automation to `bench/e2e/perf.py` or the `justfile`.

Add paired benchmark cases to `bench/e2e/tests.json`. Use edge-only commands because `pre-edge` is invalid for wildcard or plain-signal triggers. Clone the existing full-suite edge-triggered `change` case `change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk` into two new tests with names ending in `_sample_native` and `_sample_pre_edge`. Keep the same waveform, signal list, window, runs, and warmup, and append `--sample-mode native` to one command and `--sample-mode pre-edge` to the other. Include simple metadata such as `"sampling_mode": "native"` or `"sampling_mode": "pre-edge"` for report readability.

Also add a property pair to `bench/e2e/tests.json` by cloning `property_chipyard_clusteredrocketconfig_dhrystone_window_2us_match_posedge_clk` into `_sample_native` and `_sample_pre_edge` variants. This covers both commands that expose the new flag.

Add lightweight smoke pairs to `bench/e2e/tests_commit.json` so pre-commit benchmark smoke covers the new path. Use existing SCR1 FST fixtures already present in that catalog and explicit edge triggers such as `--on posedge TOP.clk`. Choose signal and expression targets that produce non-timeout, non-degenerate artifacts in both modes; do not benchmark only the clock signal if that collapses one mode to nearly empty output. Add paired `change` entries for `native` and `pre-edge` and, if runtime remains acceptable, a small paired `property` entry as well. Add matching entries with the same names and commands to `bench/e2e/tests.json`, or otherwise deliberately update the existing subset invariant test; preserving the subset invariant is preferred. These sampling-mode smoke entries may use more repetitions than the existing one-run smoke entries, such as `runs=5` and `warmup=1`, to make the normal report useful. Update `bench/e2e/test_perf.py` because it currently asserts the exact `tests_commit.json` name set, category distribution, command subset, and run/warmup values.

After changing `bench/e2e/tests.json`, regenerate `bench/e2e/tests_fsdb.json` with:

    just update-bench-e2e-fsdb-catalog

Then run:

    just check-bench-e2e-fsdb-catalog

This is a generated catalog update, not a benchmark baseline refresh. It may change `bench/e2e/tests_fsdb.json`; it must not change files under committed baseline run directories.

Update `docs/dev/benchmarking.md` only to say that paired sampling-mode cases are present for normal report-based timing inspection. Do not add a new `perf.py` command, a pair-matching threshold, or extra justfile gate around these cases.

Acceptance for this milestone is:

    python3 -m unittest bench/e2e/test_perf.py

passes, and after building a release binary in the devcontainer or CI image:

    just bench-e2e-smoke-commit

runs the paired native/pre-edge cases successfully. No files under `bench/e2e/runs/baseline_fst/` or `bench/e2e/runs/baseline_fsdb/` should be modified.

### Milestone 6: Validate, review, and commit the implementation

At the end of this milestone, tests pass, docs are embedded cleanly, benchmark smoke passes, review findings are addressed, and the branch has commits that separate the plan from the implementation if practical.

Run targeted tests first from the repository root:

    cargo test --test property_cli -- property
    cargo test --test change_cli -- change
    cargo test --test docs_cli -- docs
    python3 -m unittest bench/e2e/test_perf.py

Then run the local pre-handoff gate:

    just check

Run the benchmark smoke gate in the devcontainer or CI image with RTL artifacts available:

    just bench-e2e-smoke-commit

If time and environment allow, run the full standard gate and the full E2E benchmark run:

    just ci
    just bench-e2e-run

Use the `ask-review` skill after implementation. Start read-only reviewers in parallel for at least these lanes:

- Code lane focused on `src/cli/`, `src/engine/change.rs`, `src/engine/property.rs`, and tests.
- Docs lane focused on `docs/public/commands/change.md`, `docs/public/commands/property.md`, `docs/public/reference/expression-language.md`, `docs/public/troubleshooting/clock-edge-sampling.md`, and `docs/dev/benchmarking.md`.
- Architecture/performance lane focused on the choice to route `change --sample-mode pre-edge` through a correctness-first baseline path, avoiding accidental changes to native engine behavior, and the paired benchmark coverage.

Apply fixes in the main session, rerun relevant tests, rerun benchmark smoke when performance-related code or catalog changes, and run one final control review pass on the consolidated diff. Stop when reviewers report no substantive findings or only minor nits that do not affect correctness, compatibility, performance coverage, or user-facing clarity.

## Concrete Steps

Work from the repository root:

    cd /workspaces/feat-rtl-sampling

Before editing implementation, confirm the reviewed plan is present and the worktree is clean:

    git status --short
    git log --oneline -3

Expected state before implementation starts: no uncommitted files, and the latest commit is this reviewed plan update. Do not commit the same plan again unless you intentionally revise it. If this file has just been edited from review findings, commit it before changing implementation files.

Implement the public enum and CLI fields:

    edit src/cli/mod.rs to add pub mod sampling;
    write src/cli/sampling.rs with the SampleMode enum
    edit src/cli/change.rs to add SampleMode to ChangeArgs
    edit src/cli/property.rs to add SampleMode to PropertyArgs

Add validation helpers in `src/engine/change.rs` and `src/engine/property.rs`. The logic should run after `bind_waveform_event_expr` so it can inspect `BoundEventExpr`, and before candidate collection so invalid pre-edge requests fail early.

Implement property pre-edge sampling by changing the evaluation timestamp passed to `eval_bound_logical_truth` while leaving event matching at the trigger timestamp.

Implement change pre-edge sampling as a separate path selected before native engine dispatch. Preserve all native engine behavior when `sample_mode == Native`.

Add or update tests. Inline VCD fixtures are acceptable for small command behavior tests and are already used in `tests/property_cli.rs` and `tests/change_cli.rs`. Use the exact VCD in Milestone 2 for the core native/pre-edge assertions so expected rows are not left to interpretation.

Update docs. Each new public Markdown topic must have valid front matter with `id`, `title`, `description`, `section`, and `see_also` where appropriate.

Extend E2E benchmark coverage without adding comparison automation:

    edit bench/e2e/tests.json to add full-suite native/pre-edge pairs for change and property, including any names used by tests_commit.json
    edit bench/e2e/tests_commit.json to add smoke native/pre-edge pairs with enough runs and warmup for useful report inspection
    run just update-bench-e2e-fsdb-catalog after editing tests.json
    edit bench/e2e/test_perf.py to update catalog-shape assertions
    edit docs/dev/benchmarking.md to document that paired sampling-mode tests are inspected through normal reports

Run format and tests through the repository gates. If a command fails, fix the issue and rerun the smallest failing command before rerunning `just check`.

## Validation and Acceptance

The main behavioral acceptance is the exact VCD fixture from Milestone 2. It updates `valid` and `data` at the same timestamp as a `posedge clk`. Native mode reports the condition at that same edge, while pre-edge mode reports it at the next relevant edge:

    wavepeek property --waves <fixture> --from 0ns --to 20ns --scope top --on 'posedge clk' --eval valid --capture assert
    expected stdout:
      @5ns assert

    wavepeek property --waves <fixture> --from 0ns --to 20ns --scope top --on 'posedge clk' --eval valid --capture assert --sample-mode pre-edge
    expected stdout:
      @15ns assert

    wavepeek property --waves <fixture> --from 0ns --to 20ns --scope top --on 'edge clk' --eval valid --capture assert --sample-mode pre-edge
    expected stdout:
      @10ns assert

For `change`, the equivalent acceptance is:

    wavepeek change --waves <fixture> --from 0ns --to 15ns --scope top --signals data --on 'posedge clk'
    expected stdout:
      @5ns data=8'haa

    wavepeek change --waves <fixture> --from 0ns --to 15ns --scope top --signals data --on 'posedge clk' --sample-mode pre-edge
    expected stdout:
      @15ns data=8'haa

Boundary acceptance must also be explicit:

    wavepeek property --waves <fixture> --from 5ns --to 20ns --scope top --on 'posedge clk' --eval valid --capture assert --sample-mode pre-edge
    expected stdout is empty and stderr contains:
      warning[WPK-W0003]: no property matches found in selected time range

    wavepeek property --waves <fixture> --from 5ns --to 35ns --scope top --on 'posedge clk' --eval valid --capture deassert --sample-mode pre-edge
    expected stdout:
      @35ns deassert

    wavepeek change --waves <fixture> --from 5ns --to 15ns --scope top --signals data --on 'posedge clk' --sample-mode pre-edge
    expected stdout is empty and stderr contains:
      warning[WPK-W0003]: no signal changes found in selected time range

    wavepeek change --waves <fixture> --from 5ns --to 35ns --scope top --signals data --on 'posedge clk' --sample-mode pre-edge
    expected stdout:
      @35ns data=8'h55

Validation must also prove invalid trigger forms fail:

    wavepeek property --waves <fixture> --scope top --eval valid --sample-mode pre-edge
    expected stderr starts with:
      fatal: args:
    expected stderr contains:
      --sample-mode pre-edge requires explicit --on

    wavepeek property --waves <fixture> --scope top --on '*' --eval valid --sample-mode pre-edge
    expected failure with the same diagnostic family

    wavepeek property --waves <fixture> --scope top --on 'valid or posedge clk' --eval valid --sample-mode pre-edge
    expected failure with the same diagnostic family

    wavepeek change --waves <fixture> --scope top --signals data --on data --sample-mode pre-edge
    expected failure with the same diagnostic family

Run the repository gates:

    python3 -m unittest bench/e2e/test_perf.py
    just check-bench-e2e-fsdb-catalog
    just check

Run the E2E benchmark smoke gate in the devcontainer or CI image with RTL artifacts available:

    just bench-e2e-smoke-commit

Expected result: formatting, linting, tests, docs embedding checks, generated FSDB catalog checks, auxiliary tests, and benchmark smoke pass. The benchmark smoke must run paired native/pre-edge tests so their timings appear in normal reports. If `just ci` is run, it should also pass in the devcontainer/CI image.

## Idempotence and Recovery

All edits are ordinary text changes and can be retried safely. If a partial implementation becomes confusing, use `git status --short` and `git diff` to inspect local changes before reverting anything. Do not delete files under `tmp/` or other WIP files under `docs/tracker/wip/` that were not created for this plan.

If a pre-edge implementation breaks native output tests, first confirm that `SampleMode::Native` follows the old code path. Native compatibility is mandatory. Fix native regressions before continuing with pre-edge behavior.

If docs embedding fails after adding the troubleshooting topic, inspect the front matter first. The docs loader requires stable topic IDs and valid metadata. Use existing `docs/public/troubleshooting/*.md` files as the shape reference.

If optimized `change` paths conflict with pre-edge behavior, keep pre-edge on the dedicated baseline path and document any performance tradeoff in `Surprises & Discoveries`. Do not contort the native fast paths until correctness tests are in place.

Do not modify committed benchmark baseline run directories while implementing this feature. If a command or hook appears to require baseline updates, stop and inspect whether revised-only paired tests are being incorrectly compared to golden artifacts.

## Artifacts and Notes

The GitHub issue behind this plan is:

    https://github.com/kleverhq/wavepeek/issues/23
    Title: Clock-edge sampling semantics and RTL/SVA expectations

The issue states that waveform timestamps are not the same as RTL or SVA sampling regions. It asks the project to document current behavior, explain how current queries sample the settled dump snapshot at the selected timestamp, clarify one-cycle interpretation issues, identify affected commands, and decide whether documentation alone or a user-visible sampling mode is needed.

Relevant existing source anchors:

    src/cli/change.rs              ChangeArgs and hidden tuning flags
    src/cli/property.rs            PropertyArgs and CaptureMode
    src/engine/change.rs           change execution and native engine dispatch
    src/engine/property.rs         property execution and capture logic
    src/engine/expr_runtime.rs     event expression helpers and evaluation wrappers
    src/waveform/mod.rs            previous_sample_time and sampling APIs
    docs/public/commands/change.md public change behavior docs
    docs/public/commands/property.md public property behavior docs
    docs/public/reference/expression-language.md trigger and eval semantics reference
    bench/e2e/tests.json           full E2E benchmark catalog
    bench/e2e/tests_commit.json    lightweight pre-commit E2E benchmark catalog
    bench/e2e/tests_fsdb.json      generated FSDB benchmark catalog derived from tests.json
    bench/e2e/perf.py              E2E benchmark runner and comparison harness
    bench/e2e/test_perf.py         unit tests for the E2E benchmark harness
    docs/dev/benchmarking.md       benchmark workflow documentation
    justfile                       benchmark smoke recipes

Important semantic distinction to preserve:

    trigger timestamp T: the dump timestamp where `posedge clk`, `negedge clk`, or `edge clk` is detected
    native sample:      values sampled at T
    pre-edge sample:    values sampled at a query timestamp strictly before T; floor sampling resolves the last known value at or before that query point
    output row time:    always T

## Interfaces and Dependencies

The new public CLI enum should be shared by command argument structs:

    pub enum SampleMode {
        Native,
        PreEdge,
    }

Both `ChangeArgs` and `PropertyArgs` should expose:

    pub sample_mode: SampleMode

The engine validation helper can be duplicated locally or shared from a small private function. It must use `event_expr_is_edge_only` from `src/engine/expr_runtime.rs` and must also require `args.on.is_some()` so omitted `--on` is not accepted as implicit wildcard.

The property engine must keep using:

    event_expr_matches(event_expr_source, &bound_event, &host, &frame)
    eval_bound_logical_truth(args.eval.as_str(), &bound_eval, &host, decision_timestamp)

The change engine must keep building existing `ChangeSnapshot` rows with:

    build_snapshot(requested_signals, current_samples.as_slice(), trigger_timestamp, dump_tick)

For `pre-edge`, `current_samples` are sampled at the pre-edge query timestamp but `trigger_timestamp` is passed to `build_snapshot` so output times remain edge times.

The E2E benchmark catalog entries for sampling coverage should use paired names ending in `_sample_native` and `_sample_pre_edge` and simple metadata such as:

    "sampling_mode": "native"

or:

    "sampling_mode": "pre-edge"

No custom pair-matching command is required in `bench/e2e/perf.py`; normal benchmark run reports are the timing comparison surface for these cases.

## Revision Notes

- 2026-06-19: Initial ExecPlan created. It records the user-approved `native` and `pre-edge` names, the edge-only `--on` rule including `edge clk`, and the correctness-first implementation strategy for `property` and `change`.
- 2026-06-19: Review update. Clarified backend-neutral pre-edge query semantics, `--from` boundary behavior, mandatory reference docs note for `iff`, exact test fixture expectations, and committed-plan execution steps.
- 2026-06-19: Control-review update. Replaced invalid VCD hex-like vector values with binary VCD vectors, corrected boundary expectations to preserve the `--from` baseline, and recorded that the final control pass had no substantive findings.
- 2026-06-19: Performance coverage update. Added required E2E benchmark catalog expansion and a no-baseline-refresh constraint.
- 2026-06-19: Implementation progress update. Recorded completion of the core CLI/engine/docs slice and its targeted test evidence before starting the benchmark coverage slice.
- 2026-06-19: Performance implementation progress update. Recorded completion of the benchmark catalog/docs slice and its smoke-gate evidence before final review.
- 2026-06-19: Review-fix update. Recorded focused review findings and the fix for pre-edge transition boundary evaluation before the final control pass.
- 2026-06-19: Completion update. Recorded final `just check`, final control review result, and the completed implementation outcome.
- 2026-06-20: Benchmark refactor update. Removed the custom sampling-mode comparison command and justfile gate after maintainer feedback; kept paired benchmark catalog entries for manual timing inspection. Recorded post-refactor validation and review results.
- 2026-06-20: PR review fix update. Recorded the raw event `.triggered()` pre-edge replay bug, changed property pre-edge evaluation to use the raw tick before the trigger edge, added the regression evidence, and recorded post-fix validation and review results.
- 2026-06-22: Main merge update. Recorded merge conflict resolution against updated `main`, including the `change` streaming sink integration and benchmark catalog run-count adjustment.
