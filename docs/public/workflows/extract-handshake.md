---
id: workflows/extract-handshake
title: Extract handshakes from synchronous bus
description: Turn ready/valid clock edges into deterministic row output.
section: workflows
see_also:
  - commands/extract
  - commands/property
  - commands/value
  - reference/expression-language
---
# Extract handshakes from synchronous bus

Use `extract` commands when you need a compact table of transfer-like events from a waveform. Use `extract axi` for AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channels. Use `extract axistream` for one AXI4-Stream or AXI5-Stream interface, and `extract generic` for other protocol-neutral handshakes.

For AXI, map the clock and let include regexes find standard channel signals:

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

For AXI-Stream, map one interface. The default mapped mode requires `tready`:

```text
$ wavepeek extract axistream --waves path/to/dump.vcd \
    --scope top.dut \
    --profile axi4-stream \
    --map aclk=clk \
    --map aresetn=rst_n \
    --include '^video_out_'
name: axistream
profile: axi4-stream
issue: B
tready_mode: mapped
mappings:
  aclk = clk
  aresetn = rst_n
  tvalid = video_out_tvalid
  tready = video_out_tready
  tdata = video_out_tdata
transfers:
@25ns sample@24999ps tdata=32'hdeadbeef
```

Use `--tready-mode implicit-high` only when the physical stream interface omits `TREADY`; do not map `tready` in that mode. The AXI-Stream adapter emits every handshake row, including identical consecutive payloads, and does not reconstruct packets from `tlast`.

For generic handshakes, start by selecting a scope and an edge-only event:

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

For several generic source types, or for a reusable AXI or AXI-Stream profile/mapping setup, write a source file and pass `--source`. Use `wavepeek schema --input` to fetch the exact input schema for that file.
