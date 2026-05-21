# FSDB Reader API for `wavepeek property`

Note date: 2026-05-21

Purpose: record only the FSDB Reader API calls and data normalization needed to match the current VCD/FST behavior of `wavepeek property`. No additional FSDB-specific features are added to the command.

## `wavepeek property` contract to support

The command evaluates a logical expression at timestamps selected by an event expression:

- `--on <event_expr>`: wildcard `*`, named event/signal, `posedge`, `negedge`, `edge`, unions via `or`/`,` and `iff`;
- `--eval <logical_expr>`: bit/integral values, enum labels, real, string, casts/selections/operators already supported by the current expression engine;
- `--scope <path>`: relative name resolution;
- `--from` / `--to`: inclusive bounds in dump time units;
- `--capture`: `match`, `switch`, `assert`, `deassert`.

JSON result rows:

```json
[
  {"time": "10ns", "kind": "match"},
  {"time": "20ns", "kind": "assert"}
]
```

The FSDB Reader API must provide the following minimum data for this:

1. time metadata;
2. hierarchy/signal index;
3. signal types for expression binding;
4. sampled value at-or-before a timestamp;
5. exact occurrence for raw events;
6. candidate timestamps from value changes of selected signals.

## Opening and time metadata

Recommended calls:

```cpp
ffrObject::ffrIsFSDB(path)
ffrObject::ffrGetFSDBInfo(path, info)   // useful for preflight metadata
ffrObject::ffrOpen3(path)
obj->ffrGetScaleUnit()
obj->ffrGetXTagType()
obj->ffrGetMinFsdbTag64(&min_tag)
obj->ffrGetMaxFsdbTag64(&max_tag)
obj->ffrClose()
```

Required mapping:

| FSDB Reader | Wavepeek |
|---|---|
| `ffrGetScaleUnit()` / `info.scale_unit` | `WaveformMetadata.time_unit` |
| `ffrGetMinFsdbTag64()` / `info.min_xtag` | raw start tick -> normalized `time_start` |
| `ffrGetMaxFsdbTag64()` / `info.max_xtag` | raw end tick -> normalized `time_end` |

For digital FSDB, support integer tag time:

```text
raw = (uint64_t(tag.H) << 32) | uint64_t(tag.L)

tag.H = raw >> 32
tag.L = raw & 0xffffffff
```

When using `fsdbXTag`, read `tag.hltag.H` / `tag.hltag.L`.

Supported `ffrGetXTagType()` values for this minimum:

```cpp
FSDB_XTAG_TYPE_L
FSDB_XTAG_TYPE_HL
```

Treat `FSDB_XTAG_TYPE_FLOAT` / `FSDB_XTAG_TYPE_DOUBLE` as unsupported for `property` parity.

### Scale unit normalization

The wavepeek time parser expects `<integer><unit>`, for example `1ns`, `10ps`, `100fs`. FSDB may return a fractional scale string such as `0.01n`.

Normalize the scale unit to wavepeek's unit vocabulary:

| FSDB scale | Wavepeek tick |
|---|---|
| `1n` / `1ns` | `1ns` |
| `0.1n` | `100ps` |
| `0.01n` | `10ps` |
| `0.001n` | `1ps` |
| `1p` / `1ps` | `1ps` |
| `0.1p` | `100fs` |

After normalization, timestamp formatting remains the usual:

```text
formatted_time = raw * tick_factor + tick_unit_suffix
```

## Hierarchy and signal index

Recommended call:

```cpp
obj->ffrReadScopeVarTree2(tree_cb, client_data)
```

Equivalent form:

```cpp
obj->ffrSetTreeCBFunc(tree_cb, client_data);
obj->ffrReadScopeVarTree();
```

Required callback types:

```cpp
FSDB_TREE_CBT_SCOPE
FSDB_TREE_CBT_VAR
FSDB_TREE_CBT_UPSCOPE
FSDB_TREE_CBT_END_TREE
FSDB_TREE_CBT_END_ALL_TREE
```

From `fsdbTreeCBDataScope`:

```cpp
scope->name
scope->type
```

From `fsdbTreeCBDataVar`:

```cpp
var->name
var->u.idcode
var->type
var->lbitnum
var->rbitnum
var->bytes_per_bit
var->dtidcode
var->vc_dt
var->is_dummy_var
```

Required index:

```text
canonical_path -> {
  idcode,
  base_var_type,
  lbitnum,
  rbitnum,
  width,
  bytes_per_bit,
  dtidcode,
  vc_dt
}
```

Build canonical paths as dot-separated paths:

```text
scope_stack.join(".") + "." + var.name
```

`width` for packed/vector-like values:

```text
width = abs(var->lbitnum - var->rbitnum) + 1
```

Before mapping `var->type`, strip service flags:

```text
base_var_type = var->type & ~(FSDB_VT_MIDDLE | FSDB_VT_COMPUTED)
```

Store `fsdbVarIdcode` without narrowing it to `int`.

## Data type definitions for expression binding

Recommended calls:

```cpp
obj->ffrHasDataTypeDef()
obj->ffrReadDataTypeDefByBlkIdx2(dt_cb, client_data, blk_idx)
```

Alternatively, use the current tree callback:

```cpp
uint_T blk_idx = 0;
obj->ffrReadDataTypeDefByBlkIdx(blk_idx);
```

Minimum useful callback types:

```cpp
FSDB_TREE_CBT_DT_ENUM2
FSDB_TREE_CBT_DT_ENUM3

FSDB_TREE_CBT_DT_ATTR_SV_LOGIC
FSDB_TREE_CBT_DT_ATTR_SV_REG
FSDB_TREE_CBT_DT_ATTR_SV_BIT
FSDB_TREE_CBT_DT_ATTR_SV_LONG_INT
FSDB_TREE_CBT_DT_ATTR_SV_LONG_UINT
FSDB_TREE_CBT_DT_ATTR_SV_INT
FSDB_TREE_CBT_DT_ATTR_SV_UINT
FSDB_TREE_CBT_DT_ATTR_SV_INTEGER
FSDB_TREE_CBT_DT_ATTR_SV_UINTEGER
FSDB_TREE_CBT_DT_ATTR_SV_SHORT_INT
FSDB_TREE_CBT_DT_ATTR_SV_SHORT_UINT
FSDB_TREE_CBT_DT_ATTR_SV_BYTE_INT
FSDB_TREE_CBT_DT_ATTR_SV_BYTE_UINT
FSDB_TREE_CBT_DT_ATTR_SV_REAL
FSDB_TREE_CBT_DT_ATTR_SV_SHORT_REAL
FSDB_TREE_CBT_DT_ATTR_SV_TIME
FSDB_TREE_CBT_DT_ATTR_SV_STRING
FSDB_TREE_CBT_DT_ATTR_SV_EVENT
```

Build a map:

```text
dtidcode -> ExprType metadata
```

For `FSDB_TREE_CBT_DT_ENUM3`, cast callback data to `fsdbTreeCBDataEnum3*`:

```cpp
e->idcode
e->name
e->lbitnum
e->rbitnum
e->val_len
e->literal_count
e->literal_arr[i]
e->val_arr[i]
```

Convert enum literal values:

```text
for each literal i:
  bits = decode_fsdb_bit_bytes(e->val_arr[i], e->val_len)
  label = e->literal_arr[i]
```

Result for wavepeek:

```text
ExprTypeKind::EnumCore
width = e->val_len or abs(lbitnum - rbitnum) + 1
enum_type_id = e->name or synthetic id from dtidcode
enum_labels = [{ name, bits }]
```

For `FSDB_TREE_CBT_DT_ATTR_SV_*`, callback data is usually cast to `fsdbTreeCBDataAttr*`. Required fields:

```cpp
a->idcode
a->name
a->larridx
a->rarridx
a->meta_info
a->radix
```

## Mapping FSDB var/datatype to `ExprType`

Priority:

1. if `dtidcode` is present in the datatype map, use that type;
2. otherwise use `base_var_type`;
3. if the type is internal/service-only, reject it as an unsupported expression operand.

Minimum table:

| FSDB source | Wavepeek `ExprType` |
|---|---|
| enum datatype (`DT_ENUM2`/`DT_ENUM3`) | `EnumCore`, width from enum datatype/var width, labels from datatype callback |
| `FSDB_VT_VCD_EVENT`, SV event attr | `Event`, width `0` |
| `FSDB_VT_STRING`, SV string attr | `String`, width `0` |
| `FSDB_VT_VCD_REAL`, SV real attr | `Real` |
| SV short real attr | `Real` |
| `FSDB_VT_VCD_INTEGER`, SV integer attr | `IntegerLike(Integer)`, width `32` |
| `FSDB_VT_VCD_TIME`, SV time attr | `IntegerLike(Time)`, width `64` |
| SV byte/short/int/long attrs | `IntegerLike(Byte/Shortint/Int/Longint)` with width `8/16/32/64` |
| `FSDB_VT_VCD_REG`, `WIRE`, `TRI*`, `WAND`, `WOR`, `PARAMETER`, SV logic/reg attr | `BitVector`, four-state |
| SV bit attr | `BitVector`, two-state |

Signedness:

```text
SV *_INT variants       -> signed
SV *_UINT variants      -> unsigned
VCD integer             -> signed
VCD time                -> unsigned
plain wires/regs/logic  -> unsigned
```

Storage:

```text
width > 1 -> PackedVector
width <= 1 -> Scalar
real/string/event -> Scalar
```

FSDB `FSDB_VT_PROP_*`, transaction/stream/coverage/internal/MDA service variables are not part of the current `wavepeek property` contract. Reject them as unsupported/internal expression operands.

## Selecting signals for `property`

Signals to load from FSDB are derived from the parsed expressions:

- explicit handles from `--on`;
- handles from `--eval`;
- if `--on '*'`, candidate sources come from referenced signals in `--eval`;
- raw event operands are also included as candidate sources.

Required conversion before FSDB calls:

```text
SignalHandle -> canonical_path -> idcode
unique idcode set for loading
projection back to handles/paths for evaluation
```

## Loading selected value changes

Recommended calls:

```cpp
obj->ffrResetSignalList();

for each unique idcode:
    obj->ffrAddToSignalList(idcode);

obj->ffrLoadSignals();
```

After evaluation finishes:

```cpp
obj->ffrUnloadSignals();
```

## Sampling value at-or-before timestamp

Recommended calls for one signal:

```cpp
ffrVCTrvsHdl h = obj->ffrCreateVCTrvsHdl(idcode);
// alias: obj->ffrCreateVCTraverseHandle(idcode)

h->ffrHasIncoreVC();
h->ffrGotoXTag(&query_tag);
h->ffrGetXTag(&actual_tag);
h->ffrGetVC(&vc_ptr);
h->ffrGetBytesPerBit();
h->ffrGetBitSize();
h->ffrGetVarType();
h->ffrFree();
```

Semantics for wavepeek:

```text
sample_value(idcode, t):
  if no incore VC -> None
  goto query tag t
  if goto failed -> None
  read actual tag
  if actual_raw > t -> None
  decode vc_ptr according to ExprType
```

`ffrGetVC()` returns a pointer owned by the FSDB Reader/traverse handle. Copy or decode the value immediately.

### Digital/integral decode

For VCD-like digital values:

```text
bytes_per_bit == FSDB_BYTES_PER_BIT_1B
```

Mapping:

| FSDB bit byte | Wavepeek bit char |
|---|---|
| `FSDB_BT_VCD_0` | `0` |
| `FSDB_BT_VCD_1` | `1` |
| `FSDB_BT_VCD_X` | `x` |
| `FSDB_BT_VCD_Z` | `z` |

Conversion:

```text
bits = decode_fsdb_bit_bytes(vc_ptr, width)
SampledValue::Integral { bits: Some(bits), label: enum_label_for_bits(bits) }
```

For enum values, set `label` only when `bits` exactly matches one of `enum_labels`.

### Real decode

For `ExprTypeKind::Real`:

```text
if bytes_per_bit == FSDB_BYTES_PER_BIT_8B:
    value = *(double*)vc_ptr
if bytes_per_bit == FSDB_BYTES_PER_BIT_4B and type is explicitly short-real/float:
    value = double(*(float*)vc_ptr)
else:
    unsupported real encoding
```

Result:

```text
SampledValue::Real { value: Some(value) }
```

### String decode

For `ExprTypeKind::String` / `FSDB_VT_STRING`:

```text
value = nul-terminated char* from vc_ptr
```

Result:

```text
SampledValue::String { value: Some(copy_as_utf8_lossy_or_validated_string) }
```

Under current expression semantics, string truthiness remains false, but equality/operations should receive the string value.

### Event sampling

Raw events are not sampled as values:

```text
sample_value(event, t) -> error/invalid path for value sampling
```

For `.triggered()` and raw event terms, use the exact occurrence behavior below.

## Raw event occurrence at exact timestamp

Recommended calls:

```cpp
ffrVCTrvsHdl h = obj->ffrCreateVCTrvsHdl(event_idcode);
h->ffrHasIncoreVC();
h->ffrGotoXTag(&query_tag);
h->ffrGetXTag(&actual_tag);
h->ffrFree();
```

Conversion:

```text
event_occurred(event_idcode, t):
  if signal type is not Event -> invalid for this operation
  if no VC -> false
  goto query tag t
  if goto failed -> false
  return actual_raw == t
```

The `vc_ptr` value is not needed for raw events. Only the existence of a VC at the exact timestamp matters.

## Candidate timestamps for `--on` / `--eval`

Recommended calls for each candidate source:

```cpp
ffrVCTrvsHdl h = obj->ffrCreateVCTrvsHdl(idcode);
h->ffrHasIncoreVC();
h->ffrGotoXTag(&from_tag);
h->ffrGetXTag(&actual_tag);
h->ffrGotoNextVC();
h->ffrGetXTag(&actual_tag);
h->ffrFree();
```

Conversion algorithm:

```text
collect_candidate_times(idcode, from_raw, to_raw):
  if no VC -> empty
  goto from_raw
  read actual_raw

  if actual_raw < from_raw:
      gotoNext until actual_raw >= from_raw

  while actual_raw <= to_raw:
      add actual_raw
      if gotoNext fails: break
      read actual_raw
```

If `from_raw` is before the first VC, `ffrGotoXTag(from)` may align to the first VC. This is correct when `actual_raw >= from_raw`.

For multiple signals:

```text
candidate_times = sorted union of per-signal VC timestamps
```

These timestamps are then used by the command's existing semantics:

- `--on signal` / wildcard: check sampled value change;
- `posedge/negedge/edge`: compare previous/current integral LSB;
- raw event: check exact occurrence;
- `iff`: evaluate the logical expression at the same timestamp.

## Previous/current values for event semantics

For candidate timestamp `t`, two states are needed:

```text
current  = sample_value(idcode, t)
previous = sample_value(idcode, t - 1), if t > 0
```

This gives the value strictly before the candidate timestamp through at-or-before sampling.

Usage:

```text
named signal event:
  changed = previous != current, with None -> Some counted as change

posedge/negedge/edge:
  previous_bits = sample at t - 1
  current_bits  = sample at t
  classify by current wavepeek edge rules on LSB

raw event:
  event_occurred(idcode, t)
```

If `t == 0`, previous state is absent:

```text
named signal event -> false unless current semantics explicitly treats first value as change
edge event -> false
raw event -> exact occurrence at 0
```

## Command result formation

FSDB API returns raw ticks. `property` result rows contain normalized timestamps:

```text
time = format_raw_timestamp(raw, normalized_dump_tick)
```

`kind` is computed by the command from capture mode:

```text
--capture match    -> row when eval is true at candidate
--capture switch   -> assert/deassert transitions of eval truth
--capture assert   -> false -> true only
--capture deassert -> true -> false only
```

No FSDB-specific fields are added to the result.

## Minimum call sequence

```cpp
// 1. Open / metadata
if (!ffrObject::ffrIsFSDB(path)) fail_file;
ffrObject *obj = ffrObject::ffrOpen3(path);
if (!obj) fail_file;

scale = normalize_scale_unit(obj->ffrGetScaleUnit());
xtag_type = obj->ffrGetXTagType();
obj->ffrGetMinFsdbTag64(&min_tag);
obj->ffrGetMaxFsdbTag64(&max_tag);

// 2. Datatypes
if (obj->ffrHasDataTypeDef()) {
    uint_T blk_idx = 0;
    obj->ffrReadDataTypeDefByBlkIdx2(dt_cb, &dt_index, blk_idx);
}

// 3. Hierarchy/signal index
obj->ffrReadScopeVarTree2(tree_cb, &signal_index);

// 4. Resolve expression handles -> idcodes and ExprTypes
resolved = resolve_property_sources(signal_index, dt_index, on_expr, eval_expr, scope);

// 5. Load selected idcodes
obj->ffrResetSignalList();
for id in unique_idcodes:
    obj->ffrAddToSignalList(id);
obj->ffrLoadSignals();

// 6. Collect sorted union of candidate VC timestamps
for source in candidate_sources:
    scan source VC timestamps in [from_raw, to_raw]

// 7. At each candidate timestamp, use sampling/exact-event checks
for t in candidate_times:
    evaluate --on
    evaluate --eval
    emit capture row if selected by capture mode

obj->ffrUnloadSignals();
obj->ffrClose();
```
