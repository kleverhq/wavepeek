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

Use `extract` commands when you need row output that combines event selection, predicate evaluation, and payload sampling. `extract apb` classifies APB Setup and Access states. `extract axi` expands AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channels. `extract generic` is protocol-neutral.

For exact syntax and flags, run `wavepeek help extract apb`, `wavepeek help extract axi`, or `wavepeek help extract generic`.

## `extract apb`

`extract apb` emits independent sampled APB events for the `apb3`, `apb4`, and `apb5` profiles from Arm IHI 0024E Issue E. The default is APB4. At the pre-edge sample point for `posedge pclk`, a Setup event is `psel && !penable`. A completed Access is `psel && penable && pready` in mapped-PREADY mode or `psel && penable` in implicit-HIGH mode. Add `--include-wait` in mapped mode to emit one `access-wait` row per cycle where `psel && penable && !pready`. If `presetn` is mapped, every predicate is also gated by sampled known-HIGH reset.

Mapped mode is the default and requires `pready`. Implicit-HIGH mode forbids both a `pready` mapping and `--include-wait`. Unknown `psel`, `penable`, `pready`, or mapped `presetn` values do not classify as true. Setup classification does not depend on `pready`. The command preserves repeated events and does not require or remember a preceding Setup phase.

Map lowercase standard names explicitly, select candidates with include regexes, or combine both. Explicit maps win. Auto-mapping requires a complete normalized signal-name suffix, so forms such as `paddr`, `p_addr`, `apb_paddr_i`, and `apb_p_addr_i` match `paddr`, while `paddrchk`, `psel0`, and `pselx` do not. Map one concrete Completer select such as `uart_psel` to canonical `psel`; indexed selects are not discovered as one combined bus.

All profiles require `pclk`, `psel`, `penable`, and `pwrite`. APB3 accepts the base APB3 signals. APB4 adds `pprot` and `pstrb`. APB5 adds `pnse`, request/data/response user signals, and the APB4 set. `pwakeup` and APB5 check/parity signals are outside extraction. Widths, sparse-write meaning, user-field meaning, and APB protocol conformance are not validated.

Every event includes sampled `pwrite` and derives `direction` as `read`, `write`, or `unknown`. Request fields can appear on every event. Read data, error response, and response-user fields appear only on completion. Known reads omit write data/user fields, known writes omit read data/user fields, and unknown direction preserves available direction-specific observations. Sampled vectors and X/Z literals are emitted unchanged.

```text
$ wavepeek extract apb --waves path/to/dump.vcd \
    --scope top.uart \
    --profile apb4 \
    --include '^uart_apb_' \
    --include-wait
name: apb
profile: apb4
issue: E
pready_mode: mapped
include_wait: true
mappings:
  pclk = uart_apb_pclk
  psel = uart_apb_psel
  penable = uart_apb_penable
  pwrite = uart_apb_pwrite
  pready = uart_apb_pready
events:
@20ns sample@19ns [setup write] pwrite=1'h1 paddr=16'h0040 pwdata=32'hdeadbeef
@30ns sample@29ns [access-wait write] pwrite=1'h1 paddr=16'h0040 pwdata=32'hdeadbeef
@40ns sample@39ns [access-complete write] pwrite=1'h1 paddr=16'h0040 pwdata=32'hdeadbeef pslverr=1'h0
```

A source file can provide `profile`, `pready_mode`, `include_wait`, `name`, `includes`, and `maps` with `kind: "extract.apb.source"`. The parser accepts profile and PREADY-mode values case-insensitively and accepts `implicit_high` as an alias; generated schemas accept canonical lowercase values only. Source-file mode conflicts with the corresponding CLI flags. Time bounds, scope, row limit, output mode, and absolute-path rendering remain command-line concerns.

APB extraction is stateless sampled-event classification. It does not assemble transactions, pair Setup with Access, count waits into transaction records, decode registers, infer one Completer from several selects, or validate protocol sequencing, stability, parity, or errors.

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

`extract` rows use `time` for the selected event timestamp and `sample_time` for the point where predicate and payload values are read. `sample_time` is one dump tick before the selected edge.

This matches common RTL debugging expectations: the row describes the values that caused the edge to be interesting, not values updated on the edge itself.

`--from` and `--to` bound event `time` values. A row at `--from` can still use a `sample_time` before `--from` if that sample point is inside the dump.

## Output modes

Human `extract apb` output starts with name, profile, Issue E, PREADY mode, effective wait setting, resolved mappings, and then event rows. `extract apb --json` uses `command: "extract apb"` and exposes the same context plus `events`; JSONL puts the context on `begin` and one event on each `item` row. Profile, mode, wait setting, event, direction, mapping keys, and payload keys are schema-constrained. Add `--abs` to print canonical mapping and payload paths in human output.

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
