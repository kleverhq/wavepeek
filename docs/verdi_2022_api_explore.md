# Verdi 2022 FSDB Reader API Exploration for wavepeek

## Scope

This note evaluates whether the current `wavepeek` command set can be backed by
the Verdi `T-2022.06` FSDB Reader API.

Primary local sources inspected:

- `docs/fsdb_proposal.md`
- `docs/DESIGN.md`
- `docs/expression_lang.md`
- `src/cli/*.rs`
- `src/engine/*.rs`
- `src/waveform/mod.rs`
- `fsdb_research/verdi/T-2022.06/share/FsdbReader/ffrAPI.h`
- `fsdb_research/verdi/T-2022.06/share/FsdbReader/fsdbShr.h`
- `fsdb_research/verdi/T-2022.06/share/FsdbReader/example/*.cpp`
- `fsdb_research/verdi/T-2022.06/doc/.FsdbReader.txt.gz`

The analysis targets `share/FsdbReader`, matching the RFC baseline. The
`FsdbReader_pure` tree appears to expose the same public reader headers and
examples in this install, but the RFC and feasibility baseline name the normal
`FsdbReader` layout with `libnffr.so` and `libnsys.so`.

## Executive Summary

All public `wavepeek` commands look implementable for ordinary digital RTL FSDB
dumps with the Verdi 2022 reader API. There is no obvious command-level blocker
in the API.

The strongest fit is for `info`, `scope`, `signal`, `value`, and digital
`change`. `property` is also supportable in shape, but full parity depends on
validating rich value decoding and type metadata for enum, real, string, and raw
event operands. The headers expose the required metadata and traversal hooks, but
the bundled examples mostly exercise Verilog/VHDL scalar/vector values and do
not prove every expression-language value kind.

FSDB support should therefore be documented as:

- Full public-command support is feasible for digital scalar/vector signals.
- Rich expression support is feasible by API surface, but should be gated by
  licensed fixtures for enum labels, real, string, and event values.
- Analog, transaction, sparse MDA/list/class/function-recording objects are not
  required by the current `wavepeek` product contract and can be rejected
  explicitly if encountered in command paths that need plain signal values.

## Current wavepeek Surface Reviewed

| Command | Flags/features that matter for FSDB support |
|---|---|
| `schema` | No waveform input; emits the canonical JSON schema artifact. |
| `info` | `--waves`, `--json`; reports `time_unit`, `time_start`, and `time_end`. |
| `scope` | `--waves`, `--max`, `--max-depth`, `--filter`, `--tree`, `--json`; deterministic DFS hierarchy output. |
| `signal` | `--waves`, `--scope`, `--recursive`, `--max-depth`, `--max`, `--filter`, `--abs`, `--json`; direct and recursive signal listing. |
| `value` | `--waves`, `--at`, optional `--scope`, required comma-delimited `--signals`, `--abs`, `--json`; at-or-before point sampling and Verilog-literal output. |
| `change` | `--waves`, optional `--from`/`--to`, optional `--scope`, required `--signals`, optional `--on`, `--max`, `--abs`, `--json`; wildcard/named/edge/union/`iff` triggers and delta snapshots. Hidden `DEBUG=1` tuning flags are implementation controls. |
| `property` | `--waves`, optional `--from`/`--to`, optional `--scope`, optional `--on`, required `--eval`, `--capture`, `--json`; typed expression evaluation over event-selected timestamps. |

The backend-facing requirements implied by this surface are hierarchy traversal,
stable signal resolution, metadata/time normalization, at-or-before point
sampling, candidate timestamp collection, event occurrence checks, and typed
expression value access.

## Verdi 2022 Reader Surface Needed

The key reader class is `ffrObject`. The visible API version in `ffrAPI.h` is
`FFR_API_VERSION_STR 6.1`.

| Capability | Verdi API surface | Use in wavepeek |
|---|---|---|
| FSDB identification | `ffrObject::ffrIsFSDB`, `ffrObject::ffrCheckFSDB`, `ffrObject::ffrCheckFile` | High-confidence FSDB probe and differentiated unsupported-format errors. Prefer `ffrIsFSDB`; `ffrCheckFile`/`ffrCheckFSDB` are marked in the header as not-suggested-use APIs. |
| Compatibility/version check | `ffrObject::ffrCheckVersionCompatible`, `ffrObject::ffrGetLatestFSDBVersion`, `ffrObject::ffrGetVersion`, `ffrObject::ffrGetAPIVersion`, `ffrObject::ffrGetFSDBInfo` | Distinguish newer/incompatible FSDB files from generic parse failures. Record reader/API version in diagnostics or private validation logs. |
| Open/close | `ffrObject::ffrOpen3`, `ffrObject::ffrClose` | Per-command open/close inside the lazy FSDB backend. The C++ bridge should hide `str_T` mutability and raw pointers. |
| Reader messages | `ffrObject::ffrRegisterWarnCBFunc`, `ffrRegisterErrorCBFunc`, `ffrRegisterInfoCBFunc`, `ffrWarnSuppress`, `ffrInfoSuppress` | Capture or suppress vendor stdout/stderr noise so `wavepeek` error/output contracts remain stable. |
| File metadata | `ffrGetFSDBInfo`, `ffrGetScaleUnit`, `ffrExtractScaleUnit`, `ffrGetMinFsdbTag64`, `ffrGetMaxFsdbTag64`, `ffrGetXTagType`, `ffrGetFileType`, `ffrGetScopeSeparator`, `ffrGetSimVersion`, `ffrGetSimDate` | Implement `info`, time normalization, digital/analog rejection, canonical path conversion, and validation metadata. |
| Hierarchy traversal | `ffrSetTreeCBFunc`, `ffrReadScopeVarTree`, `ffrReadScopeVarTree2`, `fsdbTreeCBFunc`, `fsdbTreeCBType` | Build an owned Rust hierarchy model for `scope`, `signal`, path resolution, and expression binding. |
| Scope records | `fsdbTreeCBDataScope`, `fsdbTreeCBDataUpscope`, `fsdbScopeType` | Push/pop tree traversal stack; collect names, scope kinds, and depth. Sort in Rust for deterministic `wavepeek` output. |
| Signal records | `fsdbTreeCBDataVar`, `ffrGetVarInfoByVarIdcode`, `ffrVarInfo`, `fsdbVarType`, `fsdbBytesPerBit`, `fsdbValueChangeDataType` | Build canonical signal paths, idcode map, kind aliases, widths, data type ids, value encoding metadata, and support/rejection decisions. |
| Data type definitions | `ffrHasDataTypeDef`, `ffrReadDataTypeDefByBlkIdx`, `ffrReadDataTypeDefByBlkIdx2`, `FSDB_TREE_CBT_DT_*`, `fsdbTreeCBDataEnum2`, `fsdbTreeCBDataEnum3`, SV datatype callback types | Implement expression typing for enum labels, integer-like types, signedness, 2-state/4-state distinction, real/string/event types. Required for full `property` parity. |
| Select/load signals | `ffrAddToSignalList`, `ffrResetSignalList`, `ffrLoadSignals`, `ffrUnloadSignals`, `ffrIsInSignalList`, `ffrSetViewWindow`, `ffrResetViewWindow` | Load only command-relevant signals. Use view windows for bounded range commands when safe. Reset/unload to preserve stateless command execution. |
| Per-signal traversal | `ffrCreateVCTrvsHdl`, `ffrCreateVCTraverseHandle`, `ffrVCIterOne::ffrGotoXTag`, `ffrGotoTheFirstVC`, `ffrGotoNextVC`, `ffrGotoPrevVC`, `ffrGetXTag`, `ffrGetVC`, `ffrGetXTagVCSeqNum`, `ffrGetMinXTag`, `ffrGetMaxXTag`, `ffrHasIncoreVC`, `ffrFree` | Implement point sampling, per-signal candidate-time collection, exact event occurrence checks, and previous/current value comparisons. |
| Merged time traversal | `ffrCreateTimeBasedVCTrvsHdl`, `ffrTimeBasedVCIterOne::ffrGotoNextVC`, `ffrGetVarIdcodeXTagVCSeqNum`, `ffrGetVarIdcode`, `ffrGetXTag`, `ffrGetVC`, `ffrFree` | Efficient candidate collection for `change` and `property` over multiple source signals. This is the FSDB equivalent of pushdown candidate discovery. |
| Value metadata on handles | `ffrGetBitSize`, `ffrGetByteCount`, `ffrGetBytesPerBit`, `ffrGetVarType`, `ffrGetPowerVarRealType`, `ffrGetVwIsSet` | Decode value buffers into `wavepeek` bit strings or rich expression values. Reject unsupported encodings deterministically. |
| Glitches and sequence numbers | `ffrGotoXTag(..., ret_glitch_num)`, `ffrGetGlitchNum`, `ffrGetSeqNum`, `ffrIsThereSeqNum`, `ffrGetXTagVCSeqNum` | `wavepeek` currently emits one row per timestamp. `ffrGotoXTag` positions at the last value change at a timestamp, which matches point-sampling needs. Sequence numbers are useful for deterministic internal ordering if needed. |

## Backend Facade Shape

The FSDB bridge should expose a Rust-native facade rather than leaking Verdi
types. A minimal facade sufficient for all current public commands is:

| Facade method | Backing Verdi API | Required by |
|---|---|---|
| `open(path)` | lazy-load bridge, `ffrIsFSDB`, `ffrCheckVersionCompatible`, `ffrOpen3` | all FSDB waveform commands |
| `metadata()` | `ffrGetScaleUnit`, `ffrExtractScaleUnit`, `ffrGetMinFsdbTag64`, `ffrGetMaxFsdbTag64`, `ffrGetXTagType` | `info`, time validation in `value`, `change`, `property` |
| `scope_tree()` | `ffrReadScopeVarTree2` callbacks | `scope`, `signal`, path resolution |
| `signal_index()` | `fsdbTreeCBDataVar`, `ffrGetVarInfoByVarIdcode` | `signal`, `value`, `change`, `property` |
| `datatype_index()` | `ffrReadDataTypeDefByBlkIdx2` callbacks | full `property` expression typing |
| `resolve_signal(path)` | owned path/idcode index | `value`, `change`, `property` |
| `sample_at_or_before(idcode, raw_time)` | `ffrAddToSignalList`, `ffrLoadSignals`, `ffrCreateVCTrvsHdl`, `ffrGotoXTag`, `ffrGetXTag`, `ffrGetVC` | `value`, expression evaluation, change snapshots |
| `event_occurred(idcode, raw_time)` | per-signal traverse handle plus exact returned timestamp check | `.triggered()` and raw event terms |
| `candidate_times(idcodes, from, to)` | `ffrCreateTimeBasedVCTrvsHdl` or per-signal handles | `change`, `property` |
| `format_value_bits(idcode, bytes)` | `fsdbBytesPerBit`, `fsdbBitType`, `fsdbValueChangeDataType` | `value`, `change` |
| `sample_expr_value(idcode, raw_time)` | same as point sampling plus datatype metadata | `property`, `change --on ... iff ...` |

The facade should also expose deterministic error categories for:

- FSDB feature not compiled in.
- FSDB runtime unavailable or bridge load failure.
- Reader/file version incompatibility.
- FSDB command path not implemented yet.
- Unsupported FSDB object/value kind encountered by an otherwise implemented command.

## Command Feasibility Matrix

| Command | Current wavepeek requirements | Support with Verdi 2022 | API needed | Notes and caveats |
|---|---|---|---|---|
| `schema` | Print canonical JSON schema. No waveform input. | Supported without FSDB API. | None. | The schema should not gain backend-specific fields just because FSDB exists. |
| `info` | `time_unit`, `time_start`, `time_end`, optional JSON. | Supported for digital integer-time FSDBs. | `ffrOpen3`, `ffrClose`, `ffrGetScaleUnit`, `ffrExtractScaleUnit`, `ffrGetMinFsdbTag64`, `ffrGetMaxFsdbTag64`, `ffrGetXTagType`, optionally `ffrGetFSDBInfo`. | Reject or explicitly defer float/double XTag analog-style files because current time code assumes integer raw ticks and standard units. |
| `scope` | Deterministic DFS scope paths, depth, kind, regex filter, max/max-depth, tree rendering. | Supported. | `ffrReadScopeVarTree2`, `fsdbTreeCBDataScope`, `fsdbTreeCBDataUpscope`, `fsdbScopeType`, `ffrGetScopeSeparator`. | Verdi callback order should not be trusted as the output contract. Build an owned tree and sort children lexicographically like the wellen backend. |
| `signal` | Direct or recursive signal listing for a scope, kind, optional width, regex filter, deterministic order. | Supported for ordinary vars; partial/reject for composite-only objects if no leaf var is available. | Scope tree APIs plus `fsdbTreeCBDataVar`, `fsdbVarType`, `fsdbBytesPerBit`, `fsdbValueChangeDataType`, `ffrGetVarInfoByVarIdcode`. | Width can be derived from `lbitnum`/`rbitnum` for scalar/vector signals. Real/string/event signals should be listed with kind and no bit-vector width when appropriate, matching current optional-width schema. |
| `value` | Sample named signals at one time, at-or-before semantics, output Verilog literals, preserve input order/duplicates, reject missing/non-bit signals. | Supported for bit-vector/integer-like digital values. | `ffrAddToSignalList`, `ffrLoadSignals`, `ffrCreateVCTrvsHdl`, `ffrGotoXTag`, `ffrGetXTag`, `ffrGetVC`, `ffrGetBitSize`, `ffrGetBytesPerBit`, `ffrGetVarType`. | `ffrGotoXTag` aligns to a value-change point. If the returned time is greater than the query because the query is before the first value change, return the existing `no value at or before requested time` error. Non-bit encodings can follow the current rejection path. |
| `change` | Inclusive range, baseline at `--from`, candidate timestamps from `--on`, wildcard/named/edge/union/`iff`, delta-only rows, `--max`, JSON/human output. | Supported in public semantics for digital values. | All `value` APIs plus `ffrCreateTimeBasedVCTrvsHdl`, `ffrGetVarIdcodeXTagVCSeqNum`, `ffrGotoNextVC`, `ffrSetViewWindow`, expression datatype/sample APIs for `iff`. | Existing wellen-specific `time_table` and `SignalRef` APIs must be replaced. FSDB can avoid a global time table by collecting candidate timestamps from selected idcodes and sampling previous state at `timestamp - 1` for integer-time dumps. Hidden `--tune-*` flags are internal and should not dictate FSDB product support. |
| `property` | Event-selected timestamps, typed `--eval`, wildcard default from eval refs, `match`/`switch`/`assert`/`deassert`, raw events via `.triggered()`. | Supported in shape; full rich-type parity requires licensed validation fixtures. | All `change` APIs plus `ffrHasDataTypeDef`, `ffrReadDataTypeDefByBlkIdx2`, enum callbacks, SV datatype callbacks, rich value decoding for real/string/event. | Integral and edge/event semantics look straightforward. Enum labels, signedness, real values, string values, and raw events have visible API hooks but need fixture proof before claiming full expression-language parity. |

## Details by Command

### `schema`

`schema` is backend-independent. It should keep emitting the same schema artifact
unless a later output-contract RFC changes it. FSDB adds no required schema fields.

### `info`

The command needs only dump precision and bounds. Verdi exposes both file-level
metadata and object-level accessors:

- `ffrGetScaleUnit()` returns the X-axis scale unit string.
- `ffrExtractScaleUnit()` splits a scale unit string into numeric factor and unit.
- `ffrGetMinFsdbTag64()` and `ffrGetMaxFsdbTag64()` return digital min/max time.
- `ffrGetXTagType()` identifies whether the file uses L, HL, float, or double XTags.
- `ffrGetFSDBInfo()` can pre-read file info and version/status fields.

Current `wavepeek` time logic expects integer raw timestamps and units among
`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, and `s`. The FSDB backend should accept
integer `FSDB_XTAG_TYPE_L` and `FSDB_XTAG_TYPE_HL` digital files, convert `HL` to
`u64` with overflow checks, and reject float/double XTag files as unsupported for
the current command surface.

### `scope`

The reader's tree callback model is sufficient. `ffrReadScopeVarTree2()` can call
a bridge-owned callback without mutating the reader object's registered callback.
The callback data includes scope names, scope types, and upscope events.

Implementation notes:

- Build an owned tree during callback traversal.
- Convert FSDB scope kinds to the existing lowercase kind aliases where possible.
- Sort children lexicographically before output to preserve `wavepeek` deterministic DFS.
- Use `ffrGetScopeSeparator()` only for FSDB-native path handling; expose the existing dot-separated canonical `wavepeek` paths.

### `signal`

The same tree pass provides `fsdbTreeCBDataVar` records with the necessary signal
metadata:

- `name`
- `u.idcode`
- `lbitnum` and `rbitnum`
- `dtidcode`
- `type`
- `bytes_per_bit`
- `vc_dt`
- optional descriptor/extra info

For ordinary packed scalar/vector signals, width is `abs(lbitnum - rbitnum) + 1`.
For real, string, raw event, transaction, MDA, or composite objects, preserve the
kind and omit width unless a safe width is known. Commands that later require
bit-vector values can reject unsupported kinds at sampling time.

### `value`

The API supports the current at-or-before sampling semantics, but the bridge must
be careful with `ffrGotoXTag()` behavior. The documented jump behavior aligns the
handle to an existing value change. If the query is before the first value change,
the handle may align to that first value change. `wavepeek` should not treat that
as an at-or-before value; it should compare the returned XTag with the requested
time and return the existing missing-value error if returned time is greater than
the query.

Value decoding required for current `value` output:

- `FSDB_BYTES_PER_BIT_1B` plus `FSDB_BT_VCD_0`, `FSDB_BT_VCD_1`, `FSDB_BT_VCD_X`, `FSDB_BT_VCD_Z` maps directly to `0`, `1`, `x`, `z`.
- VHDL/SV logic states such as `U`, `W`, `L`, `H`, and `-` should be normalized consistently with the expression edge rules where values participate in edge detection.
- `FSDB_BYTES_PER_BIT_4B` and `FSDB_BYTES_PER_BIT_8B` are used for numeric non-bit encodings in examples. The current `value` command can reject those as unsupported non-bit-vector values, matching existing behavior for real/string values.

### `change`

The public `change` semantics are supportable, but the current implementation is
tightly coupled to wellen's global `time_table`, `SignalRef`, loaded-signal
offsets, and FST streaming APIs. The FSDB backend should not try to emulate those
types directly. It should provide backend-neutral operations:

- Resolve command signal paths to stable per-command handles/idcodes.
- Collect candidate timestamps over selected event-source idcodes.
- Sample requested signals at candidate timestamps and at the instant strictly before them.
- Evaluate event expressions and `iff` expressions through the shared expression engine.

For candidate collection, `ffrCreateTimeBasedVCTrvsHdl()` is the most relevant
API. It traverses value changes from multiple selected signals in time-based
fashion and returns `(idcode, xtag, value, seq_num)` through
`ffrGetVarIdcodeXTagVCSeqNum()`. If ordering is not documented strongly enough
for the output contract, collect timestamps into a sorted/deduplicated set before
evaluation.

For previous-value checks, an FSDB backend can sample `timestamp - 1` for
integer-time digital files instead of asking for the previous global dump-table
timestamp. This matches the current contract's "strictly before timestamp"
semantics and avoids requiring an all-timestamps table.

Hidden `change` tuning flags are implementation controls:

- `--tune-engine`
- `--tune-candidates`
- `--tune-edge-fast-force`

They should not be part of the FSDB support promise. If kept active under
`DEBUG=1`, `--tune-candidates=stream` can map naturally to time-based traversal,
while wellen-specific engine modes can either fall back to the supported FSDB
engine or fail with a deterministic internal-tuning error.

### `property`

The event scheduling half of `property` uses the same candidate collection as
`change`. The harder part is full expression-language parity.

The Verdi 2022 headers expose enough metadata hooks to build the expression host:

- `fsdbTreeCBDataVar::type`, `dtidcode`, `lbitnum`, `rbitnum`, `bytes_per_bit`, and `vc_dt` identify many scalar/vector value classes.
- `ffrHasDataTypeDef()` and `ffrReadDataTypeDefByBlkIdx2()` expose user-defined data type callbacks.
- `fsdbTreeCBDataEnum2` and `fsdbTreeCBDataEnum3` expose enum names, literal arrays, value arrays, and bit ranges.
- `fsdbDataType` and callback constants cover SV logic, bit, integer-like, real, shortreal, time, string, and event categories.
- `fsdbBitType` covers Verilog four-state and VHDL nine-state-style logic values.

Required validation before claiming full parity:

- Enum label recovery through `type(signal)::LABEL` and `type(signal)'(...)`.
- Signedness and 2-state/4-state mapping for SV integer-like and packed vector types.
- Real and shortreal sampling from value buffers.
- String sampling and equality semantics from FSDB value buffers.
- Raw event `.triggered()` exact-timestamp behavior.
- Edge behavior on VHDL/SV non-`0/1/x/z` states after normalization.

If these fixtures are not available for the first FSDB release, `property` can
ship partial support for integral expressions and reject unvalidated rich value
kinds with explicit unsupported-FSDB-case errors.

## Unsupported or Deferred FSDB Object Classes

These Verdi APIs exist but are not needed for the current `wavepeek` contract:

- Transaction traversal and attributes.
- Analog/SPICE/Nanosim numeric waveforms using float/double XTags.
- Sparse MDA cell traversal.
- Lists, queues, associative arrays, dynamic arrays, classes, and function-recording objects.
- Property-statistics APIs that report simulator assertion results.
- Computed value insertion APIs.
- OEM insertion and middle/computed variable APIs.

The first FSDB product surface should reject these when they appear in a command
path that expects ordinary scalar/vector RTL signals. `signal` may still list
some of them as hierarchy metadata if the reader reports them as vars, but
sampling commands should stay conservative.

## Risks and Required Validation

| Risk | Why it matters | Validation needed |
|---|---|---|
| Loader/linkage isolation regression | The RFC requires VCD/FST commands to run without Verdi at runtime. | Inspect the final `wavepeek` binary for no unconditional `libnffr.so`/`libnsys.so` dependency and run VCD/FST commands without Verdi libraries resolvable. |
| FSDB version drift | Older readers may reject newer generated files. | Use `ffrCheckVersionCompatible`, capture open failures distinctly, and test reader/file version matrix. |
| Rich expression values | `property` needs more than bit vectors. | Licensed fixtures for enum, real, string, event, signed integer-like, and VHDL/SV logic states. |
| Path canonicalization | FSDB scope separator is usually `/`, while `wavepeek` exposes dot paths. | Fixtures with nested scopes, escaped/special names, and possible dots in names. |
| Timestamp model mismatch | Current engine uses `u64` raw integer timestamps. | Reject non-integer XTag types and overflowed HL tags; test boundary times and timestamp `0`. |
| Glitches and same-timestamp VCs | `wavepeek` emits one row per timestamp. | Confirm `ffrGotoXTag()` last-at-time behavior and deterministic handling of multiple VCs with sequence numbers. |
| Performance | `ffrLoadSignals()` loads selected signals into memory. | Bench range commands with view windows and time-based traversal over large dumps. |

## Overall Conclusion

Verdi `T-2022.06` FSDB Reader has the API surface needed to support the current
`wavepeek` public command set for digital RTL FSDBs. The main implementation work
is not finding missing FSDB APIs; it is introducing a backend-neutral waveform
facade and moving the current engine away from wellen-specific `SignalRef`,
global `time_table`, and loaded-offset assumptions.

Recommended first support statement:

| Command | First FSDB support target |
|---|---|
| `schema` | Already supported; no FSDB dependency. |
| `info` | Full support for digital integer-time FSDB. |
| `scope` | Full support with deterministic sorting. |
| `signal` | Full support for ordinary hierarchy vars; conservative listing/rejection for composites. |
| `value` | Full support for bit-vector/integer-like values; reject non-bit values as today. |
| `change` | Full public semantics for digital values using time-based traversal. |
| `property` | Integral/event subset first unless rich-type fixtures prove full expression parity. |
