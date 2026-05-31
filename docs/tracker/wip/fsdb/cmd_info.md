# FSDB Reader API for `wavepeek info`

Scope: only the data needed for the existing `info` command contract:

- `time_unit`
- `time_start`
- `time_end`

No extra FSDB-specific fields are planned for `info`.

## Recommended FSDB Reader calls

Use the FSDB Reader API from `$VERDI_HOME/share/FsdbReader`.

Minimal call sequence:

```cpp
ffrObject::ffrInfoSuppress(1);
ffrObject::ffrWarnSuppress(1);

if (!ffrObject::ffrIsFSDB(path)) {
    // report unsupported / not an FSDB file
}

ffrFSDBInfo info = {};
fsdbRC rc = ffrObject::ffrGetFSDBInfo(path, info);
if (rc != FSDB_RC_SUCCESS) {
    // report FSDB metadata read failure
}
```

For `cmd info`, `ffrGetFSDBInfo` is the primary API. It provides the required metadata without opening/traversing the waveform.

## Source fields

Map FSDB Reader fields to `wavepeek info` fields as follows:

| `wavepeek info` field | FSDB Reader source |
|---|---|
| `time_unit` | `info.scale_unit` |
| `time_start` | `info.min_xtag` |
| `time_end` | `info.max_xtag` |

For digital FSDB files, read `info.min_xtag` / `info.max_xtag` through the `hltag` representation:

```cpp
uint64_t raw = (uint64_t(tag.hltag.H) << 32) | uint64_t(tag.hltag.L);
```

## Formatting rules

`wavepeek` currently emits normalized timestamps as:

```text
<raw_timestamp * time_unit_factor><time_unit_suffix>
```

So for FSDB:

1. Parse `info.scale_unit` as:
   - decimal integer factor
   - unit suffix
2. Keep the original scale string as `time_unit`.
3. Convert `min_xtag` and `max_xtag` to raw integer ticks.
4. Multiply each raw tick by the parsed factor.
5. Append the parsed suffix.

Example:

```text
info.scale_unit = "100ps"
info.min_xtag.hltag = 0:0
info.max_xtag.hltag = 0:200
```

Output:

```text
time_unit: 100ps
time_start: 0ps
time_end: 20000ps
```

Another example:

```text
info.scale_unit = "1ns"
max raw tick = 14000
```

Output:

```text
time_unit: 1ns
time_end: 14000ns
```

## Accepted time units

Use the same unit vocabulary as the existing wavepeek time parser/formatter:

- `zs`
- `as`
- `fs`
- `ps`
- `ns`
- `us`
- `ms`
- `s`

If `info.scale_unit` cannot be parsed as `<integer><unit>`, treat the FSDB metadata as unsupported/invalid for `cmd info`.

## Overflow and unsupported cases

Required checks:

- `raw = (H << 32) | L` must fit in `u64` by construction.
- `raw * scale_factor` must not overflow `u64`.
- `scale_factor` must be non-zero.
- `scale_unit` suffix must be one of the accepted units.

For non-digital FSDB variants where time tags are represented as float/double rather than `hltag`, do not guess. Return an unsupported-file/unsupported-time-tag error until explicit analog handling is designed.

## Observed sample behavior

Known sample observation from `$VERDI_HOME` FSDB files:

```text
scale_unit=100ps, max_xtag=200 -> time_end=20000ps
scale_unit=1ns,   max_xtag=14000 -> time_end=14000ns
scale_unit=1ps,   max_xtag=100000 -> time_end=100000ps
```

This matches the existing `wavepeek` behavior for VCD/FST: raw dump timestamps are multiplied by the dump time unit factor and rendered with the unit suffix.
