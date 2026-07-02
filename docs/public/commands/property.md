---
id: commands/property
title: Property command
description: Evaluate expressions on event-selected timestamps.
section: commands
see_also:
  - commands/overview
  - reference/command-model
  - reference/expression-language
  - troubleshooting/clock-edge-sampling
---
# Property command

Use `property` when you want answers like "when did this condition become true?" instead of raw signal dumps.

A good mental model is a lightweight SystemVerilog event-driven monitor:

- `--on` uses SystemVerilog-style event semantics, roughly the same surface you would write inside `@(...)`, but without the outer `@` and parentheses.
- `--eval` uses SystemVerilog-style value and logical expression semantics. By default, edge-triggered checks use pre-edge sampling: the row timestamp stays at the trigger edge, but `--eval` reads values from immediately before that edge. Use `--sample-mode native` when you intentionally want dump values from the selected timestamp.
- In practice, `--eval` supports 4-state values and the usual useful SV-style operators: logical operators, bitwise operators, comparisons and equalities, arithmetic and shifts, casts, bit-select and part-select, concatenation and replication, and related expression forms. Final property decisions are then reduced to a Boolean true/false result for capture.
- `wavepeek` supports that SV-like surface as a defined dump-oriented contract, not as full temporal SVA. The exact supported syntax and semantics live in `reference/expression-language`.

The command works in two steps:

- `--on` selects timestamps worth checking.
- `--eval` evaluates a logical expression at those timestamps.

Rough SystemVerilog pseudocode for:

```text
wavepeek property --on 'posedge clk' --eval ready
```

would look like this:

```systemverilog
bit prev = ready_at_range_start;

always @(posedge clk) begin
  bit curr = ready;
  if (!prev && curr) $display("@%0t assert", $time);
  if ( prev && !curr) $display("@%0t deassert", $time);
  prev = curr;
end
```

`--capture match` is closer to "print whenever the condition is true at a selected event". `switch`, `assert`, and `deassert` are closer to edge-detecting that Boolean result over the selected event stream.

This is useful for questions like:

- when `ready` asserted on clock edges,
- when `valid && ready` stopped being true,
- whether a state predicate ever matched in a time window.

For exact syntax and flags, run `wavepeek help property`.

## Start with the default: only report transitions

If you do not pass `--capture`, `property` uses `switch` mode. That means it prints only transitions:

- `assert` when the property becomes true,
- `deassert` when it becomes false.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on 'posedge clk' --eval ready
@15ns sample@14ns assert
@25ns sample@24ns deassert
```

Use this when you want a compact timeline of state changes, not every sampled match.

## Keep every true sample with `--capture match`

If you want every selected timestamp where the expression is true, switch to level capture:

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on 'posedge clk' --eval ready --capture match
@15ns sample@14ns match
```

This is the simplest mode when you are asking "at which sampled events was the condition true?"

## Control row count with `--max`

`property` output is bounded by default. The default limit is 50 captured rows, which keeps dense event streams from flooding a terminal or a JSON client. When more captured rows would be emitted, `property` emits `WPK-W0002`:

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on 'edge clk' --eval ready --capture match --max 1
@15ns sample@14ns match
warning[WPK-W0002]: truncated output to 1 entries (use --max to increase limit)
```

Raise the limit with `--max <N>` when you need a larger bounded sample. Use `--max unlimited` only when you intentionally want every captured row; it emits `WPK-W0001` so scripts can detect that truncation was disabled.

## Filter only one transition direction

`assert` keeps only false-to-true transitions. `deassert` keeps only true-to-false transitions.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on 'posedge clk' --eval ready --capture assert
@15ns sample@14ns assert

$ wavepeek property --waves path/to/dump.vcd --scope top --on 'posedge clk' --eval ready --capture deassert
@25ns sample@24ns deassert
```

Use this when you care about only one edge of the condition.

## Use explicit wildcard/native sampling for change-driven checks

`--on` is required. When any relevant input update should trigger evaluation, pass the wildcard trigger explicitly and use native sampling:

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on '*' --sample-mode native --eval "data == 8'h0f" --capture match
@10ns match
```

This is convenient when the property itself already tells you which signals matter, but the selected timestamps are value-change events rather than clock cycles.

## Bound the search window with `--from` and `--to`

Time bounds are inclusive. In transition modes, the baseline state is taken at `--from`, so moving the window can change which edges are visible.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --from 10ns --to 25ns --on 'posedge clk' --eval ready
@25ns sample@24ns deassert
```

In this window the property is already true at `10ns`, so there is no visible `assert` inside the selected range.

## Use JSON for scripts and agents

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on data --sample-mode native --eval "data == 8'h0f" --capture match --json
{"$schema":"https://kleverhq.github.io/wavepeek/schema-output-v2.1.json","command":"property","data":[{"time":"10ns","sample_time":"10ns","kind":"match"}],"diagnostics":[]}
```

Human output is for quick inspection. `--json` is the stable machine contract.

For long searches or incremental consumers, use `--jsonl`. It streams one JSON object per line: `begin`, one `item` per captured property row, optional `diagnostic` records, and a final `end` summary. When `--max` truncates the stream, the final summary has `"truncated":true`. Validate each line with `wavepeek schema --stream`, and require the final `end` record before treating the result as complete.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on data --sample-mode native --eval "data == 8'h0f" --capture match --jsonl
{"type":"begin","seq":0,"command":"property","$schema":"https://kleverhq.github.io/wavepeek/schema-stream-v2.1.json"}
{"type":"item","seq":1,"command":"property","item":{"time":"10ns","sample_time":"10ns","kind":"match"}}
{"type":"end","seq":2,"command":"property","summary":{"status":"ok","items":1,"diagnostics":0,"truncated":false}}
```

## Trigger expressions you will actually use

Common `--on` patterns:

- `posedge clk`
- `negedge rst_n`
- `ready` with `--sample-mode native`
- `posedge clk iff rst_n`
- `* or posedge clk` with `--sample-mode native`

Full trigger and expression syntax is defined in `reference/expression-language`. Wildcard, plain-signal, and mixed triggers are native-sampling queries; edge-only triggers can use the default pre-edge sampling.

## Choose native or pre-edge value sampling

By default, `property` uses pre-edge value sampling. A row selected by an edge-only trigger such as `--on 'posedge clk'` evaluates `--eval` from values immediately before the selected edge while keeping the reported row timestamp at the edge. Human output shows `sample@<time>` when the evaluation timestamp differs from the trigger timestamp:

```text
$ wavepeek property --waves path/to/dump.vcd --scope top \
    --on 'posedge clk' --eval valid --capture assert
@25ns sample@24999ps assert
```

Pre-edge sampling is accepted only with an explicit edge-only `--on`: `posedge`, `negedge`, or `edge`, optionally with `iff`. The trigger edge detection and any `iff` guard still use dump-native values at the edge timestamp; only the `--eval` value sampling moves to the pre-edge sample point. JSON and JSONL rows always include both `time` and `sample_time`; use `sample_time` for follow-up `value --at` checks.

Use `--sample-mode native` for wildcard, plain-signal, or mixed triggers, or when you intentionally want values from the same dump timestamp as the selected event. Use the default pre-edge mode when `property` appears one clock ahead of a SystemVerilog assertion because a value is dumped after a nonblocking assignment at the same clock edge. See `troubleshooting/clock-edge-sampling` for diagrams and boundary behavior.

## Non-obvious behavior

- VCD and FST work in default builds. FSDB works only in binaries built with the `fsdb` Cargo feature and a local Verdi FSDB Reader SDK. FSDB `property` supports digital bit-vector/integral expressions, including raw event triggers when the FSDB contains event occurrences; unsupported real or string values fail with a `signal` error.
- No output is still success. It emits `WPK-W0003` and usually means no selected timestamp satisfied the requested capture mode.
- Output is limited to 50 captured rows by default. `--max unlimited` disables truncation and emits `WPK-W0001`.
- The default capture mode is `switch`, not `match`.
- `--sample-mode pre-edge` is the default and requires an explicit edge-only trigger. Use `--sample-mode native` for wildcard, plain-signal, or mixed triggers and for same-timestamp dump sampling.
- JSON and JSONL rows always include `sample_time`. In native mode it equals `time`; in pre-edge mode it is the timestamp whose values were evaluated.
- `property` prints only trigger/sample times and result kind. If you need payload values for a matching row, query them with `value --at <sample_time>`.
- With `--scope`, names inside `--on` and `--eval` must stay scope-relative. For example, `--scope top --on 'posedge top.clk'` is an error.
- `--on` is required. Use explicit clock edges for RTL-style checks, or `--on '*' --sample-mode native` for wildcard value-change evaluation.
