# FSDB Integration Architecture for wavepeek

Status: temporary architecture proposal for implementing FSDB read support through a locally installed Synopsys Verdi / FSDB Reader SDK.

> Temporary development collateral: `docs/tracker/wip/fsdb/` exists only for the current FSDB support development branch. After FSDB support is merged, this content should either be removed or folded into the canonical documentation structure.

FSDB is treated as an **optional native read backend**, not as a separate command, converter, or external helper process. The wavepeek user contract should remain unified: the same commands, flags, JSON schema, naming rules, time rules, ordering rules, and error categories apply to VCD, FST, and FSDB where the format can provide equivalent data.

## 1. Goals and constraints

### Goals

- The default wavepeek installation works as it does today: VCD/FST through `wellen`, with no Verdi dependency.
- FSDB support is enabled explicitly at build/install time on supported Linux targets, for example with `cargo install wavepeek --features fsdb`, or through a future installer option such as `--fsdb` that enables the Cargo feature internally.
- If a binary is built without FSDB support, opening `.fsdb` input returns a clear user-facing error.
- If a binary is built with FSDB support, `info`, `scope`, `signal`, `value`, `change`, and `property` work with FSDB through the same command surface for digital bit-vector/integral signals. The current implementation still rejects unsupported real and string value decoding when a command needs those values.
- Integration is native and in-process through the FSDB Reader API. wavepeek does not launch Verdi, VIA, Tcl, Python, Perl, or other helper utilities.
- Development and public CI work without Verdi and without checked-in FSDB artifacts.
- On machines where Verdi is available, FSDB tests use the small example `.fsdb` files bundled under `$VERDI_HOME` before considering any private artifact set.

### Non-goals for the first implementation

- Do not implement a custom parser for the FSDB on-disk format. FSDB is a closed binary format; the supported path is the official Reader API.
- Do not add FSDB-specific CLI flags to the user-facing command surface.
- Do not support every FSDB data class in the first implementation. Analog/SPICE, transaction streams, power, coverage, MDA, and proprietary/internal service records are out of scope initially. The first target is digital Verilog/SystemVerilog parity for the existing commands.
- Do not support FSDB on non-Linux targets in the first implementation. The direct link + rpath design is Linux-only; non-Linux `fsdb` builds should fail with a clear build-time error.
- Do not commit Verdi headers, libraries, documentation, FSDB files, or generated bindings containing proprietary header-derived content to the public repository.

## 2. User model

### 2.1 Default installation without FSDB

Default installation:

```sh
cargo install wavepeek
```

or a downloaded public release binary supports only VCD/FST.

These commands keep working as they do today:

```sh
wavepeek info --waves dump.vcd
wavepeek info --waves dump.fst
```

Attempting to open FSDB input:

```sh
wavepeek info --waves dump.fsdb
```

should return an error along these lines:

```text
error: file: FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME
```

The exact wording may change, but it must clearly state that:

1. the file appears to be FSDB;
2. the current binary was built without FSDB support;
3. the user must rebuild or reinstall with the `fsdb` feature and a local Verdi installation.

A binary without FSDB support cannot call the FSDB Reader API, so detection is best-effort: `.fsdb` / `.fsdb.gz` extension checks plus fallback after a failed `wellen` parse. When the feature is enabled, exact probing is done with `ffrObject::ffrIsFSDB`.

### 2.2 Installation with FSDB

Recommended behavior: **if the user explicitly enables the FSDB feature, the build fails fast when `$VERDI_HOME` is unset or does not contain the expected FSDB Reader SDK**.

Example:

```sh
VERDI_HOME=/opt/verdi cargo install wavepeek --features fsdb
```

Build-time checks must verify at least:

- `$VERDI_HOME/share/FsdbReader/ffrAPI.h`;
- `$VERDI_HOME/share/FsdbReader/ffrKit.h`;
- `$VERDI_HOME/share/FsdbReader/fsdbShr.h`;
- `$VERDI_HOME/share/FsdbReader/<abi>/libnffr.so`;
- `$VERDI_HOME/share/FsdbReader/<abi>/libnsys.so`, if the selected Reader API distribution requires it at link time or runtime.

Here `<abi>` is not the FSDB file ABI. It is the directory containing a particular binary build of the FSDB Reader library inside Verdi. The inspected installation contains at least `linux64` and `linux64_gcc950`: both are Linux x86_64 builds, but they target different toolchain/C++ ABI variants. Because the wavepeek C++ shim is compiled with the local `g++` and linked with the prebuilt `libnffr.so`, the selected directory must be compatible with the local C++ toolchain.

Recommended selection policy:

1. If `WAVEPEEK_FSDB_READER_LIBDIR` is set, use that directory as an explicit override.
2. If `WAVEPEEK_FSDB_ABI` is set, use `$VERDI_HOME/share/FsdbReader/$WAVEPEEK_FSDB_ABI`.
3. If neither is set, use canonical `$VERDI_HOME/share/FsdbReader/linux64`.
4. If `linux64` is missing or the compile/link smoke test fails, the build fails with a clear error suggesting `WAVEPEEK_FSDB_READER_LIBDIR` or `WAVEPEEK_FSDB_ABI=linux64_gcc950`.

The default is the simple and predictable `linux64` path. Other directories are opt-in troubleshooting controls for non-standard Verdi installations or ABI mismatch errors.

Build-time failure is preferred over installing a non-functional binary because:

- the user explicitly requested FSDB support, so a missing SDK is an installation error;
- Cargo/build logs can identify the missing file;
- the default public build remains portable and dependency-free;
- it avoids producing a binary that appears to support FSDB but cannot open FSDB input.

The build script must set rpath/RUNPATH to the selected Reader library directory. If Verdi is moved or removed after installation, or if the dynamic loader cannot load the `.so`, an FSDB-enabled binary may fail before entering `main`; this is an accepted trade-off of direct linking.

### 2.3 Unified command surface

An FSDB-enabled binary must not require users to learn new commands. The current implementation supports `info`, `scope`, `signal`, `value`, `change`, and `property` from this surface for digital bit-vector/integral signals:

```sh
wavepeek info     --waves dump.fsdb --json
wavepeek scope    --waves dump.fsdb --max-depth 3
wavepeek signal   --waves dump.fsdb --scope top.u0 --recursive
wavepeek value    --waves dump.fsdb --scope top.u0 --signals clk,data --at 10ns
wavepeek change   --waves dump.fsdb --scope top.u0 --signals state --on 'posedge clk'
wavepeek property --waves dump.fsdb --scope top.u0 --on 'posedge clk' --eval 'valid && ready'
```

Existing rules remain in effect:

- canonical paths are dot-separated, for example `top.u0.signal`, even if FSDB internally uses `/`;
- `--scope` and relative signal resolution behave the same as for VCD/FST;
- `--json` output remains within the existing schema;
- time tokens require an integer and a unit;
- output ordering is deterministic;
- unsupported FSDB content maps to existing error categories such as `file`, `scope`, `signal`, and `expr`, not to a new FSDB-specific error envelope.

## 3. Build architecture

### 3.1 Cargo feature

Add a feature:

```toml
[features]
default = []
fsdb = []
```

In practice, the feature adds build-time C++ compilation and a dependency on `cc`. `bindgen` is not recommended: generated output derived from proprietary headers must not be committed, and generating it during every user installation would add avoidable fragility. The default feature set must not require Verdi, C++ headers, or proprietary libraries.

Public CI and default quality gates must not use `--all-features` once `fsdb` exists, because `fsdb` intentionally fails to build without a valid `$VERDI_HOME`. Repository automation should lint/check/test the default feature set by default and reserve `--features fsdb` for explicit FSDB targets after environment checks.

### 3.2 C++ shim instead of direct Rust FFI to a C++ API

The FSDB Reader API is a C++ API. Rust should not link directly against C++ class ABI. A small C ABI shim should be compiled locally when `--features fsdb` is enabled.

Proposed layout:

```text
src/
  waveform/
    mod.rs              # backend-neutral facade
    types.rs            # WaveformMetadata, ScopeEntry, SignalEntry, ids, sampled values
    wellen_backend.rs   # current VCD/FST implementation
    fsdb_backend.rs     # Rust-side FSDB backend, cfg(feature = "fsdb")
    fsdb_stub.rs        # clear error path without feature
native/
  fsdb/
    wavepeek_fsdb_shim.h
    wavepeek_fsdb_shim.cpp
build.rs
```

The shim source is wavepeek-owned code and may be stored in the public repository. It includes local headers from `$VERDI_HOME`, but does not copy their contents:

```cpp
#include "ffrAPI.h"
```

The C ABI shim should export only wavepeek-owned types and functions. Conceptually:

```c
wp_fsdb_status wp_fsdb_probe(const char *path, wp_fsdb_error *err);
wp_fsdb_status wp_fsdb_open(const char *path, wp_fsdb_reader **out, wp_fsdb_error *err);
void           wp_fsdb_close(wp_fsdb_reader *reader);

wp_fsdb_status wp_fsdb_metadata(wp_fsdb_reader *reader, wp_fsdb_metadata *out, wp_fsdb_error *err);
wp_fsdb_status wp_fsdb_read_scopes(wp_fsdb_reader *reader, scope_callback cb, void *user, wp_fsdb_error *err);
wp_fsdb_status wp_fsdb_read_signals(wp_fsdb_reader *reader, signal_callback cb, datatype_callback dt_cb, void *user, wp_fsdb_error *err);

wp_fsdb_status wp_fsdb_load_signals(wp_fsdb_reader *reader, const wp_fsdb_signal_id *ids, size_t count, wp_fsdb_error *err);
wp_fsdb_status wp_fsdb_unload_signals(wp_fsdb_reader *reader);
wp_fsdb_status wp_fsdb_sample(wp_fsdb_reader *reader, wp_fsdb_signal_id id, uint64_t raw_time, wp_fsdb_value *out, wp_fsdb_error *err);
wp_fsdb_status wp_fsdb_collect_candidate_times(wp_fsdb_reader *reader, const wp_fsdb_signal_id *ids, size_t count, uint64_t from, uint64_t to, time_callback cb, void *user, wp_fsdb_error *err);
```

This is an internal shape, not a public API. Important properties:

- C++ exceptions do not cross the FFI boundary;
- all errors become status codes plus owned error strings;
- strings from FSDB callbacks are copied on the Rust side during the callback;
- value buffers are copied or decoded before the traverse handle releases ownership;
- the shim suppresses FSDB Reader info/warning messages (`ffrInfoSuppress`, `ffrWarnSuppress`) so wavepeek stdout/stderr contracts remain stable;
- Rust code does not depend on FSDB enum numeric values.

### 3.3 Direct link + rpath

FSDB integration uses direct feature-gated linking with FSDB Reader libraries:

- `build.rs` compiles the C++ shim through `cc` when `CARGO_FEATURE_FSDB` is set;
- it adds include path `$VERDI_HOME/share/FsdbReader`;
- it adds link search path `$VERDI_HOME/share/FsdbReader/<abi>`;
- it links `nffr` and required companion libraries;
- it adds rpath/RUNPATH for the selected library directory, typically `$VERDI_HOME/share/FsdbReader/linux64`.

The resulting `wavepeek` binary contains ELF dependencies (`DT_NEEDED`) on `libnffr.so` plus any required companion libraries. This is intentional: a binary built with FSDB support requires an available Linux Verdi runtime.

## 4. Source refactor

### 4.1 Current issue

Currently `src/waveform/mod.rs` simultaneously:

- opens files through `wellen`;
- stores `wellen::simple::Waveform`;
- returns engine-layer types containing `wellen::SignalRef`;
- contains VCD/FST-specific optimization hooks: `time_table`, offsets, FST streaming candidate collection;
- implements the expression host bridge through `WaveformExprHost`.

This is acceptable for one backend, but FSDB requires a backend boundary to avoid spreading `#[cfg(feature = "fsdb")]` and format-specific dispatch throughout the engine.

### 4.2 Target shape: facade + backend-neutral IDs

Keep the engine-facing `Waveform` type, but make it a facade over backends:

```rust
pub struct Waveform {
    backend: Backend,
}

enum Backend {
    Wellen(WellenBackend),
    #[cfg(feature = "fsdb")]
    Fsdb(FsdbBackend),
}
```

A `Box<dyn WaveformBackend>` is also possible if trait-object constraints do not interfere with performance-critical paths. An enum is likely simpler because it avoids object-safety constraints and keeps Wellen-specific fast paths explicit.

Common types must stop containing `wellen::SignalRef`:

```rust
pub struct SignalId(u64);

pub struct ResolvedSignal {
    pub path: String,
    pub id: SignalId,
    pub width: u32,
}

pub(crate) struct ExprResolvedSignal {
    pub path: String,
    pub id: SignalId,
    pub expr_type: ExprType,
}
```

Each backend owns the mapping from `SignalId` to its native handle:

- WellenBackend: `SignalId -> wellen::SignalRef`;
- FsdbBackend: `SignalId -> fsdbVarIdcode` plus metadata from hierarchy traversal.

The engine must not know what `wellen::SignalRef` or `fsdbVarIdcode` are.

### 4.3 Backend interface

Minimum backend-neutral interface required by the existing commands:

```rust
trait WaveformBackend {
    fn metadata(&self) -> Result<WaveformMetadata, WavepeekError>;

    fn scopes_depth_first(&self, max_depth: Option<usize>) -> Result<Vec<ScopeEntry>, WavepeekError>;
    fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError>;
    fn signals_in_scope_recursive(&self, scope_path: &str, max_depth: Option<usize>) -> Result<Vec<SignalEntry>, WavepeekError>;

    fn resolve_signals(&self, canonical_paths: &[String]) -> Result<Vec<ResolvedSignal>, WavepeekError>;
    fn resolve_expr_signal(&self, canonical_path: &str) -> Result<ExprResolvedSignal, WavepeekError>;
    fn resolve_expr_signals(&self, canonical_paths: &[String]) -> Result<Vec<ExprResolvedSignal>, WavepeekError>;

    fn sample_resolved_optional(&mut self, resolved: &[ResolvedSignal], raw_time: u64) -> Result<Vec<SampledSignalState>, WavepeekError>;
    fn sample_expr_value(&mut self, resolved: &ExprResolvedSignal, raw_time: u64) -> Result<SampledValue, WavepeekError>;
    fn expr_event_occurred(&mut self, resolved: &ExprResolvedSignal, raw_time: u64) -> Result<bool, WavepeekError>;

    fn collect_change_times(&mut self, resolved: &[ResolvedSignal], from: u64, to: u64, mode: ChangeCandidateCollectionMode) -> Result<Vec<u64>, WavepeekError>;
    fn collect_expr_candidate_times(&mut self, resolved: &[ExprResolvedSignal], from: u64, to: u64, mode: ChangeCandidateCollectionMode) -> Result<Vec<u64>, WavepeekError>;

    fn previous_sample_time(&self, raw_time: u64) -> Option<u64>;
}
```

`previous_sample_time` supports edge semantics in `change`/`property`: the previous value must be strictly before the current timestamp. For integer raw timestamps, FSDB can return `raw_time - 1` when `raw_time > 0`; Wellen can retain its current global time table optimization. The engine must not require FSDB to provide a complete global time table.

### 4.4 Wellen-specific fast paths

Current `change` optimizations use:

- global `timestamps_raw_slice()`;
- `signal_offset_at_index()`;
- `decode_signal_at_index()`;
- FST streaming API.

These methods should not be forced onto FSDB. They should become an optional WellenBackend capability:

```rust
enum BackendCapabilities<'a> {
    IndexedWellen(&'a WellenBackend),
    Portable,
}
```

or a set of facade methods that return `None` for FSDB.

Dispatch policy:

- VCD/FST keep the existing baseline/fused/edge-fast engines and FST streaming candidate collection.
- FSDB initially uses a portable engine path: candidate times through per-signal FSDB value-change traversal, then sorted unique timestamp merging in wavepeek-owned code, and sampling through FSDB traverse handles. A merged FSDB time-based traversal can be added later as an optimization after profiling.
- An FSDB-specific fast engine can be added later if profiling shows it is needed.

The Wellen refactor behind the backend boundary must be behavior-preserving. First move the existing code and replace IDs, then add the FSDB backend.

### 4.5 File opening and format detection

`Waveform::open(path)` should:

1. Check that the file exists and is readable, as today.
2. Try to identify/read VCD/FST through the `wellen` detection/read path.
3. If VCD/FST is read successfully, return WellenBackend.
4. If VCD/FST is not read:
   - when the `fsdb` feature is enabled, call FSDB probe (`ffrIsFSDB`) and open FsdbBackend;
   - when the `fsdb` feature is disabled and the path looks like FSDB (`.fsdb`, `.fsdb.gz`), return a clear FSDB-disabled error;
   - otherwise return the normal parse/file error.

When the `fsdb` feature is enabled, `.fsdb` paths should preferably be probed as FSDB first to avoid unnecessary `wellen` parse attempts. `.vcd`/`.fst` paths should use the current path.

## 5. FSDB backend: data and commands

### 5.1 Metadata/time model

FSDB Reader provides scale unit and min/max tags. Wavepeek stores:

- `time_unit`: dump resolution string, for example `1ns`, `100ps`;
- `time_start`: normalized `raw_start * scale` string, for example `0ps`;
- `time_end`: normalized `raw_end * scale` string.

FSDB digital time tags should be supported as integer raw ticks:

```text
raw = (H << 32) | L
```

Float/double xtags are unsupported for `value`, `change`, and `property` in the first implementation. `info` should also avoid guessing. The error should be file-level: waveform uses unsupported FSDB time tag representation.

Scale unit normalization must understand at least:

- `1ns`, `10ps`, `100ps`;
- short suffix forms such as `1n`, `1p`, if the Reader returns them;
- fractional forms such as `0.1n`, `0.01n`, `0.001n`, converted to integer wavepeek units: `100ps`, `10ps`, `1ps`.

After normalization, existing `engine::time` logic is used. Do not silently round: if a user time token cannot be represented exactly in dump precision, the command fails.

### 5.2 Hierarchy index

FSDB callbacks arrive as traversal events. The backend should build an in-memory index once per command:

```text
ScopeIndex:
  canonical_scope_path -> { kind, depth, children }

SignalIndex:
  canonical_signal_path -> {
    id: SignalId,
    fsdb_idcode,
    parent_scope,
    display_name,
    kind,
    width,
    base_var_type,
    bytes_per_bit,
    vc_dt,
    dtidcode,
    expr_type,
  }
```

Canonical paths are always dot-separated. `ffrGetScopeSeparator()` must not leak into public output.

Hidden/internal scopes should be excluded with their subtrees when the Reader explicitly marks them as hidden. Multiple FSDB trees and repeated records should be deduplicated by canonical path. Sorting after index construction must match current behavior:

- scopes: pre-order DFS, children sorted lexicographically by local name/path;
- signals: sorted by `(name, path)` inside each visited scope.

### 5.3 `info`

`info` only needs file metadata:

- verify FSDB;
- read `scale_unit`, `min_xtag`, and `max_xtag`;
- normalize into `WaveformMetadata`.

Full hierarchy traversal is not required.

### 5.4 `scope`

For `scope`, prefer reading only the scope tree if the FSDB file supports it. If a separate scope tree is unavailable, fall back to the scope+var tree and ignore var records.

Return only existing fields:

```json
{"path": "top.u0", "depth": 1, "kind": "module"}
```

FSDB scope kinds map into the existing wavepeek inventory: `module`, `task`, `function`, `begin`, `fork`, `generate`, `interface`, `package`, `program`, `class`, `struct`, `union`, `unknown`. Do not add new kind strings without a schema migration.

### 5.5 `signal`

`signal` needs scope+var traversal without loading value changes.

Name normalization:

- if an FSDB var name contains a trailing packed range matching `lbitnum/rbitnum`, for example `data[7:0]`, public `name` is `data` and `width = 8`;
- array indices that are not packed ranges are preserved: `mem[3][7:0]` -> `mem[3]`, width 8;
- path is built from the normalized name.

Kind mapping should use existing stable aliases: `wire`, `reg`, `logic`, `bit`, `integer`, `time`, `real`, `string`, `event`, `enum`, and so on. If `dtidcode` indicates an enum or SystemVerilog datatype, datatype metadata has priority over base var type. Unsupported/internal FSDB var kinds must not create new public kind strings.

### 5.6 `value`

Algorithm:

1. Open FSDB and metadata.
2. Build the signal index.
3. Resolve `--scope`/`--signals` into canonical paths while preserving order and duplicates.
4. Load the unique set of FSDB idcodes through the Reader signal list/load API.
5. For each unique signal, create a traversal handle and move to value at-or-before `--at`.
6. Decode value immediately while the handle owns the buffer.
7. Project results back to the original order/duplicates.

The first supported value encoding is Verilog-like digital bit vectors with one byte per bit and states `0/1/x/z`. Literal formatting should use the existing `format_verilog_literal(width, bits)`.

Unsupported for `value` in the first version:

- real/string;
- analog/power/transaction;
- unknown bytes-per-bit encodings;
- dummy/internal variables;
- signals without meaningful width.

Error shape should match existing categories:

```text
error: signal: signal 'top.msg' has unsupported non-bit-vector encoding
```

### 5.7 `change`

Portable FSDB implementation:

1. Build metadata and signal/expression index.
2. Resolve requested output signals.
3. Parse `--on` through the existing expression engine.
4. Collect candidate source idcodes:
   - requested output signals for wildcard `*`;
   - explicit event/value operands from `--on`;
   - operands from `iff` expressions when they affect evaluation.
5. Use per-signal FSDB value-change traversal for candidate timestamps, then sort and deduplicate the raw timestamps in wavepeek-owned code. A merged FSDB time-based traversal is optional future optimization, not required behavior.
6. Public output remains per timestamp, not per value-change record.
7. At each candidate timestamp:
   - sample requested signals at-or-before `t`;
   - check requested output deltas against baseline/previous state;
   - evaluate `--on`;
   - if matched, output the snapshot in requested order.

Same-time records and glitches:

- if the Reader provides sequence numbers, use them for deterministic final value within a timestamp;
- if sequence numbers are unavailable, confirm final sampled value with normal per-signal `goto time` sampling;
- glitches are not exposed as a separate public feature; they affect only the final sampled value or event presence at the timestamp.

`--tune-candidates stream` for FSDB must not imply FST streaming. Options:

- treat it as `Auto` for FSDB and warn only if this tune flag is considered internal/debug;
- or return an internal/args error if a forced mode is not applicable.

Do not expand the user contract around tune flags. If they are hidden/debug controls, behavior may be backend-specific.

### 5.8 `property`

`property` uses the same FSDB expression host as `change`:

- event candidates from `--on`;
- eval operands from `--eval`;
- at-or-before sampling for values;
- exact occurrence for raw events;
- previous sample strictly before `t` for edge event terms.

Capture modes (`match`, `switch`, `assert`, `deassert`) remain engine-level state over the selected event stream. They must not be implemented by comparing only against the immediately previous waveform sample.

Datatype support for the first useful version:

- bit vectors / logic / reg / wire;
- integer-like SV/VCD types;
- time;
- enum labels, if datatype callbacks provide literal-to-bits mapping;
- real/string only after fixture coverage proves behavior.

`--capture match/switch/assert/deassert` remains engine-level logic. The FSDB backend only supplies candidate times and sampled values.

## 6. Errors and diagnostics

New errors should map to existing categories.

Examples:

```text
error: file: FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME
error: file: FSDB Reader SDK not found: VERDI_HOME is unset or incomplete
error: file: unsupported FSDB time tag representation: floating-point xtags are not supported
error: signal: signal 'top.a' has unsupported non-bit-vector encoding
error: scope: scope 'top.missing' not found in dump
```

Do not add `error: fsdb:` without a separate public error taxonomy decision.

FSDB Reader may write warnings/info directly. Suppress or intercept those messages. stdout belongs to the wavepeek output renderer; stderr is for wavepeek diagnostics/warnings.

## 7. Test and development infrastructure

### 7.1 Optional Verdi mount

The devcontainer may mount host Verdi at `/opt/verdi` and set `VERDI_HOME=/opt/verdi`. Treat Verdi as available only if the expected SDK files/libs exist, not merely because the environment variable is set. An empty mount directory means “Verdi unavailable”.

Use `tools/fsdb/check_fsdb_env.py` for local Verdi/FSDB SDK validation.

Checks:

- `$VERDI_HOME` is set;
- path exists;
- Reader headers exist;
- selected Reader libdir exists;
- `libnffr.so` is loadable enough for a build/link smoke test.

### 7.2 FSDB test files from `$VERDI_HOME`

Do not store FSDB fixtures in this repository. Checked public repositories with FSDB Reader integration — GTKWave/EGTKWave/Vaporview — store integration code but not `.fsdb` test fixtures. wavepeek should not download `.fsdb` files from third-party public repositories: provenance, stability, and artifact licensing would remain unclear. Tests must not perform network downloads.

For the current FSDB development branch, use the example `.fsdb` files bundled inside the local Verdi installation. The inspected `$VERDI_HOME` contains small examples under `$VERDI_HOME/share/VIA/demo/waveform`, many focused NPI examples under `$VERDI_HOME/share/NPI/example/via_examples`, and larger performance examples under `$VERDI_HOME/share/verdi_perf/perfExamples`. This keeps tests local to the same licensed Verdi installation that provides the Reader SDK and avoids creating a separate private artifact-mount contract before we need one. Splendid, one less mystery directory to appease.

The first smoke fixture is `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`: it is small, present in the inspected installation, and already proved usable by `just check-fsdb-build`. Tests may refer to this concrete path relative to `VERDI_HOME` directly; do not add another fixture-discovery layer until the bundled files stop covering a real need. Use other bundled Verdi examples only when a test needs a capability that `cpu.fsdb` does not cover, such as NPI model traversal or larger performance-style files. Do not read `.fsdb` files as text; tests must access them only through the FSDB Reader API or binary-safe file metadata operations such as `find`, `stat`, `ls`, `du`, and checksums.

Private/local artifact mounts are deferred. If bundled Verdi examples cannot cover a later command-level requirement, re-open this decision and define a separate artifact policy at that time. Until then, avoid adding `WAVEPEEK_FSDB_ARTIFACTS_DIR`, `FSDB_RTL_ARTIFACTS_DIR`, `/opt/fsdb-rtl-artifacts`, or similar paths to the architecture as required test inputs.

### 7.3 Test matrix

Required matrix:

Before adding the `fsdb` feature, update public quality gates so they do not build Verdi-gated features in no-Verdi environments. FSDB coverage belongs behind explicit targets such as `just test-fsdb`, not the default public `just ci` path.

| Environment | Command | Expected result |
|---|---|---|
| no Verdi, default features | `just ci` | full existing VCD/FST suite passes |
| no Verdi, default features | FSDB-disabled unit/integration test | verifies clear error on `.fsdb` path without a real FSDB fixture |
| no Verdi, `just check-fsdb-env` | availability probe exits 0 with skip message | developers can inspect availability without failing default automation |
| no Verdi, `just test-fsdb` | explicit FSDB target fails through `require-verdi` before Cargo builds | accidental local invocation gets a clear SDK requirement; public CI does not call this recipe |
| Verdi | `just test-fsdb` | build/link smoke + bundled-example FSDB tests run |
| public GitHub CI | `just ci` | Verdi/FSDB tests do not require proprietary payload; public CI remains green |

Rust `cargo test` does not have a native skip primitive, so integration tests should return early:

```rust
let Some(env) = fsdb_test_env() else {
    eprintln!("skipping FSDB integration test: VERDI_HOME or bundled Verdi FSDB example unavailable");
    return;
};
```

However, building with `--features fsdb` without Verdi must fail. Therefore, explicit FSDB `just` recipes must depend on a `require-verdi` gate before running Cargo commands such as:

```sh
cargo test --features fsdb --test fsdb_cli
```

Otherwise Cargo fails later and usually less politely. Let the `justfile` tripwire be the tripwire; do not make every test rediscover the same missing SDK.

### 7.4 What to test

Without Verdi:

- scale-unit parser/normalizer;
- FSDB-disabled error text;
- path normalization helpers;
- kind mapping tables, if written without proprietary numeric constants;
- backend-neutral facade using a mock backend;
- expression host behavior on a synthetic host.

With Verdi + bundled examples:

- `info`: `time_unit`, `time_start`, `time_end`;
- `scope`: depth/kind/order/filter/max-depth;
- `signal`: direct/recursive, packed range stripping, width, kind, dedup;
- `value`: exact timestamp, at-or-before, no previous value, duplicates/order;
- `change`: wildcard, named signal, posedge/negedge/edge, same-time records, `--from/--to`, truncation;
- `property`: match/switch/assert/deassert, enum/integer/real/string only where fixture-backed;
- error cases: unsupported xtag, unsupported value encoding, missing scope/signal.

The existing VCD/FST suite must remain stable. The first refactor commit should be behavior-preserving and pass `just ci` before the FSDB backend is added.

## 8. Public repository and licensing policy

For source integration, this plan accepts the GTKWave precedent: a public repository may store project-owned code that calls the FSDB Reader API when Synopsys headers/libs/docs are not vendored and the user supplies a local licensed installation.

A search of public repositories shows existing optional FSDB Reader integrations:

- GTKWave 3.x / forks: `fsdb_wrapper_api.cc` contains a GPL C++ wrapper around `ffrAPI.h`, `ffrObject::ffrOpen3`, `ffrReadScopeVarTree`, `ffrCreateVCTraverseHandle`, `ffrGetScaleUnit`, etc.; `configure.ac` searches for `FSDBREADER_HDRS` and `FSDBREADER_LIBS` and does not ship Synopsys headers/libs.
- GTKWave contrib: `contrib/fsdb2vcd/fsdb2vcd_fast.cc` publicly stores standalone converter source that `#include`s `ffrAPI.h` and is built by the user against local `/pub/FsdbReader/libnffr.*` / `libnsys.*`.
- Vaporview (`Lramseyer/vaporview`): a VS Code waveform viewer with native Node addon `src/fsdb_reader.cpp`; build configuration asks for paths to `libnffr.so`, `libnsys.so`, `ffrAPI.h`, `fsdbShr.h`; docs describe the FSDB addon as optional and requiring local Verdi/Xcelium reader libraries.
- `nayiri-k/fsdb-parse`: research C++ tools over Verdi FsdbReader, built against `$VERDI_HOME/share/FsdbReader`. Do not copy vendor sample code or code that carries proprietary copyright notices.

For wavepeek, use the GTKWave/Vaporview model: store **project-owned** source code that calls FSDB Reader API through local headers/libs. The practical safe boundary is: do not vendor headers/libs/docs, do not copy vendor sample code, do not generate or commit bindings from proprietary headers, and require the user to provide a local licensed installation.

### Acceptable in the public repository

- Project-owned Rust/C++ source code that optionally integrates with the local SDK.
- Build scripts that check for `$VERDI_HOME`.
- A C++ shim that `#include`s headers from the local installation without copying their contents.
- References to publicly observable API names/function names required to explain the integration.
- Tests that skip without local Verdi or the required bundled Verdi example file.
- Documentation stating that a licensed Verdi installation is required and that wavepeek does not ship Synopsys libraries.

### Not acceptable in the public repository

- `$VERDI_HOME` headers: `ffrAPI.h`, `ffrKit.h`, `fsdbShr.h`, or others.
- `libnffr.so`, `libnsys.so`, or Verdi runtime libraries.
- PDF/HTML/manual excerpts from Verdi documentation.
- Generated `bindgen` output from proprietary headers.
- `.fsdb` fixtures without explicit redistribution rights.
- Golden outputs that reveal proprietary design contents from Verdi-bundled or private FSDB files.

### Release policy

- Public prebuilt binaries remain default VCD/FST-only.
- FSDB-enabled binaries are not published publicly by default. Source integration is sufficient; binary distribution with proprietary runtime dependencies is a separate release decision.
- Crates.io/source release may include an optional `fsdb` feature if the source does not include proprietary payload. The user supplies licensed `$VERDI_HOME` when building.
- README/docs must state that wavepeek does not ship Synopsys libraries and that FSDB support requires a local licensed Verdi/FSDB Reader SDK installation.

## 9. Incremental implementation plan

### M0: build spike

- Verify Linux-only build gating and default `linux64` with the devcontainer compiler, plus explicit override through `WAVEPEEK_FSDB_READER_LIBDIR` / `WAVEPEEK_FSDB_ABI` for alternatives such as `linux64_gcc950`.
- Build a minimal C++ shim that opens FSDB and reads metadata.
- Verify that builds fail cleanly without `$VERDI_HOME` only when `--features fsdb` is enabled.
- Update `just ci`, `just check`, and related automation so public/no-Verdi gates do not use `--all-features` after the Verdi-gated feature exists.

### M1: backend refactor without FSDB

- Move the current `src/waveform/mod.rs` implementation into WellenBackend.
- Introduce backend-neutral `SignalId`, `ResolvedSignal`, and `ExprResolvedSignal`.
- Preserve facade `Waveform` and existing engine call sites as much as possible.
- Run `just ci`; output snapshots/schema should not change.

### M2: FSDB-disabled UX

- Add `.fsdb` path detection for the default binary.
- Add tests for the clear error without a real FSDB fixture.
- Update public docs after the optional FSDB feature is accepted.

### M3: FSDB `info` / `scope` / `signal`

- Connect C++ shim metadata and hierarchy callbacks.
- Build ScopeIndex/SignalIndex.
- Implement kind mapping, packed range normalization, deduplication, and sorting.
- Add optional integration tests.

### M4: FSDB `value`

- Implement signal loading, at-or-before sampling, and digital value decoding.
- Verify bit order on synthetic fixtures.
- Add unsupported encoding errors.

### M5: FSDB `change` / `property` portable path

Note: this milestone intentionally ships the portable correctness path. Persistent native sampler/session reuse and large-window FSDB performance tuning remain M6 work, because that needs profiling and a boring ownership design rather than another optimistic cache with a moustache.

- Implement candidate collection through per-signal FSDB value-change traversal, returning sorted unique raw timestamps to Rust.
- Connect expression host to FSDB sampled digital values and exact raw event occurrences.
- Keep Wellen-specific optimized engines for VCD/FST; FSDB uses the portable path.
- Add tests for wildcard, edge, bounds, raw events, unsupported real operands, and property capture modes.

### M6: hardening/performance

- Measure large FSDB files on a local machine with Verdi.
- Optimize index caching within one command.
- Verify memory unload/free paths.
- Verify graceful errors for missing, truncated, and clearly non-FSDB files; treat corrupted FSDB behavior as a hardening risk of direct in-process integration, not an architecture blocker.
- Add richer datatypes only after fixture coverage exists.

## 10. Recommended decision

Recommended architecture:

1. Default wavepeek remains VCD/FST-only and does not depend on Verdi.
2. FSDB is enabled only by feature-gated `fsdb` builds.
3. With the `fsdb` feature enabled, the build requires a valid `$VERDI_HOME` and FSDB Reader SDK; without the SDK, installation fails immediately.
4. Integration uses a small C++ C-ABI shim compiled locally and linked with `libnffr.so`; proprietary files are not included in the repository.
5. The engine receives a backend-neutral `Waveform` facade; existing Wellen code becomes one backend, FSDB becomes another.
6. Commands and JSON schema are not extended for FSDB; format differences are hidden in the waveform backend and expression host.
7. Tests and automation support two modes: without Verdi, FSDB integration tests are skipped; with Verdi and its bundled example FSDB files, the FSDB suite runs as far as those examples provide coverage.
8. Public releases remain without FSDB-enabled binaries unless a separate binary release decision is made.
