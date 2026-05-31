# Implement FSDB change and property portable path

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the repository `exec-plan` skill. It is intentionally self-contained: a contributor with only the current working tree and this file should be able to implement, validate, and review the change without prior conversation.

All durable repository entities created by this plan must use descriptive FSDB command names, not roadmap-label prefixes, suffixes, directory names, module names, function names, test names, fixture names, documentation anchors, or commit subjects. Use names such as `change_property_core.vcd`, `fsdb_change_edge_iff_matches_vcd_for_generated_fixture`, `wp_fsdb_collect_signal_change_times`, `FsdbCandidateTimes`, and `fsdb_change_property_portable_path`. If a new name would contain the roadmap label instead of the behavior it implements, rename it before committing. Existing historical files are not renamed by this plan.

## Purpose / Big Picture

After this change, a `wavepeek` binary built with the optional Cargo feature `fsdb` and a local licensed Synopsys Verdi FSDB Reader SDK can run the existing `change` and `property` commands directly on FSDB waveform files. Users keep the same command surface they already use for VCD and FST, for example `wavepeek change --waves dump.fsdb --scope top --signals valid,ready --on 'posedge clk'` and `wavepeek property --waves dump.fsdb --scope top --on 'posedge clk' --eval 'valid && ready' --capture switch`. The observable result is that those commands produce the same JSON and human-output contracts for generated FSDB fixtures as they do for the source VCD fixtures.

The implementation uses the already-existing portable command path in Rust. VCD and FST keep their Wellen-specific indexed and streaming optimizations. FSDB supplies only the backend operations that portable `change` and `property` need: candidate timestamps from value-change traversal, sampled expression values at or before a raw timestamp, exact raw-event occurrence checks, and a strict previous raw timestamp for event semantics. This is not the glamorous fast path. It is the reliable path, which is the one that should exist before anyone starts polishing the turbine blades.

Default builds without `--features fsdb` remain Verdi-free and continue to reject real FSDB input with the existing feature-required file error. FSDB-enabled builds continue to support `info`, `scope`, `signal`, and digital bit-vector `value`; this plan adds time-range `change` and event-selected `property` for the first useful FSDB expression subset.

## Non-Goals

This plan does not add public CLI flags, public JSON fields, schema fields, command names, or FSDB-specific output modes. FSDB `change` and `property` must fit the existing `schema/wavepeek.json` contracts.

This plan does not replace or slow down the current Wellen engines for VCD and FST. The baseline, fused, edge-fast, indexed, and FST streaming paths stay available for Wellen. FSDB may fall back to the existing portable baseline path when indexed methods return `None`.

This plan does not implement an FSDB-specific high-performance rolling engine, global FSDB time table, view-window optimization, or persistent loaded-signal session across commands. Those belong to later hardening and performance work after correctness exists.

This plan does not claim full datatype parity for every FSDB value class. The required expression support is digital/integral values that can be represented as `0`, `1`, `x`, and `z` bit strings, integer-like/time metadata already represented by the hierarchy index, and raw event occurrence checks. Enum labels should be carried through if local datatype callbacks already provide enough project-owned information, but label support is not required for acceptance unless a generated fixture proves it. Real and string expression operands are deferred unless the implementation adds generated fixture coverage and clear decoding; without that evidence they must fail with a clear signal or expression diagnostic rather than being silently misdecoded.

This plan does not commit Verdi headers, libraries, documentation excerpts, generated bindings derived from proprietary headers, `.fsdb` fixtures, native converter logs, or full golden outputs from Verdi-bundled FSDB files. Checked-in tests may add small VCD fixtures under `tests/fixtures/hand/`; generated FSDB files remain ignored local artifacts under temporary directories or `tests/fixtures/fsdb/`.

This plan does not rename existing historical fixtures or plans merely because their names contain older roadmap labels. The naming rule applies to new durable entities created while implementing this plan.

## Progress

- [x] (2026-05-27 00:00Z) Read `docs/fsdb/arch.md`, especially the FSDB `change` / `property` portable path milestone and backend-interface sections.
- [x] (2026-05-27 00:00Z) Read FSDB command research notes: `docs/fsdb/cmd_change.md` and `docs/fsdb/cmd_property.md`.
- [x] (2026-05-27 00:00Z) Read current implementation context in `src/waveform/mod.rs`, `src/waveform/fsdb_backend.rs`, `src/waveform/fsdb_native.rs`, `src/waveform/fsdb_hierarchy.rs`, `native/fsdb/wavepeek_fsdb_shim.h`, `native/fsdb/wavepeek_fsdb_shim.cpp`, `src/engine/change.rs`, `src/engine/property.rs`, `src/waveform/expr_host.rs`, and `tests/fsdb_cli.rs`.
- [x] (2026-05-27 00:00Z) Drafted this active ExecPlan at `docs/exec-plans/active/2026-05-27-fsdb-change-property-portable/PLAN.md` with descriptive names and no roadmap-label names for new entities.
- [x] (2026-05-27 00:00Z) Ran focused read-only review lanes for plan completeness, native/FFI safety, Rust backend integration, and tests/public-contract coverage; incorporated substantive findings into this plan.
- [x] (2026-05-27 00:00Z) Ran an independent control review over the revised plan, fixed its nonzero-start fixture ambiguity, and rechecked the fix with no substantive findings.
- [x] (2026-05-27 00:35Z) Confirmed baseline default and FSDB validation gates before implementation: `make check`, `make ci`, `make check-fsdb-env`, and `make test-fsdb` passed on the current Verdi-equipped container.
- [x] (2026-05-27 00:58Z) Added generated-fixture VCD coverage for `change` and `property` behavior: `change_property_core.vcd`, `change_property_offset_start.vcd`, `change_property_real_output.vcd`, and `change_property_events.vcd`.
- [x] (2026-05-27 01:45Z) Added native FSDB candidate-time traversal and exact raw-event occurrence support in `native/fsdb/wavepeek_fsdb_shim.{h,cpp}`.
- [x] (2026-05-27 01:45Z) Added safe Rust FFI wrappers for candidate times and raw-event occurrence in `src/waveform/fsdb_native.rs`.
- [x] (2026-05-27 01:45Z) Implemented FSDB backend expression sampling, candidate collection, raw-event occurrence, and strict previous timestamp behavior in `src/waveform/fsdb_backend.rs`.
- [x] (2026-05-27 01:45Z) Enabled FSDB `change` and `property` command flow by making the previous unsupported-command guard return `None`.
- [x] (2026-05-27 01:52Z) Added FSDB CLI parity tests against generated fixtures and a bounded bundled-example smoke in `tests/fsdb_cli.rs`.
- [x] (2026-05-27 02:12Z) Updated public docs, changelog, and FSDB architecture/research notes after implementation.
- [x] (2026-05-27 02:55Z) Ran four focused read-only review lanes: native/Rust code, tests, docs/contracts, and architecture/performance. Native/Rust returned no findings. Test/docs/performance findings were fixed or explicitly deferred as noted below.
- [x] (2026-05-27 03:35Z) Ran default and FSDB validation gates, requested focused implementation review, fixed findings, ran two targeted control passes, committed all fixes, and left this plan in `docs/exec-plans/active/` for user inspection.
- [x] (2026-05-27 04:05Z) At the user's follow-up request, moved this plan to `docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md` and updated `docs/fsdb/arch.md` to point at the completed implementation record.

## Surprises & Discoveries

- Observation: FSDB-enabled `value` is already implemented for digital bit-vector signals, but `change` and `property` are still blocked before normal command execution.
  Evidence: `src/waveform/mod.rs` returns `unsupported_change_collection()` for `unsupported_fsdb_command_error("change")` and `unsupported_property_evaluation()` for `unsupported_fsdb_command_error("property")`, while `value` is no longer listed there.

- Observation: the existing Rust command engines already have a portable baseline path that can work without an indexed backend if the backend implements candidate collection, sampling, event occurrence, and previous timestamp behavior.
  Evidence: `src/engine/change.rs::run_baseline` calls `collect_expr_candidate_times_with_mode`, `sample_resolved_optional`, `event_expr_matches`, and `previous_sample_time`; `src/engine/property.rs::run` follows the same candidate schedule and expression host flow.

- Observation: the FSDB backend currently implements point-in-time bit-vector sampling through `wp_fsdb_sample_signal_values`, but expression sampling and exact raw-event checks still return unsupported errors.
  Evidence: `src/waveform/fsdb_backend.rs::sample_resolved_optional` calls `FsdbReader::sample_signal_values`, while `sample_expr_value`, `expr_event_occurred`, `collect_change_times_with_mode`, and `collect_expr_candidate_times_with_mode` return unsupported helper errors.

- Observation: event edge matching in the expression runtime samples the previous raw tick with `timestamp - 1`, while named-event and wildcard change detection use the command-provided `previous_timestamp`.
  Evidence: `src/expr/eval.rs::edge_event_matches` calls `frame.timestamp.checked_sub(1)`, while `signal_changed` uses `frame.previous_timestamp` for non-event value changes. FSDB should therefore return `None` at or before the cached dump start and `raw_time.checked_sub(1)` after the dump start, giving strictly-before behavior without inventing a global FSDB time table.

- Observation: the FSDB hierarchy index already maps signal kinds into `ExprType` and records value sampleability as `FsdbValueEncoding`, but it does not store enum literal mappings, real decoding metadata, or string decoding metadata.
  Evidence: `src/waveform/fsdb_hierarchy.rs` has `RawDatatypeKind`, `FsdbValueEncoding`, and `expr_type_from_signal`, but `RawDatatypeRecord` currently stores only `idcode` and `kind`.

- Observation: FSDB integration tests already generate temporary FSDB fixtures from checked-in VCD files through `vcd2fsdb` and compare user-visible command payloads.
  Evidence: `tests/fsdb_cli.rs` defines `GeneratedFsdbFixtures`, `convert_vcd_fixture`, and existing FSDB `value` tests that compare generated FSDB results to VCD results for the same CLI arguments.

- Observation: native Reader calls are process-global enough to deserve boring serialization.
  Evidence: `native/fsdb/wavepeek_fsdb_shim.cpp` already uses a recursive mutex and output suppressor around Reader operations, and `signal_list_guard` resets/loads/unloads the Reader signal list inside the native batch value sampler.

- Observation: review found three places where the first draft was too optimistic: raw-event public support needed deterministic tests, nonzero dump starts needed explicit previous-timestamp handling, and candidate timestamp deduplication must not decide same-timestamp final values.
  Evidence: the revised plan now requires raw-event documentation to be gated by a passing FSDB raw-event test, adds `change_property_offset_start.vcd`, requires metadata-cached dump-start handling in `previous_sample_time`, and states that sampling at `t` must return the Reader's final same-time value.

- Observation: local `vcd2fsdb` preserves the new raw event fixture as a public event signal, and exact raw-event parity can be tested deterministically after candidate traversal handles the case where `--from` is before a signal's first event.
  Evidence: converting `tests/fixtures/hand/change_property_events.vcd` into an ignored temporary FSDB and running `wavepeek signal --scope top --json` reported `top.tick` with kind `event` and `top.armed` with kind `wire`; after fixing candidate traversal to jump to the signal's minimum value-change tag when `ffrGotoXTag(from)` fails before the first event, FSDB `property --on tick --eval armed --capture match --json` matched VCD at 10ns and 25ns.

- Observation: post-implementation review found that collecting every per-signal timestamp and deduplicating only after traversal can waste memory when many signals toggle at the same raw time.
  Evidence: the C++ shim now keeps a `seen_times` set during traversal and only appends a raw timestamp the first time it appears, while retaining the final sort before crossing the FFI boundary.

- Observation: post-implementation review also found a real performance risk from repeated native signal-list loading and traversal-handle creation during expression sampling on large FSDB windows.
  Evidence: this slice already caches sampled expression values and raw-event occurrence results in `FsdbBackend`, but it still crosses the native boundary per uncached signal/time. A persistent native sampler/session would require a larger ownership design and is deferred to the M6 hardening/performance milestone rather than smuggled into this correctness slice on a Friday-afternoon architecture receipt.

- Observation: the first control pass found an unsupported-value edge case that tests did not cover: wildcard expression candidate collection can be the only place where an unsupported FSDB real/string operand is noticed when the selected window has no matching value-change time.
  Evidence: `FsdbBackend::collect_expr_candidate_times_with_mode` now validates expression candidate sources before native traversal, and `fsdb_change_property_reject_unsupported_real_operands_clearly` includes a `property --on '*' --eval 'temp > 1.0' --from 6ns --to 9ns` no-candidate-window failure case.

- Observation: the targeted control recheck correctly pointed out that explicit event triggers could still hide unsupported property eval operands if no candidate occurred.
  Evidence: `property::run` now resolves and validates all logical eval operands through `Waveform::validate_expr_values_supported` before candidate collection, and `fsdb_change_property_reject_unsupported_real_operands_clearly` includes the explicit `--on 'posedge clk' --from 6ns --to 9ns --eval 'temp > 1.0'` failure case.

## Decision Log

- Decision: implement FSDB `change` and `property` by filling the backend-neutral portable operations rather than adding FSDB branches inside command engines.
  Rationale: `change` and `property` already parse arguments, bind expressions, format output, enforce capture modes, and handle warnings in backend-neutral Rust. Duplicating command logic for FSDB would create two contracts and two future bug farms. The FSDB backend should provide waveform facts; the engines should decide what those facts mean.
  Date/Author: 2026-05-27 / Grin

- Decision: make `FsdbBackend::previous_sample_time(raw_time)` return `None` at or before the cached dump start and `raw_time.checked_sub(1)` after the dump start.
  Rationale: FSDB does not expose or need a global wavepeek time table for the portable path. Existing event semantics require a value strictly before the candidate timestamp. Sampling at raw tick `t - 1` gives that meaning for integer FSDB tags after the file has begun, while the cached start check prevents a nonzero first dump timestamp from looking like a change from missing to present.
  Date/Author: 2026-05-27 / Grin

- Decision: add native candidate-time collection as a small C ABI operation that returns owned sorted raw timestamps for selected idcodes.
  Rationale: candidate scanning depends on proprietary FSDB traversal handles and time tags, so it belongs in the C++ shim that already includes Reader headers. Rust should receive only project-owned `u64` raw ticks and should not depend on proprietary struct layouts.
  Date/Author: 2026-05-27 / Grin

- Decision: add a native exact raw-event occurrence operation instead of treating event signals as sampled bit-vector values.
  Rationale: raw SystemVerilog/VCD event variables are occurrence streams, not printable values. The expression runtime already separates `sample_value` from `event_occurred`; FSDB should honor that split.
  Date/Author: 2026-05-27 / Grin

- Decision: keep real and string FSDB expression values out of required acceptance until generated fixtures prove decoding.
  Rationale: the existing expression engine can evaluate real and string values, but FSDB value buffers for those kinds need verified byte layout and fixture coverage. A clear unsupported diagnostic is safer than a plausible lie wearing a lab coat.
  Date/Author: 2026-05-27 / Grin

- Decision: treat hidden tune candidate modes as backend strategy hints for FSDB and collect candidates through the same portable FSDB collector for `Auto`, `Random`, and `Stream`.
  Rationale: `--tune-candidates` is hidden internal plumbing. Forcing `stream` should not promise FST streaming on FSDB, and failing a correct FSDB query because an internal knob says `stream` would make the knob user-visible by accident.
  Date/Author: 2026-05-27 / Grin

- Decision: keep all new durable names descriptive and command-oriented rather than roadmap-oriented.
  Rationale: names such as `change_property_core.vcd` and `wp_fsdb_collect_signal_change_times` explain their purpose after the roadmap label is stale. Milestone labels in APIs and fixtures are archaeological litter, and the dig site is crowded enough.
  Date/Author: 2026-05-27 / Grin

- Decision: make per-signal value-change traversal the required candidate collector and treat merged time-based traversal as an optional later optimization.
  Rationale: per-signal traversal is already required by the existing FSDB value sampler, while a merged traversal symbol must be verified against the local SDK before use. This avoids a plan that accidentally fails compilation on a supported SDK variant because an optimization was treated as a foundation.
  Date/Author: 2026-05-27 / Grin

- Decision: gate public raw-event support wording on deterministic FSDB test coverage.
  Rationale: the backend should expose exact event occurrence for FSDB files that contain raw events, but public docs must not promise raw-event command parity unless generated or otherwise deterministic fixtures prove it.
  Date/Author: 2026-05-27 / Grin

- Decision: cache raw FSDB metadata for previous-timestamp semantics.
  Rationale: returning `raw_time - 1` is correct only after the dump start. At a nonzero first timestamp it can turn an initial value into a false named-event change. The backend needs the raw start tick to return `None` at and before dump start.
  Date/Author: 2026-05-27 / Grin

- Decision: keep this execution plan in `docs/exec-plans/active/2026-05-27-fsdb-change-property-portable/PLAN.md` for the first handoff, then move it to `docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md` after the user asks for completion archival.
  Rationale: the user first asked to inspect the result before the plan was moved, then explicitly requested the move to completed. The plan records both steps so the historical path through active review and final archival is not a little mystery box with hinges.
  Date/Author: 2026-05-27 / Grin

- Decision: defer a persistent native FSDB sampler/session to the M6 hardening/performance milestone while keeping correctness caches in the Rust backend for this slice.
  Rationale: reusing native loaded signal lists and traversal handles across arbitrary expression evaluation calls changes native reader lifetime management and memory-retention policy. The current implementation is correct, tested, and bounded by command execution, while the obvious large-file optimization deserves profiling and a dedicated design instead of an improvised cache with interesting failure modes. The shim did take the low-risk memory fix of deduplicating candidate times during traversal.
  Date/Author: 2026-05-27 / Grin

## Outcomes & Retrospective

Plan authoring, initial focused review, independent control review, baseline validation, VCD fixture creation, implementation, documentation, final validation, review/fix/control cycles, and user-requested archival are complete. The implementation enables FSDB `change` and `property` through the existing backend-neutral engines for digital bit-vector/integral signals, with native candidate traversal, Rust FFI ownership, expression sampling, exact raw-event occurrence, nonzero-start previous timestamp handling, early unsupported-property-operand validation, and generated FSDB parity coverage. Review findings improved test breadth, stale FSDB research wording, candidate timestamp memory behavior, and unsupported real/string diagnostics. The final targeted control recheck reported no substantive findings. The plan now lives in `docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md`, and `docs/fsdb/arch.md` points to that completed record.

## Context and Orientation

`wavepeek` is a Rust command-line tool for inspecting RTL waveform dumps. VCD and FST files are read through the Rust crate `wellen`. FSDB is Synopsys's proprietary Fast Signal Database format and must be read through a local licensed Verdi FSDB Reader SDK. This repository may store project-owned Rust and C++ source that calls local FSDB Reader headers at build time, but it must not store Synopsys headers, libraries, documentation excerpts, generated bindings, copied vendor examples, or `.fsdb` files.

The public commands relevant to this plan are `change` and `property`. The `change` command scans an inclusive time window and prints a snapshot only when requested output signals changed and the event expression from `--on` matches. The `property` command evaluates a logical expression from `--eval` at timestamps selected by `--on`, then emits rows according to `--capture match`, `switch`, `assert`, or `deassert`. An event expression is the SystemVerilog-like trigger syntax accepted by this project: wildcard `*`, named signals or raw events, `posedge`, `negedge`, `edge`, unions with `or` or comma, and optional `iff` gates. A logical expression is the value expression language documented in `docs/public/reference/expression-language.md`, including integral operators and related expression forms.

A candidate timestamp is a raw dump tick where something relevant changed or occurred. For `change --on '*'`, the candidates include requested output signal changes. For `property --on '*'`, candidates are inferred from signals referenced by `--eval`. For explicit event expressions, candidates include the event operands themselves. The current engine does not add `iff` guard operands as independent candidate sources; it samples and evaluates an `iff` guard only when that guard's event term has already produced a candidate. Preserve that behavior unless a separate contract-changing plan says otherwise. A sampled value at or before timestamp `t` means the last value change for that signal whose raw time is less than or equal to `t`. An exact raw-event occurrence means a raw event variable has a value-change record exactly at `t`; it is not sampled as an ordinary value.

The command engines live in `src/engine/change.rs` and `src/engine/property.rs`. They open a shared `Waveform`, reject unsupported FSDB commands today, read metadata, bind expressions through `src/engine/expr_runtime.rs` and `src/waveform/expr_host.rs`, collect candidate timestamps through the waveform facade, evaluate event and logical expressions through the expression engine in `src/expr/`, and format stable human or JSON output. The implementation should preserve this command flow and remove only the FSDB unsupported guard once the backend methods are implemented.

The waveform facade lives in `src/waveform/mod.rs`. `Waveform` owns a backend enum. `Backend::Wellen` is the VCD/FST implementation in `src/waveform/wellen_backend.rs`. `Backend::Fsdb` is compiled only with `#[cfg(feature = "fsdb")]` and implemented in `src/waveform/fsdb_backend.rs`. Backend-neutral types live in `src/waveform/types.rs`: `SignalId` is an opaque backend id, `ResolvedSignal` is for printable signal sampling, `ExprResolvedSignal` is for expression evaluation, `SampledSignalState` carries an optional bit string for command snapshots, and `ChangeCandidateCollectionMode` represents the hidden candidate collection tuning mode.

The FSDB native boundary has two layers. `native/fsdb/wavepeek_fsdb_shim.h` and `native/fsdb/wavepeek_fsdb_shim.cpp` are wavepeek-owned C/C++ files compiled only when the `fsdb` feature is enabled. They include local SDK headers from `$VERDI_HOME/share/FsdbReader`, map proprietary Reader types into project-owned C ABI records, catch C++ exceptions, suppress native Reader stdout/stderr, and serialize Reader calls with a recursive mutex. `src/waveform/fsdb_native.rs` is the Rust wrapper around that C ABI. It owns `FsdbReader`, converts native strings and arrays into Rust values, frees native allocations, and maps native failures into `WavepeekError::File`.

The current FSDB hierarchy builder in `src/waveform/fsdb_hierarchy.rs` receives raw project-owned records from the native tree callback and builds canonical dot-separated scope and signal indexes. It normalizes packed range suffixes, deduplicates repeated tree sections, maps public scope and signal kinds, stores `FsdbValueEncoding`, and creates `ExprType` values for expression binding. After the value-command work, digital bit-vector value sampling already works through `FsdbBackend::sample_resolved_optional` and `FsdbReader::sample_signal_values`.

Development is container-first. Default quality gates must not require Verdi. Use `WAVEPEEK_IN_CONTAINER=1 make check` and `WAVEPEEK_IN_CONTAINER=1 make ci` for default validation. FSDB-specific validation runs only on Verdi-equipped Linux machines through `WAVEPEEK_IN_CONTAINER=1 make lint-fsdb`, `WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures`, and `WAVEPEEK_IN_CONTAINER=1 make test-fsdb`. FSDB Cargo invocations in the Makefile use `CARGO_TARGET_DIR=target/fsdb` to avoid feature-enabled binaries racing default builds.

The `.fst` files in this repository are binary and must never be read with the text `read` tool. Treat `.fsdb` files the same way: do not inspect them as text. Tests may access FSDB files only through the FSDB Reader API, `vcd2fsdb`, or binary-safe filesystem operations such as existence checks.

## Open Questions

No product decision blocks this plan. The exact local FSDB Reader signatures for per-signal traversal, next-value-change traversal, and event occurrence must still be verified against the locally installed SDK headers and examples during implementation. Do that verification locally, but do not copy proprietary declarations or documentation into the repository or into this plan.

Use per-signal value-change traversal as the required candidate collection implementation because the current value sampler already depends on per-signal traverse handles. A merged time-based traversal handle may be added later as an optimization only after the implementation verifies the symbol exists in the local SDK and guards any use of it without breaking compilation on supported SDK variants. Arbitrary traversal failures must remain errors; do not silently retry them as if the API were unsupported.

Raw event occurrence is a required part of this plan's public acceptance only when a deterministic test proves it. The preferred proof is a generated fixture from `change_property_events.vcd` whose `signal` output reports `tick` as kind `event` and whose VCD/FSDB `property --on tick` payloads match. If local Verdi conversion cannot preserve raw events, do not document raw-event support as shipped; instead document the limitation, keep the internal exact-event operation for future FSDB files that expose event variables, and record the missing fixture evidence in `Surprises & Discoveries`.

If real or string value decoding is implemented while working on this plan, it must be backed by small generated fixtures and explicit tests. Otherwise real and string expression operands should fail clearly and remain documented as out of the required acceptance scope for this implementation slice.

## Plan of Work

### Confirm baseline and add generated command fixtures

Start from `/workspaces/wavepeek-fsdb` on branch `feat/fsdb` with a clean tree. Record the current commit and run default validation before editing. If Verdi is available, also run the FSDB environment and test gates so failures introduced by this work are distinguishable from local setup breakage.

Run:

    cd /workspaces/wavepeek-fsdb
    git status --short --branch
    git rev-parse --short HEAD
    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci
    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env

If Verdi is available, also run:

    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Add small checked-in VCD fixtures under `tests/fixtures/hand/`. The primary fixture should be named `change_property_core.vcd`. It should contain a `top` scope with `clk`, `valid`, `ready`, `data[7:0]`, `state[1:0]`, and `pulse`. Use a simple `1ns` timescale and this intended value timeline:

    time 0ns:  clk=0 valid=0 ready=0 data=8'h00 state=2'h0 pulse=0
    time 5ns:  clk=1 valid=1 ready=0 data=8'h0f state=2'h1 pulse=0
    time 7ns:  clk=1 valid=1 ready=0 data=8'h1f state=2'h1 pulse=0
    time 10ns: clk=0 valid=1 ready=1 data=8'h1f state=2'h1 pulse=1
    time 15ns: clk=1 valid=1 ready=1 data=8'h2a state=2'h2 pulse=0
    time 20ns: clk=0 valid=0 ready=1 data=8'h2a state=2'h2 pulse=0
    time 25ns: clk=1 valid=0 ready=0 data=8'h2a state=2'h3 pulse=0
    time 30ns: clk=0 valid=1 ready=1 data=8'h3c state=2'h3 pulse=0
    time 35ns: clk=1 valid=1 ready=1 data=8'h3c state=2'h0 pulse=0

Add `change_property_offset_start.vcd` with a nonzero dump start and an exact timeline that makes a bad first-timestamp event visible. Use at least `valid` and `ready` signals with `valid=1` and `ready=1` at `100ns`, `ready=0` at `101ns`, `valid=0` at `105ns`, and `valid=1` with `ready=0` at `110ns`. It must prove that a value-change record at the dump start does not count as a named-event change from "missing" to "present". The probe `property --from 100ns --to 110ns --on valid --eval ready --capture match` should emit no rows; an incorrect FSDB `previous_sample_time` that samples before the dump start as missing would produce a false-positive row at `100ns` because `ready` is true there.

Add `change_property_real_output.vcd` with a bit-vector trigger such as `clk` and a real-valued output signal such as `temp`. This fixture exists so `change --signals temp --on clk` validates the requested output signal path and produces the existing unsupported non-bit-vector signal error, instead of failing earlier while trying to use a real signal as a wildcard candidate source.

Add `change_property_events.vcd` as a required fixture if local `vcd2fsdb` preserves VCD raw event variables. It should contain a raw event signal `tick` and an ordinary bit signal `armed`; event occurrences at `5ns`, `10ns`, `15ns`, and `25ns`; and `armed` values that make `property --on tick --eval armed --capture match` match exactly at `10ns` and `25ns`. If conversion does not preserve `tick` as a public `event` signal in generated FSDB, record that discovery in `Surprises & Discoveries`, do not commit a broken event fixture, and keep public docs from claiming raw-event parity.

Before implementing FSDB behavior, run VCD-only probes through the default binary to lock down expected command semantics:

    cargo run --quiet -- change --waves tests/fixtures/hand/change_property_core.vcd --scope top --signals data --from 0ns --to 20ns --on '*' --max 10 --json
    cargo run --quiet -- change --waves tests/fixtures/hand/change_property_core.vcd --scope top --signals valid,ready,data --from 0ns --to 35ns --on 'posedge clk iff valid' --max 10 --json
    cargo run --quiet -- property --waves tests/fixtures/hand/change_property_core.vcd --scope top --on 'posedge clk' --eval 'valid && ready' --capture switch --json
    cargo run --quiet -- property --waves tests/fixtures/hand/change_property_core.vcd --scope top --eval "data == 8'h2a" --capture match --json
    cargo run --quiet -- property --waves tests/fixtures/hand/change_property_core.vcd --scope top --from 10ns --to 25ns --on 'posedge clk' --eval 'valid && ready' --capture switch --json
    cargo run --quiet -- property --waves tests/fixtures/hand/change_property_offset_start.vcd --from 100ns --to 110ns --on valid --eval ready --capture match --json

Expected observations are: wildcard `change` on `data` emits rows at `5ns`, `7ns`, and `15ns`; edge-gated `change` emits rows at `5ns` and `15ns`; full-window `property --capture switch` over `valid && ready` on `posedge clk` emits `@15ns assert`, `@25ns deassert`, and `@35ns assert`; bounded `property --from 10ns --to 25ns` emits the expected transition at `25ns deassert`; wildcard-inferred `property --eval "data == 8'h2a" --capture match` emits a match at the first candidate where `data` changes to `8'h2a`; and the nonzero-start fixture emits no rows, especially not a false-positive row at its first dump timestamp. Capture concise JSON snippets in `Artifacts and Notes` if the actual VCD contract differs, and update the tests to match the existing VCD behavior rather than wishful memory.

Acceptance for this milestone is that the fixtures are tiny, readable, convertible by `vcd2fsdb`, and VCD-only command probes prove the exact behavior that FSDB must match.

### Add native candidate traversal and exact event occurrence

Extend `native/fsdb/wavepeek_fsdb_shim.h` with project-owned C ABI additions for candidate time collection and raw event occurrence. Use names that describe the operation:

    typedef struct wp_fsdb_time_list {
        uint64_t *times;
        size_t count;
    } wp_fsdb_time_list;

    wp_fsdb_status wp_fsdb_collect_signal_change_times(
        wp_fsdb_reader *reader,
        const uint64_t *idcodes,
        size_t count,
        uint64_t from_raw,
        uint64_t to_raw,
        wp_fsdb_time_list *out,
        char **error_message
    );

    void wp_fsdb_free_time_list(wp_fsdb_time_list *list);

    wp_fsdb_status wp_fsdb_signal_event_occurred(
        wp_fsdb_reader *reader,
        uint64_t idcode,
        uint64_t query_time_raw,
        int *occurred,
        char **error_message
    );

`wp_fsdb_collect_signal_change_times` should return success with an empty list when `count == 0` or when no selected idcode has a value-change record in the inclusive window. It should reject `from_raw > to_raw` as a native argument error only if a caller bypasses Rust validation. It should deduplicate idcodes before loading the native Reader signal list, but output one sorted unique timestamp list for the whole selected set.

Implement candidate traversal in `native/fsdb/wavepeek_fsdb_shim.cpp`. Reuse the existing `signal_list_guard`, Reader mutex, output suppressor, and integer tag conversion helpers. Initialize every output parameter on function entry: `wp_fsdb_time_list` must start as `{nullptr, 0}`, and `occurred` must start as `0`. Native free functions must be null-safe and idempotent enough that freeing an already-empty list is harmless.

Load only the selected idcodes. The required implementation creates one value-change traverse handle per unique idcode after signals are loaded, seeks to `from_raw`, skips aligned records before `from_raw`, walks forward with the Reader's next-value-change operation, filters through `to_raw`, inserts raw times into a sorted set such as `std::set<uint64_t>`, and frees every handle. If the Reader returns multiple records at the same raw timestamp, keep one candidate timestamp. Candidate deduplication only deduplicates scheduling; it must not decide the final value visible at that timestamp.

Same-timestamp value changes and glitches must be handled by the sampling path, not by candidate deduplication. When `sample_signal_values` or any new expression-sampling helper asks for value at `t`, it must return the final value at raw time `t` according to the Reader's sequence/glitch ordering. If a plain `ffrGotoXTag(t)` does not guarantee the final same-time value, the native sampler must use sequence-number-aware APIs such as the Reader's value-change sequence accessors, or continue within the same timestamp until the last sequence, then copy that value. Add or reuse a generated fixture with multiple assignments at one timestamp if local conversion preserves that case; otherwise record the Reader behavior observed from SDK examples and generated probes.

A merged time-based traversal handle may be added later as an optimization. If used, it must be compiled only after verifying the symbol exists for the supported SDK, and failures from that handle must not be swallowed as a generic fallback unless they are known nonfatal unsupported/null-handle cases. The public Rust wrapper must see the same sorted timestamp list regardless of native strategy.

Implement `wp_fsdb_signal_event_occurred` as an exact timestamp check. It should validate the file's tag representation before reading integer tag fields, load the single event idcode, create a traverse handle, seek to `query_time_raw`, read the actual aligned tag, set `*occurred = 1` only when `actual_raw == query_time_raw`, then free the handle and unload/reset the signal list. It should not decode the event value buffer. If the signal has no in-core value changes or the seek fails, return success with `*occurred = 0`.

Keep native safety properties intact: no C++ exception crosses FFI, every error returns status plus an owned error string, Reader stdout/stderr stays suppressed, Reader calls stay serialized by the native mutex, every successful signal-list load has unload/reset cleanup, every native allocation has a matching free function, and every new operation either validates integer `L`/`HL` time tags itself or is called only after a cached Rust metadata guard has proven integer tags. The safer implementation is to do both, because belts and suspenders are cheaper than debugging proprietary time encodings.

Acceptance for this milestone on a Verdi-equipped machine:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb

If focused native smoke tests are added under `#[cfg(feature = "fsdb")]`, run them by exact test name and record the output in `Artifacts and Notes`.

### Add safe Rust wrappers for native time and event operations

Update `src/waveform/fsdb_native.rs` to mirror the new C ABI and expose safe Rust methods on `FsdbReader`:

    pub(super) struct FsdbCandidateTimes {
        pub(super) times: Vec<u64>,
    }

    impl FsdbReader {
        pub(super) fn collect_signal_change_times(
            &self,
            idcodes: &[u64],
            from_raw: u64,
            to_raw: u64,
        ) -> Result<FsdbCandidateTimes, WavepeekError>;

        pub(super) fn signal_event_occurred(
            &self,
            idcode: u64,
            query_time_raw: u64,
        ) -> Result<bool, WavepeekError>;
    }

The exact wrapper type may be a plain `Vec<u64>` if that is clearer, but native ownership must be wrapped in an RAII guard so `wp_fsdb_free_time_list` runs on all paths. Initialize the raw `wp_fsdb_time_list` to null and zero before the FFI call, and assume cleanup may run after either success or error. Validate that the native result is sorted and unique. If it is not, either sort/deduplicate defensively in Rust and record a native bug observation, or return an internal error if unsorted output indicates memory corruption. Empty input should return an empty vector without calling native code.

Map native errors into `WavepeekError::File` with messages that begin with `FSDB Reader`, following the existing metadata, hierarchy, and value wrappers. Do not expose proprietary enum values or Reader struct details in Rust errors. New wrappers should be reached only after `FsdbBackend` has called a metadata guard that validates integer time tags; native code should still validate integer tags before reading tag fields so lower-level misuse fails safely.

Acceptance for this milestone:

    cargo fmt -- --check
    cargo check
    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb

### Implement FSDB backend expression sampling and candidate collection

Update `src/waveform/fsdb_backend.rs` so the FSDB backend provides the operations required by `change` and `property`.

First, add a small metadata cache to `FsdbBackend` if one does not already exist. The cache must retain the raw FSDB start time, raw end time, and validated integer tag kind from `FsdbReader::metadata()`, while still returning the existing public `WaveformMetadata` strings. All FSDB sampling, event, and candidate methods should call an `ensure_integer_time_metadata()` helper before using raw tag operations.

Then change `previous_sample_time` so it respects the dump start, not only raw zero:

    pub(super) fn previous_sample_time(&self, raw_time: u64) -> Option<u64> {
        let start = self.cached_time_start_raw()?;
        if raw_time <= start {
            None
        } else {
            raw_time.checked_sub(1)
        }
    }

The actual implementation cannot return `Result`, so `cached_time_start_raw()` should read the value populated by the earlier command metadata call and fall back conservatively only in unreachable misuse cases. The command engines call `metadata()` before building candidate schedules; add tests with a nonzero dump start so this contract stays true.

Second, implement `sample_expr_value`. For `ExprTypeKind::BitVector`, `ExprTypeKind::IntegerLike(_)`, and `ExprTypeKind::EnumCore`, verify through `FsdbHierarchyIndex::signal_value_encoding` that the resolved signal is sampleable as a bit vector and call the metadata guard. Then call `FsdbReader::sample_signal_values` for that idcode and timestamp, convert `bits: None` into `SampledValue::Integral { bits: None, label: None }`, and convert `bits: Some(bits)` into `SampledValue::Integral { bits: Some(bits), label }`. The `label` is `None` unless the resolved `ExprType` has enum label metadata and the sampled bits exactly match one of its labels. If the native bit width does not match the expression type width, prefer returning a `WavepeekError::File` that names the path and both widths; do not silently resize FSDB expression operands unless a test proves that a known FSDB datatype needs normalization.

For `ExprTypeKind::Event`, return an internal error from `sample_expr_value`, because raw events must be queried through `expr_event_occurred`. For `ExprTypeKind::Real` and `ExprTypeKind::String`, return a clear unsupported diagnostic until fixture-backed decoding exists. A good message shape is `signal '<path>' has unsupported FSDB expression value encoding`. The expression runtime will render this through its existing runtime diagnostic path.

Third, implement `expr_event_occurred`. It should require `ExprTypeKind::Event`, call the metadata guard, call `FsdbReader::signal_event_occurred`, and return the exact occurrence result. A non-event operand reaching this method is an internal error, matching Wellen behavior.

Fourth, implement `collect_change_times_with_mode` for printable `ResolvedSignal` values. If `resolved` is empty, return an empty vector. Validate each path through hierarchy value metadata and reject unsupported non-bit-vector output signals with the existing message `signal '<path>' has unsupported non-bit-vector encoding`. Call the metadata guard, deduplicate idcodes, call `FsdbReader::collect_signal_change_times`, and return the sorted timestamps. Treat `Auto`, `Random`, and `Stream` as the same FSDB portable collector.

Fifth, implement `collect_expr_candidate_times_with_mode` for `ExprResolvedSignal` values. Split the resolved inputs into value-like sources and raw event sources. Value-like sources with integral expression types should be validated like `sample_expr_value`, then included in the candidate idcode set. Raw event sources should be included too; their value-change records are occurrence candidates. Real and string sources should return the same unsupported expression value diagnostic unless this plan has been expanded with fixture-backed real/string decoding. Call the metadata guard, deduplicate idcodes across both groups, call `FsdbReader::collect_signal_change_times`, and return the sorted timestamps.

Add small per-command caches inside `FsdbBackend` if repeated expression evaluation becomes obviously noisy. Useful caches are `(SignalId, raw_time) -> SampledValue` for expression samples and `(SignalId, raw_time) -> bool` for event occurrences. These caches are safe because one `FsdbBackend` instance belongs to one opened waveform command. Do not add global static caches.

Keep `indexed_timestamps`, `indexed_signal_offset_at`, `decode_indexed_signal_at`, and `ensure_indexed_signals_loaded` returning `None` or `false` for FSDB. That forces the existing command dispatcher to use portable baseline behavior. Do not pretend FSDB has a Wellen-style global timestamp table.

Acceptance for this milestone:

    cargo fmt -- --check
    cargo test -q fsdb_hierarchy
    cargo test -q expression_event_runtime
    cargo test -q expression_rich_types
    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb

### Enable command flow and add FSDB CLI parity tests

Update `src/waveform/mod.rs` and the command tests so FSDB `change` and `property` are no longer rejected by `unsupported_fsdb_command_error`. The simplest acceptable implementation is to remove the `change` and `property` match arms so the method returns `None` for all commands on FSDB. If the method becomes unused after that, delete it and remove the command-level calls in `src/engine/change.rs` and `src/engine/property.rs`. Keep default no-feature FSDB file detection unchanged; that path is about opening files, not command support.

Extend `tests/fsdb_cli.rs`. Add `change_property_core`, `change_property_offset_start`, `change_property_real_output`, and, when conversion preserves event variables, `change_property_events` to `GeneratedFsdbFixtures`. Add generic helpers so tests can compare any command's generated FSDB JSON payload against the source VCD JSON payload, not only `value`. Avoid brittle helpers that hard-code one command name. Parity helpers must compare the stable envelope as well as `data`: `$schema`, `command`, `warnings`, and `data` should match unless a test explicitly documents a warning difference.

Required generated-fixture tests:

- `fsdb_change_wildcard_matches_vcd_for_generated_fixture`: convert `change_property_core.vcd`, run `wavepeek change --scope top --signals data --from 0ns --to 20ns --on '*' --max 10 --json` on both VCD and generated FSDB, and assert identical stable JSON envelopes. Also assert exact rows at `5ns`, `7ns`, and `15ns`, empty warnings, and the expected schema URL.
- `fsdb_change_edge_iff_matches_vcd_for_generated_fixture`: run `wavepeek change --scope top --signals valid,ready,data --from 0ns --to 35ns --on 'posedge clk iff valid' --max 10 --json` on VCD and generated FSDB, assert identical stable JSON envelopes, and assert rows at `5ns` and `15ns` only.
- `fsdb_change_bounds_truncation_and_human_output_are_stable`: verify `--from` and `--to` are inclusive, verify `--max 1` truncates with the existing warning, verify scoped human output uses relative names, and verify `--abs` uses canonical paths.
- `fsdb_change_rejects_non_bit_vector_output_signal`: use `change_property_real_output.vcd`, run `change --scope top --signals temp --from 0ns --to 10ns --on clk`, and assert exit code `1`, empty stdout, `error: signal:`, and `signal 'top.temp' has unsupported non-bit-vector encoding`. The explicit bit-vector trigger prevents a real-valued wildcard candidate from producing a different earlier diagnostic.
- `fsdb_property_switch_matches_vcd_for_generated_fixture`: run `property --scope top --on 'posedge clk' --eval 'valid && ready' --capture switch --json` on both VCD and generated FSDB, assert identical stable JSON envelopes, and assert rows `{time: 15ns, kind: assert}`, `{time: 25ns, kind: deassert}`, and `{time: 35ns, kind: assert}`.
- `fsdb_property_window_baseline_matches_vcd_for_generated_fixture`: run `property --scope top --from 10ns --to 25ns --on 'posedge clk' --eval 'valid && ready' --capture switch --json` on both VCD and generated FSDB, assert identical stable JSON envelopes, and assert the expected bounded-window transition at `25ns deassert`.
- `fsdb_property_capture_modes_match_vcd_for_generated_fixture`: verify `match`, `assert`, and `deassert` over the same fixture and expression. Assert at least one exact output per mode, not just success.
- `fsdb_property_wildcard_infers_eval_sources`: run `property --scope top --eval "data == 8'h2a" --capture match --json` with omitted `--on`, compare the VCD and FSDB stable JSON envelopes, and assert the expected first match.
- `fsdb_property_nonzero_start_named_event_matches_vcd`: use `change_property_offset_start.vcd`, run `property --from 100ns --to 110ns --on valid --eval ready --capture match --json`, compare stable JSON envelopes, and assert the `data` array is empty. This fixture must keep `ready=1` at `100ns` so an incorrect first-value-as-change implementation would produce a visible false-positive row.
- `fsdb_property_edge_and_iff_match_vcd_for_generated_fixture`: use an expression such as `--on 'posedge clk iff ready' --eval "state == 2'h2" --capture match` and compare VCD and FSDB output.
- `fsdb_property_raw_event_trigger_matches_vcd_for_generated_fixture`: require this test before documenting raw-event support. It should use `change_property_events.vcd` only after generated FSDB `signal` output reports `top.tick` as kind `event`. Compare stable JSON envelopes for `property --scope top --on tick --eval armed --capture match --json` and assert matches at `10ns` and `25ns`.

Replace `fsdb_unsupported_change_and_property_fail_clearly` with tests that prove both commands now work. Do not leave stale unsupported-message assertions. If any user-facing unsupported diagnostic remains for unsupported operands such as real/string expressions, test that diagnostic under a name that describes the operand behavior, not command-wide unsupported status.

Add a bounded bundled-example smoke only if it can be robust. A safe pattern is to discover simple bit-vector candidate paths from `cpu.fsdb`, try a short `change` query with `--max 1`, and accept either one valid row or the existing no-changes warning as long as the command succeeds and the JSON shape is correct. Do not make the test depend on proprietary full golden contents from the bundled design.

Acceptance for this milestone on a Verdi-equipped machine:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --test fsdb_cli -- --nocapture
    WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Generated `.fsdb` files created by these commands are ignored artifacts and must not be committed.

### Update public docs, changelog, and architecture traceability

Update public-facing docs after the implementation and tests prove behavior. At minimum, edit `docs/public/intro.md` and `docs/public/reference/command-model.md` so they no longer say FSDB-enabled builds support only `info`, `scope`, `signal`, and `value`. State that FSDB-enabled builds support `change` and `property` through the same command surface for digital/integral expression operands. Mention raw event occurrence only if `fsdb_property_raw_event_trigger_matches_vcd_for_generated_fixture` or an equivalent deterministic FSDB test passes. Keep real/string expression support described as limited to fixture-backed cases. Keep wording concise and user-facing.

Update command docs only if their current examples become misleading for FSDB. `docs/public/commands/change.md`, `docs/public/commands/property.md`, and `docs/public/reference/expression-language.md` should not grow FSDB-specific flag tables. If a note is needed, phrase it as a backend support caveat, not a duplicate of generated help.

Update `CHANGELOG.md` under the unreleased section with a bullet describing FSDB-enabled `change` and `property` support. Mention that default builds still require the `fsdb` feature and local Verdi SDK for FSDB input.

After implementation is complete and reviewed, move this plan directory from `docs/exec-plans/active/2026-05-27-fsdb-change-property-portable/` to `docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/`, then update the FSDB architecture roadmap section in `docs/fsdb/arch.md` to include the completed implementation record path. This archival move was done after the user's inspection request, without renaming the plan directory to a roadmap label.

Acceptance for this milestone:

    cargo fmt -- --check
    WAVEPEEK_IN_CONTAINER=1 make check

### Full validation, review, and handoff

Run default validation after all source, fixture, test, and docs changes:

    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci

Run feature-enabled validation on a Verdi-equipped machine:

    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Also run focused commands when isolating failures:

    cargo fmt -- --check
    cargo check
    cargo test -q fsdb_hierarchy
    cargo test -q expression_event_runtime
    cargo test -q expression_rich_types
    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo clippy --features fsdb --all-targets -- -D warnings
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --test fsdb_cli -- --nocapture

Request focused read-only review lanes before final handoff: native/FFI safety, Rust backend correctness, CLI tests/contracts, and documentation/plan completeness. Apply fixes in the main session only, rerun relevant validation, and run one fresh independent control review over the consolidated diff. Update `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` after each review loop and before committing.

Acceptance for final handoff is: default checks pass, FSDB checks pass where Verdi is available, review returns no substantive findings or all findings are resolved and rechecked, generated FSDB artifacts are not tracked, this plan is archived at `docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md`, `docs/fsdb/arch.md` points to the completed implementation record, and `git status --short` is clean after the final commit.

## Concrete Steps

Work from the repository root:

    cd /workspaces/wavepeek-fsdb

Create or edit the following files during implementation:

- `tests/fixtures/hand/change_property_core.vcd`: new text fixture for generated FSDB `change` and `property` parity.
- `tests/fixtures/hand/change_property_offset_start.vcd`: new text fixture for nonzero-start previous-timestamp parity.
- `tests/fixtures/hand/change_property_real_output.vcd`: new text fixture for unsupported real output-signal diagnostics with a bit-vector trigger.
- `tests/fixtures/hand/change_property_events.vcd`: new text fixture for raw event parity if `vcd2fsdb` preserves event variables; do not document raw-event support without an equivalent passing test.
- `native/fsdb/wavepeek_fsdb_shim.h`: add project-owned candidate-time list and raw-event occurrence declarations.
- `native/fsdb/wavepeek_fsdb_shim.cpp`: implement selected-idcode value-change traversal, sorted timestamp collection, exact raw-event occurrence, native allocation cleanup, and Reader signal-list guarding.
- `src/waveform/fsdb_native.rs`: mirror the new C ABI and add safe Rust wrappers that free native allocations.
- `src/waveform/fsdb_hierarchy.rs`: add any missing value metadata or tests needed for expression operand support, especially if enum labels are implemented.
- `src/waveform/fsdb_backend.rs`: implement strict previous timestamp, expression sampling, exact event occurrence, printable change candidate collection, expression candidate collection, and small per-command caches if useful.
- `src/waveform/mod.rs`: remove the FSDB `change` and `property` unsupported guard once backend methods exist.
- `src/engine/change.rs` and `src/engine/property.rs`: edit only if removing the unsupported guard method requires command-level call cleanup. Do not fork command semantics for FSDB.
- `tests/fsdb_cli.rs`: add generated-fixture parity tests for FSDB `change` and `property`; remove stale command-wide unsupported assertions.
- `docs/public/intro.md` and `docs/public/reference/command-model.md`: update FSDB support wording.
- `CHANGELOG.md`: record the user-visible FSDB command expansion.
- `docs/fsdb/arch.md`: link the completed implementation record after the user-requested archival move.
- `docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md`: final completed execution record for this work.

Do not edit `schema/wavepeek.json` unless validation proves a genuine existing schema bug. This work should fit the current schema.

## Validation and Acceptance

A human can verify the feature on a Verdi-equipped machine after implementation by running:

    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Then manually compare generated FSDB and VCD command output:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo run --features fsdb -- change --waves tests/fixtures/fsdb/change_property_core.fsdb --scope top --signals data --from 0ns --to 20ns --on '*' --max 10 --json
    cargo run --quiet -- change --waves tests/fixtures/hand/change_property_core.vcd --scope top --signals data --from 0ns --to 20ns --on '*' --max 10 --json

The two `data` payloads should be identical. Expected FSDB `change` rows for this query are `5ns`, `7ns`, and `15ns`, with `data` values `8'h0f`, `8'h1f`, and `8'h2a`.

Verify property switching:

    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo run --features fsdb -- property --waves tests/fixtures/fsdb/change_property_core.fsdb --scope top --on 'posedge clk' --eval 'valid && ready' --capture switch --json

Expected result rows are:

    {"time":"15ns","kind":"assert"}
    {"time":"25ns","kind":"deassert"}
    {"time":"35ns","kind":"assert"}

Verify the bounded-window property baseline:

    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo run --features fsdb -- property --waves tests/fixtures/fsdb/change_property_core.fsdb --scope top --from 10ns --to 25ns --on 'posedge clk' --eval 'valid && ready' --capture switch --json

Expected output contains the bounded-window transition at `25ns deassert` and matches the VCD payload for the same command.

Verify the command-wide unsupported errors are gone by running the same FSDB fixture through `change` and `property` without seeing messages that say the commands are not implemented. Unsupported operands may still fail clearly, for example real or string expressions if they are not implemented in this slice.

Full acceptance requires all of these to pass where applicable:

    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci
    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

## Idempotence and Recovery

The new VCD fixtures are durable source files and can be overwritten safely if their contents are intentionally revised. Generated `.fsdb` files under `tests/fixtures/fsdb/` are ignored artifacts; delete them and rerun `WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures` if conversion fails or if fixture contents change. Temporary converter directories created by integration tests are owned by `tempfile` and should be cleaned automatically.

Native candidate collection and event occurrence modify the Reader signal list. Keep those operations inside native transaction-style functions that load selected idcodes, perform the requested traversal, and unload/reset through guards before returning. If a native test fails after a Reader error, rerun it in a fresh process; CLI integration tests already launch child processes, which is the safest recovery boundary for proprietary in-process libraries.

If `cargo check --features fsdb` fails because `VERDI_HOME` is missing or incomplete, do not patch around `build.rs`. Run `WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env` and fix the local environment, or limit validation to default no-Verdi gates and record that FSDB gates were not runnable.

If generated FSDB output differs from VCD output, first confirm the VCD fixture expresses the intended current contract. Then inspect project-owned code and allowed local SDK examples, not binary FSDB contents. Fix decoder orientation, timestamp filtering, previous-timestamp behavior, or event occurrence logic so FSDB matches VCD unless there is a documented FSDB Reader limitation. The VCD path is the ruler; the new backend is the suspect.

## Artifacts and Notes

Record concise validation snippets here as implementation proceeds. Useful snippets include the baseline commit, fixture probe JSON excerpts, native candidate traversal evidence, FSDB/VCD parity excerpts, real/string unsupported diagnostics if exercised, final `make test-fsdb` summaries, and review outcomes. Do not paste Verdi header excerpts, proprietary documentation text, or full outputs from bundled FSDB designs.

Initial planning notes:

    Branch observed while drafting: feat/fsdb
    Original active plan path: docs/exec-plans/active/2026-05-27-fsdb-change-property-portable/PLAN.md
    Completed plan path: docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md
    Existing FSDB command state: info, scope, signal, and digital bit-vector value work; change and property are command-wide unsupported through the waveform guard.
    Existing FSDB value native operation: wp_fsdb_sample_signal_values returns owned bit strings for selected idcodes at one raw timestamp.
    Existing portable engine hook points: collect_expr_candidate_times_with_mode, sample_expr_value, expr_event_occurred, sample_resolved_optional, previous_sample_time.
    Focused review findings incorporated: previous_sample_time must respect nonzero dump starts; candidate collection must not add iff-only operands beyond current engine behavior; native time-list outputs must be initialized and freed safely; candidate deduplication must not hide same-timestamp final-value sampling; raw-event support must not be documented without deterministic FSDB coverage.
    Control review result: one medium finding on the nonzero-start fixture timeline was fixed by requiring valid=1 and ready=1 at 100ns and an empty data array assertion; follow-up recheck reported no substantive findings.
    Baseline validation before editing: WAVEPEEK_IN_CONTAINER=1 make check passed; WAVEPEEK_IN_CONTAINER=1 make ci passed; WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env reported ok; WAVEPEEK_IN_CONTAINER=1 make test-fsdb passed with 13 FSDB CLI tests.
    VCD fixture probe excerpts after adding fixtures: change_property_core wildcard change on data emitted 5ns/7ns/15ns with values 8'h0f, 8'h1f, and 8'h2a; edge-gated change emitted 5ns and 15ns; property switch emitted 15ns assert, 25ns deassert, and 35ns assert; wildcard-inferred property on data == 8'h2a emitted 15ns match; offset-start property emitted an empty data array; raw-event VCD property emitted 10ns and 25ns matches.
    Raw event conversion probe: generated temporary FSDB signal listing preserved top.tick as kind event. Initial native candidate traversal missed this event fixture because ffrGotoXTag(0) fails when a signal's first value-change tag is later than the requested from time; after falling back to ffrGetMinXTag and filtering against the requested window, raw-event property parity emitted 10ns and 25ns matches.
    Implementation validation after native/Rust/test changes: WAVEPEEK_IN_CONTAINER=1 make check passed; WAVEPEEK_IN_CONTAINER=1 make lint-fsdb passed; WAVEPEEK_IN_CONTAINER=1 make test-fsdb passed with 16 FSDB CLI tests.
    Documentation validation after public docs/changelog/FSDB notes: cargo fmt and WAVEPEEK_IN_CONTAINER=1 make check passed; WAVEPEEK_IN_CONTAINER=1 make test-fsdb still passed with 16 FSDB CLI tests.
    Focused review loop 1: native/Rust code lane reported no substantive findings; tests lane requested full envelope parity, more capture/iff coverage, truncation/human-output coverage, and a stronger real-output rejection trigger; docs lane requested stale plan status fixes, active-plan acceptance wording, per-signal traversal wording, and deferred real/string decode wording; architecture/performance lane requested in-traversal candidate deduplication and noted native sampler churn. Applied the test/doc/dedup fixes, documented the sampler churn as deferred M6 performance work, and reran cargo fmt, WAVEPEEK_IN_CONTAINER=1 make check, WAVEPEEK_IN_CONTAINER=1 make lint-fsdb, and WAVEPEEK_IN_CONTAINER=1 make test-fsdb successfully.
    Control review pass 1 found one medium issue: FSDB wildcard expression candidate collection could silently succeed for unsupported real/string operands if no candidate timestamp fell inside the requested window. Fixed by validating FSDB expression candidate source encodings before native traversal and adding a no-candidate-window property rejection case. Reran cargo fmt, WAVEPEEK_IN_CONTAINER=1 make check, WAVEPEEK_IN_CONTAINER=1 make lint-fsdb, and WAVEPEEK_IN_CONTAINER=1 make test-fsdb successfully.
    Targeted control recheck found the same unsupported-value class still possible for explicit property triggers with no candidates. Fixed by adding a backend-neutral `Waveform::validate_expr_values_supported` hook, calling it on all bound property eval operands before candidate collection, and adding an explicit-trigger no-candidate-window real operand rejection case. Reran cargo fmt, WAVEPEEK_IN_CONTAINER=1 make check, WAVEPEEK_IN_CONTAINER=1 make lint-fsdb, and WAVEPEEK_IN_CONTAINER=1 make test-fsdb successfully.
    Final validation after the explicit-trigger fix commit: WAVEPEEK_IN_CONTAINER=1 make ci passed with src/** coverage regions 94.75%, functions 95.34%, lines 95.26%, average 95.12%; WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env, check-fsdb-build, lint-fsdb, prepare-fsdb-fixtures, and test-fsdb passed with 16 FSDB CLI tests. Final targeted control recheck reported no substantive findings.
    User-requested archival: moved this plan to docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md and updated docs/fsdb/arch.md so M5 points at the completed record.

## Interfaces and Dependencies

At the end of this work, the following internal interfaces should exist or their equivalents should be clearly documented in this plan.

In `native/fsdb/wavepeek_fsdb_shim.h`, add project-owned C ABI interfaces for timestamp lists and event occurrence:

    typedef struct wp_fsdb_time_list {
        uint64_t *times;
        size_t count;
    } wp_fsdb_time_list;

    wp_fsdb_status wp_fsdb_collect_signal_change_times(
        wp_fsdb_reader *reader,
        const uint64_t *idcodes,
        size_t count,
        uint64_t from_raw,
        uint64_t to_raw,
        wp_fsdb_time_list *out,
        char **error_message
    );

    void wp_fsdb_free_time_list(wp_fsdb_time_list *list);

    wp_fsdb_status wp_fsdb_signal_event_occurred(
        wp_fsdb_reader *reader,
        uint64_t idcode,
        uint64_t query_time_raw,
        int *occurred,
        char **error_message
    );

In `src/waveform/fsdb_native.rs`, expose safe wrappers shaped like:

    pub(super) struct FsdbCandidateTimes {
        pub(super) times: Vec<u64>,
    }

    impl FsdbReader {
        pub(super) fn collect_signal_change_times(
            &self,
            idcodes: &[u64],
            from_raw: u64,
            to_raw: u64,
        ) -> Result<FsdbCandidateTimes, WavepeekError>;

        pub(super) fn signal_event_occurred(
            &self,
            idcode: u64,
            query_time_raw: u64,
        ) -> Result<bool, WavepeekError>;
    }

In `src/waveform/fsdb_backend.rs`, these methods must return working FSDB behavior instead of unsupported errors:

    pub(super) fn previous_sample_time(&self, raw_time: u64) -> Option<u64>;

    pub(super) fn sample_expr_value(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<SampledValue, WavepeekError>;

    pub(super) fn expr_event_occurred(
        &mut self,
        resolved: &ExprResolvedSignal,
        query_time_raw: u64,
    ) -> Result<bool, WavepeekError>;

    pub(super) fn collect_change_times_with_mode(
        &mut self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError>;

    pub(super) fn collect_expr_candidate_times_with_mode(
        &mut self,
        resolved: &[ExprResolvedSignal],
        from_raw: u64,
        to_raw: u64,
        mode: ChangeCandidateCollectionMode,
    ) -> Result<Vec<u64>, WavepeekError>;

The implementation depends on the existing FSDB feature gate in `Cargo.toml`, `build.rs`, and `Makefile`; the existing native Reader open/metadata/hierarchy/value sampling functions; the expression runtime in `src/expr/`; and the public command contracts documented under `docs/public/`.
