---
id: commands/extract
title: Extract command
description: Extract events, handshakes, and transfers from synchronous waveform signals.
section: commands
see_also:
  - commands/overview
  - commands/property
  - commands/value
  - reference/command-model
  - reference/expression-language
  - reference/machine-output
---
# Extract command

Use `extract` commands when you need row output that combines event selection, predicate evaluation, and payload sampling. The current extractor is `extract generic`, a protocol-neutral command for ready/valid handshakes, FIFO pushes and pops, and other clocked transfer-like events.

For exact syntax and flags, run `wavepeek help extract generic`.

## `extract generic`

`extract generic` emits one row per matching synchronous event. It avoids the manual workflow of running `property`, extracting `sample_time` values, running `value`, and joining the results externally.

The command always samples at the pre-edge sample point. It does not support `--sample-mode`.

## Single-source extraction

A single-source query defines the source directly on the command line:

```text
$ wavepeek extract generic --waves path/to/dump.vcd \
    --scope top.dut \
    --on "posedge clk iff rst_n" \
    --when "valid && ready" \
    --payload data,last
@25ns sample@24999ps data=32'hdeadbeef last=1'h1
```

`--on` selects candidate event timestamps. `extract generic` only accepts edge-only event expressions, such as `posedge clk`, `negedge clk`, or `edge clk`, with optional `iff` gating. Wildcard triggers, plain signal triggers, and mixed level/edge triggers are rejected.

`--when` is a Boolean expression evaluated at the pre-edge sample point. `--payload` is the ordered list of signals sampled at the same pre-edge point. The command emits a row only when the event matches and `--when` is true.

## Source files

Use `--source` when one query should extract several source types from the same dump:

```json
{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.1.json",
  "kind": "extract.generic.sources",
  "sources": [
    {
      "name": "fifo.write",
      "on": "posedge wclk iff wrst_n",
      "when": "wvalid && wready",
      "payload": ["wdata", "wlast"]
    },
    {
      "name": "fifo.read",
      "on": "posedge rclk iff rrst_n",
      "when": "rvalid && rready",
      "payload": ["rdata"]
    }
  ]
}
```

Then run:

```text
$ wavepeek extract generic --waves path/to/dump.vcd --scope top.dut --source sources.json --jsonl
```

Source names must be unique within the file. Payload names must be unique within one source. Source-file mode conflicts with `--name`, `--on`, `--when`, and `--payload` because those fields come from the file. The source-file contract is defined by `wavepeek schema --input`.

## Pre-edge sampling

`extract` rows use `time` for the selected event timestamp and `sample_time` for the point where predicate and payload values are read. For `extract generic`, `sample_time` is one dump tick before the selected edge.

This matches common RTL debugging expectations: the row describes the values that caused the edge to be interesting, not values updated on the edge itself.

`--from` and `--to` bound event `time` values. A row at `--from` can still use a `sample_time` before `--from` if that sample point is inside the dump.

## Output modes

Human output is compact and row-oriented:

```text
@25ns sample@24999ps data=32'hdeadbeef last=1'h1
```

For multi-source output, the source name appears after `sample@...`:

```text
@25ns sample@24999ps [fifo.write] wdata=32'hdeadbeef wlast=1'h1
```

Add `--abs` to print canonical payload paths in human output.

`--json` emits the standard envelope with `command: "extract generic"` and an array of rows. `--jsonl` streams `begin`, `item`, `diagnostic`, and `end` records; each item row has `time`, `sample_time`, `source`, and ordered `payload` values.

Repeated events are preserved even when payload values do not change. `extract` is not a delta command.

## Limits and diagnostics

`--max` limits emitted rows across all sources after sorting by event time and source declaration order. `--max unlimited` disables truncation and emits a warning diagnostic. Empty results and truncation use the same coded diagnostic model as other waveform commands.
