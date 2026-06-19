# Add JSONL Streaming Output for Waveform Commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill. It is intentionally self-contained so a new contributor can implement the feature from this file alone.

## Purpose / Big Picture

`wavepeek` already has `--json`, but that mode emits one complete JSON document after the command finishes. For large waveform queries, especially `change` and `property`, this means an automation client sees no usable results until the engine has collected every row into memory and serialized the final envelope. After this change, waveform-inspection commands can use `--jsonl` to emit newline-delimited JSON records as rows become available, so scripts and agents can consume partial results, stop early, detect incomplete streams, and avoid waiting for one giant buffered JSON document.

A user should be able to run a command such as:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top --signals data --on 'posedge clk' --jsonl

and observe stdout containing one valid JSON object per line. The first line is a `begin` record, data rows appear as `item` records, non-fatal warnings appear as `diagnostic` records, and a successful run ends with an `end` record containing counts and a `truncated` flag. The existing `--json` output must remain byte-compatible except for intentional unrelated fixture drift; the new mode is additive.

## Non-Goals

This plan does not add `--jsonl` to helper command families such as `docs`, `schema`, or `skill`. Those commands keep their existing human or `--json` surfaces. The only helper-surface exception is `wavepeek schema --stream`, which prints the JSON Schema artifact for stream records; that flag is not an output mode and does not make `schema` itself stream JSONL.

This plan does not remove or rename `--json`, does not change human-readable output, and does not change process-level fatal error formatting on stderr. Fatal errors still use `fatal: <category>: <message>` and non-zero exit codes. If a fatal error happens after a JSONL stream has begun, consumers detect the unsuccessful stream by the missing successful `end` record and by the non-zero process exit status.

This plan does not fully redesign the waveform facade to stream hierarchy traversal from the backend. The first implementation streams command rows where the engine already iterates over rows naturally, with `change` and `property` as the primary targets. `scope` and `signal` still may collect bounded results from the waveform layer before emitting item lines, but their JSONL contract is still available for consistency.

This plan does not require a runtime JSON Schema validation dependency in the shipped CLI. Tests still need a dev-only schema validator, either in Rust test dependencies or Python helper tooling. The checked-in stream schema validates one JSONL record at a time. Stateful whole-stream rules such as “first record is `begin`” and “sequence numbers increase by one” are validated by tests and by a small stream validator helper, not by JSON Schema alone.

## Progress

- [x] (2026-06-19 00:00Z) Captured product decisions from the issue discussion: add a distinct `--jsonl` mode only to waveform commands, keep `--json` unchanged, define a dedicated stream schema, and validate whole-stream ordering with stateful tests rather than JSON Schema alone.
- [x] (2026-06-19 00:00Z) Read repository guidance for source, tests, public docs, schemas, tracker WIP artifacts, architecture, style, testing, and quality gates.
- [x] (2026-06-19 00:00Z) Inspected the current command, output, schema, and representative test layout: `src/cli/*`, `src/engine/mod.rs`, `src/output.rs`, `src/schema_contract.rs`, `schema/wavepeek_v1.json`, `tests/schema_cli.rs`, `tests/change_cli.rs`, `tests/property_cli.rs`, and public docs under `docs/public/`.
- [x] (2026-06-19 00:00Z) Drafted this ExecPlan under `docs/tracker/wip/jsonl-stream-output-execplan.md` as a branch-local tracked handoff artifact.
- [x] (2026-06-19 00:00Z) Ran focused read-only review lanes for implementation architecture and public contract/schema/test coverage.
- [x] (2026-06-19 00:00Z) Revised this ExecPlan to address review findings: staged flag routing, broken-pipe handling, flush verification, schema publication, command-to-item schema constraints, runtime schema validation, and anti-cosmetic streaming tests.
- [x] (2026-06-19 00:00Z) Ran an independent control review, fixed the remaining publication and broken-pipe test gaps, and rechecked with no substantive findings.
- [x] (2026-06-19 00:00Z) Committed the reviewed ExecPlan with a conventional commit message.

## Surprises & Discoveries

- Observation: Current `CommandResult` carries a `json: bool`, and every command run returns a complete `CommandData` value before `src/output.rs` writes stdout. This is enough for `--json`, but not enough for real streaming.
  Evidence: `src/engine/mod.rs` defines `CommandResult { command, json, human_options, data, diagnostics }`, and `src/output.rs` renders after `engine::run` completes.

- Observation: The current schema command prints a checked-in artifact byte-for-byte rather than generating schema dynamically at runtime.
  Evidence: `tests/schema_cli.rs` has `schema_command_prints_canonical_artifact_bytes`, and `tools/schema/check_schema_contract.py` compares `schema/wavepeek_v1.json` with `cargo run --quiet -- schema`.

- Observation: JSON Schema can validate one JSON object, but JSONL is a sequence of JSON objects separated by newlines. A stream-level invariant such as “the last successful record is `end`” needs a stateful validator.
  Evidence: This is a property of JSONL as used here: each stdout line is parsed with `serde_json::from_str`, while order and counters require remembering earlier records.

- Observation: The docs publication tooling currently collects root schema artifacts by the `wavepeek_v*.json` pattern, so a new stream schema named `wavepeek-stream-v1.json` would not automatically be published to the root Pages URL.
  Evidence: `tools/docs/publish_docs.py` has `collect_root_artifacts` globbing `schema/wavepeek_v*.json`, and `tools/docs/check_deploy.py` validates only the current major envelope schema artifact.

- Observation: A writer test backed only by `Vec<u8>` cannot prove per-record flushing, and a final stdout-shape integration test cannot prove that `change` and `property` avoided the old buffered-vector path.
  Evidence: A `Vec<u8>` implements `Write` but ignores `flush`, and a buffered adapter can produce identical final JSONL lines after collecting all rows first.

## Decision Log

- Decision: Add `--jsonl` only to waveform-inspection commands: `info`, `scope`, `signal`, `value`, `change`, and `property`.
  Rationale: The user explicitly requested that the new output mode be limited to waveform commands. Helper commands already have deterministic small outputs and are not the large-waveform result path this feature targets.
  Date/Author: 2026-06-19 / Grin

- Decision: Keep `--json` unchanged and mutually exclusive with `--jsonl`.
  Rationale: Existing automation may depend on the exact single-document envelope. A separate mode avoids smuggling stream semantics into the old contract and lets clap report a clear argument error when both flags are present.
  Date/Author: 2026-06-19 / Grin

- Decision: Use a dedicated stream schema artifact at `schema/wavepeek-stream-v1.json`, exposed by `wavepeek schema --stream` and by a stream URL such as `https://kleverhq.github.io/wavepeek/wavepeek-stream-v1.json`.
  Rationale: The existing `schema/wavepeek_v1.json` describes one complete `--json` envelope. JSONL records have a different top-level shape and need a separate contract. Exposing it through `schema --stream` lets installed binaries provide the exact contract they ship without adding `--jsonl` to the schema command itself.
  Date/Author: 2026-06-19 / Grin

- Decision: Validate JSONL one line at a time with JSON Schema and validate whole-stream rules with Rust tests.
  Rationale: JSON Schema is stateless. It can say “this line is a valid `item` record,” but it cannot count prior item records or prove that `end.summary.items` matches the number of item lines. Tests can and should cover those stateful rules.
  Date/Author: 2026-06-19 / Grin

- Decision: In successful streams, emit records in this order: one `begin`, zero or more `item`, zero or more `diagnostic`, and one `end`.
  Rationale: This lets data consumers process rows early while keeping non-fatal diagnostics in the same relative order they currently occupy in the `diagnostics` array. Diagnostics are small, so buffering them until after items is acceptable.
  Date/Author: 2026-06-19 / Grin

- Decision: The first implementation treats a missing `end` record as the signal for an incomplete or failed stream rather than trying to convert all mid-stream fatal errors into JSONL `error` diagnostics.
  Rationale: Current fatal errors have a stable stderr and exit-code contract. Retrofitting all mid-stream failures into terminal JSONL records would complicate error ownership and could weaken fail-fast behavior. Consumers already need to handle missing `end` for cancellation and timeouts.
  Date/Author: 2026-06-19 / Grin

- Decision: Milestone 1 must not leave `--jsonl` accepted but silently rendered through the old path. If the flag is parsed before full routing exists, dispatch must return a clear temporary argument or unimplemented error.
  Rationale: Independent milestones should be safe to test. A half-wired output flag that falls back to human or buffered JSON would create misleading behavior and make later failures harder to diagnose.
  Date/Author: 2026-06-19 / Grin

- Decision: Treat `std::io::ErrorKind::BrokenPipe` during JSONL writing as a normal early-consumer-stop path, not as `fatal: internal` noise.
  Rationale: JSONL consumers often pipe to tools such as `head` or stop reading once they have enough rows. The CLI should stop cleanly and silently on a closed stdout pipe, while still reporting other write failures.
  Date/Author: 2026-06-19 / Grin

- Decision: The stream schema must bind each `command` value to its matching `item` payload shape, and tests must reject representative mismatches.
  Rationale: A per-record schema that accepts `command: "property"` with a `change` snapshot payload is technically valid JSON but not a valid wavepeek stream record. The schema should catch that class of client/server drift.
  Date/Author: 2026-06-19 / Grin

- Decision: Runtime JSONL output must be validated against the new stream schema in tests, using a dev-only validator dependency or Python helper rather than a runtime dependency.
  Rationale: Parsing lines and checking sequence order proves stream mechanics, but it does not prove that the emitted records match the published schema. A schema-backed test closes that gap without adding dependencies to the shipped binary.
  Date/Author: 2026-06-19 / Grin

- Decision: Publishing the stream schema is part of the feature, not a later docs chore.
  Rationale: JSONL `begin` records will point at `https://kleverhq.github.io/wavepeek/wavepeek-stream-v1.json`. If the Pages pipeline does not publish and check that artifact, the URL can 404 even when local `schema --stream` works.
  Date/Author: 2026-06-19 / Grin

## Outcomes & Retrospective

This feature is not implemented yet, but the reviewed ExecPlan is ready for execution. When implemented, it should make large `change` and `property` runs observable before completion while keeping existing human and `--json` behavior stable. The review tightened the plan around the main risks: accidental cosmetic JSONL over already-buffered vectors, unpublished schema URLs, stream schema drift from runtime records, and normal broken-pipe behavior for early-stopping consumers.

## Context and Orientation

The repository is a Rust CLI named `wavepeek`. A “waveform” is a digital simulation dump, such as VCD, FST, or optional FSDB. A “waveform-inspection command” is one of the commands that opens a waveform file and returns information from it: `info`, `scope`, `signal`, `value`, `change`, and `property`.

The current top-level flow is: `src/main.rs` calls `wavepeek::run_cli()`, `src/lib.rs` exposes that function, `src/cli/mod.rs` parses clap arguments and dispatches to the engine, `src/engine/mod.rs` chooses the command implementation, and `src/output.rs` writes stdout and non-fatal diagnostics. The command-specific CLI argument structs live under `src/cli/`. The command-specific engine implementations live under `src/engine/`.

The current machine output contract is the `--json` envelope. That envelope is documented in `docs/public/reference/machine-output.md`, referenced by `docs/public/reference/command-model.md`, and encoded in `schema/wavepeek_v1.json`. The runtime exposes that checked-in artifact through `wavepeek schema`, and `tools/schema/check_schema_contract.py` verifies that the runtime output matches the artifact.

The current command data types are these Rust structures:

- `src/engine/info.rs`: `InfoData` with `time_unit`, `time_start`, and `time_end`.
- `src/engine/scope.rs`: `ScopeEntry` with `path`, `depth`, and `kind`.
- `src/engine/signal.rs`: `SignalEntry` with serialized fields `name`, `path`, `kind`, and optional `width`; the `display` helper field is skipped in JSON.
- `src/engine/value.rs`: `ValueSnapshot` with `time` and `signals`; each signal serializes `path` and `value`, while `display` is skipped.
- `src/engine/change.rs`: `ChangeSnapshot` with `time` and `signals`; each signal serializes `path` and `value`, while `display` is skipped.
- `src/engine/property.rs`: `PropertyCaptureRow` with `time` and `kind`, where `kind` serializes as `match`, `assert`, or `deassert`.
- `src/diagnostic.rs`: `Diagnostic` with `kind`, optional `code`, and `message`. Warning codes use `WPK-W####`; error codes use `WPK-E####`; info diagnostics have no code.

The new JSONL contract uses one JSON object per stdout line. “JSONL” means newline-delimited JSON: each line is complete valid JSON, and the whole stdout stream is not wrapped in an array or object. The planned record shapes are:

    {"type":"begin","seq":0,"command":"change","$schema":"https://kleverhq.github.io/wavepeek/wavepeek-stream-v1.json"}
    {"type":"item","seq":1,"command":"change","item":{"time":"5ns","signals":[{"path":"top.data","value":"8'h0f"}]}}
    {"type":"diagnostic","seq":2,"command":"change","diagnostic":{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entry (use --max to increase limit)"}}
    {"type":"end","seq":3,"command":"change","summary":{"status":"ok","items":1,"diagnostics":1,"truncated":true}}

The `seq` field starts at `0` on `begin` and increases by one for every following record. The `command` field is one of `info`, `scope`, `signal`, `value`, `change`, or `property`. The `item` payload is the command-specific row type listed above. `begin` is emitted once, `end` is emitted once only on successful completion, and a missing `end` means the process did not complete a successful stream.

## Open Questions

There are no blocking open questions. The stream schema will validate individual records, while tests validate whole-stream order and counters. The stream schema will be exposed by `wavepeek schema --stream`, while `wavepeek schema` remains the existing single-document `--json` schema.

## Plan of Work

### Milestone 1: Add the output-mode model and CLI surface

The first milestone creates the output-mode model and a safe public flag surface. Define a shared `OutputMode` enum in a new source file, `src/output_mode.rs`, with variants `Human`, `Json`, and `Jsonl`. Export it from `src/lib.rs` or keep it crate-private according to the surrounding module style. Add helper constructors so a pair of booleans from clap can become one mode, and so old helper commands with only `--json` can continue to map to `Human` or `Json`.

Update `src/cli/info.rs`, `src/cli/scope.rs`, `src/cli/signal.rs`, `src/cli/value.rs`, `src/cli/change.rs`, and `src/cli/property.rs` to add a `jsonl: bool` field with long flag `--jsonl`, help heading `Output options`, and wording such as “Stream newline-delimited JSON output”. Mark it as conflicting with `--json`. Keep helper command arg structs under `src/cli/docs.rs`, `src/cli/schema.rs`, and `src/cli/skill.rs` without `--jsonl`.

Update `src/engine/mod.rs` so `CommandResult` stores `output_mode: OutputMode` instead of `json: bool`. Add an `output_mode()` method on `engine::Command` or equivalent command-specific helper so `src/cli/mod.rs` can decide whether to run the normal buffered path or the JSONL path. Existing command implementations should initially set `OutputMode::Json` when `--json` is present, `OutputMode::Jsonl` when `--jsonl` is present, and `OutputMode::Human` otherwise.

Do not leave the new flag silently falling through to human or old buffered output. Until Milestone 4 adds the JSONL execution path, `src/cli/mod.rs` must detect `OutputMode::Jsonl` and return a clear temporary failure such as `WavepeekError::Unimplemented("JSONL output routing")` or `WavepeekError::Args("--jsonl is not wired yet")`. That temporary guard is removed when `engine::run_jsonl` exists.

At the end of this milestone, `wavepeek change --help` and the other waveform command help outputs include `--jsonl`; `wavepeek docs topics --help` does not. Passing both `--json` and `--jsonl` to a waveform command fails with a clap argument error before opening the waveform file. Passing only `--jsonl` fails clearly with the temporary routing error rather than producing a misleading non-JSONL result.

### Milestone 2: Add the stream schema artifact and schema command exposure

Create `schema/wavepeek-stream-v1.json`. This schema must use JSON Schema draft 2020-12 and describe one JSONL record, not the whole stream. The top-level schema should be an object with `oneOf` or command-specific conditional branches for these record variants: `begin`, `item`, `diagnostic`, and `end`. Each variant must reject unexpected properties with `additionalProperties: false`.

The schema must include definitions for the shared command enum, diagnostics, stream summary, and each waveform item payload. Copy or mirror the serialized shapes from `schema/wavepeek_v1.json` for the waveform data structures only. Do not include `docs topics` or `docs search` payloads, because `--jsonl` is not available for helper commands. The schema must bind `item.command` to the matching `item` payload shape: an `item` record with `command: "property"` must require a `PropertyCaptureRow`, and an `item` record with `command: "change"` must require a `ChangeSnapshot`. Use `oneOf` branches or `if`/`then` conditionals to reject representative command/payload mismatches.

Update `src/schema_contract.rs` to define stream schema constants next to the existing canonical envelope schema constants. The stream schema URL should be `https://kleverhq.github.io/wavepeek/wavepeek-stream-v1.json`, and the embedded bytes should come from `include_str!("../schema/wavepeek-stream-v1.json")` or the existing style used by that file. The existing `wavepeek schema` behavior must not change.

Update `src/cli/schema.rs` to add `--stream` with help text such as “Print the JSONL stream record schema instead of the JSON envelope schema”. Update `src/engine/schema.rs` so `wavepeek schema --stream` returns the stream schema bytes, while `wavepeek schema` returns the existing envelope schema bytes. This is not a `--jsonl` mode; it is a schema selector.

Update `tools/schema/check_schema_contract.py` so `just check-schema` verifies both canonical artifacts. The new checks should prove that `schema/wavepeek-stream-v1.json` exists, is valid JSON, ends with a trailing newline, has the expected stream URL pattern or constant, exactly matches `cargo run --quiet -- schema --stream`, and rejects representative invalid command/payload pairs. Keep the existing envelope checks intact.

Update the docs publication helpers so the public stream schema URL resolves after release. In `tools/docs/publish_docs.py`, make `collect_root_artifacts` copy `schema/wavepeek-stream-v*.json` artifacts in addition to `wavepeek_v*.json`. Also update every downstream publication guard that currently only knows `wavepeek_v*.json`: `stage_publication_artifacts` must stage the stream schema path, `allowed_path_patterns` and `path_allowed` must allow `wavepeek-stream-v*.json`, `required_pages_artifact_paths` must require the stream schema in the exported Pages artifact, and `verify_root_artifacts` must verify the stream schema blob on the staged `gh-pages` branch. In `tools/docs/check_deploy.py`, add a stream-schema artifact name such as `wavepeek-stream-v1.json`, fetch it from the Pages root, and validate its basic JSON Schema shape and title. Update `tools/docs/test_publish_docs.py`, `tools/docs/test_check_deploy.py`, and `tools/docs/README.md` so helper tests and helper docs describe both envelope and stream schema artifacts.

At the end of this milestone, `cargo run --quiet -- schema --stream` prints the checked-in stream schema exactly, `cargo run --quiet -- schema` still prints `schema/wavepeek_v1.json` exactly, `just check-schema` validates both artifacts, and the docs-site helper tests prove both schema artifacts are staged and deploy-checked.

### Milestone 3: Implement a reusable JSONL writer

Add reusable JSONL rendering support to `src/output.rs`. A simple design is a generic writer type such as `JsonlWriter<W: std::io::Write>`. It owns the command name, next sequence number, item count, diagnostic count, and a line-oriented writer. Its methods should be:

    begin(command) -> Result<(), WavepeekError>
    item<T: serde::Serialize>(&mut self, item: &T) -> Result<(), WavepeekError>
    diagnostic(&mut self, diagnostic: &Diagnostic) -> Result<(), WavepeekError>
    end(&mut self, truncated: bool) -> Result<(), WavepeekError>

The `begin` method writes the first line with `seq: 0`, the command name, and the stream schema URL. `item` writes `type: "item"` records and increments the item count. `diagnostic` writes `type: "diagnostic"` records and increments the diagnostic count. `end` writes `type: "end"` with `summary.status: "ok"`, `summary.items`, `summary.diagnostics`, and the supplied `summary.truncated` value.

Each method must serialize exactly one JSON object, append one newline byte, and flush after the newline. Flushing is necessary so a consumer reading from a pipe sees rows during execution rather than after a process-level buffer fills. Convert serialization failures and ordinary I/O failures into `WavepeekError::Internal` unless the repository already has a more precise output error variant.

Handle `std::io::ErrorKind::BrokenPipe` specially. The preferred design is to add a non-reporting broken-pipe path, such as `WavepeekError::BrokenPipe` with exit code `0` and CLI handling that suppresses stderr, or an internal stream-stop signal that `src/cli/mod.rs` treats as successful early termination. Do not print `fatal: internal` when a downstream consumer closes stdout early, for example `wavepeek change ... --jsonl | head -n 5`.

Add unit tests in `src/output.rs` or a nearby module that parse each line as `serde_json::Value` and assert begin/item/diagnostic/end order, sequence numbers, counters, and newline termination. Use both a `Vec<u8>` sink for easy output inspection and a custom `Write` test sink that records `flush()` calls, then assert one flush happens after every emitted record. Add a broken-pipe test sink that returns `ErrorKind::BrokenPipe` and assert the writer returns the chosen non-reporting broken-pipe path.

At the end of this milestone, there is a tested writer that can stream records independently of the command engines.

### Milestone 4: Route JSONL execution from the CLI without disturbing existing modes

Update the CLI dispatch in `src/cli/mod.rs` and the engine dispatch in `src/engine/mod.rs` so waveform commands with `OutputMode::Jsonl` use a JSONL execution path. One practical shape is to add:

    pub fn run_jsonl<W: std::io::Write>(command: Command, writer: &mut JsonlWriter<W>) -> Result<(), WavepeekError>

in `src/engine/mod.rs`. It should match only `Info`, `Scope`, `Signal`, `Value`, `Change`, and `Property`. Helper commands should be unreachable through clap because they have no `--jsonl`; if defensive handling is needed, return a `WavepeekError::Args` saying JSONL is available only for waveform commands.

In `src/cli/mod.rs`, after parsing and converting to `engine::Command`, choose the execution path from the command output mode. For `Human` and `Json`, keep calling the existing buffered `engine::run` and `output::write`. For `Jsonl`, remove the temporary Milestone 1 guard, lock stdout, construct a `JsonlWriter`, and call `engine::run_jsonl`. Keep process-level fatal error handling unchanged so fatal errors still go to stderr and set the same exit code, except for the explicit broken-pipe early-termination path, which must be silent and successful.

For the first pass of this milestone, it is acceptable for `run_jsonl` to call existing buffered command implementations and then adapt the returned `CommandResult` into JSONL records. This temporary adapter is useful for proving routing and contract tests. It must be replaced or bypassed for `change` and `property` in later milestones so those commands actually emit rows during execution.

At the end of this milestone, all waveform commands can produce syntactically correct JSONL, but `change` and `property` may still be internally buffered until the next milestones are complete.

### Milestone 5: Refactor `change` to emit snapshots during execution

This is the most important milestone because `change` is the hot path for large result sets. The current implementation in `src/engine/change.rs` returns `ChangeRunOutput { snapshots: Vec<ChangeSnapshot>, truncated: bool }`, and the baseline, fused, and edge-fast engines push snapshots into a vector. Refactor the shared execution helpers so they accept an emitter callback or a small sink trait and return counts plus truncation state instead of requiring a vector.

A target shape is:

    struct ChangeRunStats {
        emitted: usize,
        truncated: bool,
    }

    fn run_baseline(..., emit: impl FnMut(ChangeSnapshot) -> Result<(), WavepeekError>) -> Result<ChangeRunStats, WavepeekError>

Apply the same shape to the fused and edge-fast paths. The old `--json` and human paths can pass an emitter that pushes snapshots into a vector. The `--jsonl` path passes an emitter that calls `JsonlWriter::item(&snapshot)` immediately.

Keep all existing engine-selection behavior and debug trace events deterministic. Update debug counts to use emitted counts rather than vector lengths where necessary. Preserve diagnostics: if no snapshots are emitted, add the existing empty-result warning; if truncation occurs, add the existing truncation warning. In JSONL mode, buffer those diagnostics until after all item records, then write them as diagnostic records before the final `end`.

Add focused engine-level tests that call the new `change` streaming helper with a test emitter. One test should make the emitter return a sentinel error after the first emitted snapshot and assert that the helper propagates that error immediately; this proves the helper is driving emission during execution rather than after a completed `Vec` has already been returned. Also add a structural assertion in review or tests that the final `change --jsonl` path does not call the old buffered `change::run(args)` adapter.

At the end of this milestone, `change --jsonl` begins stdout before the row loop, writes each `ChangeSnapshot` as it is found, does not store the complete result set solely for output, and writes `end.summary.truncated` accurately.

### Milestone 6: Refactor `property` to emit rows during execution

Refactor `src/engine/property.rs` similarly. The current `run` builds a `Vec<PropertyCaptureRow>`. Identify the loop that evaluates the logical expression at selected event timestamps and pushes rows for `match`, `assert`, or `deassert` capture. Extract a helper that accepts an emitter callback for `PropertyCaptureRow` and returns the emitted row count.

The old buffered path passes an emitter that pushes into a vector. The JSONL path passes an emitter that writes each row immediately. Preserve the existing empty-result diagnostic `WPK-W0003` when no rows are emitted. `property` currently has no `--max` flag, so `end.summary.truncated` should be `false` unless a future limit is added.

Add focused engine-level tests like the `change` tests: call the property streaming helper with an emitter that returns a sentinel error after the first emitted row and assert immediate propagation. Also assert by test or final review that the final `property --jsonl` path does not call the old buffered `property::run(args)` adapter.

At the end of this milestone, `property --jsonl` streams property capture rows during execution and ends with accurate row and diagnostic counts.

### Milestone 7: Finish JSONL coverage for `info`, `scope`, `signal`, and `value`

Implement direct JSONL functions for the remaining waveform commands. For `info`, open the waveform, gather metadata, write one `InfoData` item, and end with `items: 1`. For `value`, emit one `ValueSnapshot` per requested timestamp as each snapshot is sampled. For `scope` and `signal`, it is acceptable in this plan to reuse existing bounded collection helpers and then write one item per entry, because their current waveform facade APIs return collections. Preserve existing diagnostics such as `WPK-W0001` limit-disabled and `WPK-W0002` truncation warnings, writing them after items and before `end`.

At the end of this milestone, every waveform command has a first-class JSONL path, and the temporary buffered adapter from Milestone 4 can be removed if it exists.

### Milestone 8: Add integration tests for JSONL behavior

Create a new integration test file, `tests/jsonl_cli.rs`. Add helpers that run the CLI, split stdout on newlines, parse each line as `serde_json::Value`, validate each record against `schema/wavepeek-stream-v1.json`, and validate the whole stream with stateful checks. Use a dev-only schema validator: either a Rust dev-dependency such as a JSON Schema validator crate used only in tests, or a Python helper backed by a pinned test/dev dependency. Do not add a runtime dependency to the shipped binary. The helper should assert:

- stdout is non-empty on success;
- every line is valid JSON;
- every line validates against `schema/wavepeek-stream-v1.json`;
- the first record is `type: "begin"` and `seq: 0`;
- all records have the same `command`;
- `seq` increases by one with no gaps;
- item records appear before diagnostic records;
- the last successful record is `type: "end"`;
- `end.summary.status` is `ok`;
- `end.summary.items` equals the number of `item` records;
- `end.summary.diagnostics` equals the number of `diagnostic` records;
- `end.summary.truncated` matches the presence of the truncation diagnostic for commands with limits.

Add focused cases for:

- `change --jsonl` on `tests/fixtures/hand/m2_core.vcd`, asserting item shape and command name.
- `change --max 1 --jsonl`, asserting one item, one truncation diagnostic, and `truncated: true`.
- an empty `change --jsonl` or `property --jsonl` query, asserting zero items, one empty-result diagnostic, and `truncated: false`.
- `property --jsonl` with a small inline VCD fixture like the existing `tests/property_cli.rs` fixture, asserting `kind` values.
- `info --jsonl`, asserting one item with time metadata.
- `scope --jsonl`, `signal --jsonl`, and `value --jsonl`, asserting representative item payloads.
- `--json` plus `--jsonl` conflict for at least one waveform command, asserting failure, empty stdout, and a clap conflict message on stderr.
- helper commands do not accept `--jsonl`, either by checking help text or by asserting `docs topics --jsonl` fails as an unexpected argument.
- a process-level broken-pipe case: spawn a `change --jsonl` command that can emit several records, read only the first few stdout lines, close/drop the stdout reader, wait for the process, and assert the process exits successfully with empty stderr. This catches CLI-level regressions where writer unit tests pass but `run_cli` still prints `fatal: internal` or returns a non-zero status on early consumer shutdown.

Update existing tests only where the internal `CommandResult` field rename requires it. Existing `--json` CLI tests should continue to pass unchanged.

At the end of this milestone, the new mode is covered end-to-end through the public binary.

### Milestone 9: Add schema and docs tests

Extend `tests/schema_cli.rs` to cover the stream schema. Add tests that:

- `wavepeek schema --stream` prints exactly `schema/wavepeek-stream-v1.json`;
- the stream schema output is valid JSON;
- the stream schema has the expected title and URL pattern;
- the stream schema command enum contains exactly the waveform commands;
- the stream schema includes `begin`, `item`, `diagnostic`, and `end` record definitions;
- representative valid records for every waveform command validate against the stream schema;
- representative invalid command/payload pairs fail validation, such as `command: "property"` with a `change` snapshot payload and `command: "change"` with a property capture row payload;
- `wavepeek schema` remains byte-identical to `schema/wavepeek_v1.json`.

The schema-backed checks are required. If no validator is already available in the devcontainer, add a dev-only validation dependency and document it in the appropriate dependency file. Acceptable shapes are a Rust dev-dependency used only by `tests/schema_cli.rs` and `tests/jsonl_cli.rs`, or a Python helper under `tools/schema/` backed by a pinned package in the container requirements. Do not add a runtime dependency to `wavepeek` for schema validation.

At the end of this milestone, schema exposure, per-record schema conformance, command-to-item schema constraints, and stateful stream invariants are tested.

### Milestone 10: Update public docs and architecture notes

Update `docs/public/reference/machine-output.md` to add a JSONL section. Explain the difference between `--json` and `--jsonl`, define every record type, state that each line validates independently against `wavepeek-stream-v1.json`, and state that consumers must require a final `end` record with `summary.status: "ok"` before treating the stream as complete.

Update `docs/public/reference/command-model.md` to mention that waveform commands support `--jsonl` and helper commands do not. Keep exact flag tables out of narrative docs; generated help remains the exact flag authority.

Update `docs/public/commands/overview.md` and the waveform command topics under `docs/public/commands/` so users know when to choose `--jsonl`: large result sets, incremental processing, and automation that wants partial rows. At minimum update `change.md`, `property.md`, and any shared command overview text. Update `docs/public/commands/schema.md` to document `wavepeek schema --stream`.

Update `docs/dev/architecture.md` to mention that `src/output.rs` now owns both the single-document JSON renderer and the JSONL stream writer, while `change` and `property` can emit rows through a streaming sink instead of returning only fully buffered vectors.

At the end of this milestone, embedded docs and maintainer architecture notes describe the new behavior without duplicating generated clap flag tables.

### Milestone 11: Run quality gates and fix fallout

Run focused commands while iterating, then the repository gates before handoff. The exact commands from the repository root are:

    cargo fmt
    cargo test jsonl_cli --test jsonl_cli
    cargo test schema_cli --test schema_cli
    cargo test change_cli --test change_cli
    cargo test property_cli --test property_cli
    python3 -B -m unittest discover -s tools/docs -p "test_*.py"
    just check-schema
    just check

If the environment is outside the devcontainer and `just` recipes fail because `WAVEPEEK_IN_CONTAINER=1` is not set, record that exact failure and run the narrow Cargo tests that can run locally. In the intended container environment, `just check` is the pre-handoff gate.

At the end of this milestone, tests pass, schema artifacts match runtime output, documentation builds through the standard gate, and the branch is ready for code review.

## Concrete Steps

Start from the repository root:

    cd /workspaces/feat-jsonl
    git status --short

Expect no unrelated changes except this ExecPlan if implementation has not started. If unrelated user changes are present, do not overwrite them.

Implement Milestone 1 first. After editing CLI arg structs, run:

    cargo test cli_contract --test cli_contract

The expected result is that the test binary passes, or any failures point to help snapshots/assertions that must be updated for the new waveform-only `--jsonl` flag.

Implement Milestones 2 and 3 next. After adding the stream schema, docs publication helper changes, and writer unit tests, run:

    cargo test schema_cli --test schema_cli
    cargo test output --lib
    python3 -B -m unittest discover -s tools/docs -p "test_*.py"

The first command should pass existing schema tests plus the new stream schema tests. The second command name may need adjustment if output tests are ordinary library unit tests; `cargo test --lib jsonl` is also acceptable. The Python unittest command should prove that both envelope and stream schema artifacts are copied into root Pages artifacts and deploy-checked.

Implement Milestones 4 through 7 command by command. After each command’s JSONL path exists, run the matching integration test while developing:

    cargo test jsonl_change --test jsonl_cli
    cargo test jsonl_property --test jsonl_cli
    cargo test jsonl_info --test jsonl_cli
    cargo test jsonl_scope --test jsonl_cli
    cargo test jsonl_signal --test jsonl_cli
    cargo test jsonl_value --test jsonl_cli

If test names differ, use the nearest exact test filter. Keep each command independently demonstrable before moving on.

After docs updates, stream schema validation, and publication helper updates, run:

    just check-schema
    just test-aux
    just check

In a successful handoff, `just check` exits zero. If it fails for an environmental reason, capture the command, exit status, and relevant stderr in the final handoff notes.

## Validation and Acceptance

A successful implementation satisfies these observable behaviors:

Running `change --json` still prints one JSON envelope with `$schema`, `command`, `data`, and `diagnostics`, and existing tests continue to pass.

Running `change --jsonl` prints multiple newline-delimited records. For a truncated query, a representative stdout shape is:

    {"type":"begin","seq":0,"command":"change","$schema":"https://kleverhq.github.io/wavepeek/wavepeek-stream-v1.json"}
    {"type":"item","seq":1,"command":"change","item":{"time":"...","signals":[...]}}
    {"type":"diagnostic","seq":2,"command":"change","diagnostic":{"kind":"warning","code":"WPK-W0002","message":"..."}}
    {"type":"end","seq":3,"command":"change","summary":{"status":"ok","items":1,"diagnostics":1,"truncated":true}}

Every stdout line parses as a complete JSON value and validates against `schema/wavepeek-stream-v1.json`. The stream is complete only if the process exits successfully and the last line is an `end` record with `summary.status` equal to `ok`.

Running a conflict such as:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top --signals data --json --jsonl

fails before command execution, prints no stdout, and reports a clap conflict on stderr.

Running:

    cargo run --quiet -- schema --stream

prints the exact bytes from `schema/wavepeek-stream-v1.json`, and running:

    cargo run --quiet -- schema

still prints the exact bytes from `schema/wavepeek_v1.json`.

The final implementation must pass at least:

    cargo fmt
    cargo test jsonl_cli --test jsonl_cli
    cargo test schema_cli --test schema_cli
    python3 -B -m unittest discover -s tools/docs -p "test_*.py"
    just check-schema
    just check

## Idempotence and Recovery

All source edits are additive or refactoring edits and can be retried safely with git. Generated or scratch outputs must go under repository-root `tmp/` and should not delete existing files there. Do not read `.fst` or `.fsdb` files as text; use fixtures through the CLI or waveform helpers.

If `schema/wavepeek-stream-v1.json` becomes invalid while editing, recover by reverting that file with git and recreating it from the shapes in `schema/wavepeek_v1.json`. If `wavepeek schema --stream` does not match the file, check `src/schema_contract.rs` first; the runtime should embed the checked-in file rather than hand-assembling JSON strings.

If a JSONL stream test fails because an `end` record is missing, inspect whether the command returned an error after writing `begin`. Fatal errors after `begin` are allowed to omit `end`, but successful commands must always call `JsonlWriter::end` exactly once. If a stream test fails because diagnostics precede items, either fix the command to buffer non-fatal diagnostics until after item emission or update this plan and the docs with a deliberate new ordering decision.

If the `change` refactor becomes too large, keep the old buffered helper and introduce a parallel streaming helper first. Once both paths pass tests, remove duplication in a later cleanup. Do not leave `change --jsonl` implemented only by printing a completed `Vec<ChangeSnapshot>`; that would miss the purpose of the feature.

## Artifacts and Notes

The primary new tracked artifacts are expected to be:

- `schema/wavepeek-stream-v1.json` for the per-record JSONL schema.
- `tests/jsonl_cli.rs` for end-to-end stream behavior.
- Public docs updates under `docs/public/reference/` and `docs/public/commands/`.
- Docs publication helper updates under `tools/docs/` so the stream schema is copied to and checked from the Pages root.
- Stream-schema validation support in tests, either through a Rust dev-dependency or a Python helper plus its pinned container requirement.
- Internal architecture updates in `docs/dev/architecture.md`.

The expected branch-local planning artifact is this file, `docs/tracker/wip/jsonl-stream-output-execplan.md`. It should be removed before merging to the default branch unless a maintainer explicitly wants to keep the handoff record.

## Interfaces and Dependencies

Use existing runtime dependencies: `serde`, `serde_json`, `clap`, and the standard library. Do not add a new runtime dependency for JSONL. A dev-only JSON Schema validator is allowed and required if no suitable validator is already present, because tests must validate representative JSONL records against `schema/wavepeek-stream-v1.json`. If the validator is Python-based, pin it in the same container requirements path used by the test helper; if it is Rust-based, add it under `[dev-dependencies]` only.

The implementation should expose or define these internal interfaces:

    enum OutputMode {
        Human,
        Json,
        Jsonl,
    }

    struct JsonlWriter<W: std::io::Write> { ... }

    impl<W: std::io::Write> JsonlWriter<W> {
        fn begin(&mut self, command: CommandName) -> Result<(), WavepeekError>;
        fn item<T: serde::Serialize>(&mut self, item: &T) -> Result<(), WavepeekError>;
        fn diagnostic(&mut self, diagnostic: &Diagnostic) -> Result<(), WavepeekError>;
        fn end(&mut self, truncated: bool) -> Result<(), WavepeekError>;
    }

    struct ChangeRunStats {
        emitted: usize,
        truncated: bool,
    }

The exact names can change if repository style suggests better names, but the behavior must remain: one output mode value instead of scattered booleans, one reusable writer for line-delimited records, and streaming command helpers that can emit rows without accumulating the complete result for JSONL.

## Revision Notes

- 2026-06-19 / Grin: Initial ExecPlan drafted from issue discussion and repository inspection. It records the waveform-only `--jsonl` scope, dedicated per-record stream schema, stateful stream validation approach, and the requirement to refactor `change` and `property` beyond cosmetic JSONL printing.
- 2026-06-19 / Grin: Revised after architecture and contract review. The plan now requires safe staged flag routing, broken-pipe handling, flush-aware writer tests, docs publication of the stream schema, command-to-item schema constraints, runtime JSONL schema validation, and anti-buffered-adapter tests for `change` and `property`.
- 2026-06-19 / Grin: Revised after control review to name all docs publication guard functions that must learn `wavepeek-stream-v*.json` and to require a process-level broken-pipe integration test. A targeted re-check reported no substantive findings.
