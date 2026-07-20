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

Use `extract` commands when you need row output that combines event selection, predicate evaluation, and payload sampling. `extract generic` is protocol-neutral. `extract axi` expands AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channels into generic extraction sources. `extract axistream` does the same for one AXI4-Stream or AXI5-Stream interface.

For exact syntax and flags, run `wavepeek help extract axi`, `wavepeek help extract axistream`, or `wavepeek help extract generic`.

## `extract axi`

`extract axi` emits one row per completed AXI-family transfer on each mapped ready/valid channel. Supported profiles are `axi3`, `axi4`, `axi4-lite`, `axi5`, `axi5-lite`, `ace`, `ace-lite`, `ace5`, `ace5-lite`, `ace5-lite-dvm`, and `ace5-lite-acp`; the default profile is `axi4`. AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 use Arm IHI 0022H.c Issue H.c signal definitions. AXI5, AXI5-Lite, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP use Arm IHI 0022L Issue L ready/valid signal definitions. A completed transfer requires both channel `VALID` and channel `READY` to be true at the pre-edge sample point for `posedge aclk`. If `aresetn` is mapped, it must also be true at that sample point.

The CLI and source parser accept `ace5_lite` for ACE5-Lite. ACE5-LiteDVM additionally accepts `ace5-litedvm`, `ace5_litedvm`, and `ace5_lite_dvm`; ACE5-LiteACP additionally accepts `ace5-liteacp`, `ace5_liteacp`, and `ace5_lite_acp`. Generated schemas accept canonical hyphenated profile names only.

Map signals explicitly with repeated `--map standard=waveform` options, auto-map candidates selected by repeated `--include REGEX`, or combine both. Standard signal names are lowercase AXI names such as `awvalid`, `awready`, `wdata`, `rresp`, and `acvalid`; explicit mappings override auto-mapping for the same standard signal. With `--scope`, mapped waveform names and include regexes are scope-relative.

AXI5 and ACE5-LiteDVM add the `ac` and `cr` DVM channels after the base `aw`, `w`, `b`, `ar`, and `r` channels when those signals are mapped; neither adds a `cd` channel. AXI5-Lite, ACE5-Lite, and ACE5-LiteACP use only the five base channels. ACE and ACE5 add the `ac`, `cr`, and `cd` coherency channels. ACE-Lite uses only the five base channels and accepts its read/write address additions, including optional `awunique`. ACE5 does not accept the removed `awbar` or `arbar` signals. Optional and conditional payload signals are extracted when mapped and are not required.

AXI-family extraction reports functional ready/valid channel transfers only. The Issue L profiles do not accept credited transport signals. Extraction does not include standalone `rack` or `wack` acknowledgements, interface-level wakeup or coherency-connection signals, QoS-accept controls, or check/parity signals. It does not reconstruct bursts, ordering, DVM messages, or coherency state.

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

## `extract axistream`

`extract axistream` emits one row per completed transfer on one mapped stream interface. Its profiles are `axi4-stream` and `axi5-stream`; both use Arm IHI 0051B Issue B and the default is `axi4-stream`. The CLI and source parser accept profile names case-insensitively and accept the underscore aliases `axi4_stream` and `axi5_stream`. Generated schemas accept canonical hyphenated profile names only.

Map `aclk`, optional `aresetn`, handshake signals, and any payload signals with `--map`, `--include`, or both. Accepted payload standard names are `tdata`, `tstrb`, `tkeep`, `tlast`, `tid`, `tdest`, and `tuser`. Payload signals are optional and omitted from mappings and rows when unmapped. `twakeup` and check/parity signals are not part of transfer extraction.

The default `--tready-mode mapped` requires a `tready` mapping and recognizes a transfer when `tvalid && tready` is true at the pre-edge sample point for `posedge aclk`. Use `--tready-mode implicit-high` only when the physical interface omits `TREADY`; this mode forbids a `tready` mapping and recognizes transfers from `tvalid`. If `aresetn` is mapped, it gates either predicate. `aclk` and `tvalid` are always required.

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
  tlast = video_out_tlast
transfers:
@25ns sample@24999ps tdata=32'hdeadbeef tlast=1'h1
```

One invocation maps one interface. Rows do not contain a synthetic channel field, and a handshake-only transfer still emits its event and sample timestamps. The command does not reconstruct packets, interpret byte qualifiers, check protocol timing, or validate AXI5-Stream wake-up or parity.

A source file uses singular kind `extract.axistream.source` and can provide `profile`, `tready_mode`, `name`, `includes`, and `maps`. The defaults are `axi4-stream`, `mapped`, and `axistream`. Source-file mode conflicts with those CLI mapping/configuration options; time bounds and scope remain command-line options.

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

`extract axistream` uses the same context-first layout plus `tready_mode`, but its rows have no `channel`. JSON uses `command: "extract axistream"`; JSONL puts name, profile, Issue B, TREADY mode, and mappings in the `begin` context and emits one independently profile-typed transfer per `item`.

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
