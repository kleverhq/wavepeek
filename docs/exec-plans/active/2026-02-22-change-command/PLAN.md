# Implement `change` Command End-to-End

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

This work turns `wavepeek change` from a documented stub into a working command that returns value snapshots over time. After completion, users and agents can ask, in one command, how selected signals evolve in a time window, either on any tracked transition or sampled at a clock posedge cadence.

The change is visible immediately from CLI behavior. `wavepeek change ...` must stop returning `error: unimplemented` and instead return deterministic human output by default and strict JSON output under `--json`, with stable warnings for truncation and empty matches.

## Non-Goals

This plan does not implement `at` or `when`, does not add new flags beyond the existing `change` surface, does not add alias compatibility, and does not redesign generic output policy outside what is needed for `change` data rendering.

## Progress

- [x] (2026-02-22 14:22Z) Drafted initial ExecPlan with milestones, acceptance criteria, and repository context.
- [x] (2026-02-22 14:41Z) Revised plan after review feedback: made command contract self-contained, added exact expected outputs/errors, and added milestone-level proof criteria.
- [x] (2026-02-22 14:52Z) Revised plan after independent control review: fixed canonical command invocation guidance and expanded JSON envelope acceptance checks.
- [x] (2026-02-22 16:02Z) Revised plan after final control findings: aligned milestone sequencing for help contract, added missing human/error acceptance cases, and added explicit exit-code verification steps.
- [ ] Add failing tests that define `change` behavior (JSON contract, warnings, clocked/unclocked semantics, args errors).
- [ ] Implement waveform-layer primitives for time windowing, signal resolution, and value sampling needed by `change`.
- [ ] Implement engine-layer `change` execution and integrate output rendering for human and JSON modes.
- [ ] Update schema/docs/collateral and run full validation (`make test`, targeted deterministic checks, `make check`).

## Surprises & Discoveries

- Observation: During initial repository scan (before this plan file existed), `docs/exec-plans/active/` was absent and had to be created for this plan.
  Evidence: early filesystem read showed only `docs/exec-plans/AGENTS.md` and `docs/exec-plans/completed/`.

- Observation: `change` is fully wired in CLI parsing and dispatch but intentionally fails in engine.
  Evidence: `src/engine/change.rs` currently returns `WavepeekError::Unimplemented("`change` command execution is not implemented yet")`.

- Observation: No reusable time parser exists yet for command inputs (`--from`, `--to`, `--time` style), so `change` must introduce one in waveform/engine code now.
  Evidence: no `parse_time` helper exists in `src/engine/` or `src/waveform/`; only metadata normalization exists for dump internals.

## Decision Log

- Decision: Use TDD by writing `tests/change_cli.rs` first and intentionally observing red tests before implementing command logic.
  Rationale: `change` has dense semantics (windowing, dedup, posedge filtering, warnings). Contract-first tests prevent silent behavioral drift.
  Date/Author: 2026-02-22 / OpenCode

- Decision: Keep `change` on the existing envelope pipeline (`CommandResult` -> `output::write`) instead of command-specific serialization.
  Rationale: Existing commands already enforce deterministic JSON envelope behavior and warning routing.
  Date/Author: 2026-02-22 / OpenCode

- Decision: Freeze expected warning/error strings in tests where behavior must be parseable (`--max` validation, invalid time syntax, empty result, truncation).
  Rationale: This command is aimed at machine consumers; exact strings materially affect automation reliability.
  Date/Author: 2026-02-22 / OpenCode

- Decision: Make `cargo run --quiet -- ...` the canonical execution form for all acceptance examples so the plan works from a fresh checkout without global install assumptions.
  Rationale: Stateless novice execution must not depend on whether `wavepeek` binary is pre-installed in PATH.
  Date/Author: 2026-02-22 / OpenCode

- Decision: Move `change` help-text contract update to Milestone 1 so `cli_contract` can be green immediately after test-contract setup.
  Rationale: Keeping help text in Milestone 4 while expecting early `cli_contract` alignment created an avoidable sequencing contradiction.
  Date/Author: 2026-02-22 / OpenCode

## Outcomes & Retrospective

Implementation has not started yet. At completion this section must state what became user-visible, what trade-offs were made in waveform sampling APIs, and any residual risks around time normalization and parser edge cases.

## Context and Orientation

The repository is a single Rust crate with clear layers. `src/cli/` defines clap args and help text, `src/engine/` performs command logic, `src/waveform/mod.rs` is the parser adapter over `wellen`, and `src/output.rs` renders human/JSON outputs. Errors are normalized by `src/error.rs` and top-level CLI parse behavior is centralized in `src/cli/mod.rs`.

`change` already has argument shape in `src/cli/change.rs` and dispatch in `src/engine/mod.rs`, but command execution is a stub in `src/engine/change.rs`. JSON schema is stored in `schema/wavepeek.json`; `make update-schema` regenerates it from runtime output and `make check-schema` verifies no drift.

Relevant files for this plan are `src/engine/change.rs`, `src/engine/mod.rs`, `src/waveform/mod.rs`, `src/output.rs`, `src/cli/mod.rs`, `tests/change_cli.rs` (new), `tests/cli_contract.rs`, `tests/fixtures/hand/change_edge_cases.vcd` (new), `schema/wavepeek.json`, `README.md`, and `CHANGELOG.md`.

## Normative `change` Contract (Authoritative in This Plan)

This section is the execution contract. Implementers must follow this section even if wording elsewhere differs.

Command surface is:

    wavepeek change --waves <file> [--from <time>] [--to <time>] [--scope <path>] --signals <names> [--clk <name>] [--max <n>] [--json]

`--waves` and `--signals` are required. `--max` defaults to `50` and must be greater than `0`; `--max 0` is an `args` error. `--signals` accepts comma-separated names and preserves user order in output rows.

Name resolution rules are strict. Without `--scope`, each `--signals` entry and optional `--clk` entry must be a full path (example `top.cpu.valid`). With `--scope top.cpu`, short names are resolved relative to that scope (example `valid` -> `top.cpu.valid`). Mixed input is allowed: full paths remain full paths, short names resolve relative to scope.

Time strings require explicit units (`fs`, `ps`, `ns`, `us`, `ms`, `s`). Bare numbers are rejected as `args` errors. Parsed times must convert exactly into dump precision; non-exact conversion is `args` error. `--from` and `--to` define an inclusive candidate window. Missing `--from` means dump start, missing `--to` means dump end, both missing means full dump. If both are present and `from > to`, return `args` error.

Unclocked mode is used when `--clk` is absent. A snapshot is emitted for each timestamp in the inclusive window where at least one tracked signal changed. If several tracked signals change at the same timestamp, emit one snapshot for that timestamp after applying all changes at that timestamp.

Clocked mode is used when `--clk` is present. A snapshot is emitted only at timestamps where the clock signal has a clean posedge transition `0 -> 1`. Any transition involving `x` or `z` is not a posedge. Clocked mode forbids including the clock in `--signals`; if present, return `args` error. Clock itself is never emitted in snapshot signal arrays unless user explicitly tracks it in unclocked mode.

Each snapshot row contains normalized timestamp plus all requested signals in user order. Signal values are formatted as Verilog literals `<width>'h<digits>` using lowercase hex digits, including `x` and `z` where applicable.

Successful output in `--json` mode is one envelope object with `command: "change"`, `data` as an array of snapshots, and `warnings` as an array of strings. In human mode, stdout format is exact and deterministic: one line per snapshot using `<time> <requested_name_1>=<value_1> <requested_name_2>=<value_2> ...` with names shown exactly as provided in `--signals` and in that same order. In human mode, warnings go to stderr as `warning: <text>`, and warning text must match JSON warning text byte-for-byte.

If there are no trigger timestamps in window, return success with empty `data` and one warning exactly `no signal changes found in selected time range`. If snapshots exceed `--max`, truncate to first `--max` snapshots in time order and add warning exactly `truncated output to <n> entries (use --max to increase limit)`.

Runtime lookup failures for unknown scope/signal use existing categories (`error: scope:` or `error: signal:`) and non-zero exit. Parse/validation failures use `error: args:`.

For non-exact time conversion, the canonical error message fragment is `cannot be represented exactly in dump precision` under `error: args:`.

## Open Questions

There are no open design questions blocking implementation. Resolved ambiguities in this plan are: initial timestamp behavior is tested via explicit ranges that avoid relying on implicit pre-time-zero state, mixed full and scoped signal naming is allowed, and warning strings are fixed as exact literals above.

## Plan of Work

Milestone 1 defines behavior with failing tests and aligns help contract early. Add `tests/change_cli.rs`, update `tests/cli_contract.rs` so `change` is no longer marked unimplemented, and update `src/cli/mod.rs` help text for `change` accordingly. Milestone 1 is done when `cargo test --test change_cli` fails due to missing implementation rather than malformed tests, while `cargo test --test cli_contract` is green with `change` implemented-in-help and `at`/`when` still unimplemented-in-help.

Milestone 2 builds waveform-side sampling primitives. Implement resolution, time parsing, timestamp iteration, and value sampling helpers in `src/waveform/mod.rs`, and add unit tests for edge behavior (same timestamp dedup inputs, clean posedge discrimination, exact unit conversion). Milestone 2 is done when waveform unit tests for these helpers pass and expose stable APIs consumed by engine code.

Milestone 3 implements engine execution and rendering integration. Replace `src/engine/change.rs` stub, extend `src/engine/mod.rs` enums with `change` data variants, and add `change` branch in `src/output.rs` human renderer. Milestone 3 is done when `cargo test --test change_cli` passes and command output matches exact JSON, human stdout, and error/warning examples in this plan.

Milestone 4 updates schema and collateral. Extend `schema/wavepeek.json`, switch README command status to available, and add changelog entry. Milestone 4 is done when `make update-schema` followed by `make check-schema` succeeds and no doc/test mismatch remains.

## Concrete Steps

Run all commands from `/workspaces/wavepeek`.

Use `cargo run --quiet -- ...` as the canonical way to execute CLI examples in this plan. If you prefer running the built binary directly, ensure behavior and output are byte-identical.

1. Create failing contract tests first.

   Create `tests/change_cli.rs` with exact assertions from the examples in this plan, including exact human stdout lines and stderr warnings. Update `tests/cli_contract.rs` help assertions so only `at` and `when` remain unimplemented, and update `src/cli/mod.rs` help text for `change` in the same step.

   Run:

       cargo test --test change_cli
       cargo test --test cli_contract

   Expected now: `change_cli` fails due to unimplemented runtime path; `cli_contract` is green after help-text and assertion updates.

2. Add waveform primitives and unit tests.

   Implement helper types/functions in `src/waveform/mod.rs` and add `tests/fixtures/hand/change_edge_cases.vcd` for same-timestamp and x/z clock transitions. Add focused unit tests in `src/waveform/mod.rs` for exact time parsing and posedge filtering.

   Run:

       cargo test waveform::tests

   Expected now: waveform helper tests pass; integration tests still fail until engine wiring is complete.

3. Implement engine `change` and output wiring.

   Replace stub in `src/engine/change.rs`, extend `CommandName` and `CommandData` in `src/engine/mod.rs`, and add human rendering branch in `src/output.rs`.

   Run:

       cargo test --test change_cli
       cargo test --test cli_contract

   Expected now: both suites pass with exact JSON payload, exact human stdout line format, and exact warning/error text assertions.

4. Sync schema and docs.

   Update `schema/wavepeek.json`, `README.md`, and `CHANGELOG.md`.

   Run:

       make update-schema
       make check-schema

   Expected now: no schema drift and command enum in schema includes `change`.

5. Run full quality gates.

   Run:

       make test
       make check

   Expected now: all tests, format, lint, schema contract, and build checks pass.

## Validation and Acceptance

The implementation is accepted only when command behavior matches the exact examples below.

Example A, unclocked snapshots in a bounded range:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --json

Expected top-level JSON envelope keys are exactly `$schema`, `command`, `data`, and `warnings`. `schema_version` must be absent. `command` must be exactly `change`.

Expected envelope shape:

    {
      "$schema": "https://github.com/kleverhq/wavepeek/blob/v<crate-version>/schema/wavepeek.json",
      "command": "change",
      "data": <exact data array below>,
      "warnings": []
    }

Expected `data` exactly:

    [
      {
        "time": "5ns",
        "signals": [
          {"name": "top.clk", "path": "top.clk", "value": "1'h1"},
          {"name": "top.data", "path": "top.data", "value": "8'h00"}
        ]
      },
      {
        "time": "10ns",
        "signals": [
          {"name": "top.clk", "path": "top.clk", "value": "1'h1"},
          {"name": "top.data", "path": "top.data", "value": "8'h0f"}
        ]
      }
    ]

and `warnings` is `[]`.

Example B, clocked snapshots:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 0ns --to 10ns --scope top --clk clk --signals data --json

Expected `data` exactly one row at `5ns` with signal `{ "name": "data", "path": "top.data", "value": "8'h00" }` and `warnings` `[]`.

Example C, empty window warning:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 6ns --to 9ns --signals top.clk,top.data --json

Expected `data` exactly `[]` and `warnings` exactly `["no signal changes found in selected time range"]`.

Example D, truncation warning:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --max 1 --json

Expected `data` length `1` and `warnings` exactly `["truncated output to 1 entries (use --max to increase limit)"]`.

Example E, human-mode stdout shape and order:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data

Expected stdout exactly:

    5ns top.clk=1'h1 top.data=8'h00
    10ns top.clk=1'h1 top.data=8'h0f

and expected stderr exactly empty.

Example F, human-mode warning routing:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 6ns --to 9ns --signals top.clk,top.data

Expected stdout exactly empty and expected stderr exactly:

    warning: no signal changes found in selected time range

Example G, mixed scope-relative and absolute signal names in human mode:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,top.clk

Expected stdout exactly:

    5ns data=8'h00 top.clk=1'h1
    10ns data=8'h0f top.clk=1'h1

Example H, human-mode truncation warning parity with JSON warning text:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --max 1

Expected stdout exactly:

    5ns top.clk=1'h1 top.data=8'h00

Expected stderr exactly:

    warning: truncated output to 1 entries (use --max to increase limit)

Error acceptance checks (exact prefix and category):

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --signals top.clk --max 0

must fail with stderr starting `error: args: --max must be greater than 0.`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 10ns --to 1ns --signals top.clk

must fail with stderr starting `error: args:` and include `--from must be less than or equal to --to`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 10 --signals top.clk

must fail with stderr starting `error: args:` and include `requires units`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top --clk clk --signals clk

must fail with stderr starting `error: args:` and include `--clk must not be included in --signals`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --signals top.nope

must fail with stderr starting `error: signal:`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top.nope --signals clk

must fail with stderr starting `error: scope:`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1.5ns --signals top.clk

must fail with stderr starting `error: args:` and include `cannot be represented exactly in dump precision`.

All success examples above must exit with code `0`. All error examples above must exit with code `1`.

Exit-code verification pattern for novice execution:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --json >/tmp/change_ok.json 2>/tmp/change_ok.err; echo $?

Expected printed code is `0`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --signals top.clk --max 0 >/tmp/change_err.out 2>/tmp/change_err.err; echo $?

Expected printed code is `1`.

Help acceptance:

    cargo run --quiet -- --help

must no longer contain `change` marked as not implemented, while `at` and `when` remain marked as not implemented.

## Idempotence and Recovery

All steps are safe to repeat. Tests, schema regeneration, and formatting/lint/build checks are idempotent when run from a clean checkout.

If schema checks fail after command implementation, regenerate schema (`make update-schema`) and re-run `make check-schema`. If mismatch persists, compare runtime `wavepeek schema` output against `schema/wavepeek.json` to isolate whether drift is intentional contract evolution or accidental serialization change.

If implementation is partially complete and failing, recover in this order: compile green with stubs, then re-enable unclocked snapshots, then clocked snapshots, then warning/error exact-string assertions, then collateral updates.

## Artifacts and Notes

Primary files to modify are:

    src/engine/change.rs
    src/engine/mod.rs
    src/waveform/mod.rs
    src/output.rs
    src/cli/mod.rs
    tests/change_cli.rs
    tests/cli_contract.rs
    tests/fixtures/hand/change_edge_cases.vcd
    schema/wavepeek.json
    README.md
    CHANGELOG.md

Boundary behavior that must be explicitly tested includes `--from == --to`, omitted bounds (full-dump default window), default `--max = 50`, mixed absolute and scope-relative signal names, and deterministic row order when multiple signals update at the same timestamp.

## Interfaces and Dependencies

No new dependencies are required.

In `src/engine/change.rs`, add serializable result types and use them directly in `CommandData`:

    pub struct ChangeSignalValue {
        pub name: String,
        pub path: String,
        pub value: String,
    }

    pub struct ChangeSnapshot {
        pub time: String,
        pub signals: Vec<ChangeSignalValue>,
    }

In `src/engine/mod.rs`, extend enums with exact names:

    CommandName::Change
    CommandData::Change(Vec<crate::engine::change::ChangeSnapshot>)

In `src/waveform/mod.rs`, add helper interfaces with fixed names so engine code stays parser-agnostic:

    pub struct ResolvedSignal {
        pub requested_name: String,
        pub path: String,
        pub width: u32,
    }

    pub fn resolve_signals(&self, scope: Option<&str>, names: &[String]) -> Result<Vec<ResolvedSignal>, WavepeekError>
    pub fn resolve_clock(&self, scope: Option<&str>, name: &str) -> Result<ResolvedSignal, WavepeekError>
    pub fn parse_time_bound(&self, value: Option<&str>) -> Result<Option<u64>, WavepeekError>
    pub fn change_timestamps(&self, signals: &[ResolvedSignal], from: Option<u64>, to: Option<u64>) -> Result<Vec<u64>, WavepeekError>
    pub fn posedge_timestamps(&self, clock: &ResolvedSignal, from: Option<u64>, to: Option<u64>) -> Result<Vec<u64>, WavepeekError>
    pub fn snapshot_values(&self, signals: &[ResolvedSignal], time: u64) -> Result<Vec<String>, WavepeekError>
    pub fn normalize_time_string(&self, time: u64) -> Result<String, WavepeekError>

These names and responsibilities are part of this plan contract to reduce implementation ambiguity.

Revision Note: 2026-02-22 / OpenCode - Revised after final independent review: resolved milestone sequencing for help contract, added human truncation + `error: scope:` acceptance cases, and added explicit exit-code verification procedure.
