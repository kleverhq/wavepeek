# FSDB Reader API notes for `wavepeek signal`

Scope: research notes for implementing FSDB-backed parity for the existing
`wavepeek signal` command. This file only records FSDB Reader calls and the data
normalization needed for the command contract. It intentionally does not cover
integration architecture.

## Command contract to match

`wavepeek signal` needs the same information it already gets from VCD/FST:

- exact selected scope path from `--scope`;
- direct signals in that scope, or recursive signals below it when `--recursive` is set;
- stable traversal/order semantics;
- per signal:
  - `name` — declared signal name inside its immediate parent scope;
  - `path` — canonical dot-separated full path;
  - `kind` — stable `wavepeek` signal kind alias;
  - `width` — emitted only when the backend reports useful packed bit width metadata.

No value-change loading is needed for this command.

## Minimum FSDB Reader calls

Recommended FSDB Reader call surface for `signal`:

1. Optional preflight / metadata:
   - `ffrObject::ffrIsFSDB(path)`
   - `ffrObject::ffrGetFSDBInfo(path, info)`
2. Open/close:
   - `ffrObject::ffrOpen3(path)`
   - `obj->ffrClose()`
3. Hierarchy and variable definitions:
   - `obj->ffrReadScopeVarTree2(tree_cb, ctx)`
   - equivalent form: `obj->ffrSetTreeCBFunc(tree_cb, ctx)` then
     `obj->ffrReadScopeVarTree()`
4. Data type definitions, when present:
   - `obj->ffrHasDataTypeDef()`
   - `obj->ffrReadDataTypeDefByBlkIdx2(dt_cb, ctx, blk_idx)`

Do **not** use these for `signal`:

- `ffrAddToSignalList`
- `ffrLoadSignals`
- `ffrCreateVCTrvsHdl` / `ffrCreateVCTraverseHandle`
- `ffrCreateTimeBasedVCTrvsHdl`
- value-change traversal APIs

Those are for sampling/change commands, not for hierarchy listing.

## Tree callback data to collect

Relevant tree callback types:

- `FSDB_TREE_CBT_BEGIN_TREE` — start of a hierarchy tree section.
- `FSDB_TREE_CBT_SCOPE` — push a scope onto the current stack.
- `FSDB_TREE_CBT_VAR` — record one signal/variable in the current scope.
- `FSDB_TREE_CBT_UPSCOPE` — pop current scope.
- `FSDB_TREE_CBT_END_TREE` — end of one hierarchy tree section.
- `FSDB_TREE_CBT_END_ALL_TREE` — no more tree sections, if emitted.

From `fsdbTreeCBDataScope`, collect at least:

- `name` — scope path component.
- `type` — not emitted by `signal`, but useful if shared hierarchy storage also feeds
  scope queries later.

From `fsdbTreeCBDataVar`, collect at least:

- `name`
- `u.idcode`
- `type`
- `dtidcode`
- `lbitnum`
- `rbitnum`
- `bytes_per_bit`
- `vc_dt` if needed for later type disambiguation

`direction`, descriptors, and extra info are not required for current `signal`
output.

## Data type callback data to collect

For parity with existing stable aliases, `dtidcode` must be honored where it
changes how a signal should be classified.

Minimum useful datatype callbacks:

- `FSDB_TREE_CBT_DT_ENUM`
- `FSDB_TREE_CBT_DT_ENUM2`
- `FSDB_TREE_CBT_DT_ENUM3`

Build a small map:

```text
dtidcode -> { stable_kind, optional_width_hint }
```

Known observed case: a variable may have a plain Verilog storage `type` such as
`reg`, while its `dtidcode` points to an enum definition. In that case the stable
`wavepeek` kind should be `enum`, not `reg`.

For `DT_ENUM3`, width can be derived from its left/right bit bounds when useful:

```text
width = abs(lbitnum - rbitnum) + 1
```

Other datatype callbacks can be added only when they map to existing stable
`signalKind` values. Do not invent new `kind` strings for `signal`.

## Canonical path construction

Current `wavepeek` canonical paths are dot-separated dump-derived paths.

For FSDB:

1. Maintain a scope stack from `SCOPE` / `UPSCOPE` callbacks.
2. Ignore `obj->ffrGetScopeSeparator()` for canonical output; FSDB samples often
   report `/`, but `wavepeek` output must remain dot-separated.
3. Scope path is `scope_stack.join(".")`.
4. Signal path is `scope_path + "." + normalized_signal_name`.
5. Preserve case and spelling from FSDB names except for packed-range
   normalization below.

## Signal name normalization

FSDB Reader often reports packed ranges as part of `var->name`, for example:

```text
data[7:0]
```

while VCD/FST-backed `wavepeek signal` exposes the base name plus `width`:

```text
name = data
width = 8
```

Normalization rule for FSDB variables:

- If `var->name` ends with a trailing packed range matching
  `var->lbitnum`/`var->rbitnum`, strip only that trailing range from `name`.
- Then compute `width` from the bit bounds.
- Do not strip array indices that are not the packed range.

Examples:

```text
A[3:0], l=3, r=0        -> name A,       width 4
clk,    l=0, r=0        -> name clk,     width 1
a[0][1][7:0], l=7, r=0  -> name a[0][1], width 8
B[3], l=0, r=0          -> name B[3],    width 1  # array element, not packed range
```

This is needed for parity; otherwise FSDB paths would become `top.data[7:0]`
where VCD/FST paths are `top.data`.

## Width mapping

For bit-like digital values:

```text
width = abs(lbitnum - rbitnum) + 1
```

Emit `width` when it is meaningful for the stable kind. Typically emit it for:

- scalar/vector net and variable kinds (`wire`, `reg`, `logic`, `bit`, etc.);
- integer-like fixed-width kinds when the width is known;
- `enum` when datatype metadata provides width.

Usually omit `width` for non-bit-vector payloads such as `real` and `string`.
For `event`, mirror the existing command contract/tests rather than assigning
new semantics from FSDB metadata alone.

## Stable signal kind mapping

Map FSDB Reader var/data types only into existing schema `signalKind` aliases.
The obvious Verilog mappings are:

| FSDB Reader type | `wavepeek` kind |
|---|---|
| `FSDB_VT_VCD_EVENT` | `event` |
| `FSDB_VT_VCD_INTEGER` | `integer` |
| `FSDB_VT_VCD_PARAMETER` | `parameter` |
| `FSDB_VT_VCD_REAL` | `real` |
| `FSDB_VT_VCD_REG` | `reg` |
| `FSDB_VT_VCD_REG2` | `reg` |
| `FSDB_VT_VCD_SUPPLY0` | `supply0` |
| `FSDB_VT_VCD_SUPPLY1` | `supply1` |
| `FSDB_VT_VCD_TIME` | `time` |
| `FSDB_VT_VCD_TRI` | `tri` |
| `FSDB_VT_VCD_TRIAND` | `triand` |
| `FSDB_VT_VCD_TRIOR` | `trior` |
| `FSDB_VT_VCD_TRIREG` | `trireg` |
| `FSDB_VT_VCD_TRI0` | `tri0` |
| `FSDB_VT_VCD_TRI1` | `tri1` |
| `FSDB_VT_VCD_WAND` | `wand` |
| `FSDB_VT_VCD_WIRE` | `wire` |
| `FSDB_VT_VCD_WOR` | `wor` |
| `FSDB_VT_VCD_PORT` | `port` |
| `FSDB_VT_STRING` | `string` |

Data type override examples:

| FSDB datatype evidence | `wavepeek` kind |
|---|---|
| enum datatype for `dtidcode` | `enum` |
| SystemVerilog logic datatype, when identifiable | `logic` |
| SystemVerilog bit datatype, when identifiable | `bit` |
| SystemVerilog int/short/long/byte datatypes, when identifiable | `int`, `short_int`, `long_int`, `byte` |
| SystemVerilog short real datatype, when identifiable | `short_real` |

If `type` has FSDB middle/computed marker bits OR-ed into it, clear those marker
bits before applying the base Verilog mapping.

For analog, transaction, property, coverage, MDA/list/object, and other
FSDB-specific kinds: only emit them through `signal` if they can be mapped to an
existing stable `signalKind`. Do not add new command output kinds as part of
FSDB parity.

## Deduplication

FSDB files may contain multiple hierarchy tree sections. Observed behavior: the
same logical hierarchy can be reported more than once, with `total_var_count`
greater than `unique_var_count`.

For `signal`, deduplicate by canonical signal path:

```text
scope_path + "." + normalized_signal_name
```

Do **not** deduplicate solely by `idcode`: the same underlying idcode may appear
through different ports/scopes, and those distinct paths should remain visible.

If the same path appears repeatedly with identical metadata, keep the first entry
and ignore later duplicates for deterministic output.

## Applying `signal` selection semantics

After building the FSDB-backed hierarchy index, apply the existing command logic:

1. Resolve `--scope` by exact canonical dot path.
2. If not recursive, list only signals whose immediate parent scope is that path.
3. If recursive, traverse child scopes pre-order depth-first from selected scope.
4. Treat the selected scope as depth `0` for `--max-depth`.
5. Sort child scopes lexicographically by scope name/path.
6. Sort signals within each visited scope by `(name, path)`.
7. Apply `--filter` to the displayed/normalized signal `name`.
8. Apply `--max` truncation and warnings exactly as the current command does.

## Error handling expectations

Map FSDB Reader failures to existing user-facing categories:

- open/check failure -> `file` error;
- non-FSDB input -> `file` error;
- selected scope absent -> `scope` error;
- unsupported FSDB file/type content that cannot be represented with current
  stable `signal` output -> `file` or `signal` error, depending on whether the
  whole file or only a specific entry is unsupported.

Keep JSON output within the existing schema; errors are not JSON-wrapped.
