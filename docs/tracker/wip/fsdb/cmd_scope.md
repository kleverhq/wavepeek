# FSDB Reader API for `wavepeek scope`

Note date: 2026-05-20

Goal: extract from FSDB the same minimum that `scope` already extracts from VCD/FST: a list of scope entries with `path`, `depth`, and `kind`.

## Minimum parity result

`wavepeek scope` expects each scope to provide:

```text
path:  canonical dot-separated hierarchy path
 depth: zero-based depth from dump root
 kind:  stable wavepeek scope kind alias
```

No new fields or FSDB-specific command flags are needed.

## Recommended FSDB Reader API calls

Primary path for FSDB 2.x+:

```cpp
ffrObject::ffrIsFSDB(path)
ffrObject::ffrOpen3(path)
obj->ffrHasScopeTree()
obj->ffrReadScopeTree2(tree_cb, client_data)
obj->ffrClose()
```

Purpose:

- `ffrIsFSDB(path)` — check that the input file is an FSDB file.
- `ffrOpen3(path)` — open the file without immediately traversing the hierarchy.
- `ffrHasScopeTree()` — check whether the separate scope-tree representation is available.
- `ffrReadScopeTree2(tree_cb, client_data)` — read only hierarchy scopes through the callback.
- `ffrClose()` — close the reader object.

Fallback for old FSDB files without a scope tree:

```cpp
obj->ffrReadScopeVarTree2(tree_cb, client_data)
```

In this fallback mode, the callback may also receive signal callbacks; `scope` must ignore them.

## Why `ffrReadScopeTree2`, not `ffrReadScopeVarTree2`

The `scope` command only needs hierarchy scopes. `ffrReadScopeTree2` reads the scope hierarchy without variables/signals. `ffrReadScopeVarTree2` reads both scopes and signals, so it does unnecessary work for the normal FSDB 2.x+ path.

`ffrReadScopeVarTree2` is useful only as a compatibility fallback when no separate scope tree is available.

## Callback types needed by `scope`

Handle:

```cpp
FSDB_TREE_CBT_BEGIN_TREE
FSDB_TREE_CBT_SCOPE
FSDB_TREE_CBT_UPSCOPE
FSDB_TREE_CBT_END_TREE
FSDB_TREE_CBT_END_ALL_TREE
```

Ignore for `scope`:

```cpp
FSDB_TREE_CBT_VAR
```

Also ignore datatype, value-change, transaction, and other callback types: they are not part of the `scope` command contract.

## Data from `FSDB_TREE_CBT_SCOPE`

For `FSDB_TREE_CBT_SCOPE`, cast callback data to `fsdbTreeCBDataScope*`.

Required fields:

- `name` — local scope name; the segment for the future canonical path.
- `type` — FSDB scope type; maps to wavepeek `kind`.
- `is_hidden_scope` — internal/hidden scope; user-facing `scope` output should preferably exclude these scopes together with their subtree.

Fields not needed for `scope`:

- `module`
- `scale_unit`
- `time_unit`
- `full_decl_name`
- `var_start_log_uoff`
- `has_memory_var`
- `has_property_var`

They may be useful for other commands, but they are not required for `scope` parity.

## Building `path` and `depth`

FSDB Reader reports traversal events, so the required shape is built with a stack:

1. On `FSDB_TREE_CBT_BEGIN_TREE`, start a new traversal context.
2. On `FSDB_TREE_CBT_SCOPE`:
   - if the scope is not hidden, push `name` onto the stack;
   - `depth = stack.len() - 1`;
   - `path = stack.join(".")`;
   - `kind = map_scope_type(scope->type)`;
   - add the entry to an intermediate tree/set.
3. On `FSDB_TREE_CBT_UPSCOPE`, pop the current scope from the stack.
4. On `FSDB_TREE_CBT_END_TREE`, clear the stack.
5. On `FSDB_TREE_CBT_END_ALL_TREE`, traversal is complete.

The public path must stay dot-separated, matching VCD/FST behavior in wavepeek. FSDB `ffrGetScopeSeparator()` must not change the public command format.

## Multiple trees and deduplication

FSDB may report multiple trees, including repeated or overlapping scopes. For stable `scope` output, entries must be merged by canonical `path`.

Recommended rule:

- key: canonical `path`;
- if the same `path` appears again with the same `kind`, keep a single entry;
- if the same `path` appears again with a different `kind`, keep the first stable kind or normalize the conflict to `unknown` — this is a separate contract decision, but duplicate output entries must not be emitted;
- hidden scopes and their subtrees must not create visible entries.

After merging, the final list must be sorted/traversed according to the existing wavepeek contract: pre-order DFS, children ordered lexicographically by local scope name, with full path as a tie-breaker.

## Mapping `fsdbScopeType` to wavepeek `kind`

Keep the current `scopeKind` inventory from the schema; do not add new kinds.

| FSDB scope type | wavepeek `kind` |
|---|---|
| `FSDB_ST_VCD_MODULE` | `module` |
| `FSDB_ST_SV_MODULE` | `module` |
| `FSDB_ST_SC_MODULE` | `module` |
| `FSDB_ST_VCD_TASK` | `task` |
| `FSDB_ST_VCD_FUNCTION` | `function` |
| `FSDB_ST_VCD_BEGIN` | `begin` |
| `FSDB_ST_VCD_FORK` | `fork` |
| `FSDB_ST_VCD_GENERATE` | `generate` |
| `FSDB_ST_SV_INTERFACE` | `interface` |
| `FSDB_ST_SV_PACKAGE` | `package` |
| `FSDB_ST_SV_PROGRAM` | `program` |
| `FSDB_ST_SV_CLASS` | `class` |
| `FSDB_ST_VCD_LIST` | `unknown` |
| `FSDB_ST_VCD_ARRAY_ELEM` | `unknown` |
| `FSDB_ST_SV_MODPORT` | `unknown` |
| `FSDB_ST_SV_INTERFACEPORT_REF` | `unknown` |
| `FSDB_ST_SV_MODPORT_REF` | `unknown` |
| VHDL-specific scope types | `unknown` |
| power/spice/AMS/internal/special scope types | `unknown` |
| any unrecognized type | `unknown` |

`struct` and `union` remain valid wavepeek scope kinds in the current schema, but the FSDB Reader scope-tree minimum does not require emitting them unless they arrive as normal scope records.

## wavepeek command flags

FSDB extraction should return the full normalized list of scope entries. Existing command semantics are then applied exactly as for VCD/FST:

- `--max-depth` filters by `depth`;
- `--filter` is applied to the full canonical `path`;
- `--max` limits the number of entries and emits a truncation warning;
- `--tree` affects only human rendering;
- `--json` keeps a flat array in the envelope.

## Verification sources

- FSDB Reader API docs: `/opt/verdi/doc/HTML/pdf/verdi_fsdb_reader.pdf`
  - traversal overview and `ffrReadScopeTree` vs `ffrReadScopeVarTree`: p. 25, p. 28, p. 30-31
  - `ffrOpen3`: p. 104
  - `ffrReadScopeTree` / `ffrReadScopeTree2`: p. 107-108
  - `ffrReadScopeVarTree` / `ffrReadScopeVarTree2`: p. 108-109
  - callback data mapping: p. 144
- Headers for exact enum/field names:
  - `/opt/verdi/share/FsdbReader/ffrAPI.h`
  - `/opt/verdi/share/FsdbReader/fsdbShr.h`
