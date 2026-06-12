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
- [x] (2026-06-12T11:20:44Z) Committed this ExecPlan as branch-local WIP artifact in commit `1b233b8`.
- [x] (2026-06-12T11:33:17Z) Ran focused read-only plan review with code-feasibility and docs/schema/test-collateral lanes; reviewers found missing validation commands, benchmark baselines, FSDB tests, schema/output tests, overview docs, top-level help, and README table coverage.
- [x] (2026-06-12T11:33:17Z) Revised this ExecPlan to include the review findings before implementation.
- [x] (2026-06-12T11:33:17Z) Committed this reviewed ExecPlan revision before implementation.
- [x] (2026-06-12T11:46:32Z) Implemented parser, runtime, output, schema, docs, changelog, benchmark baseline, and test updates.
- [x] (2026-06-12T11:46:32Z) Ran targeted tests and schema checks, including `cargo test --test value_cli`, `cargo test --test cli_contract`, `cargo test --test schema_cli`, `cargo test --test docs_cli`, `cargo test --test command_fixture_contract`, `CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --test fsdb_cli`, `just check-schema`, benchmark helper tests, benchmark baseline self-compares, and `just check`.
- [x] (2026-06-12T12:06:54Z) Ran focused read-only implementation review across code/tests, docs/schema, and benchmark collateral lanes; fixed stale CLI-contract help expectation and added generated-help wording for the new human row format.
- [x] (2026-06-12T12:06:54Z) Ran an independent read-only control pass after fixes; it reported no substantive findings.
- [x] (2026-06-12T12:06:54Z) Committed implementation work in a conventional feature commit.

## Surprises & Discoveries

- Observation: The current schema artifact is checked in at `schema/wavepeek_v1.json`, and the `wavepeek schema` runtime command embeds that artifact through `src/schema_contract.rs` instead of generating it from Rust types.
  Evidence: `src/engine/schema.rs` returns `CANONICAL_SCHEMA_JSON`, and `schema/AGENTS.md` allows deliberate edits to the current-major artifact when the runtime embeds that artifact, followed by `just check-schema`.
- Observation: `value --at` currently parses a single string in `src/engine/value.rs`; `--signals` is already comma-split by clap in `src/cli/value.rs`.
  Evidence: `ValueArgs` has `pub at: String` and `pub signals: Vec<String>` with `value_delimiter = ','`.
- Observation: Cargo integration-test files must be selected with `cargo test --test <target>`, not by using the file stem as a test-name filter.
  Evidence: Plan review identified that `cargo test value_cli`, `cargo test cli_contract`, and `cargo test schema_cli` can run zero tests, so the plan now uses `cargo test --test value_cli`, `cargo test --test cli_contract`, and `cargo test --test schema_cli`.
- Observation: The checked-in benchmark corpus includes `.wavepeek.json` value artifacts under `bench/e2e/runs/`, and pre-commit includes an E2E smoke check.
  Evidence: Plan review found old object-shaped `value.data` baselines would need refreshing after the JSON array-shape change.
- Observation: A full FSDB-vs-FST functional baseline compare still reports existing non-value differences in scope and signal artifacts.
  Evidence: `python3 bench/e2e/perf.py compare --functional-only --revised bench/e2e/runs/baseline_fsdb --golden bench/e2e/runs/baseline_fst --verbose` reported mismatches for `scope_scr1_all_depth7_json` and `signal_scr1_top_recursive_depth2_json`; value artifacts were refreshed separately and each baseline directory self-compares successfully.

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
- Decision: In `schema/wavepeek_v1.json`, make `valueData` an array whose items reference the same `changeSnapshot` definition used by `changeData`.
  Rationale: The user asked for the JSON form to be effectively the same as `change` where practical. Reusing the snapshot definition keeps schema drift low while the envelope `command` still disambiguates semantics.
  Date/Author: 2026-06-12 / Grin

## Outcomes & Retrospective

Implementation and review are complete. Single-time `value` now emits one `change`-style row and one JSON snapshot in an array; comma-separated `--at` emits one row and one JSON snapshot per requested time. Public docs, help text, schema, tests, FSDB assertions, and benchmark value baselines were updated together. Review findings were fixed, the control pass was clean, and `just check` passed.

## Context and Orientation

`wavepeek` is a Rust command-line tool for inspecting waveform dumps. A waveform dump is a file such as `.vcd`, `.fst`, or `.fsdb` that records signal values over simulation time. The `value` command samples selected signals at explicit time points. The `change` command samples selected signals over a time range and prints rows when a trigger event fires and the sampled values changed.

The relevant files are:

`src/cli/value.rs` defines command-line arguments for `wavepeek value`. Today `--at` is a required string containing one time token, and `--signals` is a required comma-separated list parsed by clap into `Vec<String>`.

`src/cli/mod.rs` contains top-level and per-command help text. Its top-level command summary and its `Value` command section must describe one or more comma-separated time points and the new row output.

`src/engine/time.rs` defines `validate_time_token_to_raw`, which validates one time token against dump bounds and resolution and returns the raw backend timestamp. A raw timestamp is the integer tick used by the waveform backend. The engine should continue using this helper for each token.

`src/engine/value.rs` opens the waveform, validates the selected time, resolves requested signals, samples values, formats values as Verilog-style literals, and returns `CommandData::Value`. This file is the main implementation target.

`src/engine/change.rs` defines the target JSON row shape through `ChangeSnapshot { time, signals }` and `ChangeSignalValue { display, path, value }`. The `display` field is skipped in JSON and is used only for human rendering.

`src/engine/mod.rs` defines `CommandData`. It currently stores `Value(value::ValueData)` and `Change(Vec<change::ChangeSnapshot>)`. After this change, `value::ValueData` should be an ordered collection of snapshots rather than a single snapshot object.

`src/output.rs` renders human and JSON output. Its current `Value` human branch prints a header line `@10ns` followed by one `name value` line per signal. Its `Change` branch already renders row output as `@10ns clk=1'h1 data=8'h0f`. The `Value` branch should render the same row style.

`schema/wavepeek_v1.json` is the current major JSON Schema contract. The `valueData` definition currently describes one object with `time` and `signals`; it must become an array of snapshot objects, preferably by referencing the same snapshot definition used by `changeData`.

`docs/public/commands/value.md`, `docs/public/commands/overview.md`, `docs/public/commands/change.md`, `docs/public/reference/machine-output.md`, `docs/public/reference/command-model.md`, `docs/skills/wavepeek.md`, `README.md`, and `CHANGELOG.md` are user-facing collateral that mention `value`, `--at`, human output, JSON output, or the standard workflow.

`tests/value_cli.rs` owns the direct CLI behavior contract for `value`. `tests/change_cli.rs` is useful as the reference for `change` row and JSON expectations. `tests/cli_contract.rs` checks help text. `tests/schema_cli.rs` and `tools/schema/check_schema_contract.py` check schema availability and drift. `src/output.rs` also has renderer tests that can mention value shape. `tests/fsdb_cli.rs` has FSDB-enabled value JSON assertions that must move to the new array shape. The command fixture manifest under `tests/fixtures/cli/` should only be changed if a durable runtime fixture needs `value` coverage. Checked-in benchmark outputs under `bench/e2e/runs/` may include value `.wavepeek.json` artifacts and must be refreshed if pre-commit smoke compares them against current output.

## Open Questions

None. The user has clarified that `--at` is one argument, that a single time point should also use the new row format, that JSON should be as close to `change` as possible, and that compatibility with the previous human and JSON shape is not required for the v1 release candidate branch.

## Plan of Work

First, commit this plan and send it through focused read-only review. If review finds ambiguity, revise the plan and commit the revision before implementation.

In `src/cli/value.rs`, keep `pub at: String` but update the help text to say that `--at` accepts one or more comma-separated time points with explicit units, for example `10ns` or `5ns,10ns`.

In `src/cli/mod.rs`, update both the top-level `value` summary and the `Value` command long help. They should say that the command prints values for requested signals at each selected time point, that `--at` accepts a comma-separated list in one argument, that output preserves the `--at` order and the `--signals` order, and that time tokens must include units and align to dump precision.

In `src/engine/value.rs`, replace the single-time `ValueData { time, signals }` model with an array model. Define a snapshot struct analogous to `ChangeSnapshot`, for example `ValueSnapshot { time: String, signals: Vec<ValueSignalValue> }`, and define `pub type ValueData = Vec<ValueSnapshot>` or an equivalent transparent collection. Keep `ValueSignalValue.display` skipped from JSON so human rendering can use either requested names or canonical paths while JSON only exposes `path` and `value`.

Still in `src/engine/value.rs`, add a helper such as `parse_at_tokens(at: &str, metadata: &WaveformMetadata, dump_time: DumpTimeContext) -> Result<Vec<u64>, WavepeekError>`. It should split the single string on commas, trim whitespace around each token, reject empty tokens as an args error with the `wavepeek value --help` hint, validate each token with `validate_time_token_to_raw`, and map validation failures through the existing `map_value_time_validation_error`. It should return raw timestamps in the same order as the input tokens and should not sort or deduplicate.

In `src/engine/value.rs`, resolve requested signals once, collect canonical paths once, and sample for each raw timestamp. Sampling can call `waveform.sample_signals_at_time(&canonical_paths, raw_time)` for each time; this preserves current missing-value error behavior and duplicate signal order. For each time, build a `ValueSnapshot` with `format_raw_timestamp(raw_time, dump_time.dump_tick)` and formatted signal values. Return `CommandData::Value(snapshots)`.

In `src/output.rs`, update the `CommandData::Value` human branch to iterate snapshots and render each row with the same style as `CommandData::Change`: `@<time>` followed by `display=value` fields separated by spaces. Respect `--abs` the same way the old renderer did: when `args.abs` sets `HumanRenderOptions.signals_abs`, use the canonical `signal.path`; otherwise use the requested display name.

In `schema/wavepeek_v1.json`, change `valueData` from an object to an array of snapshots. The preferred schema is `"valueData": { "type": "array", "items": { "$ref": "#/$defs/changeSnapshot" } }` if the existing `changeSnapshot` description is general enough, or a new `valueSnapshot` definition with the same required fields and properties if wording must remain value-specific. If a shared snapshot definition is introduced, both `valueData` and `changeData` should reference it so the shape stays visibly unified. Do not modify prior-major schema artifacts.

Update tests. In `tests/value_cli.rs`, change the existing human expectations from the old multi-line block to one row. Change existing JSON expectations from an object to a one-element array. Add explicit coverage for `--at 5ns,10ns` in human and JSON modes, preserving input order. Add coverage that `--at 10ns,5ns,10ns` preserves duplicates and order. Add coverage that an empty comma entry such as `--at 5ns,,10ns` fails with `fatal: args:` and the value help hint. Keep existing error tests for invalid, decimal, out-of-range, and unaligned single tokens, because those should still fail the same way. Update unit tests in `src/engine/value.rs` to assert the returned payload is an array and that parsing multiple time points works. Update `src/output.rs` tests for the new value human rendering if they assert command rendering. Update `tests/schema_cli.rs` assertions that currently inspect `$defs.valueData.properties.time` so they assert the new array/shared snapshot schema. Update `tests/fsdb_cli.rs` value JSON assertions to use `data[0].time` and `data[0].signals` or the equivalent array-shape checks.

Update docs and user collateral. In `docs/public/commands/value.md`, describe single and comma-separated time sampling, the new human row output, and the JSON array shape. In `docs/public/commands/overview.md`, replace wording that says `value` samples one normalized timestamp. In `docs/public/reference/machine-output.md`, update the `value` row contract so `data` is an array of snapshots like `change`. In `docs/public/reference/command-model.md`, update point sampling language if it says `value` only has one selected time. In `README.md`, change the quick-start comment to mention one or more timestamps and update the command table row that says `value` is for a specific time. In `docs/skills/wavepeek.md`, change the workflow example to show `--at <TIME[,TIME...]>` or equivalent concise wording. In `CHANGELOG.md`, add an Unreleased `Changed` entry that says `value --at` now accepts comma-separated time points and emits `change`-style snapshot arrays and human rows.

Refresh benchmark collateral if current pre-commit smoke or tracked functional baselines compare stored `value` JSON output. Check `bench/e2e/runs/*/value_*.wavepeek.json` and update affected artifacts after the new `value.data` array shape lands, using the repository benchmark workflow rather than hand-editing opaque performance data where practical.

Finally, run formatting and checks. Use targeted tests first, then schema checks, then the local pre-handoff gate if time permits.

## Concrete Steps

Work from the repository root `/workspaces/wavepeek`.

After writing or revising this plan, commit it:

    git add docs/tracker/wip/value-at-list-plan.md
    git commit -m "docs(tracker): plan value multi-time sampling"

Run plan review through read-only subagents. The reviewer should inspect this plan and relevant code/docs paths, then return only concrete findings with file and line references.

Implement the code and collateral changes described above. Then run:

    cargo fmt --all
    cargo test --test value_cli
    cargo test --test cli_contract
    cargo test --test schema_cli
    cargo test --test fsdb_cli
    just check-schema

If those pass, run the repository pre-handoff gate:

    just check

Expected targeted test result is that the selected Rust integration suites finish successfully. `cargo test --test fsdb_cli` may skip or exercise feature-dependent cases depending on the local environment; if it cannot run due to missing FSDB SDK support, capture the exact result and rely on `just check` or CI for the supported matrix. Expected schema check result is no output ending in a failure and exit code `0`. If `just check` takes too long or fails for environment reasons unrelated to this change, capture the exact command and failure in this plan and report it in the final handoff.

After implementation and targeted checks, run focused read-only implementation review. Use separate lanes if the diff is broad: code/tests for engine and rendering behavior, and docs/schema for public collateral. Apply substantive fixes, rerun affected checks, and run one independent control pass if the first review required changes.

Commit implementation work with conventional messages. A likely final commit is:

    git add src tests docs schema bench README.md CHANGELOG.md
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
- 2026-06-12 / Grin: Plan review revision. It adds missing integration-test target commands, benchmark baseline handling, FSDB/schema/output test updates, public command overview docs, top-level help summary, and README command-table collateral.
- 2026-06-12 / Grin: Implementation progress revision. It records completed code, docs, schema, tests, benchmark baseline updates, validation commands, and the schema decision to reuse `changeSnapshot` for `valueData`.
- 2026-06-12 / Grin: Review completion revision. It records implementation review findings, the clean control pass, final validation, and the implementation commit state.
