---
id: commands/value
title: Value command
description: Sample signal values at one or more explicit timestamps.
section: commands
see_also:
  - commands/overview
  - commands/info
  - commands/scope
  - commands/signal
  - reference/command-model
  - reference/machine-output
---
# Value command

A useful mental model: `value` is a rough CLI equivalent of an `initial` block that waits until each requested `--at` time and then `$display`s the selected signals — except it samples an existing dump instead of modifying the simulation.

Use `value` when you want explicit point snapshots instead of a time range.

It is the fastest way to answer questions like:

- what was `state` at reset release,
- what did `valid`, `ready`, and `data` look like on selected cycles,
- did two related signals agree at specific timestamps.

In practice, `value` usually comes after `info` (to get time units and bounds) and `scope` or `signal` (to find the right names).

For exact syntax and flags, run `wavepeek help value`.

## Start with one point snapshot

If you already know the canonical signal paths, sample them directly:

```text
$ wavepeek value --waves path/to/dump.vcd --at 10ns --signals top.clk,top.data
@10ns top.clk=1'h1 top.data=8'h0f
```

Use this when you already have full paths from `signal`, logs, or earlier queries.

## Sample several explicit points

Pass a comma-separated list as the single `--at` argument:

```text
$ wavepeek value --waves path/to/dump.vcd --at 5ns,10ns --scope top --signals clk,data
@5ns clk=1'h1 data=8'h00
@10ns clk=1'h1 data=8'h0f
```

Rows follow the `--at` order. Duplicate time points are preserved.

## Shorten deep names with `--scope`

When several signals live in the same scope, set that scope once and keep `--signals` relative:

```text
$ wavepeek value --waves path/to/dump.vcd --at 10ns --scope top --signals clk,data
@10ns clk=1'h1 data=8'h0f
```

This is usually the most convenient form for manual debugging.

## Keep short input names but print canonical paths

Add `--abs` when you want scope-relative input but fully qualified output:

```text
$ wavepeek value --waves path/to/dump.vcd --at 10ns --scope top --signals clk,data --abs
@10ns top.clk=1'h1 top.data=8'h0f
```

Use this when you plan to paste results into notes, bugs, or follow-up commands.

## Follow `sample_time` from event-driven commands

`change` and `property` JSON rows include both `time` and `sample_time`. Use `sample_time` for follow-up `value --at` queries when you want to inspect the values that were printed or evaluated by that row.

This matters for `--sample-mode pre-edge`: `time` remains the trigger edge, while `sample_time` is the point just before that edge. Querying `value --at <time>` can show the next-cycle payload instead of the values that made the `property` row match.

## Remember that sampling is state-at-time, not change-at-time

`value` returns the latest known value at or before each `--at` timestamp. A timestamp does not need to be a transition point:

```text
$ wavepeek value --waves path/to/dump.vcd --at 7ns --scope top --signals clk,data
@7ns clk=1'h1 data=8'h00
```

Use this for spot checks between edges or between visible value changes.

## Use JSON for scripts and agents

`--json` returns a stable machine-readable envelope. The `data` field is an ordered array of snapshots, matching the row shape used by `change`:

```text
$ wavepeek value --waves path/to/dump.vcd --at 5ns,10ns --scope top --signals clk,data --json
{"$schema":"https://kleverhq.github.io/wavepeek/wavepeek_v1.json","command":"value","data":[{"time":"5ns","signals":[{"path":"top.clk","value":"1'h1"},{"path":"top.data","value":"8'h00"}]},{"time":"10ns","signals":[{"path":"top.clk","value":"1'h1"},{"path":"top.data","value":"8'h0f"}]}],"diagnostics":[]}
```

Use this when another tool needs deterministic parsing instead of human formatting.

## Non-obvious behavior

- `--at` accepts one time token or a comma-separated list in one argument.
- `--at` order is preserved exactly, including duplicates.
- `--signals` order is preserved exactly, including duplicates.
- Without `--scope`, names in `--signals` are treated as canonical full paths.
- With `--scope`, names in `--signals` must stay scope-relative. Do not mix relative names and full paths in one request.
- `--abs` affects only human output. With `--json`, canonical paths are emitted either way.
- `--at` accepts dump start and dump end; both bounds are inclusive.
- Time tokens must be integer plus unit. `10ns` is valid; `10` and `1.5ns` are errors.
- Empty comma entries such as `5ns,,10ns` are errors.
- If a signal has no sampled value at or before a requested time, the command fails instead of inventing a default.
- `value` does not truncate output and does not use `--max`; result size is bounded by the number of times and signals you requested.
- When a timestamp comes from `change` or `property`, use the row's `sample_time` for follow-up payload inspection. `time` can be a trigger edge whose sampled values came from an earlier point.
- Values are printed as Verilog literals. Non-bit-vector dump encodings are currently rejected by this command.
