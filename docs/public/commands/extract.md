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

Use `extract` commands when you need row output that combines event selection, predicate evaluation, and payload sampling. `extract generic` is protocol-neutral. `extract axi` expands AXI3, AXI4, and AXI4-Lite ready/valid channels into generic extraction sources for common bus debug.

For exact syntax and flags, run `wavepeek help extract axi` or `wavepeek help extract generic`.

## `extract axi`

`extract axi` emits one row per completed AXI transfer on each mapped ready/valid channel. Supported profiles are `axi3`, `axi4`, and `axi4-lite`; the default profile is `axi4`. A completed transfer requires both channel `VALID` and channel `READY` to be true at the pre-edge sample point for `posedge aclk`. If `aresetn` is mapped, it must also be true at that sample point.

Map signals explicitly with repeated `--map standard=waveform` options, auto-map candidates selected by repeated `--include REGEX`, or combine both. Standard signal names are lowercase AXI names such as `awvalid`, `awready`, `wdata`, and `rresp`; explicit mappings override auto-mapping for the same standard signal. With `--scope`, mapped waveform names and include regexes are scope-relative.

```text
$ wavepeek extract axi --waves path/to/dump.vcd \
    --scope top.dut \
    --profile axi4-lite \
    --map aclk=clk \
    --map aresetn=rst_n \
    --include '^axi_(aw|w|b|ar|r)_'
name: axi
profile: axi4-lite
issue: H.c
mappings:
  aclk = clk
  aresetn = rst_n
  awaddr = axi_aw_addr
  awvalid = axi_aw_valid
  awready = axi_aw_ready
transfers:
@25ns sample@24999ps [aw] awaddr=32'h00000040
```

A source file can provide `profile`, `name`, `includes`, and `maps` with `kind: "extract.axi.source"`. Source-file mode conflicts with `--profile`, `--name`, `--map`, and `--include`; time bounds and scope still come from the command line.

Machine-readable AXI output is typed by profile and channel. JSON transfer rows and JSONL item rows include `profile`; the schemas enumerate allowed payload keys for each profile/channel pair while allowing omitted keys for unmapped signals.

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
  "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.2.json",
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

Human `extract axi` output starts with name, profile, issue, resolved mappings, and then transfer rows. Add `--abs` to print canonical mapping and payload paths in human output.

`extract axi --json` emits the standard envelope with `command: "extract axi"` and a data object containing `name`, `profile`, `issue`, `mappings`, and `transfers`. `extract axi --jsonl` streams a `begin` record with AXI context, one transfer per `item`, optional diagnostics, and an `end` summary.

Human `extract generic` output is compact and row-oriented:

```text
@25ns sample@24999ps data=32'hdeadbeef last=1'h1
```

For multi-source output, the source name appears after `sample@...`:

```text
@25ns sample@24999ps [fifo.write] wdata=32'hdeadbeef wlast=1'h1
```

Add `--abs` to print canonical payload paths in human output.

`extract generic --json` emits the standard envelope with `command: "extract generic"` and an array of rows. `extract generic --jsonl` streams `begin`, `item`, `diagnostic`, and `end` records; each item row has `time`, `sample_time`, `source`, and ordered `payload` values.

Repeated events are preserved even when payload values do not change. `extract` is not a delta command.

## Limits and diagnostics

`--max` limits emitted rows across all sources after sorting by event time and source declaration order. `--max unlimited` disables truncation and emits a warning diagnostic. Empty results and truncation use the same coded diagnostic model as other waveform commands.
