---
id: workflows/extract-handshake-rows
title: Extract handshake rows
description: Turn ready/valid clock edges into deterministic row output.
section: workflows
see_also:
  - commands/extract-generic
  - commands/property
  - commands/value
  - reference/expression-language
---
# Extract handshake rows

Use `extract generic` when you need a compact table of transfer-like events from a waveform.

Start by selecting a scope and an edge-only event:

```text
$ wavepeek extract generic --waves path/to/dump.vcd \
    --scope top.dut \
    --on "posedge clk iff rst_n" \
    --when "valid && ready" \
    --payload data,last
@25ns sample@24999ps data=32'hdeadbeef last=1'h1
```

The event `time` is the clock edge. Payload values are sampled at `sample_time`, one dump tick before the edge.

For automation, prefer JSONL when the result may be large:

```text
$ wavepeek extract generic --waves path/to/dump.vcd \
    --scope top.dut \
    --on "posedge clk iff rst_n" \
    --when "valid && ready" \
    --payload data,last \
    --jsonl
```

For several source types in one pass, write a source file and pass `--source`. Use `wavepeek schema --input` to fetch the exact input schema for that file.
