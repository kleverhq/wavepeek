# FSDB Reader API for `wavepeek change`

Scope: recommended **FSDB Reader API** calls and the data shaping needed to feed the existing `wavepeek change` semantics. This note intentionally avoids integration architecture details.

## Target shape expected by `change`

`change` needs these backend-facing facts/operations:

- dump metadata: time unit, start/end raw timestamps;
- hierarchy: canonical dot-separated scope and signal paths;
- signal metadata: stable signal id, width, value kind, value encoding;
- candidate timestamps: ordered unique times where trigger candidate signals changed or raw events occurred;
- sampled values: signal value at-or-before a raw timestamp;
- printable snapshots: requested bit-vector values formatted by wavepeek as Verilog literals.

## File validation and opening

Recommended calls:

- `ffrObject::ffrIsFSDB(str_T file_name)`
  - Use as quick format validation.
- `ffrObject::ffrCheckFile(str_T file_name)` / `ffrObject::ffrCheckFSDB(str_T fsdb_fname)`
  - Optional pre-open diagnostics for missing/unreadable/empty/non-FSDB input.
- `ffrObject::ffrGetFSDBInfo(str_T fsdb_fname, ffrFSDBInfo &info)`
  - Use for file-level metadata available before full traversal.
- `ffrObject::ffrOpen3(str_T fname)`
  - Open reader object.
- `obj->ffrClose()`
  - Close when command finishes.

Useful `ffrFSDBInfo` fields:

- `file_type`
- `scope_separator`
- `scale_unit`
- `min_xtag`
- `max_xtag`
- `unique_var_count`
- `total_var_count`
- `max_glitch_num`
- `is_view_window_available`

For `change` parity, accept digital Verilog/SystemVerilog-style FSDB first. At minimum, `FSDB_FT_VERILOG` is directly relevant; mixed Verilog/VHDL needs separate compatibility decisions.

## Time metadata

Recommended calls:

- `obj->ffrGetScaleUnit()`
  - Returns scale string such as `1ns`, `10ps`, etc.
- `ffrObject::ffrExtractScaleUnit(scale_unit, digit, unit)`
  - Parse scale string into numeric tick and unit suffix.
- `obj->ffrGetMinFsdbTag64(fsdbTag64 *tag64)`
  - Digital dump start time.
- `obj->ffrGetMaxFsdbTag64(fsdbTag64 *tag64)`
  - Digital dump end time.
- `obj->ffrGetXTagType()`
  - Confirm tag representation; `change` wants integer raw ticks.

Convert `fsdbTag64` to wavepeek raw `u64`:

```text
raw = (u64(tag64.H) << 32) | u64(tag64.L)
```

Convert FSDB scale unit to wavepeek metadata:

```text
ffrGetScaleUnit() -> "<integer><unit>"
```

Then expose:

```text
time_unit  = scale unit, e.g. "1ns"
time_start = raw_min * scale, formatted with same unit
time_end   = raw_max * scale, formatted with same unit
```

`change --from/--to` already converts user tokens back to raw timestamps using this metadata, so the FSDB side should keep raw times as unscaled ticks.

## Hierarchy and signal metadata

Recommended calls:

- `obj->ffrReadScopeVarTree2(fsdbTreeCBFunc tree_cb_func, void *tree_client_data)`
  - Prefer this direct form for building a one-shot in-memory hierarchy map.
- Alternative equivalent sequence:
  - `obj->ffrSetTreeCBFunc(cb, user)`
  - `obj->ffrReadScopeVarTree()`

Tree callback signature:

```cpp
bool_T (*fsdbTreeCBFunc)(fsdbTreeCBType type, void *client, void *tree_cb_data)
```

Relevant callback types:

- `FSDB_TREE_CBT_BEGIN_TREE`
- `FSDB_TREE_CBT_SCOPE`
- `FSDB_TREE_CBT_VAR`
- `FSDB_TREE_CBT_UPSCOPE`
- `FSDB_TREE_CBT_END_TREE`
- `FSDB_TREE_CBT_END_ALL_TREE`
- enum datatype callbacks if enum expression parity is required:
  - `FSDB_TREE_CBT_DT_ENUM`
  - `FSDB_TREE_CBT_DT_ENUM2`
  - `FSDB_TREE_CBT_DT_ENUM3`

### Scope records

For `FSDB_TREE_CBT_SCOPE`, cast callback data to `fsdbTreeCBDataScope *` / `fsdbScopeRec *`.

Useful fields:

- `name`
- `module`
- `scale_unit`
- `time_unit`
- `type`
- `full_decl_name`

Build a scope stack from `SCOPE` / `UPSCOPE`. Wavepeek canonical paths are dot-separated, so join scope names and signal names as:

```text
scope.subscope.signal
```

FSDB has `obj->ffrGetScopeSeparator()` / `ffrFSDBInfo.scope_separator`, but for wavepeek command resolution normalize internal paths to dot-separated canonical paths.

### Variable records

For `FSDB_TREE_CBT_VAR`, cast callback data to `fsdbTreeCBDataVar *` / `fsdbVarRec *`.

Useful fields:

- `name`
- `u.idcode`
- `lbitnum`
- `rbitnum`
- `dtidcode`
- `direction`
- `type`
- `bytes_per_bit`
- `vc_dt`
- `len_descriptor`
- `descriptor`
- `extra_inf`

Build signal table entries:

```text
path       = current_scope_stack + "." + var.name
idcode     = var.u.idcode
width      = abs(var.lbitnum - var.rbitnum) + 1
var_type   = var.type
bytes/bit  = var.bytes_per_bit
vc_dt      = var.vc_dt
dtidcode   = var.dtidcode
```

This table feeds:

- `--scope` validation;
- `--signals` path resolution;
- `--on` signal/event/edge operand resolution;
- output snapshot widths.

## Mapping FSDB variable kinds to `change`

Common FSDB variable types from `fsdbVarType`:

- `FSDB_VT_VCD_EVENT`
  - Raw event. Use as event occurrence source for event expressions. Do not treat as printable bit-vector value.
- `FSDB_VT_VCD_INTEGER`
  - Integral expression operand. Width should normally be integer-like width if available; otherwise use FSDB bit range/metadata.
- `FSDB_VT_VCD_TIME`
  - Integral expression operand, unsigned time-like width.
- `FSDB_VT_VCD_REG`, `FSDB_VT_VCD_WIRE`, `FSDB_VT_VCD_TRI`, `FSDB_VT_VCD_TRIAND`, `FSDB_VT_VCD_TRIOR`, `FSDB_VT_VCD_TRIREG`, `FSDB_VT_VCD_TRI0`, `FSDB_VT_VCD_TRI1`, `FSDB_VT_VCD_WAND`, `FSDB_VT_VCD_WOR`, `FSDB_VT_VCD_PORT`
  - Bit-vector/integral operands and printable `change --signals` values.
- `FSDB_VT_VCD_REAL`
  - Real expression operand. Not printable by `change` requested-signal output parity unless the current wavepeek command gains explicit support.
- `FSDB_VT_STRING`
  - String expression operand. Not printable by `change` requested-signal output parity unless explicitly supported.

For requested output signals, current `change` parity should require bit-vector-compatible signals and reject unsupported non-bit-vector encodings with a signal error.

## Loading selected variables

Recommended calls:

- `obj->ffrResetSignalList()`
  - Clear any previous selection.
- `obj->ffrAddToSignalList(fsdbVarIdcode idcode)`
  - Add every selected variable needed by the command.
- `obj->ffrLoadSignals()`
  - Load value changes for selected variables.
- `obj->ffrUnloadSignals()`
  - Release selected loaded values after use.
- `obj->ffrUnloadSignals(fsdbVarIdcode idcode)` or `obj->ffrUnloadSignals(id_array, count)`
  - Optional targeted unload.

The selected set for `change` should include:

- requested output signals from `--signals`;
- candidate sources from `--on`:
  - requested output signals if `--on` contains `*`;
  - explicit named/edge event operands;
- operands referenced by `iff` expressions, if they need sampling during event evaluation.

## Candidate timestamps

Current implementation calls:

- `obj->ffrCreateVCTrvsHdl(fsdbVarIdcode idcode)` for each selected candidate variable.
- `hdl->ffrGotoXTag(&from_tag)` to align near the window start.
- `hdl->ffrGetMinXTag(&tag)` when `ffrGotoXTag(from)` fails because the signal's first value change is later than `from`.
- `hdl->ffrGetXTag(&tag)` and `hdl->ffrGotoNextVC()` to collect value-change times through `to`.
- `hdl->ffrFree()` to release each traverse handle.

A future optimization may use `obj->ffrCreateTimeBasedVCTrvsHdl(uint_T num, fsdbVarIdcode sig_arr[])` to create a merged time-ordered traversal, but the portable implementation does not depend on that symbol.

Shape into wavepeek candidate data:

```text
candidate event:
  raw_time = xtag.hltag as u64
  idcode   = changed variable id
  seq_num  = FSDB sequence number, if present
  vc       = raw FSDB value pointer for this value change
```

For `change`, group traversal records by `raw_time` and produce one candidate timestamp per unique time.

Candidate source mapping:

- value source changed at `t` -> candidate `t`;
- raw event source occurred at `t` -> candidate `t`;
- multiple candidate records at the same `t` -> one candidate timestamp.

Do not emit a row merely because a candidate occurred. Existing `change` semantics require a requested output signal delta and a matching `--on` expression.

## Sampling at-or-before a timestamp

Recommended calls:

- `obj->ffrCreateVCTrvsHdl(fsdbVarIdcode idcode)`
  - Create per-signal traversal handle.
- On `ffrVCTrvsHdl`:
  - `hdl->ffrGotoXTag(void *xtag, int *ret_glitch_num = NULL)`
  - `hdl->ffrGetXTagVCSeqNum(&xtag, &vc, &seq_num)` or `hdl->ffrGetXTag(&xtag)` + `hdl->ffrGetVC(&vc)`
  - `hdl->ffrGotoPrevVC()` when an explicit previous value is needed and the handle is at the current timestamp.
  - `hdl->ffrFree()`

Required wavepeek sampling semantics:

```text
sample(signal, t) = last value change of signal at or before raw time t
```

Use cases:

- snapshot value at candidate timestamp `t`;
- baseline value at `--from`;
- previous value for edge classification, equivalent to sampling at `t - 1` when `t > 0`;
- previous/current comparison for requested output delta.

If no value exists at-or-before `t`, return missing value. For printable requested signals, a missing value at emitted snapshot time is an error in the current command contract.

## Value decoding

Recommended calls on traversal handles:

- `hdl->ffrGetVC(byte_T **ret_vc)`
- `hdl->ffrGetBitSize()`
- `hdl->ffrGetBytesPerBit()`
- `hdl->ffrGetByteCount()`
- `hdl->ffrGetVarType()`

Relevant byte-per-bit enum:

- `FSDB_BYTES_PER_BIT_1B`
- `FSDB_BYTES_PER_BIT_2B`
- `FSDB_BYTES_PER_BIT_4B`
- `FSDB_BYTES_PER_BIT_8B`
- `FSDB_BYTES_PER_BIT_UNKNOWN`

### Digital 4-state vector values

For ordinary Verilog digital vectors, expected encoding is usually `FSDB_BYTES_PER_BIT_1B` with one byte per bit.

Map FSDB bit values to wavepeek bit strings:

```text
FSDB_BT_VCD_0 -> '0'
FSDB_BT_VCD_1 -> '1'
FSDB_BT_VCD_X -> 'x'
FSDB_BT_VCD_Z -> 'z'
other         -> 'x' for command-level conservative handling
```

Wavepeek wants bit strings in display order compatible with existing `format_verilog_literal(width, bits)`. Preserve FSDB traversal bit order as validated against generated fixtures; if FSDB returns bits in declaration order, this directly matches the current formatter expectation.

### Real values

For `FSDB_VT_VCD_REAL`, examples decode by casting the VC pointer according to bytes-per-bit:

- `FSDB_BYTES_PER_BIT_4B` -> `float`
- `FSDB_BYTES_PER_BIT_8B` -> `double`

Use for `--on ... iff ...` expression parity when real operands are referenced. Do not use as printable `change --signals` output unless command contract changes.

### Raw events

For `FSDB_VT_VCD_EVENT`, treat a VC record at time `t` as event occurrence at `t`.

Needed behavior:

```text
event_occurred(event_signal, t) = true iff event signal has a VC exactly at t
```

Raw event signals should not be sampled as ordinary printable values.

### Strings

`FSDB_VT_STRING` exists, but exact value representation should be verified with a fixture before claiming full expression parity. If unsupported initially, report string operands as unsupported rather than silently converting garbage. The machine spirits enjoy silent garbage; users less so.

## Event expression shaping

`change --on` needs these backend observations:

- wildcard `*`:
  - candidate if any requested output signal changed;
- named event/value signal:
  - candidate if that signal changed;
- raw event signal:
  - candidate if event occurred exactly at timestamp;
- `posedge` / `negedge` / `edge`:
  - sample previous integral value strictly before `t`;
  - sample current integral value at `t`;
  - classify only the least-significant bit.

Edge normalization must match existing wavepeek behavior:

```text
0 -> 1/x/z       = posedge
x/z -> 1         = posedge
1 -> 0/x/z       = negedge
x/z -> 0         = negedge
h/u/w/l/-/other  = x for edge classification
```

## Delta and snapshot shaping

For each candidate timestamp `t`:

1. Coalesce all FSDB records at the same `t`.
2. Ensure current values for requested output signals are known, either from rolling traversal state or per-signal sampling.
3. Compare requested output signal values with their previous sampled values.
4. If no requested output bit-vector value changed, skip the timestamp.
5. Evaluate `--on` at `t`.
6. If it matches, build snapshot:

```text
time    = raw t formatted using dump time unit
signals = requested signals in CLI order
value   = wavepeek Verilog literal from width + bit string
```

Duplicate requested signal tokens should preserve output order and duplicates, while resolution/loading can internally deduplicate by canonical path/idcode.

## Sequence numbers, same-time records, and glitches

Relevant calls:

- `obj->ffrIsThereSeqNum()`
- `hdl->ffrGetSeqNum(&seq_num)`
- `hdl->ffrGetVarIdcodeXTagVCSeqNum(...)`
- `hdl->ffrGetGlitchNum(&glitch_num)`
- `obj->ffrHasGlitch(idcode)`
- `obj->ffrGetVCCount(idcode)`

Shaping rule for `change`:

- output is per timestamp, not per individual VC record;
- multiple records at the same timestamp should collapse to the final sampled value used at that timestamp;
- when sequence numbers are present, use them to preserve deterministic same-time ordering;
- when sequence numbers are absent and same-time ordering matters, prefer confirming final value with per-signal `ffrGotoXTag(t)`.

Glitch-specific data is not a separate `change` feature. Only its effect on final sampled values and event occurrence matters for parity.

## View-window caution

Relevant calls:

- `obj->ffrSetViewWindow(fsdbXTag *start_xtag, fsdbXTag *close_xtag)`
- `obj->ffrResetViewWindow(...)`
- `obj->ffrIsViewWindowSet()`
- `obj->ffrSetFsdbTag64LoadRange(fsdbTag64 *start, fsdbTag64 *end)`

For exact `change` semantics, be careful with view-window loading: FSDB Reader examples note that values outside a view window may be aligned to the window boundary. That can affect baseline and first-candidate behavior.

Safe command-level shape:

- keep raw command window as `[from_raw, to_raw]`, inclusive;
- baseline is sampled at `from_raw`;
- no snapshot is emitted at `t <= from_raw`;
- candidate records are filtered to `from_raw <= t <= to_raw`;
- values used at `t` must represent at-or-before sampling in the full dump, not an artificial boundary value.
