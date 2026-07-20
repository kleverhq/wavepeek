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

Use `extract` commands when you need row output that combines event selection, protocol state, predicate evaluation, or payload sampling. `extract ahb` follows the pipelined AHB address/data relationship. `extract axi` reports AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channel transfers. `extract generic` is protocol-neutral.

For exact syntax and flags, run `wavepeek help extract ahb`, `wavepeek help extract axi`, or `wavepeek help extract generic`.

## `extract ahb`

`extract ahb` emits deterministic manager-facing AHB pipeline events using Arm IHI 0033C, Issue C. The supported profiles are `ahb-lite` and `ahb5`; the default is `ahb-lite`. The CLI and source parser also accept `ahb_lite`, while generated schemas accept canonical profile names only.

AHB overlaps each transfer's address phase with the previous transfer's data phase. The extractor therefore keeps one pipeline slot instead of treating every high-`HREADY` clock as a completed transfer. At each rising `HCLK` edge it samples mapped values one dump tick before the edge, then:

1. if `HREADY` is high, completes a previously accepted data phase;
2. on that same edge, accepts current `HTRANS=NONSEQ` or `SEQ` into the next data-phase slot;
3. treats `IDLE` and `BUSY` as slots without a following data phase.

When a completion and a new address occur on the same edge, the `data-complete` row appears before the `address` row. A low `HREADY` extends a pending data phase; `--include-stall` emits one `data-stall` row for each such sampled cycle. `--include-idle` and `--include-busy` independently add address-slot rows. These optional rows are disabled by default.

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

The required standard mappings are `hclk`, `htrans`, `hready`, and `hwrite`. Optional AHB-Lite mappings are `hresetn`, `haddr`, `hburst`, `hmastlock`, `hprot`, `hsize`, `hauser`, `hwdata`, `hwstrb`, `hwuser`, `hrdata`, `hruser`, `hbuser`, and `hresp`. AHB5 additionally accepts `hnonsec`, `hexcl`, `hmaster`, and `hexokay`. Standard keys are lowercase.

Map signals explicitly with repeated `--map standard=waveform` options, auto-map candidates selected by repeated `--include REGEX`, or combine both. Explicit mappings override auto-mapping. Normalized full-suffix matching accepts common forms such as `haddr`, `h_addr`, `ahb_haddr_i`, and `ahb_h_addr_i`. It does not map `hreadyout` to `hready`, and it ignores parity/check lookalikes. With `--scope`, mapped waveform names and include regexes are scope-relative.

The command uses the manager-facing selected `HREADY`. It does not accept subordinate-local `HREADYOUT`, `HSELx`, or parity/check signals as standard mappings. It also does not reconstruct bursts, join address and data rows, assign transaction IDs, count stalls, mask byte lanes, validate protocol rules, or infer completions hidden by unknown history.

Mapped active-low `HRESETn` clears pending state. A consecutive known-low reset episode emits one `reset` boundary. Unknown reset, unknown `HREADY`, or unknown accepted `HTRANS` history moves the walker to `desynchronized`; one boundary is emitted when that state is entered. A later known-high `HREADY` edge establishes the next slot from current `HTRANS` without inventing an old completion. Unknown `HWRITE` preserves an accepted phase with `direction: "unknown"` rather than discarding it.

Before an inclusive `--from` bound, the walker warms from dump start without emitting or counting rows. Machine context records the resulting `initial_data_phase` as `empty`, `desynchronized`, or `pending`. A pending state retains the earlier accepted address snapshot so a later in-range completion has explicit provenance.

Address and BUSY rows contain mapped address/control observations. IDLE rows contain only signals valid for IDLE. A write stall or completion can contain write data, strobes, and write user data. A known-direction read stall does not claim read data validity. Read completion preserves mapped `hrdata`, including on ERROR. `hruser`, `hbuser`, and AHB5 `hexokay` appear only when mapped `hresp` is known low. Unknown direction preserves both mapped data sides as observations. X/Z values remain sized Verilog literals.

A source file can provide canonical or aliased `profile`, `name`, `include_stall`, `include_idle`, `include_busy`, `includes`, and `maps` with `kind: "extract.ahb.source"`. Source-file mode conflicts with the matching CLI configuration flags; time bounds and scope remain command-line options. The exact source contract comes from `wavepeek schema --input`.

Machine-readable AHB output is typed by profile and event. JSON uses `command: "extract ahb"` and carries Issue C context, inclusion flags, initial pipeline state, canonical mappings, and ordered `events`. JSONL begins with the same context, emits one event per `item`, then diagnostics and an `end` summary. Only emitted public events count toward `--max`, so a limit can stop between a same-edge completion/address pair.

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

`extract` rows use `time` for the selected event timestamp and `sample_time` for the point where predicate and payload values are read. For `extract ahb` and `extract generic`, `sample_time` is one dump tick before the selected edge. `extract axi` uses the generic pre-edge engine and follows the same sampling rule.

This matches common RTL debugging expectations: the row describes the values that caused the edge to be interesting, not values updated on the edge itself.

`--from` and `--to` bound event `time` values. A row at `--from` can still use a `sample_time` before `--from` if that sample point is inside the dump.

## Output modes

Human `extract ahb` output starts with name, profile, issue, inclusion flags, initial data-phase state, resolved mappings, and then event rows. Add `--abs` to print canonical mapping and payload paths. JSON and JSONL carry the full retained pending-address snapshot when `initial_data_phase` is `pending`.

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

For `extract generic`, `--max` limits emitted rows across all sources after sorting by event time and source declaration order. For `extract ahb`, it limits public event rows after warm-up and completion-before-address ordering. For `extract axi`, it limits ready/valid transfer rows. `--max unlimited` disables truncation and emits a warning diagnostic. Empty results and truncation use the same coded diagnostic model as other waveform commands.
