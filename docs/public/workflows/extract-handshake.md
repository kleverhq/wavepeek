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

Use `extract` commands when you need a compact table of transfer-like events from a waveform. Use `extract ahb` for pipelined AHB-Lite or AHB5 address, completion, reset, and synchronization events. Use `extract axi` for AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channels. AXI5 and ACE5-LiteDVM can include DVM `ac` and `cr` transfers without `cd`; AXI5-Lite, ACE5-Lite, and ACE5-LiteACP use only the five base channels. Use `extract generic` for other protocol-neutral handshakes.

For AHB, map manager-facing `HREADY` together with the clock and address-phase controls. Do not substitute subordinate-local `HREADYOUT`:

```text
$ wavepeek extract ahb --waves path/to/dump.vcd \
    --scope top.dut \
    --profile ahb-lite \
    --map hclk=clk \
    --map hresetn=rst_n \
    --include '^ahb_'
name: ahb
profile: ahb-lite
issue: C
include_stall: false
include_idle: false
include_busy: false
initial_data_phase: desynchronized
mappings:
  hclk = clk
  hresetn = rst_n
  htrans = ahb_htrans
  hready = ahb_hready
events:
@25ns sample@24999ps [address nonseq read] htrans=2'h2 hwrite=1'h0 haddr=32'h00000040
@35ns sample@34999ps [data-complete read] hresp=1'h0 hrdata=32'hdeadbeef
```

The AHB walker retains a pending phase across low-`HREADY` cycles, emits a real completion when it advances, and warms state before `--from`. Add `--include-stall`, `--include-idle`, or `--include-busy` only when cycle-level rows are useful.

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

For several generic source types, or for a reusable AHB or AXI profile/mapping setup, write a source file and pass `--source`. Use `wavepeek schema --input` to fetch the exact input schema for that file.
