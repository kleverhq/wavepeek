# Add Multi-Time `value --at` Support

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, a user can sample selected waveform signals at several explicit time points with one `value` command invocation, for example `wavepeek value --waves dump.vcd --at 5ns,10ns --scope top --signals clk,data`. The command will still accept one time point, but both one-time and multi-time human output will use the compact row format already used by `change`: one row per sampled time, beginning with `@<time>` and followed by `name=value` fields. JSON clients will receive `value.data` as an ordered array of time snapshots with the same `time` and `signals[{path,value}]` structure as `change.data`.

This matters because point sampling and range change inspection will have compatible output shapes. A human can see it working by running a single-time command and observing one row such as `@10ns clk=1'h1 data=8'h0f`, then running a comma-separated `--at 5ns,10ns` command and observing two rows in the requested time order.

## Non-Goals

This change does not add repeated `--at` flags. The supported multi-time syntax is one command-line argument containing a comma-separated list, such as `--at 5ns,10ns`. This change does not add time ranges, sorting, deduplication, or interpolation. It does not change signal name resolution rules: without `--scope`, `--signals` entries are top-related canonical paths; with `--scope`, entries are scope-relative names. It does not change `change` or `property` expression semantics.

## Progress

- [x] (2026-06-12T11:20:44Z) Confirmed requirements with the user: one `--at` argument, comma-separated time tokens, single-time output also changes to `change`-style rows, JSON shape should match `change` as closely as practical, and the plan is committed before implementation.
- [x] (2026-06-12T11:20:44Z) Read repository guidance for source, tests, public docs, schema, and WIP tracker artifacts.
- [x] (2026-06-12T11:20:44Z) Inspected current `value` and `change` implementation and tests: `src/engine/value.rs` currently returns one `ValueData { time, signals }`, while `src/engine/change.rs` returns `Vec<ChangeSnapshot>` and `src/output.rs` renders change rows as `@time name=value ...`.
- [ ] Commit this ExecPlan as a branch-local WIP artifact.
- [ ] Run focused read-only review of this ExecPlan and revise it if reviewers find gaps.
- [ ] Implement parser, runtime, output, schema, docs, changelog, and test updates.
- [ ] Run targeted tests and schema checks, then run the local pre-handoff gate if practical.
- [ ] Run focused read-only review of the implementation and revise the code or docs if reviewers find substantive issues.
- [ ] Commit implementation work in one or more conventional commits.

## Surprises & Discoveries

- Observation: The current schema artifact is checked in at `schema/wavepeek_v1.json`, and the `wavepeek schema` runtime command embeds that artifact through `src/schema_contract.rs` instead of generating it from Rust types.
  Evidence: `src/engine/schema.rs` returns `CANONICAL_SCHEMA_JSON`, and `schema/AGENTS.md` allows deliberate edits to the current-major artifact when the runtime embeds that artifact, followed by `just check-schema`.
- Observation: `value --at` currently parses a single string in `src/engine/value.rs`; `--signals` is already comma-split by clap in `src/cli/value.rs`.
  Evidence: `ValueArgs` has `pub at: String` and `pub signals: Vec<String>` with `value_delimiter = ','`.

## Decision Log

- Decision: Keep `ValueArgs.at` as one `String` and split it inside the engine instead of making clap produce `Vec<String>`.
  Rationale: The requested interface is one argument containing a comma-separated list. Engine-side splitting gives deterministic validation of empty entries and makes it clear that repeated `--at` flags are not part of the contract.
  Date/Author: 2026-06-12 / Grin
- Decision: Represent `value.data` as an array of snapshots, where each snapshot has `time` and `signals`, and each signal has `path` and `value` in JSON.
  Rationale: This matches the durable `change.data` shape closely and preserves the old single-time capability as a one-element array. The command envelope still says `"command":"value"`.
  Date/Author: 2026-06-12 / Grin
- Decision: Render human `value` output as one compact row per requested time, using the existing `change` row style `@<time> <display>=<value> ...`.
  Rationale: The user explicitly requested the human format to become analogous to `change` for both one and multiple time points.
  Date/Author: 2026-06-12 / Grin
- Decision: Preserve the user-provided order of `--at` tokens and preserve duplicate time points.
  Rationale: Existing `value` preserves `--signals` order and duplicates; doing the same for time points is deterministic and avoids hidden sorting or deduplication semantics.
  Date/Author: 2026-06-12 / Grin

## Outcomes & Retrospective

No implementation outcome yet. This plan records the intended behavior and the repository surfaces that must move together before code changes begin.

## Context and Orientation

`wavepeek` is a Rust command-line tool for inspecting waveform dumps. A waveform dump is a file such as `.vcd`, `.fst`, or `.fsdb` that records signal values over simulation time. The `value` command samples selected signals at explicit time points. The `change` command samples selected signals over a time range and prints rows when a trigger event fires and the sampled values changed.

The relevant files are:

`src/cli/value.rs` defines command-line arguments for `wavepeek value`. Today `--at` is a required string containing one time token, and `--signals` is a required comma-separated list parsed by clap into `Vec<String>`.

`src/cli/mod.rs` contains long help text for the top-level command model and per-command behavior. Its `Value` command section must describe one or more comma-separated time points and the new row output.

`src/engine/time.rs` defines `validate_time_token_to_raw`, which validates one time token against dump bounds and resolution and returns the raw backend timestamp. A raw timestamp is the integer tick used by the waveform backend. The engine should continue using this helper for each token.

`src/engine/value.rs` opens the waveform, validates the selected time, resolves requested signals, samples values, formats values as Verilog-style literals, and returns `CommandData::Value`. This file is the main implementation target.

`src/engine/change.rs` defines the target JSON row shape through `ChangeSnapshot { time, signals }` and `ChangeSignalValue { display, path, value }`. The `display` field is skipped in JSON and is used only for human rendering.

`src/engine/mod.rs` defines `CommandData`. It currently stores `Value(value::ValueData)` and `Change(Vec<change::ChangeSnapshot>)`. After this change, `value::ValueData` should be an ordered collection of snapshots rather than a single snapshot object.

`src/output.rs` renders human and JSON output. Its current `Value` human branch prints a header line `@10ns` followed by one `name value` line per signal. Its `Change` branch already renders row output as `@10ns clk=1'h1 data=8'h0f`. The `Value` branch should render the same row style.

`schema/wavepeek_v1.json` is the current major JSON Schema contract. The `valueData` definition currently describes one object with `time` and `signals`; it must become an array of snapshot objects, preferably by referencing the same snapshot definition used by `changeData`.

`docs/public/commands/value.md`, `docs/public/commands/change.md`, `docs/public/reference/machine-output.md`, `docs/public/reference/command-model.md`, `docs/skills/wavepeek.md`, `README.md`, and `CHANGELOG.md` are user-facing collateral that mention `value`, `--at`, human output, JSON output, or the standard workflow.

`tests/value_cli.rs` owns the direct CLI behavior contract for `value`. `tests/change_cli.rs` is useful as the reference for `change` row and JSON expectations. `tests/cli_contract.rs` checks help text. `tests/schema_cli.rs` and `tools/schema/check_schema_contract.py` check schema availability and drift. The command fixture manifest under `tests/fixtures/cli/` should only be changed if a durable runtime fixture needs `value` coverage.

## Open Questions

None. The user has clarified that `--at` is one argument, that a single time point should also use the new row format, that JSON should be as close to `change` as possible, and that compatibility with the previous human and JSON shape is not required for the v1 release candidate branch.

## Plan of Work

First, commit this plan and send it through focused read-only review. If review finds ambiguity, revise the plan and commit the revision before implementation.

In `src/cli/value.rs`, keep `pub at: String` but update the help text to say that `--at` accepts one or more comma-separated time points with explicit units, for example `10ns` or `5ns,10ns`.

In `src/cli/mod.rs`, update the `Value` command long help. It should say that the command prints values for requested signals at each selected time point, that `--at` accepts a comma-separated list in one argument, that output preserves the `--at` order and the `--signals` order, and that time tokens must include units and align to dump precision.

In `src/engine/value.rs`, replace the single-time `ValueData { time, signals }` model with an array model. Define a snapshot struct analogous to `ChangeSnapshot`, for example `ValueSnapshot { time: String, signals: Vec<ValueSignalValue> }`, and define `pub type ValueData = Vec<ValueSnapshot>` or an equivalent transparent collection. Keep `ValueSignalValue.display` skipped from JSON so human rendering can use either requested names or canonical paths while JSON only exposes `path` and `value`.

Still in `src/engine/value.rs`, add a helper such as `parse_at_tokens(at: &str, metadata: &WaveformMetadata, dump_time: DumpTimeContext) -> Result<Vec<u64>, WavepeekError>`. It should split the single string on commas, trim whitespace around each token, reject empty tokens as an args error with the `wavepeek value --help` hint, validate each token with `validate_time_token_to_raw`, and map validation failures through the existing `map_value_time_validation_error`. It should return raw timestamps in the same order as the input tokens and should not sort or deduplicate.

In `src/engine/value.rs`, resolve requested signals once, collect canonical paths once, and sample for each raw timestamp. Sampling can call `waveform.sample_signals_at_time(&canonical_paths, raw_time)` for each time; this preserves current missing-value error behavior and duplicate signal order. For each time, build a `ValueSnapshot` with `format_raw_timestamp(raw_time, dump_time.dump_tick)` and formatted signal values. Return `CommandData::Value(snapshots)`.

In `src/output.rs`, update the `CommandData::Value` human branch to iterate snapshots and render each row with the same style as `CommandData::Change`: `@<time>` followed by `display=value` fields separated by spaces. Respect `--abs` the same way the old renderer did: when `args.abs` sets `HumanRenderOptions.signals_abs`, use the canonical `signal.path`; otherwise use the requested display name.

In `schema/wavepeek_v1.json`, change `valueData` from an object to an array of snapshots. The preferred schema is `"valueData": { "type": "array", "items": { "$ref": "#/$defs/changeSnapshot" } }` if the existing `changeSnapshot` description is general enough, or a new `valueSnapshot` definition with the same required fields and properties if wording must remain value-specific. If a shared snapshot definition is introduced, both `valueData` and `changeData` should reference it so the shape stays visibly unified. Do not modify prior-major schema artifacts.

Update tests. In `tests/value_cli.rs`, change the existing human expectations from the old multi-line block to one row. Change existing JSON expectations from an object to a one-element array. Add explicit coverage for `--at 5ns,10ns` in human and JSON modes, preserving input order. Add coverage that `--at 10ns,5ns,10ns` preserves duplicates and order. Add coverage that an empty comma entry such as `--at 5ns,,10ns` fails with `fatal: args:` and the value help hint. Keep existing error tests for invalid, decimal, out-of-range, and unaligned single tokens, because those should still fail the same way. Update unit tests in `src/engine/value.rs` to assert the returned payload is an array and that parsing multiple time points works.

Update docs and user collateral. In `docs/public/commands/value.md`, describe single and comma-separated time sampling, the new human row output, and the JSON array shape. In `docs/public/reference/machine-output.md`, update the `value` row contract so `data` is an array of snapshots like `change`. In `docs/public/reference/command-model.md`, update point sampling language if it says `value` only has one selected time. In `README.md`, change the quick-start comment to mention one or more timestamps. In `docs/skills/wavepeek.md`, change the workflow example to show `--at <TIME[,TIME...]>` or equivalent concise wording. In `CHANGELOG.md`, add an Unreleased `Changed` entry that says `value --at` now accepts comma-separated time points and emits `change`-style snapshot arrays and human rows.

Finally, run formatting and checks. Use targeted tests first, then schema checks, then the local pre-handoff gate if time permits.

## Concrete Steps

Work from the repository root `/workspaces/wavepeek`.

After writing or revising this plan, commit it:

    git add docs/tracker/wip/value-at-list-plan.md
    git commit -m "docs(tracker): plan value multi-time sampling"

Run plan review through read-only subagents. The reviewer should inspect this plan and relevant code/docs paths, then return only concrete findings with file and line references.

Implement the code and collateral changes described above. Then run:

    cargo fmt --all
    cargo test value_cli
    cargo test cli_contract
    cargo test schema_cli
    just check-schema

If those pass, run the repository pre-handoff gate:

    just check

Expected targeted test result is that the selected Rust integration suites finish successfully. Expected schema check result is no output ending in a failure and exit code `0`. If `just check` takes too long or fails for environment reasons unrelated to this change, capture the exact command and failure in this plan and report it in the final handoff.

After implementation and targeted checks, run focused read-only implementation review. Use separate lanes if the diff is broad: code/tests for engine and rendering behavior, and docs/schema for public collateral. Apply substantive fixes, rerun affected checks, and run one independent control pass if the first review required changes.

Commit implementation work with conventional messages. A likely final commit is:

    git add src tests docs schema README.md CHANGELOG.md
    git commit -m "feat(value): support multi-time sampling"

Split documentation/schema/test commits only if the implementation naturally lands in separate reviewed slices.

## Validation and Acceptance

The implemented behavior is accepted when the following commands have the stated behavior against `tests/fixtures/hand/m2_core.vcd`.

Single-time human output:

    cargo run --quiet -- value --waves tests/fixtures/hand/m2_core.vcd --at 10ns --scope top --signals clk,data

Expected stdout:

    @10ns clk=1'h1 data=8'h0f

Expected stderr is empty and exit code is `0`.

Multi-time human output:

    cargo run --quiet -- value --waves tests/fixtures/hand/m2_core.vcd --at 5ns,10ns --scope top --signals clk,data

Expected stdout:

    @5ns clk=1'h1 data=8'h00
    @10ns clk=1'h1 data=8'h0f

Expected stderr is empty and exit code is `0`.

Single-time JSON output:

    cargo run --quiet -- value --waves tests/fixtures/hand/m2_core.vcd --at 10ns --scope top --signals clk,data --json

Expected JSON envelope fields include `"command":"value"`, an empty `diagnostics` array, and this `data` payload:

    [
      {
        "time": "10ns",
        "signals": [
          {"path": "top.clk", "value": "1'h1"},
          {"path": "top.data", "value": "8'h0f"}
        ]
      }
    ]

Multi-time JSON output for `--at 5ns,10ns` should be the same shape with two array entries in that order. `wavepeek schema` and `schema/wavepeek_v1.json` must both describe `valueData` as an array of snapshot objects, not a single object.

Invalid list syntax:

    cargo run --quiet -- value --waves tests/fixtures/hand/m2_core.vcd --at 5ns,,10ns --scope top --signals clk

Expected stdout is empty, exit code is `1`, and stderr starts with `fatal: args:` and contains `See 'wavepeek value --help'.`.

## Idempotence and Recovery

The code and docs edits are ordinary file changes and can be retried safely. `cargo fmt --all`, `cargo test ...`, `just check-schema`, and `just check` are safe to run repeatedly. Use repository-root `tmp/` only for disposable logs if needed and do not delete unrelated files there.

If implementation changes need to be abandoned, use `git diff` to inspect the current state and `git restore <path>` only for files owned by this task. Do not remove unrelated WIP artifacts under `docs/tracker/wip/`. If schema edits drift from runtime output, rerun `just check-schema` to get the exact mismatch, then make `schema/wavepeek_v1.json` and `wavepeek schema` agree before committing.

## Artifacts and Notes

Current implementation evidence before changes:

    src/engine/value.rs: ValueData is a struct with fields `time` and `signals`.
    src/output.rs: Value human output is `@time` on one line followed by `name value` lines.
    src/output.rs: Change human output is one `@time name=value ...` row per snapshot.
    schema/wavepeek_v1.json: valueData is an object; changeData is an array of changeSnapshot.

The desired durable contract after changes is:

    value.data == [ { time, signals: [ { path, value }, ... ] }, ... ]
    change.data == [ { time, signals: [ { path, value }, ... ] }, ... ]

The only JSON difference between `value` and `change` envelopes should be the `command` string and the semantics of which timestamps are included.

## Interfaces and Dependencies

No new external crates are required.

At the end of implementation, `src/engine/value.rs` should expose serializable types equivalent to:

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct ValueSignalValue {
        #[serde(skip_serializing)]
        pub display: String,
        pub path: String,
        pub value: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct ValueSnapshot {
        pub time: String,
        pub signals: Vec<ValueSignalValue>,
    }

    pub type ValueData = Vec<ValueSnapshot>;

The exact type alias is optional, but `CommandData::Value` must serialize as an array of snapshot objects. `src/output.rs` must treat `CommandData::Value` as a slice-like collection of snapshots for human rendering. `src/engine/value.rs` must continue using `crate::engine::time::validate_time_token_to_raw` for each token and `crate::engine::value_format::format_verilog_literal` for each sampled signal value.

## Plan Revision Notes

- 2026-06-12 / Grin: Initial plan. It records the user-confirmed CLI contract, identifies implementation and collateral files, defines acceptance commands, and preserves the requirement to review and commit the plan before implementation.
