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

Use `extract` commands when you need row output that combines event selection, predicate evaluation, and payload sampling. `extract generic` is protocol-neutral. `extract atb` expands AMBA ATB transfer, flush, and synchronization-request conditions into generic extraction sources. `extract axi` expands AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channels for common bus debug.

For exact syntax and flags, run `wavepeek help extract atb`, `wavepeek help extract axi`, or `wavepeek help extract generic`.

## `extract atb`

`extract atb` emits stateless AMBA ATB interface events using Arm IHI 0032C Issue C definitions. The supported profiles are `atb-a`, `atb-b`, and `atb-c`; the default is `atb-c`. The CLI and source parser also accept underscore aliases, and accept legacy `atbv1.0` and `atbv1.1` as aliases for ATB-A and ATB-B. Generated schemas accept only the canonical hyphenated profile names.

Every configuration maps `atclk` and at least one complete handshake pair. A transfer event requires known-true `atvalid && atready`; a flush event requires known-true `afvalid && afready`. Mapping `syncreq` on ATB-B or ATB-C adds a synchronization-request source whose predicate is known-true `syncreq`, but does not replace the required transfer or flush pair. `atresetn` is optional; when mapped, each predicate is additionally gated by known-true `atresetn` at the pre-edge sample point. Unknown control or reset values do not produce the affected event.

Transfer payload mappings are optional and stay raw. A transfer row can include mapped `atbytes`, `atdata`, and `atid` values in that order. `ATBYTES + 1` is the number of valid low-order `ATDATA` bytes, but the command preserves the complete observed vectors without trimming or masking; upper bytes outside that count are observations and are not claimed as protocol-valid trace bytes. This permits handshake-only extraction and 8-bit `ATDATA` interfaces where `ATBYTES` is absent. Flush and synchronization-request rows have empty payloads. `ATID` values are observations rather than decoded trigger or protocol semantics.

The extraction profiles deliberately exclude `atclken` and `atwakeup`. ATB-A also excludes `syncreq`; ATB-B and ATB-C accept it. The initial ATB-B and ATB-C extraction signal sets are otherwise identical.

Map signals explicitly or select automatic candidates with include regexes. Matching is case-insensitive after separator normalization, accepts leading interface prefixes and common direction suffixes, and requires a complete standard-signal suffix. Explicit mappings win. With `--scope`, waveform mapping names and include candidates are relative to that scope.

```text
$ wavepeek extract atb --waves path/to/dump.vcd \
    --scope top.etm \
    --profile atb-c \
    --map atclk=trace_clk \
    --map atresetn=trace_reset_n \
    --include '^trace_(at|af|sync)'
name: atb
profile: atb-c
issue: C
mappings:
  atclk = trace_clk
  atresetn = trace_reset_n
  atvalid = trace_at_valid
  atready = trace_at_ready
  atbytes = trace_at_bytes
  atdata = trace_at_data
  atid = trace_at_id
  afvalid = trace_af_valid
  afready = trace_af_ready
  syncreq = trace_sync_req
events:
@25ns sample@24999ps [transfer] atbytes=2'h3 atdata=32'h44332211 atid=7'h10
@25ns sample@24999ps [flush]
@25ns sample@24999ps [sync-request]
```

When several event conditions are true at one edge, rows appear in `transfer`, `flush`, then `sync-request` order. Every sampled high `syncreq` produces a row independently, and repeated transfer or flush handshakes are preserved even when values do not change.

A source file can provide `profile`, `name`, `includes`, and `maps` with `kind: "extract.atb.source"`. Source-file mode conflicts with the corresponding command-line configuration options; time bounds and scope remain command-line settings.

ATB extraction does not reconstruct trace packets, derive byte counts, decode trace triggers, verify legal encodings, or infer cross-cycle transfer, flush, synchronization, or wake-up episodes. Use the raw event rows as evidence tied to their sampled edge.

Arm IHI 0032C sections 3.1-3.2 define ATB transfer sampling and the `ATVALID`/`ATREADY` handshake. Section 4.2 defines the flush handshake, section 4.4 defines synchronization requests, and Appendix A Table A-1 defines the interface signal matrix.

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

`extract` rows use `time` for the selected event timestamp and `sample_time` for the point where predicate and payload values are read. For `extract generic`, `sample_time` is one dump tick before the selected edge.

This matches common RTL debugging expectations: the row describes the values that caused the edge to be interesting, not values updated on the edge itself.

`--from` and `--to` bound event `time` values. A row at `--from` can still use a `sample_time` before `--from` if that sample point is inside the dump.

## Output modes

Human `extract atb` output starts with name, profile, issue, resolved mappings, and then event rows. `extract atb --json` emits `command: "extract atb"` with `name`, `profile`, `issue`, `mappings`, and `events`. JSONL puts ATB context on the `begin` record and streams one event per `item`. Add `--abs` to print canonical mapping and payload paths in human output.

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
