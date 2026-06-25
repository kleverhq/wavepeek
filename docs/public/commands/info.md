---
id: commands/info
title: Info command
description: Show waveform metadata.
section: commands
see_also:
  - commands/overview
  - reference/command-model
  - reference/machine-output
---
# Info command

Use `info` as the first sanity check on a dump. It answers three questions quickly:

- what time unit the dump uses,
- where the dump starts,
- where the dump ends.

That is usually enough to choose correct `--at`, `--from`, and `--to` values for later `value`, `change`, or `property` queries.

For exact syntax and flags, run `wavepeek help info`.

## Getting the dump time base before querying

Run `info` on the waveform file:

```text
$ wavepeek info --waves path/to/dump.vcd
time_unit: 1ns
time_start: 0ns
time_end: 10ns
```

Exact values depend on the dump. Use this when you are about to write a time-based query and do not want to guess the unit.

## Getting the same metadata in a stable machine-readable form

Add `--json`:

```text
$ wavepeek info --waves path/to/dump.vcd --json
{"$schema":"https://kleverhq.github.io/wavepeek/wavepeek_v2.0.json","command":"info","data":{"time_unit":"1ns","time_start":"0ns","time_end":"10ns"},"diagnostics":[]}
```

Use this in scripts and agents. The exact JSON shape is defined by `wavepeek schema` and explained in `reference/machine-output`.

## Non-obvious outcomes

- `time_start` and `time_end` are already normalized to the dump's `time_unit`. These are the bounds the rest of `wavepeek` uses.
- `info` is intentionally minimal. It does not print scope counts, signal counts, or extra time-precision fields.
- Dumps do not have to start at zero, and they do not all share the same unit.
- `info` does not emit truncation diagnostics; the result is always a small fixed metadata record.
