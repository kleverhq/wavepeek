# Add native and pre-edge sampling modes for event-driven waveform commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. It is intentionally self-contained so a contributor can resume the work with only this file and the repository checkout.

## Purpose / Big Picture

Users debugging RTL handshakes and bus transfers often compare `wavepeek property --on 'posedge clk'` output with simulator logs or SystemVerilog assertions. Today `wavepeek` evaluates values at the dump timestamp where the clock edge appears. That is useful and must remain the default, but it differs from RTL and SVA-style sampled semantics where values are sampled immediately before the clock edge and values driven on that edge are normally observed by a property on the next clock edge.

After this change, `change` and `property` will accept `--sample-mode native|pre-edge`. `native` keeps the existing dump-native behavior. `pre-edge` is an opt-in mode for edge-triggered `--on` expressions that evaluates selected values at the previous dump timestamp strictly before the triggering edge while reporting the original edge timestamp. A user should be able to run the same `property` query once with `native` and once with `pre-edge` and observe a same-edge update move from the edge where it was dumped to the following edge, matching the common RTL/SVA debugging mental model.

## Non-Goals

This plan does not implement full SystemVerilog scheduler reconstruction. Waveform dumps do not preserve all Active, NBA, Observed, Reactive, or Postponed region ordering, so the implementation will be a dump-based approximation of pre-edge sampling.

This plan does not change the default behavior of existing commands. `native` remains the default so existing scripts and JSON consumers continue to see the same rows unless they opt in.

This plan does not add sampling modes to `value`, because `value` has no `--on` edge trigger. Documentation will explain that `value` is explicit timestamp sampling; users who need clock-edge pre-sampling should use `change` or `property`, or manually query an explicit timestamp before the edge if they know one.

This plan does not change JSON output shape. The new flag changes which rows are produced and which values appear in existing fields, but it does not add new JSON fields.

This plan does not optimize `pre-edge` through every existing fast path. Correctness is the first contract. `change --sample-mode pre-edge` may use a dedicated baseline path even when the native engine would select a fused or edge-fast path.

## Progress

- [x] (2026-06-19T20:04Z) Researched the relevant repository structure, command docs, expression runtime, event trigger binding, and existing `change`/`property` engine paths.
- [x] (2026-06-19T20:04Z) Confirmed GitHub issue #23 is titled "Clock-edge sampling semantics and RTL/SVA expectations" and asks for documentation and a decision on user-visible sampling modes.
- [x] (2026-06-19T20:04Z) Confirmed the chosen public mode names are `native` and `pre-edge`, with `native` as the compatibility default.
- [x] (2026-06-19T20:04Z) Drafted this ExecPlan with implementation, documentation, validation, and review steps.
- [ ] Commit this ExecPlan as a branch-local WIP artifact under `docs/tracker/wip/`.
- [ ] Run focused read-only review on this ExecPlan before implementation.
- [ ] Update this ExecPlan from review findings until no substantive plan blockers remain.
- [ ] Begin implementation only after the plan is reviewed and ready to execute.

## Surprises & Discoveries

- Observation: The expression runtime already has an `event_expr_is_edge_only` helper that accepts `posedge`, `negedge`, and `edge` terms and rejects wildcard and plain signal terms.
  Evidence: `src/engine/expr_runtime.rs` defines `event_expr_is_edge_only` over `BoundEventKind::Posedge`, `BoundEventKind::Negedge`, and `BoundEventKind::Edge`.

- Observation: `property` transition capture already documents that its baseline state is taken at `--from` and that moving `--from` can change visible `assert` and `deassert` rows.
  Evidence: `docs/public/commands/property.md` states, "Time bounds are inclusive. In transition modes, the baseline state is taken at `--from`."

- Observation: `change` has multiple optimized engines. A pre-edge implementation that only compares requested signal values at the raw edge timestamp would miss the main use case, because a register updated at edge T should first appear in pre-edge sampling at the next edge, where the register may not change at that exact timestamp.
  Evidence: `src/engine/change.rs` currently uses native fast paths selected by `select_engine_mode`; the planned pre-edge path must compare sampled values across selected edge firings rather than only immediate dump deltas at the edge timestamp.

## Decision Log

- Decision: Expose a shared CLI enum named `SampleMode` with values `native` and `pre-edge`.
  Rationale: `native` describes the dump-native timestamp behavior better than `current`, which can be misread as "the current release behavior" rather than "the value at the selected timestamp." `pre-edge` is short, explicit, and close to SystemVerilog's preponed or `#1step` mental model without promising exact simulator-region reconstruction.
  Date/Author: 2026-06-19 / Grin

- Decision: Keep `native` as the default for both `change` and `property`.
  Rationale: Existing scripts, tests, and workflows depend on current row timing and value sampling. Changing defaults would silently alter debug conclusions, which is exactly the sort of helpful surprise that sets fire to the bench.
  Date/Author: 2026-06-19 / Grin

- Decision: Permit `pre-edge` only when the user explicitly passes `--on` and every event term is `posedge`, `negedge`, or `edge`, with optional `iff` guards.
  Rationale: `pre-edge` is meaningful for synchronous edge-driven sampling. Plain signal triggers and wildcard triggers do not have a single clock edge whose pre-edge value should govern the query. Requiring explicit `--on` also avoids treating omitted `--on` as wildcard by accident.
  Date/Author: 2026-06-19 / Grin

- Decision: Evaluate edge detection and `iff` guards at the native trigger timestamp, but evaluate `property --eval` and `change --signals` values at the pre-edge sample timestamp in `pre-edge` mode.
  Rationale: The clock edge must be detected at the dump timestamp where it occurs; otherwise there is no edge to trigger on. SystemVerilog also treats clocking-event expressions differently from sampled assertion values: concurrent assertion values are sampled, while the clock expression controls when the assertion triggers. This preserves useful gated clock expressions such as `posedge clk iff rst_n`.
  Date/Author: 2026-06-19 / Grin

- Decision: Keep output row timestamps equal to the triggering edge timestamp in both modes.
  Rationale: Users ask "which clock edge did this property report on?" The value sample point changes in `pre-edge`, but the selected event remains the clock edge. Reporting the previous dump timestamp would make rows harder to compare with logs and SVA reports.
  Date/Author: 2026-06-19 / Grin

- Decision: For `pre-edge`, if an edge has no prior dump timestamp, skip that trigger for value evaluation.
  Rationale: There is no recorded value at T- to sample. Fabricating SystemVerilog default sampled values would require simulator-state knowledge that the dump does not contain.
  Date/Author: 2026-06-19 / Grin

- Decision: Implement `change --sample-mode pre-edge` with a correctness-first path that compares values across selected edge firings and uses the range start as the transition baseline.
  Rationale: This is the behavior users need for off-by-one-clock debugging. Reusing the native delta-oriented optimized paths would be easy and wrong, an old and popular engineering tradition best avoided here.
  Date/Author: 2026-06-19 / Grin

## Outcomes & Retrospective

No implementation outcome yet. The current artifact is a reviewed-ready design target: once review finishes, the next contributor should be able to implement the feature, tests, and documentation from this file alone.

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

A `Waveform` in `src/waveform/mod.rs` can find the previous dump timestamp with `previous_sample_time(raw_time)`. This returns the greatest dump timestamp strictly before `raw_time`, or `None` if no earlier timestamp exists. Wellen-backed VCD/FST waveforms expose indexed timestamp tables; FSDB may not always expose the same indexed fast path, so the plan should not depend on indexed access for correctness.

The public documentation source lives in `docs/public/`. It is embedded into the binary by `src/docs/mod.rs` using `include_dir!("$CARGO_MANIFEST_DIR/docs/public")`, so adding a Markdown topic under `docs/public/troubleshooting/` automatically packages it if the front matter is valid. Public command topics explain behavior and edge cases but should not duplicate exact generated flag tables. Exact flag syntax belongs to generated help from `clap`.

The relevant SystemVerilog concept is sampled value timing. In IEEE 1800-2023, the Preponed region is immediately before the current simulation time slot, and `#1step` sampling is equivalent to taking samples there. Clocking block inputs default to `#1step`, meaning they sample the value at the end of the previous time step before the clock event. Concurrent assertions use sampled values, generally from the Preponed region, and evaluate property expressions later in the Observed region. `wavepeek` cannot reconstruct the full scheduler from a dump, but `pre-edge` approximates this by sampling at the previous recorded dump timestamp before the selected edge.

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

At the end of this milestone, `property` still detects `--on` events at the native edge timestamp, but evaluates `--eval` at the previous dump timestamp in `pre-edge` mode. Row timestamps remain the native edge timestamp.

In `src/engine/property.rs`, keep the existing native path for `SampleMode::Native`. Introduce a small helper:

    fn value_sample_time_for_mode(
        waveform: &Waveform,
        sample_mode: SampleMode,
        trigger_timestamp: u64,
    ) -> Option<u64>

For `Native`, return `Some(trigger_timestamp)`. For `PreEdge`, return `waveform.previous_sample_time(trigger_timestamp)`. If `None` is returned, skip value evaluation for that trigger because no recorded T- sample exists.

In the loop over `candidate_schedule`, keep the existing call to `event_expr_matches` with `frame.timestamp == trigger timestamp`. That preserves edge detection and `iff` guard behavior at the event timestamp. After the event matches, compute `decision_timestamp` with the helper above and pass that timestamp to `eval_bound_logical_truth`.

For `CaptureMode::Match`, push a row at `trigger_timestamp` when the decision is true. For transition captures, preserve the existing documented baseline rule: initial `previous_state` is evaluated at `from_raw` for both modes, and selected event rows compare their decision against that state. If a trigger has no pre-edge sample timestamp, it does not update `previous_state`.

Add focused tests in `tests/property_cli.rs` using a tiny inline VCD. The fixture should have `clk`, `valid`, and optionally `data`. At `#5`, put a `posedge clk` and a same-timestamp `valid` update from 0 to 1. At `#15`, put the next `posedge clk` without changing `valid`. Then assert:

    property --on 'posedge clk' --eval valid --capture assert
    => @5ns assert

    property --on 'posedge clk' --eval valid --capture assert --sample-mode pre-edge
    => @15ns assert

Also add a test proving `edge clk` is accepted in `pre-edge` mode. With the same fixture, `edge clk` will see the value before the `#10` falling edge, so an assert may appear at `@10ns` depending on the exact fixture. Keep the expected output explicit.

Add negative tests for `pre-edge` with omitted `--on`, wildcard `--on '*'`, plain signal `--on valid`, and mixed `--on 'valid or posedge clk'`. These may be direct CLI assertions rather than shared manifest cases, because they are command-level argument contract tests.

### Milestone 3: Implement `change --sample-mode pre-edge`

At the end of this milestone, `change --sample-mode pre-edge --on 'posedge clk'` reports selected signal values sampled before each clock edge, but prints rows at the edge timestamps. A register that changes at edge T should appear in `change` output at the next selected edge, not at T.

Keep the existing native engine dispatch untouched for `SampleMode::Native` to preserve compatibility. For `SampleMode::PreEdge`, route to a new correctness-first function in `src/engine/change.rs`, for example:

    fn run_pre_edge(...same core inputs as run_baseline...) -> Result<ChangeRunOutput, WavepeekError>

This function should collect candidate times from edge event sources only. The earlier validation ensures wildcard and named signal triggers are impossible, so `candidate_sources` should come from `event_candidate_handles(&bound_event)` rather than requested signal changes. It should iterate candidate timestamps, call `event_expr_matches` at the native trigger timestamp, and only then sample requested signals at `waveform.previous_sample_time(trigger_timestamp)`. If there is no previous sample time, skip that trigger.

Maintain a persistent vector of previous requested values initialized from `baseline_raw` using native sampling at the range start. For each matching trigger with a valid pre-edge sample time, sample all requested signals at that pre-edge timestamp, compare against the persistent vector, update the persistent vector to the current sampled values, and emit a row at the trigger timestamp only when at least one requested value changed. This comparison is intentionally across selected trigger firings, not merely across adjacent dump timestamps.

Do not pass `pre-edge` through `run_fused` or `run_edge_fast` in this first implementation. The optimized native paths compare offsets and candidate changes in ways that are not equivalent to pre-edge event-stream sampling. A future performance pass can optimize after behavior is locked by tests.

Add focused tests in `tests/change_cli.rs` using the same inline VCD idea. Assert:

    change --signals data --on 'posedge clk'
    => row at @5ns with the same-timestamp data value

    change --signals data --on 'posedge clk' --sample-mode pre-edge
    => row at @15ns with that data value

Also add at least one negative CLI test for `change --sample-mode pre-edge --on data`, expecting a `fatal: args:` error.

### Milestone 4: Update public docs and troubleshooting

At the end of this milestone, installed docs explain why dump-native edge sampling differs from RTL/SVA-style sampled values, how to choose a mode, and why timestamps may look one clock off.

Update `docs/public/commands/property.md`. Add a short section near the mental model explaining `--sample-mode native|pre-edge`. It should say that `native` evaluates `--eval` at the selected dump timestamp, while `pre-edge` evaluates `--eval` immediately before an edge trigger and reports the edge timestamp. Mention that `pre-edge` requires explicit edge-only `--on` terms.

Update `docs/public/commands/change.md`. Add a section explaining that `native` samples displayed `--signals` at the trigger timestamp, and `pre-edge` samples displayed values before the trigger edge. Make clear that `change` rows remain delta snapshots over selected trigger firings, and `pre-edge` is useful when a registered value is driven at one edge but should be observed by RTL-like checks at the next edge.

Update `docs/public/reference/expression-language.md` only if needed to clarify that `--on` event detection is unchanged by sampling mode and that `iff` guards remain part of event selection. Do not duplicate command help.

Create a new troubleshooting topic, likely `docs/public/troubleshooting/clock-edge-sampling.md`, with front matter similar to existing topics. Use an ID such as `troubleshooting/clock-edge-sampling`, title `Clock-edge sampling`, and `see_also` links to `commands/change`, `commands/property`, and `reference/expression-language`.

The troubleshooting topic must explain the mismatch that issue #23 describes. Include concise ASCII diagrams. Avoid fenced code blocks inside this plan if quoting examples here, but the actual docs file can use normal Markdown fences if appropriate. The topic should include at least these concepts:

- A waveform dump records a value at a timestamp after the simulator has chosen what value is stored for that timestamp; `native` samples that stored value.
- RTL/SVA-style checks commonly sample before the clock edge. Values assigned by flip-flops or nonblocking assignments on that edge are therefore observed by the next edge.
- `pre-edge` approximates this by using the previous recorded dump timestamp before the edge.
- Row timestamps still name the edge that triggered the command.
- `pre-edge` does not duplicate one-cycle pulses and cannot reconstruct simulator regions that were not recorded in the dump.

An ASCII diagram should be similar to:

    time:        0ns      5ns      10ns     15ns
    clk:         0   ____/‾‾‾\____/‾‾‾\____
    valid dump:  0        1-------------------

    native @5ns:   valid=1
    pre-edge @5ns: valid=0
    pre-edge @15ns: valid=1

Update `docs/public/intro.md` only if a new troubleshooting topic should be easier to find from the documentation map. Usually the topic list/search is enough, so this may not be necessary.

### Milestone 5: Validate, review, and commit the implementation

At the end of this milestone, tests pass, docs are embedded cleanly, review findings are addressed, and the branch has commits that separate the plan from the implementation if practical.

Run targeted tests first from the repository root:

    cargo test --test property_cli -- property
    cargo test --test change_cli -- change
    cargo test --test docs_cli -- docs

Then run the local pre-handoff gate:

    just check

If time and environment allow, run the full standard gate:

    just ci

Use the `ask-review` skill after implementation. Start read-only reviewers in parallel for at least these lanes:

- Code lane focused on `src/cli/`, `src/engine/change.rs`, `src/engine/property.rs`, and tests.
- Docs lane focused on `docs/public/commands/change.md`, `docs/public/commands/property.md`, `docs/public/reference/expression-language.md`, and the new troubleshooting topic.
- Architecture/performance lane focused on the choice to route `change --sample-mode pre-edge` through a correctness-first baseline path and on avoiding accidental changes to native engine behavior.

Apply fixes in the main session, rerun relevant tests, and run one final control review pass on the consolidated diff. Stop when reviewers report no substantive findings or only minor nits that do not affect correctness, compatibility, or user-facing clarity.

## Concrete Steps

Work from the repository root:

    cd /workspaces/feat-rtl-sampling

Before editing implementation, confirm the plan is present and committed:

    git status --short
    git add docs/tracker/wip/rtl-sampling-execplan.md
    git commit -m "docs: add rtl sampling execution plan"

Implement the public enum and CLI fields:

    edit src/cli/mod.rs to add pub mod sampling;
    write src/cli/sampling.rs with the SampleMode enum
    edit src/cli/change.rs to add SampleMode to ChangeArgs
    edit src/cli/property.rs to add SampleMode to PropertyArgs

Add validation helpers in `src/engine/change.rs` and `src/engine/property.rs`. The logic should run after `bind_waveform_event_expr` so it can inspect `BoundEventExpr`, and before candidate collection so invalid pre-edge requests fail early.

Implement property pre-edge sampling by changing the evaluation timestamp passed to `eval_bound_logical_truth` while leaving event matching at the trigger timestamp.

Implement change pre-edge sampling as a separate path selected before native engine dispatch. Preserve all native engine behavior when `sample_mode == Native`.

Add or update tests. Inline VCD fixtures are acceptable for small command behavior tests and are already used in `tests/property_cli.rs` and `tests/change_cli.rs`.

Update docs. Each new public Markdown topic must have valid front matter with `id`, `title`, `description`, `section`, and `see_also` where appropriate.

Run format and tests through the repository gates. If a command fails, fix the issue and rerun the smallest failing command before rerunning `just check`.

## Validation and Acceptance

The main behavioral acceptance is this observable scenario. Given a VCD where `valid` and `data` are updated at the same timestamp as a `posedge clk`, native mode reports the condition at that same edge, while pre-edge mode reports it at the next edge:

    wavepeek property --waves <fixture> --scope top --on 'posedge clk' --eval valid --capture assert
    expected stdout:
      @5ns assert

    wavepeek property --waves <fixture> --scope top --on 'posedge clk' --eval valid --capture assert --sample-mode pre-edge
    expected stdout:
      @15ns assert

For `change`, the equivalent acceptance is:

    wavepeek change --waves <fixture> --scope top --signals data --on 'posedge clk'
    expected stdout contains:
      @5ns data=8'haa

    wavepeek change --waves <fixture> --scope top --signals data --on 'posedge clk' --sample-mode pre-edge
    expected stdout contains:
      @15ns data=8'haa

Validation must also prove invalid trigger forms fail:

    wavepeek property --waves <fixture> --scope top --eval valid --sample-mode pre-edge
    expected stderr starts with:
      fatal: args:
    expected stderr contains:
      --sample-mode pre-edge requires explicit --on

    wavepeek property --waves <fixture> --scope top --on '*' --eval valid --sample-mode pre-edge
    expected failure with the same diagnostic family

    wavepeek change --waves <fixture> --scope top --signals data --on data --sample-mode pre-edge
    expected failure with the same diagnostic family

Run the repository gates:

    just check

Expected result: formatting, linting, tests, docs embedding checks, and other configured local checks pass. If `just ci` is run, it should also pass in the devcontainer/CI image.

## Idempotence and Recovery

All edits are ordinary text changes and can be retried safely. If a partial implementation becomes confusing, use `git status --short` and `git diff` to inspect local changes before reverting anything. Do not delete files under `tmp/` or other WIP files under `docs/tracker/wip/` that were not created for this plan.

If a pre-edge implementation breaks native output tests, first confirm that `SampleMode::Native` follows the old code path. Native compatibility is mandatory. Fix native regressions before continuing with pre-edge behavior.

If docs embedding fails after adding the troubleshooting topic, inspect the front matter first. The docs loader requires stable topic IDs and valid metadata. Use existing `docs/public/troubleshooting/*.md` files as the shape reference.

If optimized `change` paths conflict with pre-edge behavior, keep pre-edge on the dedicated baseline path and document any performance tradeoff in `Surprises & Discoveries`. Do not contort the native fast paths until correctness tests are in place.

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

Important semantic distinction to preserve:

    trigger timestamp T: the dump timestamp where `posedge clk`, `negedge clk`, or `edge clk` is detected
    native sample:      values sampled at T
    pre-edge sample:    values sampled at previous dump timestamp strictly before T
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

For `pre-edge`, `current_samples` are sampled at the pre-edge timestamp but `trigger_timestamp` is passed to `build_snapshot` so output times remain edge times.

## Revision Notes

- 2026-06-19: Initial ExecPlan created. It records the user-approved `native` and `pre-edge` names, the edge-only `--on` rule including `edge clk`, and the correctness-first implementation strategy for `property` and `change`.
