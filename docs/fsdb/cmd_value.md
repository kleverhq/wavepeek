# FSDB Reader API for `wavepeek value`

Goal: capture the minimal FSDB Reader API calls and data transformations needed to make `wavepeek value` behave on FSDB files the same way it currently behaves on VCD/FST files. This note intentionally does not cover integration architecture, FFI/C-shims, build/link details, or similar plumbing. That belongs in a separate design. One pile of machinery at a time.

## `wavepeek value` contract to preserve

`value` performs a point query over selected signals:

- accepts `--at <integer><unit>`;
- validates that the timestamp is inside the dump bounds, inclusive;
- validates that the timestamp is aligned to the dump resolution;
- resolves signals either as canonical paths or as names relative to `--scope`;
- preserves `--signals` order, including duplicates;
- returns the latest known value **at or before** the requested timestamp;
- fails instead of inventing a default when a signal has no value at-or-before the requested timestamp;
- supports bit-vector-like values and renders them as Verilog literals: `1'h1`, `8'h0f`, `4'hx`, `4'hz`;
- rejects real/string/non-bit-vector encodings for `value`.

Command JSON payload:

```json
{
  "time": "10ns",
  "signals": [
    {"path": "top.clk", "value": "1'h1"},
    {"path": "top.data", "value": "8'h0f"}
  ]
}
```

Human output:

```text
@10ns
clk 1'h1
addr 32'h00000010
```

With `--abs`, human output uses canonical paths. JSON always uses canonical paths.

## Opening FSDB and basic validation

Recommended calls:

```cpp
ffrObject::ffrIsFSDB(path)
ffrObject::ffrOpen3(path)
// or ffrObject::ffrOpenNonSharedObj(path)
obj->ffrClose()
```

Minimum behavior for `value`:

- if the file is not FSDB or cannot be opened, return a file error;
- use only FSDB Reader / FFR API;
- do not use NPI.

## Metadata: timescale and time bounds

Recommended calls:

```cpp
obj->ffrGetScaleUnit()
obj->ffrGetXTagType()
obj->ffrGetMinFsdbTag64(&min_tag)
obj->ffrGetMaxFsdbTag64(&max_tag)
```

Additional static helper:

```cpp
ffrObject::ffrExtractScaleUnit(scale_unit, digit, unit)
```

Required mapping into wavepeek's model:

| FSDB Reader | Wavepeek |
|---|---|
| `ffrGetScaleUnit()` -> string such as `1ns`, `10ps` | `WaveformMetadata.time_unit` |
| `ffrGetMinFsdbTag64()` | raw `time_start` |
| `ffrGetMaxFsdbTag64()` | raw `time_end` |
| `raw * time_unit` | normalized time string, for example `10ns` |

For digital FSDB, expect `fsdbTag64` / HL-style time. Raw time:

```text
raw = (uint64_t(tag.H) << 32) | uint64_t(tag.L)
```

Back to `fsdbTag64`:

```text
tag.H = raw >> 32
tag.L = raw & 0xffffffff
```

For `value`, supporting digital time tags is sufficient. Float/double xtags should be treated as unsupported for this minimal parity target.

`--at` validation remains wavepeek-side:

- token must be `<integer><unit>`;
- token is converted to zeptoseconds;
- `[time_start, time_end]` is checked;
- dump tick alignment is checked;
- raw timestamp is then computed.

## Reading hierarchy and building the signal index

Recommended call:

```cpp
obj->ffrReadScopeVarTree2(tree_cb_func, client_data)
```

Relevant callback types:

```cpp
FSDB_TREE_CBT_SCOPE
FSDB_TREE_CBT_VAR
FSDB_TREE_CBT_UPSCOPE
FSDB_TREE_CBT_END_TREE
```

For scopes, use `fsdbTreeCBDataScope`:

```cpp
scope->name
scope->type
scope->is_hidden_scope
scope->full_decl_name
```

For variables, use `fsdbTreeCBDataVar`:

```cpp
var->name
var->u.idcode
var->lbitnum
var->rbitnum
var->type
var->bytes_per_bit
var->vc_dt
var->dtidcode
var->is_dummy_var
```

Index needed by `value`:

```text
canonical_path -> {
  idcode,
  width,
  var_type,
  bytes_per_bit,
  vc_dt,
  dtidcode
}
```

The wavepeek canonical path must be dot-separated:

```text
scope_stack.join(".") + "." + var.name
```

Even if FSDB reports a different scope separator, wavepeek's public contract currently uses `.`.

Width:

```text
width = abs(var->lbitnum - var->rbitnum) + 1
```

Do not narrow `fsdbVarIdcode` to `int`: in newer/64-bit builds it may be `longlong_T`.

## Resolving `--scope` and `--signals`

Behavior must match current `value`:

- if `--scope top.u0` is provided, first verify that scope `top.u0` exists;
- `--signals clk,data` becomes canonical paths:
  - `top.u0.clk`
  - `top.u0.data`
- if `--scope` is not provided, `--signals` entries are treated as canonical paths;
- a full path passed under `--scope` should not be specially corrected: current semantics concatenate `scope + token`, and the resulting signal usually fails to resolve;
- input order and duplicates must be preserved in the result.

In practice: load a unique set of `idcode`s through FSDB Reader, then project sampled results back into the original `--signals` order.

## Selecting and loading only requested signals

Recommended calls:

```cpp
obj->ffrResetSignalList();

for each unique idcode:
    obj->ffrAddToSignalList(idcode);

obj->ffrLoadSignals();
```

After sampling:

```cpp
obj->ffrUnloadSignals();
obj->ffrFreeNavDB(); // if available and nav db cleanup is needed
```

For initial `value` parity, do not rely on a narrow `ffrSetViewWindow()` around the query timestamp: `value` must find the previous value at-or-before the query time, and an overly narrow window can cut off the needed previous VC. Optimize later, after fixture coverage. The floor is usually where optimism stores its knives.

## Sampling: value at-or-before

Recommended calls per selected signal:

```cpp
ffrVCTrvsHdl h = obj->ffrCreateVCTrvsHdl(idcode);
// alias also exists: ffrCreateVCTraverseHandle

h->ffrHasIncoreVC();
h->ffrGotoXTag(&query_tag, &glitch_num);
h->ffrGetXTag(&actual_tag);
h->ffrGetVC(&vc_ptr);
h->ffrFree();
```

`ffrGotoXTag()` matches `wavepeek value` semantics:

- if there is a value change at `query_tag`, the handle points to the last VC at that time;
- if there is no exact VC, FSDB Reader aligns the handle to the nearest preceding VC;
- if no suitable value exists, the call fails.

Required defensive handling:

```text
if no handle -> backend/internal file error
if !ffrHasIncoreVC() -> no value at-or-before
if ffrGotoXTag(query) fails -> no value at-or-before
if actual_time > query_time -> no value at-or-before  // defensive guard
```

Decode `vc_ptr` immediately: the memory belongs to FSDB Reader / the traverse handle.

## Decoding Verilog digital values

Minimal VCD/FST parity for `value` is covered by VCD-like digital bits.

Supported conditions:

```text
bytes_per_bit == FSDB_BYTES_PER_BIT_1B
var/value is digital bit-vector-like
width >= 1
```

Mapping `fsdbBitType` -> wavepeek bit char:

| FSDB bit | Char |
|---|---|
| `FSDB_BT_VCD_0` | `0` |
| `FSDB_BT_VCD_1` | `1` |
| `FSDB_BT_VCD_X` | `x` |
| `FSDB_BT_VCD_Z` | `z` |

Example decoder:

```cpp
for (i = 0; i < width; i++) {
    switch (vc_ptr[i]) {
    case FSDB_BT_VCD_0: bits.push_back('0'); break;
    case FSDB_BT_VCD_1: bits.push_back('1'); break;
    case FSDB_BT_VCD_X: bits.push_back('x'); break;
    case FSDB_BT_VCD_Z: bits.push_back('z'); break;
    default: unsupported_or_x;
    }
}
```

Pass the resulting `bits` into wavepeek's existing formatter:

```text
format_verilog_literal(width, bits)
```

Examples:

| width | bits | value |
|---:|---|---|
| 1 | `1` | `1'h1` |
| 8 | `00001111` | `8'h0f` |
| 4 | `zzzz` | `4'hz` |
| 4 | `10xz` | `4'hx` |

Bit order must be verified on real fixtures. FSDB examples print `vc_ptr[0..bitSize)` directly, but wavepeek parity requires matching the VCD/FST canonical bit string.

## VHDL / 9-state values

FSDB has separate codes for VHDL std_logic/std_ulogic:

| FSDB VHDL std logic | Char |
|---|---|
| `FSDB_BT_VHDL_STD_LOGIC_U` / `STD_ULOGIC_U` | `u` |
| `..._X` | `x` |
| `..._0` | `0` |
| `..._1` | `1` |
| `..._Z` | `z` |
| `..._W` | `w` |
| `..._L` | `l` |
| `..._H` | `h` |
| `..._DASH` | `-` |

This matches wellen-style nine-value bit strings, which the current formatter reduces to hex/`x`/`z` using nibble rules. However, for initial Verilog-oriented parity, VHDL decoding can stay out unless there is fixture coverage for `dtidcode` / datatype definitions. Without reliable type identification, reject rather than print confident nonsense. Machines respect honesty. Eventually.

## Unsupported encodings for `value`

The following are not needed for the minimal `value` parity target and should produce signal/file unsupported errors, not partial decoding:

- `FSDB_BYTES_PER_BIT_2B`, `4B`, `8B`, unless covered by an explicit digital decoder;
- real / float / double;
- string;
- analog / SPICE / power signals;
- memory range/depth service variables;
- transaction/stream/property/internal variables;
- dummy vars;
- signal with no width.

Error text should align with current wavepeek behavior, for example:

```text
signal '<path>' has unsupported non-bit-vector encoding
```

If no value exists:

```text
signal '<path>' has no value at or before requested time
```

## Minimal call sequence for `value`

```cpp
// 1. Open
if (!ffrObject::ffrIsFSDB(path)) fail_file;
ffrObject *obj = ffrObject::ffrOpen3(path);
if (!obj) fail_file;

// 2. Metadata
scale_unit = obj->ffrGetScaleUnit();
xtag_type = obj->ffrGetXTagType();
obj->ffrGetMinFsdbTag64(&min_tag);
obj->ffrGetMaxFsdbTag64(&max_tag);

// 3. Build hierarchy/signal index
obj->ffrReadScopeVarTree2(tree_cb_build_index, &index);

// 4. Resolve requested canonical paths to idcodes
resolved = resolve_from_index(scope, signals);
unique_ids = dedupe_preserving_projection(resolved);

// 5. Load selected signals
obj->ffrResetSignalList();
for id in unique_ids:
    obj->ffrAddToSignalList(id);
obj->ffrLoadSignals();

// 6. Sample each unique signal at-or-before query time
for signal in unique_resolved:
    h = obj->ffrCreateVCTrvsHdl(signal.idcode);
    query = raw_to_tag64(query_raw);
    if h && h->ffrHasIncoreVC() && h->ffrGotoXTag(&query) == FSDB_RC_SUCCESS:
        h->ffrGetXTag(&actual);
        h->ffrGetVC(&vc_ptr);
        bits = decode_vc(signal.metadata, vc_ptr);
    else:
        no_value_at_or_before;
    if h:
        h->ffrFree();

// 7. Project duplicate-preserving results back to requested order
// 8. Format bits as Verilog literals in wavepeek
// 9. Cleanup
obj->ffrUnloadSignals();
obj->ffrClose();
```

## Fixture checks before declaring parity

Minimum checks:

1. `1ns` and `10ps` scale units convert to the same normalized time semantics as VCD/FST.
2. `--at` exactly at a transition returns that timestamp's value.
3. `--at` between transitions returns the previous value.
4. `--at` before the first signal VC fails with a no-value error.
5. Query at dump start/end works when in bounds.
6. Signal order and duplicates in `--signals` are preserved.
7. `--scope` relative names and `--abs` human output match current behavior.
8. Bit order matches VCD/FST for scalar and vector signals.
9. `0/1/x/z` values format to expected Verilog literals.
10. Unsupported real/string/non-bit-vector variables fail as signal errors.
