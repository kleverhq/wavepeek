# Implement `change` Command End-to-End (Aligned with `at` Contracts)

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

This work turns `wavepeek change` from a documented stub into a working command that returns deterministic value snapshots over time. After completion, users and agents can ask, in one command, how selected signals evolve in a time window, either on any tracked transition or sampled at a clock posedge cadence.

This revision also aligns `change` output semantics with shipped `at` behavior: no duplicate identity fields in JSON, canonical path identity in JSON, default human rendering that mirrors user input tokens, and `--abs` for canonical human display. Time uses compact `@<time>` only in human output; JSON keeps plain normalized time.

## Non-Goals

This plan does not implement `when`, does not introduce compatibility aliases, does not change JSON envelope behavior outside `change` payload shape, and does not redesign command contracts that are already shipped for `info`/`scope`/`signal`/`at`.

This plan does not add timestamp prefixes to JSON payloads. `@` is human-only by design.

## Progress

- [x] (2026-02-22 14:22Z) Drafted initial ExecPlan with milestones, acceptance criteria, and repository context.
- [x] (2026-02-22 14:41Z) Revised plan after review feedback: made command contract self-contained, added exact expected outputs/errors, and added milestone-level proof criteria.
- [x] (2026-02-22 14:52Z) Revised plan after independent control review: fixed canonical command invocation guidance and expanded JSON envelope acceptance checks.
- [x] (2026-02-22 16:02Z) Revised plan after final control findings: aligned milestone sequencing for help contract, added missing human/error acceptance cases, and added explicit exit-code verification steps.
- [x] (2026-02-23 20:16Z) Rebased plan contract on already-shipped `at` behavior: added `--abs` for `change`, removed JSON `name/path` duplication, made `@` human-only, and switched implementation strategy from new parsing logic to shared/refactored `at` helpers.
- [ ] Add failing tests that define `change` behavior under the aligned contract (JSON shape, human `@` lines, `--abs`, warning parity, clocked/unclocked semantics, args errors).
- [ ] Extract shared time/value formatting logic from `at` into reusable engine helpers, then update `at` to use those helpers without behavior changes.
- [ ] Implement waveform-side `change` primitives (resolution, trigger timestamps, posedge filtering, value snapshotting) and integrate engine/output wiring.
- [ ] Update schema/docs/changelog/collateral and run full validation (`make test`, deterministic checks, `make check`).

## Surprises & Discoveries

- Observation: `change` is fully wired in CLI parsing and dispatch but intentionally fails in engine.
  Evidence: `src/engine/change.rs` currently returns `WavepeekError::Unimplemented("`change` command execution is not implemented yet")`.

- Observation: `at` now contains working, tested helpers for time parsing/conversion and Verilog literal formatting that can be reused.
  Evidence: `src/engine/at.rs` implements `parse_time_token`, cross-unit conversion via zeptoseconds, exact alignment checks, `format_raw_timestamp`, and `format_verilog_literal` with unit tests.

- Observation: current JSON schema includes `at` but not `change`; `change` contract additions must extend command enum/conditional branches.
  Evidence: `schema/wavepeek.json` command enum is `["info", "scope", "signal", "at"]`.

- Observation: top-level/help contracts still mark `change` as unimplemented.
  Evidence: `tests/cli_contract.rs` expects unimplemented markers for `change` and `when`.

## Decision Log

- Decision: Keep TDD-first flow by writing `tests/change_cli.rs` before runtime implementation.
  Rationale: `change` has dense semantics (windowing, trigger rules, warnings, rendering mode differences). Contract tests prevent drift.
  Date/Author: 2026-02-22 / OpenCode

- Decision: Keep `change` on the existing envelope pipeline (`CommandResult` -> `output::write`) instead of command-specific serialization.
  Rationale: Existing commands already enforce deterministic JSON envelope behavior and warning routing.
  Date/Author: 2026-02-22 / OpenCode

- Decision: Make `cargo run --quiet -- ...` the canonical execution form for acceptance examples.
  Rationale: Stateless execution should not rely on globally installed binaries.
  Date/Author: 2026-02-22 / OpenCode

- Decision: Align `change` naming semantics with shipped `at` semantics.
  Rationale: One consistent display-identity model lowers user/agent confusion: default human shows requested tokens, `--abs` shows canonical paths, JSON always carries canonical identity.
  Date/Author: 2026-02-23 / OpenCode

- Decision: Use `@` prefix only in human output, not in JSON.
  Rationale: Human output benefits from compact visual marker; JSON should stay plain normalized time for machine stability and consistency with existing payload style.
  Date/Author: 2026-02-23 / OpenCode

- Decision: Refactor and share `at` time/value helpers instead of duplicating parsing/formatting logic in `change`.
  Rationale: Reuse reduces bugs, keeps contracts synchronized across commands, and simplifies future maintenance.
  Date/Author: 2026-02-23 / OpenCode

## Outcomes & Retrospective

Implementation has not started yet. At completion this section must state what became user-visible, what shared helper APIs were introduced, how `at` compatibility was preserved during refactor, and any residual risks around time normalization or trigger edge cases.

## Context and Orientation

The repository is a single Rust crate with clear layers. `src/cli/` defines clap args and help text, `src/engine/` performs command logic, `src/waveform/mod.rs` is the parser adapter over `wellen`, and `src/output.rs` renders human/JSON outputs. Errors are normalized by `src/error.rs` and top-level parse behavior is centralized in `src/cli/mod.rs`.

`change` argument shape exists in `src/cli/change.rs`, dispatch exists in `src/engine/mod.rs`, and execution is still a stub in `src/engine/change.rs`. `at` is already implemented end-to-end in `src/cli/at.rs`, `src/engine/at.rs`, `src/waveform/mod.rs`, `src/output.rs`, `schema/wavepeek.json`, and `tests/at_cli.rs`; that implementation is now the contract template for naming and shared time/value semantics.

Primary files for this plan are `src/engine/change.rs`, `src/engine/mod.rs`, `src/engine/at.rs`, `src/waveform/mod.rs`, `src/output.rs`, `src/cli/change.rs`, `src/cli/mod.rs`, `tests/change_cli.rs` (new), `tests/cli_contract.rs`, `tests/fixtures/hand/change_edge_cases.vcd` (new), `schema/wavepeek.json`, `README.md`, `docs/DESIGN.md`, and `CHANGELOG.md`.

## Normative `change` Contract (Authoritative in This Plan)

This section is the execution contract. Implementers must follow this section even if wording elsewhere differs.

Command surface is:

    wavepeek change --waves <file> [--from <time>] [--to <time>] [--scope <path>] --signals <names> [--clk <name>] [--max <n>] [--abs] [--json]

`--waves` and `--signals` are required. `--signals` is a comma-separated list and output order must match user order exactly (including duplicates). `--max` defaults to `50` and must be greater than `0`; `--max 0` is an `args` error.

Name resolution matches `at`:

- Without `--scope`, each `--signals` token and optional `--clk` token is interpreted as a canonical full path.
- With `--scope <path>`, each `--signals` token and optional `--clk` token is interpreted as a short name relative to that scope.

`--scope` mode does not accept canonical full-path tokens in `--signals`/`--clk`; those must fail as `error: signal:` lookup failures under scoped resolution, consistent with `at` behavior.

Time strings require explicit units (`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`) and integer numeric parts. Bare numbers are rejected as `args` errors. Parsed times must convert exactly into dump precision; non-exact conversion is an `args` error with fragment `cannot be represented exactly in dump precision`. `--from` and `--to` define an inclusive candidate window. Missing `--from` means dump start, missing `--to` means dump end, both missing means full dump. If both are present and `from > to`, return `args` error.

Unclocked mode is used when `--clk` is absent. Emit one snapshot per timestamp in the inclusive window where at least one tracked signal changed. If multiple tracked signals change at the same timestamp, emit exactly one snapshot after applying all changes at that timestamp.

Clocked mode is used when `--clk` is present. Emit snapshots only at timestamps where the clock signal has a clean posedge `0 -> 1`. Transitions involving `x` or `z` are not posedges. Clocked mode forbids including the clock in `--signals`; if present, return `args` error `--clk must not be included in --signals`.

Values are formatted as Verilog literals `<width>'h<digits>` with lowercase hex digits and support for `x`/`z`.

JSON mode (`--json`) returns one envelope object with `command: "change"`, `data` as an array of snapshots, and `warnings` as an array of strings. Each snapshot object has:

- `time`: normalized plain time string (for example `5ns`, no `@`)
- `signals`: ordered array of objects `{ "path": <canonical path>, "value": <literal> }`

Human mode defaults to one line per snapshot:

    @<time> <display_1>=<value_1> <display_2>=<value_2> ...

`<display_i>` uses exact tokens passed in `--signals` by default. With `--abs`, `<display_i>` switches to canonical full paths. `--abs` does not alter JSON payload content.

Warnings must match byte-for-byte between JSON warning strings and human stderr warning text after `warning: ` prefixing.

If there are no trigger timestamps in range, return success with empty `data` and one warning exactly `no signal changes found in selected time range`.

If snapshots exceed `--max`, truncate to first `--max` snapshots in time order and add warning exactly `truncated output to <n> entries (use --max to increase limit)`.

Runtime lookup failures for unknown scope/signal use existing categories (`error: scope:` or `error: signal:`) and non-zero exit. Parse/validation failures use `error: args:`. Success exits with code `0`; errors exit with code `1`.

## Open Questions

No blocking open questions remain. This revision resolves prior ambiguity by fixing: `--abs` semantics for `change`, human-only `@` prefixes, JSON canonical identity fields (`path` only), and `at`-style scoped signal resolution.

## Plan of Work

Milestone 1 defines and locks the aligned contract with failing tests. Add `tests/change_cli.rs` and update `tests/cli_contract.rs` + `src/cli/mod.rs` help wording so `change` is no longer marked unimplemented in help. This milestone is complete when `cargo test --test change_cli` fails because runtime behavior is not yet implemented (without unrelated contract/help regressions) and `cargo test --test cli_contract` is green with `change` marked implemented while `when` remains unimplemented.

Milestone 2 extracts shared helper logic from `at` and adds waveform-side `change` primitives. Move time parsing/conversion/normalization and Verilog literal formatting into shared engine helpers consumed by both `at` and `change`; keep `at` behavior unchanged and covered by existing tests. In parallel, add/change waveform APIs for resolved signal lookup, trigger timestamp enumeration, posedge filtering, and snapshot sampling.

Milestone 3 replaces `change` runtime stub and wires output rendering. Implement `src/engine/change.rs`, extend `src/engine/mod.rs` with `CommandName::Change` and `CommandData::Change`, add human renderer branch in `src/output.rs`, and thread `--abs` from CLI args into `HumanRenderOptions`.

Milestone 4 updates schema and collateral. Extend `schema/wavepeek.json` with `change` command/data definitions and conditionals, update `docs/DESIGN.md` + `README.md` command status, and add changelog notes.

Milestone 5 runs full validation and deterministic checks, then records evidence.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-change`.

1. Add contract tests first (`tests/change_cli.rs`) and update help contracts.

       cargo test --test change_cli
       cargo test --test cli_contract

   Expected now: `change_cli` fails because `change` runtime behavior is still missing (not because of malformed test/help assumptions); `cli_contract` is green with updated help expectations.

2. Extract shared helpers from `at` and add waveform `change` primitives + unit tests.

       cargo test --test at_cli
       cargo test waveform::tests

   Expected now: `at_cli` remains green after refactor; waveform helper tests are green.

3. Implement engine `change` and output wiring.

       cargo test --test change_cli
       cargo test --test cli_contract

   Expected now: both suites pass with exact JSON shape, exact human `@` rendering, `--abs` behavior, and exact warning/error text.

4. Sync schema/docs/changelog.

       make update-schema
       make check-schema

   Expected now: no schema drift; `command` enum includes `change` and conditionals validate `change` payload.

5. Run full quality gates.

       make test
       make check

   Expected now: tests, format, lint, schema checks, and build checks pass.

## Validation and Acceptance

Acceptance is behavioral and requires exact outputs for commands below.

Example A, unclocked JSON snapshots:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --json

Expected `data` exactly:

    [
      {
        "time": "5ns",
        "signals": [
          {"path": "top.clk", "value": "1'h1"},
          {"path": "top.data", "value": "8'h00"}
        ]
      },
      {
        "time": "10ns",
        "signals": [
          {"path": "top.clk", "value": "1'h1"},
          {"path": "top.data", "value": "8'h0f"}
        ]
      }
    ]

with `warnings` exactly `[]`.

Example B, unclocked human default (`@` prefix + requested tokens):

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data

Expected stdout exactly:

    @5ns top.clk=1'h1 top.data=8'h00
    @10ns top.clk=1'h1 top.data=8'h0f

and stderr exactly empty.

Example C, scoped human default uses short requested tokens:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk

Expected stdout exactly:

    @5ns data=8'h00 clk=1'h1
    @10ns data=8'h0f clk=1'h1

Example D, scoped human `--abs` uses canonical paths:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk --abs

Expected stdout exactly:

    @5ns top.data=8'h00 top.clk=1'h1
    @10ns top.data=8'h0f top.clk=1'h1

Example E, clocked JSON snapshots:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 0ns --to 10ns --scope top --clk clk --signals data --json

Expected `data` exactly one row at `5ns`:

    [
      {
        "time": "5ns",
        "signals": [
          {"path": "top.data", "value": "8'h00"}
        ]
      }
    ]

with `warnings` exactly `[]`.

Example F, empty window warning:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 6ns --to 9ns --signals top.clk,top.data --json

Expected `data` exactly `[]` and `warnings` exactly `["no signal changes found in selected time range"]`.

Example G, truncation warning:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --max 1 --json

Expected `data` length `1` and `warnings` exactly `["truncated output to 1 entries (use --max to increase limit)"]`.

Example H, human-mode warning routing and parity with JSON warning text:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 6ns --to 9ns --signals top.clk,top.data

Expected stdout exactly empty and stderr exactly:

    warning: no signal changes found in selected time range

and warning text (after removing `warning: `) must match JSON warning text byte-for-byte.

Example I, human-mode truncation warning parity:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --max 1

Expected stdout exactly:

    @5ns top.clk=1'h1 top.data=8'h00

Expected stderr exactly:

    warning: truncated output to 1 entries (use --max to increase limit)

and warning text (after removing `warning: `) must match JSON warning text byte-for-byte.

Example J, JSON payload parity with and without `--abs`:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk --json > /tmp/change_json_default.json
    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk --json --abs > /tmp/change_json_abs.json
    cmp /tmp/change_json_default.json /tmp/change_json_abs.json

Expected: `cmp` exits `0`.

Example K, clocked mode excludes x/z transitions from posedge detection:

    cargo run --quiet -- change --waves tests/fixtures/hand/change_edge_cases.vcd --from 0ns --to 30ns --scope top --clk clk --signals data --json

Expected behavior: `data` contains rows only for clean `0->1` clock transitions and contains no rows for `x->1` or `z->1` transitions.

Error acceptance checks:

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

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top --signals top.clk

must fail with stderr starting `error: signal:` (scoped mode expects short names only).

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 15ps --signals top.clk

must fail with stderr starting `error: args:` and include `cannot be represented exactly in dump precision`.

All success examples must exit `0`. All error examples must exit `1`.

## Idempotence and Recovery

All steps are safe to repeat. Tests, schema regeneration, and checks are idempotent from a clean checkout.

If schema checks fail after command implementation, run `make update-schema` and then `make check-schema`. If mismatch persists, compare runtime `wavepeek schema` output to `schema/wavepeek.json` to separate intentional contract changes from accidental serialization drift.

If implementation is partially complete and failing, recover in this order: keep compile green, keep `at` tests green after helper extraction, re-enable unclocked snapshots, then clocked snapshots, then exact warning/error strings, then collateral updates.

## Artifacts and Notes

Primary files to modify:

    src/cli/change.rs
    src/cli/mod.rs
    src/engine/change.rs
    src/engine/at.rs
    src/engine/mod.rs
    src/waveform/mod.rs
    src/output.rs
    tests/change_cli.rs
    tests/cli_contract.rs
    tests/fixtures/hand/change_edge_cases.vcd
    schema/wavepeek.json
    docs/DESIGN.md
    README.md
    CHANGELOG.md

Boundary behavior that must be explicitly tested includes `--from == --to`, omitted bounds, default `--max = 50`, duplicate signal tokens preserving order, scoped short-name mode parity with `at`, and deterministic row order when multiple tracked signals change at one timestamp.

## Interfaces and Dependencies

No new dependencies are required.

Add `--abs` to `src/cli/change.rs` mirroring `at`:

    #[arg(long)]
    pub abs: bool,

In `src/engine/change.rs`, define payload types aligned with `at` identity model:

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct ChangeSignalValue {
        #[serde(skip_serializing)]
        pub display: String,
        pub path: String,
        pub value: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct ChangeSnapshot {
        pub time: String,
        pub signals: Vec<ChangeSignalValue>,
    }

In `src/engine/mod.rs`, extend enums with exact names:

    CommandName::Change
    CommandData::Change(Vec<crate::engine::change::ChangeSnapshot>)

In `src/output.rs`, add deterministic human rendering for `CommandData::Change`:

    @<time> <display_or_path_1>=<value_1> <display_or_path_2>=<value_2> ...

where `display_or_path` uses requested tokens by default and canonical paths when `signals_abs` is true.

Extract shared time/literal helpers from `src/engine/at.rs` into reusable engine-level helpers (module naming is implementation choice, but helper behavior must remain byte-compatible with current `at` tests), then make both `at` and `change` consume them.

In `src/waveform/mod.rs`, add/extend parser-agnostic helper APIs for `change` execution:

    pub struct ResolvedSignal {
        pub display: String,
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
Revision Note: 2026-02-23 / OpenCode - Updated plan after shipped `at` implementation to align `change` contract with `at` naming semantics (`--abs`, canonical JSON identity, no `name/path` duplication), make `@` time prefix human-only, and switch implementation strategy to shared-helper refactoring instead of duplicating time/value logic.
