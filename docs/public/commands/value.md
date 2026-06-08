---
id: commands/value
title: Value command
description: Sample signal values at one timestamp.
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

A useful mental model: `value` is a rough CLI equivalent of an `initial` block that waits until `<at>` and then `$display`s the selected signals — except it samples an existing dump instead of modifying the simulation.

Use `value` when you want one trustworthy snapshot instead of a time range.

It is the fastest way to answer questions like:

- what was `state` at reset release,
- what did `valid`, `ready`, and `data` look like on this cycle,
- did two related signals agree at a specific timestamp.

In practice, `value` usually comes after `info` (to get time units and bounds) and `scope` or `signal` (to find the right names).

For exact syntax and flags, run `wavepeek help value`.

## Start with one point-in-time snapshot

If you already know the canonical signal paths, sample them directly:

```text
$ wavepeek value --waves path/to/dump.vcd --at 10ns --signals top.clk,top.data
@10ns
top.clk 1'h1
top.data 8'h0f
```

Use this when you already have full paths from `signal`, logs, or earlier queries.

## Shorten deep names with `--scope`

When several signals live in the same scope, set that scope once and keep `--signals` relative:

```text
$ wavepeek value --waves path/to/dump.vcd --at 10ns --scope top --signals clk,data
@10ns
clk 1'h1
data 8'h0f
```

This is usually the most convenient form for manual debugging.

## Keep short input names but print canonical paths

Add `--abs` when you want scope-relative input but fully qualified output:

```text
$ wavepeek value --waves path/to/dump.vcd --at 10ns --scope top --signals clk,data --abs
@10ns
top.clk 1'h1
top.data 8'h0f
```

Use this when you plan to paste results into notes, bugs, or follow-up commands.

## Remember that sampling is state-at-time, not change-at-time

`value` returns the latest known value at or before `--at`. The timestamp does not need to be a transition point:

```text
$ wavepeek value --waves path/to/dump.vcd --at 7ns --scope top --signals clk,data
@7ns
clk 1'h1
data 8'h00
```

Use this for spot checks between edges or between visible value changes.

## Use JSON for scripts and agents

`--json` returns a stable machine-readable envelope:

```text
$ wavepeek value --waves path/to/dump.vcd --at 10ns --scope top --signals clk,data --json
{"$schema":"https://raw.githubusercontent.com/kleverhq/wavepeek/main/schema/wavepeek_v0.json","command":"value","data":{"time":"10ns","signals":[{"path":"top.clk","value":"1'h1"},{"path":"top.data","value":"8'h0f"}]},"warnings":[]}
```

Use this when another tool needs deterministic parsing instead of human formatting.

## Non-obvious behavior

- `--signals` order is preserved exactly, including duplicates.
- Without `--scope`, names in `--signals` are treated as canonical full paths.
- With `--scope`, names in `--signals` must stay scope-relative. Do not mix relative names and full paths in one request.
- `--abs` affects only human output. With `--json`, canonical paths are emitted either way.
- `--at` accepts dump start and dump end; both bounds are inclusive.
- Time tokens must be integer plus unit. `10ns` is valid; `10` and `1.5ns` are errors.
- If a signal has no sampled value at or before the requested time, the command fails instead of inventing a default.
- `value` does not truncate output and does not use `--max`; result size is bounded by the signals you requested.
- Values are printed as Verilog literals. Non-bit-vector dump encodings are currently rejected by this command.
