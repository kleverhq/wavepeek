# Implement FSDB value command support

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the repository `exec-plan` skill. It is intentionally self-contained: a contributor with only the current working tree and this file should be able to implement, validate, and review the change without prior conversation.

All durable repository entities created by this plan must use descriptive FSDB command names, not roadmap milestone labels. Do not add milestone-label prefixes, suffixes, directory names, module names, function names, test names, documentation anchors, or commit subjects. Use names such as `value_vectors.vcd`, `fsdb_value_json_matches_vcd_for_generated_vector_fixture`, `RawValueKind`, `FsdbSignalValueInfo`, `wp_fsdb_load_signals`, and `wp_fsdb_sample_bit_vector`. If a new name would contain a milestone label, especially the roadmap label for this value slice, rename it before committing. Existing ignored build artifacts produced by broad repository helper scripts are not durable entities for this plan, but do not mention or depend on them. Future archaeologists have suffered enough.

## Purpose / Big Picture

After this change, a `wavepeek` binary built with the optional Cargo feature `fsdb` and a local licensed Synopsys Verdi FSDB Reader SDK can run the existing `value` command directly on FSDB waveform files. Users still run the same command shape they use for VCD and FST, for example `wavepeek value --waves dump.fsdb --scope top --signals clk,data --at 10ns --json`; the command resolves scopes and signals the same way, validates the same time tokens, preserves signal order and duplicates, prints the same Verilog literal format, and uses the same JSON schema.

The visible proof is a Verdi-equipped `make test-fsdb` run. The tests generate local ignored FSDB files from small checked-in VCD fixtures, run `wavepeek value` on both the VCD and generated FSDB versions with the same arguments, and compare the payloads. They also prove exact timestamp sampling, at-or-before sampling between transitions, duplicate preservation, scope-relative and absolute human output, missing-prior-value errors, bit-vector orientation, unknown/high-impedance formatting, and unsupported real-valued signal errors. A separate bundled-example smoke test samples at least one discovered bit-vector signal from `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` without storing that proprietary waveform’s full contents in Git.

Default builds without `--features fsdb` remain Verdi-free and continue to produce the existing clear FSDB-disabled error. FSDB `change` and `property` remain explicitly unsupported after this plan; this slice implements only point-in-time value sampling. One command at a time, because the machine spirits punish ambition without tests.

## Non-Goals

This plan does not implement FSDB `change`, FSDB `property`, event candidate collection, raw event occurrence checks, expression evaluation over FSDB values, or FSDB-specific indexed fast paths. The existing Wellen-optimized paths for VCD/FST remain untouched except for behavior-preserving facade adjustments.

This plan does not add public CLI flags, public JSON fields, JSON schema changes, public error categories, or new `signal.kind` strings. FSDB `value` output must fit the existing `schema/wavepeek.json` command contract.

This plan supports the first useful digital Verilog/SystemVerilog value encoding: bit-vector-like values that can be decoded to one character per bit using `0`, `1`, `x`, and `z`. It intentionally rejects real, string, analog, transaction, power, coverage, dummy/internal variables, unknown byte encodings, and signals without meaningful width. VHDL nine-state decoding and enum label formatting are deferred until fixture coverage proves the exact behavior.

This plan does not commit Verdi headers, libraries, documentation excerpts, generated bindings derived from proprietary headers, `.fsdb` fixtures, native converter logs, full golden outputs from Verdi-bundled FSDB files, or copied vendor sample code. Checked-in tests may add small VCD fixtures under `tests/fixtures/hand/`; generated FSDB files remain ignored local artifacts under temporary directories or `tests/fixtures/fsdb/`.

This plan does not rename existing modules for aesthetics. It extends the current `FsdbBackend`, `fsdb_native`, `fsdb_hierarchy`, and native shim layers because they are already the backend boundary established by previous FSDB work.

## Progress

- [x] (2026-05-24 20:26Z) Read `docs/fsdb/arch.md`, `docs/fsdb/cmd_value.md`, current FSDB hierarchy command plan, `src/waveform/mod.rs`, `src/waveform/fsdb_backend.rs`, `src/waveform/fsdb_native.rs`, `src/waveform/fsdb_hierarchy.rs`, `native/fsdb/wavepeek_fsdb_shim.{h,cpp}`, `src/engine/value.rs`, `tests/fsdb_cli.rs`, `Makefile`, and `docs/DEVELOPMENT.md`.
- [x] (2026-05-24 20:26Z) Drafted this active ExecPlan under `docs/exec-plans/active/2026-05-24-fsdb-value-command/PLAN.md` with descriptive names and no milestone-labelled created durable entities.
- [x] (2026-05-24 20:43Z) Ran four read-only review lanes for plan completeness, native/FFI safety, Rust backend/test contracts, and architecture/scope risk; incorporated their substantive findings into the plan.
- [x] (2026-05-24 20:51Z) Ran an independent control review over the revised plan; it returned no substantive findings.
- [x] (2026-05-25 00:18Z) Confirmed starting branch `feat/fsdb`, current head `9d13d31`, default `make check`, default `make ci`, FSDB environment probe, and pre-change `make test-fsdb` all pass.
- [x] (2026-05-25 00:26Z) Added checked-in VCD fixtures for value-vector parity, delayed first value, and unsupported real signal coverage; VCD-only `wavepeek value` probes match the intended Wellen contract.
- [x] (2026-05-25 00:55Z) Extended FSDB hierarchy metadata with project-owned value-encoding classification so the backend can reject unsupported signal classes before native traversal.
- [x] (2026-05-25 00:55Z) Added native FSDB batch at-or-before bit-vector sampling through the wavepeek-owned C ABI with signal-list load/unload guarding and traverse-handle cleanup.
- [x] (2026-05-25 01:02Z) Wrapped the new native sample array safely in Rust, including owned sample cleanup, native error conversion, and decoded bit-string ownership.
- [x] (2026-05-25 01:04Z) Implemented `FsdbBackend::sample_resolved_optional` for digital bit vectors and removed the FSDB `value` unsupported guard while leaving expression sampling unsupported.
- [x] (2026-05-25 01:12Z) Added feature-gated FSDB CLI integration coverage for generated fixtures, bundled-example smoke, missing initial values, and unsupported real encodings.
- [ ] Run default and feature-enabled validation gates, record outcomes, run focused reviews, apply fixes, and keep this plan updated.

## Surprises & Discoveries

- Observation: FSDB-enabled builds currently open real FSDB files for `info`, `scope`, and `signal`, but `value` is deliberately blocked before metadata parsing.
  Evidence: `src/engine/value.rs` calls `waveform.unsupported_fsdb_command_error("value")` immediately after `Waveform::open`, and `src/waveform/mod.rs` returns `unsupported_value_sampling()` for `Backend::Fsdb`.

- Observation: the current FSDB hierarchy index stores path, kind, optional width, and idcode, but not value encoding metadata such as dummy status, bytes-per-bit, or whether native sampling should be attempted.
  Evidence: `src/waveform/fsdb_hierarchy.rs` defines `RawSignalRecord` with `idcode`, `name`, `kind`, `left`, `right`, and `datatype_id`, and `FsdbSignalInfo` with `name`, `path`, `kind`, `width`, and `idcode` only.

- Observation: the existing `Waveform::sample_signals_at_time` already deduplicates requested paths, calls `sample_resolved_optional`, projects results back to the original order, and maps `bits: None` to the public “no value at or before requested time” signal error.
  Evidence: `src/waveform/mod.rs` uses `duplicate_preserving_projection`, then converts `SampledSignalState { bits: None }` into `WavepeekError::Signal("signal '<path>' has no value at or before requested time")`. The FSDB backend should plug into this rather than duplicating command-level value semantics.

- Observation: generated FSDB integration tests already exist and can convert checked-in VCD fixtures through `vcd2fsdb` in temporary directories.
  Evidence: `tests/fsdb_cli.rs` has `GeneratedFsdbFixtures`, `require_vcd2fsdb`, and `convert_vcd_fixture`, while `scripts/prepare_fsdb_fixtures.sh` and `make prepare-fsdb-fixtures` generate ignored FSDB files from `tests/fixtures/hand/*.vcd`.

- Observation: a VCD fixture can represent a valid time range where a signal has no value at the dump start.
  Evidence: a local probe with a VCD containing `#0` followed by the first value change at `#5` made current Wellen-backed `wavepeek value --at 0ns` fail with `error: signal: signal 'top.late' has no value at or before requested time`, which is the FSDB behavior this plan must preserve.

- Observation: a simple VCD real-valued signal converts through the local `vcd2fsdb` and appears through the current FSDB `signal` command as `kind: real`.
  Evidence: a local probe converted a VCD containing `$var real 64 ! temp $end`; `wavepeek signal --features fsdb` listed `top.temp` with kind `real`. This gives an actionable generated-fixture path for unsupported value encoding tests.

- Observation: FSDB Reader hierarchy metadata from `fsdbTreeCBDataVar` can mark VCD-derived vector wires with `bytes_per_bit` / `vc_dt` values that are not useful as a durable prefilter for `value` support; using those fields too strictly rejected ordinary vector data converted from VCD.
  Evidence: the first FSDB probe against converted `value_vectors.fsdb` failed with `error: signal: signal 'top.data' has unsupported non-bit-vector encoding` even though the same signal sampled correctly through `ffrGetVC()` after relaxing the prefilter.

- Observation: `ffrGotoXTag()` aligns a request before the first value change to the first value change rather than failing, so native code must compare the aligned value-change time to the original query time.
  Evidence: the delayed fixture preserves the existing contract because native sampling returns `has_value = false` when the aligned FSDB value time is greater than the requested raw time; the CLI then emits `signal 'top.late' has no value at or before requested time`.

- Observation: the Verdi-bundled `cpu.fsdb` contains straightforward digital signals under `system.i_cpu` that can exercise real FSDB sampling without committing proprietary fixtures.
  Evidence: `wavepeek value --waves $VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb --scope system.i_cpu --signals clock,addr,data --at 0ns --json` returned `clock=1'h0`, `addr=8'hxx`, and `data=8'hxx`.

## Decision Log

- Decision: decode FSDB digital value bytes inside the C++ shim and return an owned ASCII bit string to Rust.
  Rationale: proprietary bit constants and value buffer lifetimes belong on the C++ side that includes the local SDK headers. Rust receives only wavepeek-owned data: `has_value` plus a heap-owned string of `0`, `1`, `x`, and `z`, so Rust does not depend on FSDB enum numeric values or borrowed Reader buffers. Raw SDK diagnostics such as byte-encoding enum values may be used inside C++ for classification, but they must not be copied into Rust records, tests, or public error messages.
  Date/Author: 2026-05-24 / Grin

- Decision: store project-owned value metadata in `FsdbHierarchyIndex` before sampling.
  Rationale: unsupported encodings should produce `WavepeekError::Signal` with the user’s canonical signal path, not a generic native file error. The hierarchy callback already sees variable metadata, so it can classify each signal once and let `FsdbBackend` reject real/string/event/dummy/internal variables before loading signals.
  Date/Author: 2026-05-24 / Grin

- Decision: use generated VCD-derived fixtures for exact `value` parity and a bundled Verdi FSDB only for structural smoke.
  Rationale: generated fixtures provide known expected values and catch bit-order mistakes by comparing the same command on VCD and FSDB. The bundled `cpu.fsdb` proves the code works on a vendor-created FSDB without committing proprietary golden contents.
  Date/Author: 2026-05-24 / Grin

- Decision: keep `sample_expr_value` unsupported for FSDB in this slice even though `sample_resolved_optional` becomes implemented.
  Rationale: `wavepeek value` samples requested signal paths directly through `sample_resolved_optional`. Expression sampling drives `change` and `property`, which need candidate collection, event semantics, and richer type handling that are explicitly out of scope here.
  Date/Author: 2026-05-24 / Grin

- Decision: do not expose an FSDB global time table or indexed signal offset capability while implementing `value`.
  Rationale: `value` needs at-or-before sampling for selected signals at one raw timestamp. Wellen indexed fast paths are not required, and pretending FSDB has the same global table would couple later `change` work to the wrong abstraction.
  Date/Author: 2026-05-24 / Grin

- Decision: make the Rust FSDB wrapper provide one transaction-scoped value sampling operation, even if the C ABI remains split into load/sample/unload calls.
  Rationale: the native Reader signal list is mutable reader-global state. A native mutex held only per FFI call does not protect the sequence `load -> sample -> unload`. The safe Rust API must hold a per-reader transaction lock across the whole sequence, or the native layer must expose a single batch sampling call. The first implementation should use the Rust transaction lock because it keeps error handling and per-signal ordering in Rust.
  Date/Author: 2026-05-24 / Grin

- Decision: look up value metadata by canonical signal path, not by idcode alone.
  Rationale: FSDB can expose aliases or repeated tree sections where more than one canonical signal path shares an idcode. The user-facing error and declared range belong to the resolved path. Idcodes are still deduplicated for loading and sampling, but they are not the primary metadata key.
  Date/Author: 2026-05-24 / Grin

- Decision: reject non-integer FSDB time tags before value sampling.
  Rationale: the existing FSDB metadata reader supports integer `L`/`HL` tags only. `value` converts query time to the same raw integer representation as `info`; float or double tags are out of scope and must fail clearly before native bit traversal can mis-sample.
  Date/Author: 2026-05-24 / Grin

- Decision: keep new durable names descriptive and command-oriented rather than milestone-oriented.
  Rationale: names like `value_vectors.vcd` and `fsdb_value_rejects_real_signal_encoding` explain their job after roadmap labels fade. Milestone labels in filenames and APIs are stale metadata disguised as structure. Naturally they breed. Existing broad helper targets may generate ignored files from historical fixture names; do not depend on or mention those side effects for this plan.
  Date/Author: 2026-05-24 / Grin

- Decision: implement the native value operation as one batch C ABI call, `wp_fsdb_sample_signal_values`, rather than exposing separate load, sample, and unload calls to Rust.
  Rationale: the FSDB Reader signal list is reader-global mutable state, and a single native function can hold the native mutex, suppress Reader chatter, load selected signals, sample every requested idcode in order, and unload/reset through RAII guards even on failures. This is less clever than a half-transaction spread over FFI calls, which is another way of saying it has fewer places to leak state.
  Date/Author: 2026-05-25 / Grin

- Decision: classify only obvious non-vector signal classes as unsupported in hierarchy metadata and let native traversal validate bytes-per-bit at sampling time.
  Rationale: local evidence showed VCD-derived digital vectors can carry hierarchy `bytes_per_bit` / `vc_dt` metadata that looks unsupported while `ffrGetVC()` still returns ordinary 0/1/x/z bytes. Real, string, event, sparse-array, dummy/internal, and unknown kind records are still rejected before sampling so user-facing errors name the canonical signal path.
  Date/Author: 2026-05-25 / Grin

## Outcomes & Retrospective

Implementation is underway. The FSDB-enabled `wavepeek value` command now samples digital bit-vector signals through the native Reader, preserves duplicate/request order through the existing waveform facade, emits the same Verilog literal formatting as VCD/FST, maps missing prior values to the existing signal error, rejects real-valued signals with an unsupported non-bit-vector diagnostic, and leaves FSDB `change` / `property` explicitly unsupported. Targeted feature validation has passed so far: `cargo fmt -- --check`, `cargo check --features fsdb`, `cargo test -q fsdb_hierarchy --features fsdb`, `cargo test --features fsdb -q --test fsdb_cli -- --nocapture`, and `WAVEPEEK_IN_CONTAINER=1 make test-fsdb`. Full final validation and review cycles are still pending; naturally the machine wants its paperwork.

## Context and Orientation

`wavepeek` is a Rust command-line tool for inspecting RTL waveform dumps. VCD and FST files are read through the Rust crate `wellen`. FSDB is Synopsys’s proprietary Fast Signal Database format and must be read through a local licensed Verdi FSDB Reader SDK. The repository may store project-owned Rust and C++ source that calls local FSDB Reader headers at build time, but it must not store Synopsys headers, libraries, documentation excerpts, generated bindings, copied vendor examples, or `.fsdb` files.

The current command flow for `wavepeek value` is in `src/engine/value.rs`. It opens a `Waveform`, rejects FSDB `value` today through `unsupported_fsdb_command_error("value")`, reads metadata, resolves requested signals either as canonical paths or as names relative to `--scope`, then validates `--at <integer><unit>` through `src/engine/time.rs`, calls `Waveform::sample_signals_at_time`, and formats returned bit strings through `src/engine/value_format.rs`. Preserve this ordering so public error precedence does not drift: scope/signal resolution errors should not become time parsing errors just because FSDB sampling was added. JSON output contains `data.time` plus a `signals` array of objects with `path` and `value`. Human output prints `@<time>` and one line per signal; without `--abs`, it prints the original requested token, while `--abs` prints the canonical path.

The waveform facade lives in `src/waveform/mod.rs`. `Waveform` owns a backend enum. `Backend::Wellen` is the existing VCD/FST implementation in `src/waveform/wellen_backend.rs`. `Backend::Fsdb` is compiled only with `#[cfg(feature = "fsdb")]` and is implemented in `src/waveform/fsdb_backend.rs`. The FSDB backend already supports `metadata`, `scopes_depth_first`, `signals_in_scope`, `signals_in_scope_recursive`, and signal path resolution through a lazy hierarchy index. Its sampling methods currently return unsupported-operation errors.

Backend-neutral waveform types live in `src/waveform/types.rs`. `SignalId` is an opaque `u64` wrapper; the engine must not know whether it represents a Wellen signal reference or an FSDB idcode. `ResolvedSignal` contains `path`, `id`, and `width`. `SampledSignalState` contains `path`, `width`, and `bits: Option<String>`. `bits: None` means the signal has no value at or before the requested timestamp. The public `SampledSignal` used by `value` always has a concrete bit string.

The FSDB native boundary has two layers. `native/fsdb/wavepeek_fsdb_shim.{h,cpp}` is wavepeek-owned C++ code compiled only when the `fsdb` feature is enabled. It includes local SDK headers from `$VERDI_HOME/share/FsdbReader`, maps proprietary Reader types into project-owned C ABI records, catches C++ exceptions, suppresses native Reader stdout/stderr, and serializes Reader calls through a recursive mutex. `src/waveform/fsdb_native.rs` is the Rust FFI wrapper. It owns `FsdbReader`, converts native strings into Rust `String`s, turns native failures into `WavepeekError::File`, and builds `FsdbHierarchyIndex` from callback records.

The current FSDB hierarchy builder lives in `src/waveform/fsdb_hierarchy.rs`. It receives project-owned raw records such as `RawScopeRecord`, `RawSignalRecord`, and `RawDatatypeRecord`, normalizes public dot-separated paths, strips trailing packed ranges that match bit bounds, deduplicates repeated tree sections by canonical path, maps scope/signal kinds to stable schema aliases, and resolves paths into `ResolvedSignal` / `ExprResolvedSignal`. This plan extends it with value-sampling metadata while preserving the existing `info`, `scope`, and `signal` behavior.

Development is container-first. Default quality gates must not require Verdi. Use `WAVEPEEK_IN_CONTAINER=1 make check` and `WAVEPEEK_IN_CONTAINER=1 make ci` for default validation. FSDB-specific validation runs only on Verdi-equipped Linux machines through `WAVEPEEK_IN_CONTAINER=1 make lint-fsdb`, `WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures`, and `WAVEPEEK_IN_CONTAINER=1 make test-fsdb`. The existing broad fixture-preparation script converts every checked-in hand VCD fixture and may create ignored generated files for historical fixture names; those side effects are not durable entities created by this plan. Value-specific tests should reference the new `value_*` fixtures directly. FSDB Cargo invocations in Makefile use `CARGO_TARGET_DIR=target/fsdb` to avoid feature-enabled binaries racing default builds.

The `.fst` files in this repository are binary and must never be read with the text `read` tool. Treat `.fsdb` files the same way: do not inspect them as text. Tests may access FSDB files through the FSDB Reader API, `vcd2fsdb`, or binary-safe filesystem operations such as existence checks.

## Open Questions

No product decision blocks this plan. The exact local FSDB Reader signatures were verified from `$VERDI_HOME/share/FsdbReader/ffrAPI.h` and SDK examples without copying proprietary declarations into the repository. The implemented native path loads selected idcodes, creates one value-change traverse handle per idcode, calls `ffrGotoXTag`, reads the aligned value-change time and value buffer, decodes immediately into an owned project string, frees the handle, and unloads/reset signals through guards.

The implementation resolved the two known technical questions. In this SDK, `ffrGotoXTag` accepts the requested time pointer and an optional glitch-number pointer; the shim does not need the glitch count for point sampling. When the requested time is earlier than the first value change, `ffrGotoXTag` can align to the first value change, so the shim compares aligned raw time to requested raw time and returns `bits: None` when aligned time is greater. `FsdbBackend::sample_resolved_optional` now calls `metadata()` before sampling so lower-level calls get the same non-integer time-tag rejection as the command path.

## Milestones

### Confirm baseline and add focused value fixtures

Start from `/workspaces/wavepeek-fsdb` on branch `feat/fsdb` with a clean tree. Record the current commit and run the default checks before editing. On a Verdi-equipped machine, also confirm the FSDB environment and current FSDB tests. This gives a useful before-picture; apparently machines prefer evidence to optimism.

Run:

    cd /workspaces/wavepeek-fsdb
    git status --short --branch
    git rev-parse --short HEAD
    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci
    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env

If Verdi is available, also run:

    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Add three checked-in VCD fixtures under `tests/fixtures/hand/`:

- `value_vectors.vcd`, containing a `top` scope with scalar `clk`, 8-bit `data[7:0]`, 4-bit descending `nibble[3:0]`, 4-bit `status[3:0]`, and 4-bit ascending `asc[0:3]` signals. Use this exact value table:

      time 0ns:  clk=0, data=b00000000, nibble=b1010, status=bzzzz, asc=b1100
      time 5ns:  clk=1, data=b00001111, nibble=b10xz, status=b0011, asc=b0011
      time 10ns: clk=0, data=b11110000, nibble=b0101, status=bxxxx, asc=b1010

  The expected rendered literals are `1'h0`, `8'h00`, `4'ha`, `4'hz`, `4'hc` at `0ns`; `1'h1`, `8'h0f`, `4'hx`, `4'h3`, `4'h3` at `5ns` and between `5ns` and `10ns`; and `1'h0`, `8'hf0`, `4'h5`, `4'hx`, `4'ha` at `10ns`. The ascending-range signal exists because declared range direction is exactly the kind of tiny thing that ruins a Friday.
- `value_delayed.vcd`, containing `top.anchor` and `top.late`. Give `anchor` a concrete value at `0ns` so converters preserve a real dump start, but leave `late` unset until `5ns`, then change it again at `10ns`. Querying `late` at `0ns` should pass time-bounds validation and then fail with “no value at or before requested time.” Querying `late` at `7ns` should return the value from `5ns`.
- `value_real.vcd`, containing a `top.temp` real-valued signal with at least two real value changes. Querying it through `wavepeek value` should fail as an unsupported non-bit-vector encoding.

Before adding FSDB behavior, run VCD-only probes through the default binary to confirm the fixtures express the intended Wellen contract:

    cargo run --quiet -- value --waves tests/fixtures/hand/value_vectors.vcd --scope top --signals data,clk,data,nibble,status,asc --at 7ns --json
    cargo run --quiet -- value --waves tests/fixtures/hand/value_delayed.vcd --scope top --signals late --at 0ns
    cargo run --quiet -- value --waves tests/fixtures/hand/value_real.vcd --scope top --signals temp --at 0ns

Expected observations are: the first command succeeds with duplicate `data` preserved as `8'h0f`, `clk` as `1'h1`, `nibble` as `4'hx`, `status` as `4'h3`, and `asc` as `4'h3`; the second fails with a `signal` error saying no value exists at or before the requested time; the third fails with a `signal` error saying the signal has unsupported non-bit-vector encoding. Record the exact output snippets in `Artifacts and Notes` if they differ in punctuation.

Acceptance for this milestone is that the new VCD fixtures are tiny, readable, committed as text, covered by default Wellen behavior, and convertible by the existing FSDB fixture helpers without adding any checked-in FSDB file. Running the broad `make prepare-fsdb-fixtures` target may also generate ignored FSDB files for pre-existing hand fixtures; those artifacts are not named, referenced, committed, or required by this plan.

### Extend hierarchy metadata for value sampling decisions

Extend the FSDB hierarchy path so Rust can decide whether a resolved signal is eligible for `value` before native sampling. Add a project-owned value classification enum to the C ABI header, for example `wp_fsdb_value_kind` with variants for bit vector, real, string, event, and unsupported. Do not encode proprietary FSDB numeric constants in Rust. In `native/fsdb/wavepeek_fsdb_shim.cpp`, map FSDB variable metadata from the tree callback into this project-owned classification. The mapping should treat one-byte-per-bit digital Verilog-like values with meaningful width as sampleable bit vectors, and should classify real, string, event, dummy/internal, analog, transaction, power, coverage, unknown byte encodings, and missing-width variables as unsupported for `value` even if `signal` can list them.

Extend `wp_fsdb_signal_record` with only project-owned value metadata needed by Rust: the value classification, whether the signal has a declared packed range, the declared left and right bounds already exposed by the hierarchy callback, and a project-owned unsupported/dummy classification. Do not carry raw SDK diagnostic fields such as byte-encoding enum values or value-change data type numbers through the Rust FFI mirror. Those details may inform C++ classification, but Rust tests and errors must not depend on proprietary numeric semantics. Update the Rust FFI mirror in `src/waveform/fsdb_native.rs`, add a Rust `RawValueKind`, and extend `RawSignalRecord` in `src/waveform/fsdb_hierarchy.rs` accordingly.

Extend `FsdbSignalInfo` with value metadata. A useful shape is:

    pub(super) struct FsdbSignalValueInfo {
        pub(super) path: String,
        pub(super) id: SignalId,
        pub(super) idcode: u64,
        pub(super) width: u32,
        pub(super) declared_range: Option<FsdbSignalRange>,
        pub(super) kind: FsdbSignalValueKind,
    }

    pub(super) struct FsdbSignalRange {
        pub(super) left: i32,
        pub(super) right: i32,
    }

    pub(super) enum FsdbSignalValueKind {
        BitVector,
        Real,
        String,
        Event,
        Unsupported,
    }

Keep these types `pub(super)` unless a narrower visibility works. Add an `FsdbHierarchyIndex` method that lets `FsdbBackend` retrieve value metadata by canonical path without exposing the whole internal signal table. Use the resolved path as the primary key because aliases or repeated tree sections can share an idcode while still needing path-specific width/range/error text. It is fine to keep an idcode-to-sample cache inside the backend after metadata has been collected by path, but do not make idcode-only lookup the source of truth.

When resolving `wavepeek value` paths, keep the current behavior that `ResolvedSignal.width` is `1` if width is missing. However, `FsdbBackend::sample_resolved_optional` must reject missing or zero width for FSDB value sampling before calling native code, because a default width is fine for legacy path resolution but not enough evidence for decoding an FSDB value buffer.

Add pure Rust tests in `src/waveform/fsdb_hierarchy.rs` for value metadata:

- bit-vector raw signals with scalar and vector widths produce `FsdbSignalValueKind::BitVector` and the expected width;
- real, string, event, unknown, dummy, and missing-width raw signals produce non-sampleable metadata;
- datatype overrides still affect public `signal.kind` without accidentally making real/string/event sampleable as bit vectors;
- lookup by canonical path returns the correct metadata even when two records share an idcode;
- declared range metadata preserves descending and ascending bounds for later decoder orientation;
- unsupported value metadata includes enough information for `FsdbBackend` to produce the canonical-path signal error.

Acceptance for this milestone:

    cargo fmt -- --check
    cargo test -q fsdb_hierarchy
    cargo test -q fsdb_time

Default builds must still pass without Verdi.

### Add native signal loading and at-or-before bit-vector sampling

Extend `native/fsdb/wavepeek_fsdb_shim.h` and `native/fsdb/wavepeek_fsdb_shim.cpp` with a small batch C ABI for value sampling. Add `#include <stddef.h>` to the shim header when introducing `size_t`; the previous header only needed integer types, and literal implementation without that include would be a charmingly avoidable build failure. The implemented names are:

    typedef struct wp_fsdb_sample_record {
        uint64_t idcode;
        int has_value;
        uint32_t bit_width;
        uint64_t value_time_raw;
        char *bits;
    } wp_fsdb_sample_record;

    wp_fsdb_status wp_fsdb_sample_signal_values(
        wp_fsdb_reader *reader,
        const uint64_t *idcodes,
        size_t count,
        uint64_t query_time_raw,
        wp_fsdb_sample_record **out,
        char **error_message
    );

    void wp_fsdb_free_samples(wp_fsdb_sample_record *samples, size_t count);

The names describe the operation and do not contain milestone labels. The exact discriminants and structures are wavepeek-owned and mirrored only by wavepeek Rust code.

`wp_fsdb_sample_signal_values` clears the Reader signal list, adds each requested idcode, loads those signals, samples each idcode in request order, then unloads and resets through guards. Treat an empty idcode list as success at the Rust wrapper layer. Do not load all signals. The native function holds the Reader mutex across the complete operation so no other native call can interleave with the mutable signal list.

Each sampled idcode creates a value-change traverse handle after signals are loaded. Convert `query_time_raw` into the SDK’s integer tag representation with `H = raw_time >> 32` and `L = raw_time & 0xffffffff` only after the backend has confirmed the file metadata uses an integer `L` or `HL` time-tag type. Move the handle to the value at or before the query tag. If the Reader aligns to a value-change time after the query, return `WP_FSDB_STATUS_OK` with `has_value = 0` and `bits = nullptr`. If a handle cannot be created, if `ffrGetVC` fails after a successful seek, or if the value buffer contains an unsupported digital bit code for a signal that was classified as bit-vector, return a native error.

Decode the value buffer immediately while the traverse handle owns it. Allocate a NUL-terminated string of exactly `width` bit characters plus the terminator. The first implementation supports `0`, `1`, `x`, and `z` states. Unknown Verilog-style codes should map to `x` only when the SDK clearly documents them as unknown-strength variants; otherwise return unsupported. Do not let pointers from the Reader escape the native function. Always free the traverse handle on every path.

Bit order is important. `format_verilog_literal(8, "00001111")` renders `8'h0f`; reversing the FSDB buffer would render `8'hf0`, which is wrong. Use generated fixture tests to prove the native decoder returns the same bit string orientation as Wellen for both descending and ascending declared ranges. If local Reader examples imply the buffer order is least-significant first or declaration-order-dependent for some case, normalize in the C++ shim or Rust wrapper so the backend-neutral bit string remains most-significant-bit first, matching Wellen and existing `format_verilog_literal` expectations. Record the evidence in `Surprises & Discoveries`.

Keep the existing native safety properties: C++ exceptions do not cross FFI, errors become status plus owned strings, Reader stdout/stderr remains suppressed unless debug verbosity is explicitly enabled, and all suppressing Reader calls stay serialized by the native recursive mutex. Because load, sample, and unload operate on the same Reader object and signal list, native per-call locking is not sufficient on its own. The implemented single native batch transaction avoids interleaving by keeping load/sample/unload in one mutex-protected call.

Extend `src/waveform/fsdb_native.rs` with a transaction-scoped safe wrapper. `FsdbBackend` calls a single operation shaped like this:

    pub(super) struct FsdbNativeSample {
        pub(super) idcode: u64,
        pub(super) bit_width: u32,
        pub(super) value_time_raw: Option<u64>,
        pub(super) bits: Option<String>,
    }

    impl FsdbReader {
        pub(super) fn sample_signal_values(&self, idcodes: &[u64], query_time_raw: u64) -> Result<Vec<FsdbNativeSample>, WavepeekError>;
    }

`sample_signal_values` returns early for empty input, calls the native batch operation once, converts each native row into owned Rust data, and frees the native sample array on all paths. Integer time-tag support is verified in `FsdbBackend::sample_resolved_optional` by calling `metadata()` before sampling. Add focused feature-gated CLI coverage rather than proprietary native golden outputs; the CLI smoke against `cpu.fsdb` gives stronger user-visible evidence.

Acceptance for this milestone on a Verdi-equipped machine:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

If a new native sampling smoke is added, run it by exact test name too. Default `cargo test` must still compile without Verdi.

### Implement FSDB backend sampling and enable `wavepeek value`

Update `src/waveform/fsdb_backend.rs` so `sample_resolved_optional` implements FSDB digital bit-vector sampling. The method should preserve the backend-neutral contract:

1. If `resolved` is empty, return an empty vector.
2. Ensure cached metadata has accepted the file’s integer time-tag type. The command path already reads metadata before sampling; lower-level calls must not bypass this guard.
3. For every requested `ResolvedSignal`, fetch the corresponding FSDB value metadata from the hierarchy index by `resolved.path`.
4. If any signal is real, string, event, dummy/internal, missing width, zero width, or otherwise unsupported, return `WavepeekError::Signal("signal '<path>' has unsupported non-bit-vector encoding")`. Use the resolved canonical path in the message.
5. Build one ordered idcode list from the original resolved entries. The native batch loader deduplicates only the Reader signal-list load step while still returning one sampled row per requested idcode, preserving result order.
6. Call `FsdbReader::sample_signal_values` once with the ordered idcodes and raw query time. Do not call separate load/sample/unload wrappers from the backend.
7. Return one `SampledSignalState` per original `resolved` entry, with `bits: Some(bits)` for sampled values and `bits: None` for no prior value. The facade will turn `None` into the public `value` command error.

Do not implement `sample_expr_value`, `expr_event_occurred`, `collect_change_times_with_mode`, or `collect_expr_candidate_times_with_mode` here. They should keep returning the existing FSDB unsupported errors for `change` and `property` paths. `previous_sample_time` may continue returning `None` for FSDB because the command implemented here does not call it.

Update the FSDB unsupported-command guard. In `src/waveform/mod.rs`, `unsupported_fsdb_command_error("value")` must return `None` for `Backend::Fsdb` after this work, or `src/engine/value.rs` should stop calling that guard for `value`. Prefer keeping the guard method and changing its FSDB match arm so `change` and `property` remain blocked in their current command modules while `value` proceeds normally. Update any tests whose names or assertions still say value is unsupported.

Keep error categories aligned with existing behavior:

- unsupported value encoding is `WavepeekError::Signal` and renders like `error: signal: signal 'top.temp' has unsupported non-bit-vector encoding`;
- no prior value is represented as `bits: None` so `Waveform::sample_signals_at_time` renders the existing `error: signal: signal 'top.late' has no value at or before requested time`;
- native Reader failures such as load failure, traverse handle allocation failure, or corrupt value buffers are `WavepeekError::File` messages beginning with `FSDB Reader`;
- missing scope and missing signal resolution keep using existing `scope` and `signal` errors from the hierarchy index.

Acceptance for this milestone:

    cargo fmt -- --check
    cargo check
    cargo test -q value_helpers_exercise_resolution_time_errors_and_public_run

With Verdi available, also run:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

### Add FSDB value CLI tests and update FSDB command coverage

Extend `tests/fsdb_cli.rs` with feature-gated tests for the new `value` behavior. Reuse existing helpers where possible, but avoid overfitting to the bundled Verdi design. Add helper methods to `GeneratedFsdbFixtures` for `value_vectors`, `value_delayed`, and `value_real`, or replace the helper with a fixture-name-driven converter if that keeps the code shorter. Keep conversion in temporary directories and capture converter stdout/stderr.

Required generated-fixture tests:

- `fsdb_value_json_matches_vcd_for_generated_vector_fixture`: convert `value_vectors.vcd`, run `wavepeek value --scope top --signals data,clk,data,nibble,status,asc --at 7ns --json` on both the VCD and FSDB through the same feature-enabled binary, and assert the JSON `data` payloads are identical. Also assert exact values: duplicate `data` appears twice as `8'h0f`, `clk` is `1'h1`, `nibble` is `4'hx`, `status` is `4'h3`, `asc` is `4'h3`, `time` is `7ns`, `$schema` is the expected URL, command is `value`, warnings are empty, and stderr is empty.
- `fsdb_value_samples_exact_transition_and_dump_end`: query the same fixture at `0ns`, `5ns`, and `10ns` for vector signals and assert the literals from the fixture table: at `0ns` expect `8'h00`, `4'ha`, `4'hz`, and `4'hc`; at `5ns` expect `8'h0f`, `4'hx`, `4'h3`, and `4'h3`; at `10ns` expect `8'hf0`, `4'h5`, `4'hx`, and `4'ha`. This catches exact timestamp handling, bit order, and declared range direction.
- `fsdb_value_preserves_scope_relative_human_output_and_abs`: run human output without `--abs` using `--scope top --signals data,clk` and assert lines use `data` and `clk`; run with `--abs` and assert lines use `top.data` and `top.clk`.
- `fsdb_value_uses_previous_sample_and_reports_missing_prior_value`: use `value_delayed.vcd`; query `7ns` and assert it returns the `5ns` value; query `0ns` and assert exit code `1`, empty stdout, stderr begins with `error: signal:`, and the message contains `signal 'top.late' has no value at or before requested time`.
- `fsdb_value_rejects_real_signal_encoding`: use `value_real.vcd`; query `top.temp` and assert exit code `1`, empty stdout, stderr begins with `error: signal:`, and the message contains `signal 'top.temp' has unsupported non-bit-vector encoding`.

Required bundled-example smoke:

- `fsdb_bundled_cpu_value_smoke_samples_discovered_bit_signal`: use `cpu.fsdb`, run `info` to get `time_end`, discover candidate signals whose public kind and positive width make them plausibly bit-vector-like, then try `value --waves <cpu.fsdb> --signals <path> --at <time_end> --json` for each candidate until one succeeds. Skip candidates that fail with unsupported non-bit-vector encoding or no-prior-value signal errors, and keep a bounded diagnostic list of skipped paths so a total failure is debuggable. When a candidate succeeds, assert empty stderr, command `value`, empty warnings, one signal row, the row path equals the selected path, and the value matches the existing Verilog literal shape `<width>'h<hex-or-x-or-z-digits>`. Do not select a candidate solely by public `signal.kind`; hidden FSDB value metadata is the real support boundary.

Update the existing unsupported command test in `tests/fsdb_cli.rs`. It currently expects `value`, `change`, and `property` to fail as unsupported. After this plan, it must cover only `change` and `property`, with a new name such as `fsdb_unsupported_change_and_property_fail_clearly`. Keep the existing clear messages for those commands until their own implementation slice changes them.

Run the feature-gated integration tests directly and through the Makefile:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --test fsdb_cli
    WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

`make prepare-fsdb-fixtures` may also create ignored generated FSDB files for older hand fixtures because the script is intentionally broad. Do not reference those artifacts from the new tests, do not commit them, and do not introduce new milestone-labelled fixture names while touching this area.

Acceptance for this milestone is that generated fixture `value` outputs match VCD payloads exactly, bundled FSDB sampling succeeds structurally without checked-in golden contents, and `change`/`property` still fail clearly on FSDB input.

### Full validation, review, and handoff

Run default validation after all source and test changes:

    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci

Run feature-enabled validation on a Verdi-equipped machine:

    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Also run focused commands if a failure needs isolation:

    cargo fmt -- --check
    cargo check
    cargo test -q fsdb_hierarchy
    cargo test -q fsdb_time
    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo clippy --features fsdb --all-targets -- -D warnings
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --test fsdb_cli

Request focused read-only review lanes before final handoff: native/FFI safety, Rust backend correctness, CLI tests/contracts, and documentation/plan completeness. Apply fixes in the main session only, rerun the relevant validation, and run one fresh independent control review over the consolidated diff. Update this plan’s `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` before committing.

Acceptance for final handoff is: default checks pass, FSDB checks pass where Verdi is available, review returns no substantive findings or all findings are resolved and rechecked, generated FSDB artifacts are not tracked, and `git status --short` shows only intentional source, fixture, test, documentation, and plan changes before commit.

## Concrete Steps

Work from the repository root:

    cd /workspaces/wavepeek-fsdb

Create or edit the following files:

- `tests/fixtures/hand/value_vectors.vcd`, `tests/fixtures/hand/value_delayed.vcd`, and `tests/fixtures/hand/value_real.vcd`: new text fixtures for generated FSDB value parity.
- `native/fsdb/wavepeek_fsdb_shim.h`: add value classification, sampled-bit output type, and signal load/unload/sample function declarations.
- `native/fsdb/wavepeek_fsdb_shim.cpp`: map Reader variable metadata into value classification, load selected signals, create/free traverse handles, seek at-or-before raw time, decode digital bits, unload signals, and keep native locking/output suppression intact.
- `src/waveform/fsdb_native.rs`: mirror the new C ABI records and functions, add safe wrappers, free owned native strings and sampled values, and pass value classification into `FsdbHierarchyBuilder`.
- `src/waveform/fsdb_hierarchy.rs`: add raw and indexed value metadata, lookup helpers, unit tests, and unchanged public scope/signal output behavior.
- `src/waveform/fsdb_backend.rs`: implement `sample_resolved_optional` using the hierarchy metadata and native wrappers; keep expression/change/property methods unsupported.
- `src/waveform/mod.rs`: allow `value` to proceed for FSDB while leaving `change` and `property` blocked.
- `tests/fsdb_cli.rs`: add FSDB value integration tests and update the unsupported-command test.
- `docs/exec-plans/active/2026-05-24-fsdb-value-command/PLAN.md`: keep this plan current while implementation and review proceed.

Do not edit `schema/wavepeek.json` unless validation proves a genuine existing schema bug. This work should fit the current schema.

## Validation and Acceptance

A human can verify the feature with these commands on a Verdi-equipped machine after implementation:

    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Then manually sample a generated fixture:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures
    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo run --features fsdb -- value --waves tests/fixtures/fsdb/value_vectors.fsdb --scope top --signals data,clk,data,nibble,status,asc --at 7ns --json

Expected JSON shape is one `value` command object with empty warnings and data equivalent to:

    {
      "time": "7ns",
      "signals": [
        {"path": "top.data", "value": "8'h0f"},
        {"path": "top.clk", "value": "1'h1"},
        {"path": "top.data", "value": "8'h0f"},
        {"path": "top.nibble", "value": "4'hx"},
        {"path": "top.status", "value": "4'h3"},
        {"path": "top.asc", "value": "4'h3"}
      ]
    }

The same binary must still reject an unsupported real signal:

    VERDI_HOME="$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo run --features fsdb -- value --waves tests/fixtures/fsdb/value_real.fsdb --scope top --signals temp --at 0ns

Expected stderr contains:

    error: signal: signal 'top.temp' has unsupported non-bit-vector encoding

The same binary must still reject `change` and `property` on FSDB until their own implementation work exists. Default builds must still reject FSDB input with the existing feature-required message and must not require Verdi for `make ci`.

Full acceptance requires all of these to pass where applicable:

    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci
    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

## Idempotence and Recovery

The new VCD fixtures are durable source files and can be overwritten safely if their contents are intentionally revised. Generated `.fsdb` files under `tests/fixtures/fsdb/` are ignored artifacts; delete them and rerun `WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures` if conversion fails or if fixture contents change. Temporary converter directories created by integration tests are owned by `tempfile` and should be cleaned automatically.

Native signal loading modifies Reader object state. The implemented native batch operation holds the native Reader mutex for the whole load/sample/unload sequence and uses guards so every successful load has a corresponding unload/reset even if one signal sample fails. If a test fails after a native error, rerun it in a fresh process; the CLI tests already launch child processes, which is the safest recovery boundary for proprietary in-process libraries.

If `cargo check --features fsdb` fails because `VERDI_HOME` is missing or incomplete, do not patch around the build script. Run `WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env` and fix the local environment, or limit validation to default no-Verdi gates and record that FSDB gates were not runnable.

If bit-order or ascending-range tests fail, do not “fix” expected values. Compare VCD and generated FSDB outputs for the same fixture, inspect only project-owned code and allowed local SDK examples, then correct the decoder orientation so FSDB matches Wellen. The fixture is the ruler; the decoder is the suspect.

## Artifacts and Notes

Record concise validation snippets here as implementation proceeds. Useful snippets include the baseline commit, a successful generated fixture value JSON excerpt, the real-signal unsupported error, and final `make test-fsdb` / `make ci` summaries. Do not paste Verdi header excerpts, proprietary documentation text, or full outputs from bundled FSDB designs.

Current implementation notes:

    Branch before implementation: feat/fsdb
    Current head observed before implementation: 9d13d31
    Baseline default gate: WAVEPEEK_IN_CONTAINER=1 make check passed
    Baseline CI gate: WAVEPEEK_IN_CONTAINER=1 make ci passed with src coverage regions=94.69%, functions=95.31%, lines=95.19%
    Baseline FSDB environment: WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env reported Verdi FSDB Reader SDK found
    Baseline FSDB gate: WAVEPEEK_IN_CONTAINER=1 make test-fsdb passed 8 fsdb_cli tests plus native smokes
    Existing FSDB value state: explicit unsupported error before metadata parsing
    Expected new active plan path: docs/exec-plans/active/2026-05-24-fsdb-value-command/PLAN.md

    VCD fixture probe at 7ns for value_vectors.vcd produced duplicate top.data rows as 8'h0f, top.clk as 1'h1, top.nibble as 4'hx, top.status as 4'h3, and top.asc as 4'h3.
    VCD fixture probe for value_delayed.vcd at 0ns failed with: error: signal: signal 'top.late' has no value at or before requested time
    VCD fixture probe for value_real.vcd at 0ns failed with: error: signal: signal 'top.temp' has unsupported non-bit-vector encoding

    Generated FSDB probe at 7ns for value_vectors.fsdb produced the same JSON value payload as the VCD fixture, including duplicate top.data rows and asc/status as 4'h3.
    Generated FSDB delayed probe at 0ns failed with: error: signal: signal 'top.late' has no value at or before requested time
    Generated FSDB real probe at 0ns failed with: error: signal: signal 'top.temp' has unsupported non-bit-vector encoding
    Bundled cpu.fsdb probe succeeded at 0ns for system.i_cpu clock/addr/data with values 1'h0, 8'hxx, and 8'hxx.
    Targeted validation passed: cargo fmt -- --check; cargo check --features fsdb; cargo test -q fsdb_hierarchy --features fsdb; cargo test --features fsdb -q --test fsdb_cli -- --nocapture; WAVEPEEK_IN_CONTAINER=1 make test-fsdb.

## Interfaces and Dependencies

At the end of this work, the following internal interfaces should exist or their equivalents should be clearly documented in this plan:

In `native/fsdb/wavepeek_fsdb_shim.h`, wavepeek-owned C ABI additions are now:

    typedef enum wp_fsdb_value_encoding {
        WP_FSDB_VALUE_ENCODING_BIT_VECTOR = 0,
        WP_FSDB_VALUE_ENCODING_UNSUPPORTED = 1
    } wp_fsdb_value_encoding;

    typedef struct wp_fsdb_sample_record {
        uint64_t idcode;
        int has_value;
        uint32_t bit_width;
        uint64_t value_time_raw;
        char *bits;
    } wp_fsdb_sample_record;

    wp_fsdb_status wp_fsdb_sample_signal_values(wp_fsdb_reader *reader, const uint64_t *idcodes, size_t count, uint64_t query_time_raw, wp_fsdb_sample_record **out, char **error_message);
    void wp_fsdb_free_samples(wp_fsdb_sample_record *samples, size_t count);

The exact enum discriminants are project-owned. They are not copied from proprietary FSDB headers.

In `src/waveform/fsdb_native.rs`, the backend-facing interface is transaction-scoped in one method:

    pub(super) struct FsdbNativeSample {
        pub(super) idcode: u64,
        pub(super) bit_width: u32,
        pub(super) value_time_raw: Option<u64>,
        pub(super) bits: Option<String>,
    }

    impl FsdbReader {
        pub(super) fn sample_signal_values(&self, idcodes: &[u64], query_time_raw: u64) -> Result<Vec<FsdbNativeSample>, WavepeekError>;
    }

The Rust wrapper owns the returned native array until it has converted each row into Rust data, then calls `wp_fsdb_free_samples` in `Drop`.

In `src/waveform/fsdb_hierarchy.rs`, the implemented durable metadata surface is intentionally smaller than the first sketch:

    pub(super) enum FsdbValueEncoding { BitVector, Unsupported }

    impl FsdbHierarchyIndex {
        pub(super) fn signal_value_encoding(&self, canonical_path: &str) -> Result<FsdbValueEncoding, WavepeekError>;
    }

The backend still relies on `ResolvedSignal.width` for public width and the native sample record for decoded width; declared range did not need a public Rust metadata type because the Reader returns bytes in renderable bit order for the covered digital fixtures.

In `src/waveform/fsdb_backend.rs`:

    impl FsdbBackend {
        pub(super) fn sample_resolved_optional(
            &mut self,
            resolved: &[ResolvedSignal],
            query_time_raw: u64,
        ) -> Result<Vec<SampledSignalState>, WavepeekError>;
    }

The implementation may choose borrowed return types instead of cloned `FsdbSignalValueInfo` if Rust lifetimes stay simple. Preserve the behavior, not the decorative exact shape. Decorative exact shapes are how plans become cages.

## Revision Notes

- 2026-05-24 / Grin: Initial active plan drafted from the FSDB architecture proposal, command-specific value research, and current hierarchy-command implementation. The plan resolves the implementation boundary around C++ bit decoding, Rust value metadata, generated VCD-derived FSDB tests, and keeping `change`/`property` unsupported.
- 2026-05-24 / Grin: Revised after focused review to clarify generated fixture side effects, exact fixture values, delayed-value anchoring, bundled smoke candidate iteration, native transaction locking, declared range direction, path-based metadata lookup, integer time-tag checks, and proprietary metadata boundaries. A follow-up control review reported no substantive findings.
