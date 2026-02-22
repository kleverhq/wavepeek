# Implement `at` Command End-to-End

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, users can query signal values at a specific timestamp from both VCD and FST files with `wavepeek at`, in either human-readable output (default) or strict JSON envelope mode (`--json`). Today, `at` exists in CLI help but fails with `error: unimplemented`. When this plan is complete, the command will be fully usable for scripted debug flows and agent-driven workflows, including deterministic output ordering and stable error categories.

The behavior is observable by running `wavepeek at --waves tests/fixtures/hand/m2_core.vcd --time 10ns --scope top --signals clk,data` and seeing concrete values instead of an unimplemented error.

## Non-Goals

This plan does not implement `change` or `when`, does not introduce radix selection flags (for example `--radix`), and does not change the global output envelope model beyond adding the new `at` payload shape. This plan also does not introduce compatibility aliases for old command names.

## Progress

- [x] (2026-02-22 11:14Z) Mapped existing CLI/engine/output/schema integration points and current `at` stub behavior.
- [x] (2026-02-22 11:14Z) Researched `wellen` APIs needed for timestamp sampling (`time_table`, `load_signals`, `get_offset`, `get_value_at`).
- [x] (2026-02-22 11:20Z) Incorporated two independent review passes and tightened validation/TDD steps to avoid false-green checks.
- [x] (2026-02-22 13:01Z) Added failing integration suite `tests/at_cli.rs` covering success, error, deterministic, duplicate-order, mixed-mode, and VCD/FST parity scenarios.
- [x] (2026-02-22 13:16Z) Implemented strict `--time` parsing/normalization for `at` with mandatory units, inclusive bounds, and explicit args-category failures.
- [x] (2026-02-22 13:24Z) Added waveform canonical-path sampling API and replaced `engine::at::run` stub with end-to-end value extraction.
- [x] (2026-02-22 13:29Z) Wired `at` through command enums, human renderer, and JSON schema (`$defs.atData` + command-conditioned validation).
- [x] (2026-02-22 13:42Z) Updated help/contracts/docs/changelog and passed `cargo test -q`, `make check`, and `make ci`.

## Surprises & Discoveries

- Observation: `wellen::simple::Waveform` requires explicit signal loading before value access for robust on-demand sampling.
  Evidence: `wellen-0.20.2/src/simple.rs` exposes `load_signals(...)` and `get_signal(...)` paths used for value retrieval.
- Observation: `Signal::get_offset(...)` returns the nearest change at or before a time-table index, which matches `at` semantics for timestamps between change events.
  Evidence: `wellen-0.20.2/src/signals.rs` documents offset retrieval and subsequent `get_value_at(...)` usage.
- Observation: Design text still mentions `time_precision` wording in places, while runtime metadata already uses `time_unit` and normalized strings.
  Evidence: `docs/DESIGN.md` section 3.2.4 vs `src/engine/info.rs` + existing JSON outputs.
- Observation: `cargo clippy -D warnings` enforces `manual_div_ceil`, so helper formatting code must use `usize::div_ceil(...)` to pass repository gates.
  Evidence: `make check` failed on first pass until `format_verilog_literal` switched from `(len + 3) / 4` to `len.div_ceil(4)`.
- Observation: `at` scope semantics were implemented correctly but help wording was ambiguous enough to trigger reviewer confusion about full-path behavior with `--scope`.
  Evidence: Independent review pass flagged `--scope top --signals top.clk` expectation mismatch; wording/tests were tightened to reflect short-name-only scoped mode.
- Observation: Clap `Vec` args are optional unless explicitly marked required, even with `num_args = 1..`; this let `at` run without `--signals` until review caught it.
  Evidence: Independent review found `wavepeek at --waves ... --time ...` succeeding with empty signal output before `required = true` was added.

## Decision Log

- Decision: `at` treats `--time` as an absolute query time with mandatory unit suffix and requires it to be within dump bounds (`time_start <= time <= time_end`).
  Rationale: This is explicit, deterministic, and prevents ambiguous behavior before/after recorded dump range.
  Date/Author: 2026-02-22 / OpenCode
- Decision: Without `--scope`, each `--signals` entry is interpreted as a full canonical path; with `--scope`, each entry is interpreted as a short name relative to that scope and expanded to full path.
  Rationale: Matches product contract in `docs/DESIGN.md` section 3.2.4.
  Date/Author: 2026-02-22 / OpenCode
- Decision: JSON `data.signals` preserves the exact input order from `--signals` (including duplicates if provided).
  Rationale: Deterministic behavior and explicit contract requirement.
  Date/Author: 2026-02-22 / OpenCode
- Decision: Values are emitted as Verilog-style hex literals `<width>'h<digits>` with `x`/`z` support, using lowercase deterministic digits.
  Rationale: Required by command contract and consistent machine-readable representation across VCD/FST.
  Date/Author: 2026-02-22 / OpenCode
- Decision: Once implementation lands, `at` help text must no longer advertise "not implemented yet".
  Rationale: Help output must reflect shipped behavior and existing CLI contract tests must be updated accordingly.
  Date/Author: 2026-02-22 / OpenCode
- Decision: `at` rejects query times that are not exactly representable in dump resolution (`time_unit`) as `error: args:`.
  Rationale: Prevents silent rounding and keeps sampled timestamp deterministic and explicit for scripts/agents.
  Date/Author: 2026-02-22 / OpenCode
- Decision: `at --time` accepts integer tokens with mandatory units and rejects decimal tokens (for example `1.5ns`) as `error: args:`.
  Rationale: Keeps parsing deterministic and aligned with current command contract/tests while avoiding floating-point ambiguities.
  Date/Author: 2026-02-22 / OpenCode

## Outcomes & Retrospective

Completed all four milestones in one implementation pass with TDD-first ordering preserved.

What shipped: `at` now executes end-to-end for VCD and FST, including deterministic human/JSON output, input-order preservation (including duplicates), scope-aware and full-path resolution modes, fail-fast signal errors, inclusive bounds checks, and strict schema coverage (`command=at`, `data=atData`). CLI help and contract tests now treat only `change` and `when` as unimplemented.

What remains: no functional `at` items remain in this plan. Future work is still tracked separately for `change` and `when`.

Lessons learned: a focused waveform-layer sampling API kept engine logic concise while preserving deterministic ordering guarantees, and running full repository gates (`make check`, `make ci`) early surfaced style/lint constraints that would not appear in narrow test runs.

## Context and Orientation

The repository is a single Rust crate. CLI parsing lives in `src/cli/`, command execution in `src/engine/`, waveform access through `src/waveform/mod.rs`, rendering in `src/output.rs`, and JSON schema contract in `schema/wavepeek.json`.

Today, `at` arguments are defined in `src/cli/at.rs`, command routing is wired in `src/cli/mod.rs` and `src/engine/mod.rs`, but execution is a stub in `src/engine/at.rs` returning `WavepeekError::Unimplemented`. Because `output.rs` and `schema/wavepeek.json` currently only support `schema/info/scope/signal`, adding `at` requires extending command enums, data models, human renderer branch, and schema `command`/`data` unions.

Tests are split into command-specific integration files in `tests/` (for example `tests/info_cli.rs`, `tests/signals_cli.rs`) plus CLI surface contracts in `tests/cli_contract.rs`. New `at` tests should follow the same style: assert exit code, stdout/stderr shape, JSON structure, and deterministic repeated output.

Two terms used throughout this plan:

- "time table" means the sorted list of timestamps present in the dump as exposed by `wellen`.
- "canonical path" means dot-separated full signal path (for example `top.cpu.clk`) already used by existing `scope`/`signal` outputs.

Additional term definitions used in milestones:

- "timescale" means the waveform base time resolution (for example `1ns`) used by `wellen` raw timestamps.
- "raw timestamp" means the integer timestamp stored in the dump time table before string formatting.
- "Verilog literal" here means `<width>'h<digits>` output string (for example `8'h0f`, `4'hx`, `1'hz`).
- "fail fast" means returning the first error immediately with empty stdout and correct `error: <category>:` prefix.
- "VCD/FST parity" means equivalent queries over paired fixtures produce identical JSON payload content.

## Open Questions

No blocking open questions remain for implementation kickoff. If a non-blocking nuance appears (for example mixed-nibble `x/z` formatting policy), document it in `Decision Log` before proceeding.

## Plan of Work

The implementation will proceed in four milestones. Each milestone ends with executable proof (tests/commands) so a stateless contributor can continue safely from this file alone.

### Milestone 1: Lock behavior with failing tests first (TDD entry)

This milestone defines final behavior before implementation. Add a new integration suite `tests/at_cli.rs` only (do not change `tests/cli_contract.rs` yet). Cover both success and failure modes: default human output, `--json` envelope shape, required flags, invalid time format, out-of-range time, scope-not-found, signal-not-found, deterministic repeated runs, duplicate-signal ordering, mixed-mode name resolution errors, and VCD/FST parity with hand fixtures `tests/fixtures/hand/m2_core.vcd` and `tests/fixtures/hand/m2_core.fst`.

At the end of this milestone, `cargo test -q --test at_cli` should fail specifically because `at` is still unimplemented, proving the tests are meaningful.

### Milestone 2: Implement time and waveform sampling primitives

Add shared helpers for parsing `--time` with mandatory units and converting it into waveform raw timestamp units based on dump timescale. Add waveform-layer APIs that resolve canonical signal paths, load referenced signals, and sample value state at or before the requested timestamp index.

This milestone must also define deterministic value literal formatting into `<width>'h<digits>` while preserving unknown/high-impedance states (`x`/`z`). Keep helper functions unit-tested in the module where they are introduced.

At the end of this milestone, low-level unit tests for time conversion and value formatting pass, but end-to-end `at` may still fail until engine/output wiring is completed.

### Milestone 3: Wire `at` into engine, output, and schema

Replace the `src/engine/at.rs` stub with real execution logic. The command must open waveform file, parse and validate time, resolve requested signals (scope-aware), sample values in input order, and return `CommandResult` with `command="at"`, `warnings=[]` for normal success, and data payload:

- `time`: normalized timestamp string in dump time unit.
- `signals`: ordered array of `{ name, path, value }`.

Extend `src/engine/mod.rs` to include `CommandName::At` and `CommandData::At(...)`. Extend `src/output.rs` human rendering for the new payload. Extend `schema/wavepeek.json` so `command` enum includes `at`, `data.oneOf` includes `atData`, and command-conditioned validation supports `at`.

In this milestone, update `tests/cli_contract.rs` assertions that currently treat `at` as unimplemented. The expected shift is explicit TDD red-to-green:

- Before wiring: `at`-specific contract expectations fail.
- After wiring: those contract expectations pass and only `change`/`when` remain marked unimplemented in help output.

At the end of this milestone, `tests/at_cli.rs` and updated contract tests pass.

### Milestone 4: Collateral alignment and full validation

Update user-facing docs and release notes so they do not claim `at` is merely planned. Specifically update `README.md` command status table and `CHANGELOG.md` `Unreleased` section with behavior-level notes. If `docs/DESIGN.md` time wording is inconsistent (`time_precision` versus actual normalized unit), align it to the implemented contract in this milestone.

Run the relevant project gates and capture concise evidence in this plan.

### Edit Map

Edit `src/engine/at.rs` to replace the stub with full command execution. Keep parsing/validation and payload assembly in this file so behavior is discoverable in one place. Add focused unit tests in the same module for time parsing and Verilog literal formatting helpers.

Edit `src/waveform/mod.rs` to add one sampling API that accepts canonical paths + raw query time, loads required `SignalRef`s, and returns sampled bits/width in request order. Keep scope/name interpretation out of waveform layer; the waveform layer should work only with canonical paths.

Edit `src/engine/mod.rs` to add `At` to `CommandName` and `CommandData`, then edit `src/output.rs` to render `CommandData::At` deterministically in both human and JSON modes.

Edit `schema/wavepeek.json` to introduce `at` in `command` enum and a new `$defs.atData` object schema, then update conditional validation branches so `command=at` strictly maps to `atData`.

Edit `tests/cli_contract.rs` only after Milestone 3 behavior lands, removing `at` from unimplemented-help expectations.

### Concrete Steps

Run all commands from `/workspaces/wavepeek`.

1. Add tests first.

       cargo test -q --test at_cli

   Expected before implementation: failure containing the current unimplemented message for `at`.

2. Implement waveform/time helpers and command wiring.

       cargo test -q waveform::
       cargo test -q --test at_cli
       cargo test -q --test cli_contract

   Expected during iteration: `waveform::` filter runs more than zero tests, unit/helper tests pass first, then integration tests turn green after `engine::at` + output/schema wiring.

3. Validate command behavior manually on fixtures.

       cargo run --quiet -- at --waves tests/fixtures/hand/m2_core.vcd --time 10ns --scope top --signals clk,data
       cargo run --quiet -- at --waves tests/fixtures/hand/m2_core.vcd --time 10ns --scope top --signals clk,data --json
       cargo run --quiet -- at --waves tests/fixtures/hand/m2_core.vcd --time 10ns --scope top --signals clk,data --json > /tmp/at-vcd-1.json
       cargo run --quiet -- at --waves tests/fixtures/hand/m2_core.vcd --time 10ns --scope top --signals clk,data --json > /tmp/at-vcd-2.json
       cmp /tmp/at-vcd-1.json /tmp/at-vcd-2.json
       cargo run --quiet -- at --waves tests/fixtures/hand/m2_core.fst --time 10ns --scope top --signals clk,data --json > /tmp/at-fst.json
       cmp /tmp/at-vcd-1.json /tmp/at-fst.json

4. Run broader regression suite.

       cargo test -q

5. If running in devcontainer/CI image with `WAVEPEEK_IN_CONTAINER=1`, run full local gate and CI gate.

       make check
       make ci

   If not in container, record that `make` gates are intentionally skipped and keep `cargo test -q` evidence.

### Validation and Acceptance

Acceptance is behavioral and must be observable from CLI:

- Success path (human mode): querying known signals at `10ns` on `tests/fixtures/hand/m2_core.vcd` exits `0`, prints one `time:` line plus one line per requested signal, and emits no stderr warnings/errors.
- Success path (`--json`): output is valid JSON envelope with `$schema`, `command="at"`, empty `warnings`, and `data` object containing normalized `time` and ordered `signals` entries.
- Scope behavior: with `--scope top --signals clk,data`, output `name` values are `clk` and `data`, while `path` values are `top.clk` and `top.data`.
- Full-path behavior: without `--scope`, `--signals top.clk,top.data` resolves directly and preserves input order.
- Error path: invalid time token (for example `--time 100`) returns `error: args:` with hint `See 'wavepeek at --help'.`
- Error path: out-of-range time returns `error: args:` explaining bounds and requested value.
- Error path: missing signal returns `error: signal:` and fails fast without partial success output.
- Boundary path: `--time` exactly equal to `time_start` and `time_end` is accepted (inclusive bounds).
- Duplicate-input path: repeated signal names in `--signals` are preserved in output order, including duplicates.
- Mixed-mode path: without `--scope`, short names fail as `error: signal:`; with `--scope`, scoped short names resolve successfully.
- Cross-format parity: equivalent VCD/FST fixture queries produce identical JSON payload content.
- Determinism: two identical invocations produce byte-for-byte identical stdout and stderr.
- CLI help contract: top-level and `at --help` no longer include "not implemented yet" for `at`.

### Idempotence and Recovery

All edits are additive and can be re-applied safely. Re-running tests is idempotent. If schema assertions fail after payload updates, re-open `schema/wavepeek.json` and align enum/`oneOf`/`$defs` branches for `at`; then re-run `cargo test -q --test schema_cli` and `cargo test -q`. If a partially implemented state leaves `cli_contract` failing due to stale "not implemented" assertions, complete Milestone 3 wiring and update those assertions in the same change before final validation.

### Artifacts and Notes

Expected JSON shape example (content values depend on fixture and query):

    {
      "$schema": "https://github.com/kleverhq/wavepeek/blob/v0.2.0/schema/wavepeek.json",
      "command": "at",
      "data": {
        "time": "10ns",
        "signals": [
          {"name": "clk", "path": "top.clk", "value": "1'h1"},
          {"name": "data", "path": "top.data", "value": "8'h0f"}
        ]
      },
      "warnings": []
    }

Expected human output example:

    time: 10ns
    clk path=top.clk value=1'h1
    data path=top.data value=8'h0f

Expected out-of-range error example:

    error: args: time '11ns' is outside dump bounds [0ns, 10ns]. See 'wavepeek at --help'.

### Interfaces and Dependencies

Use existing dependencies only (`wellen`, `serde`, `clap`, `regex`, `thiserror`); no new crate is required.

At the end of implementation, these interfaces must exist:

- In `src/engine/at.rs`:

      #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
      pub struct AtSignalValue {
          pub name: String,
          pub path: String,
          pub value: String,
      }

      #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
      pub struct AtData {
          pub time: String,
          pub signals: Vec<AtSignalValue>,
      }

      pub fn run(args: AtArgs) -> Result<CommandResult, WavepeekError>;

- In `src/engine/mod.rs`:

      pub enum CommandName { Schema, Info, Scope, Signal, At }
      pub enum CommandData { Schema(...), Info(...), Scope(...), Signal(...), At(at::AtData) }

- In `src/waveform/mod.rs`, add a focused sampling API callable from `engine::at`:

      pub struct SampledSignal {
          pub path: String,
          pub width: u32,
          pub bits: String,
      }

      impl Waveform {
          pub fn sample_signals_at_time(
              &mut self,
              canonical_paths: &[String],
              query_time_raw: u64,
          ) -> Result<Vec<SampledSignal>, WavepeekError>;
      }

`sample_signals_at_time` must preserve input order and return one result per requested path, or fail with `WavepeekError::Signal` on first unresolved signal.

- In `src/output.rs`, add rendering branch for `CommandData::At` with a deterministic human format used by tests.

- In `schema/wavepeek.json`, add `$defs.atData` and update `command`/`data` unions and conditional `allOf` branches so `command=at` validates against `atData`.

Revision note (2026-02-22): Initial plan created to deliver full `at` command implementation with TDD-first milestones, schema/output integration, and explicit acceptance criteria.
Revision note (2026-02-22): Updated after independent double review to remove false-green test commands, clarify TDD staging for `cli_contract`, add glossary terms, and make parity/determinism validation fully executable.
Revision note (2026-02-22): Updated after implementation to mark milestones complete, record final decisions/discoveries, and capture validation evidence (`cargo test -q`, `make check`, `make ci`).
Revision note (2026-02-22): Updated after review findings to clarify integer-only `--time` grammar in design/decision log and add explicit regression coverage for decimal-token rejection.
Revision note (2026-02-22): Clarified scoped-signal resolution wording and added regression coverage so `--scope` mode remains explicitly short-name relative.
Revision note (2026-02-22): Added mandatory `--signals` enforcement in CLI parsing and regression test after final independent review found the missing required-flag contract.
