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
- `--eval` uses SystemVerilog-style value and logical expression semantics. In default native sampling it is evaluated at each timestamp selected by `--on`; with `--sample-mode pre-edge`, the row timestamp stays at the trigger edge but `--eval` reads values from immediately before that edge.
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
@15ns assert
@25ns deassert
```

Use this when you want a compact timeline of state changes, not every sampled match.

## Keep every true sample with `--capture match`

If you want every selected timestamp where the expression is true, switch to level capture:

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on 'posedge clk' --eval ready --capture match
@15ns match
```

This is the simplest mode when you are asking "at which sampled events was the condition true?"

## Filter only one transition direction

`assert` keeps only false-to-true transitions. `deassert` keeps only true-to-false transitions.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on 'posedge clk' --eval ready --capture assert
@15ns assert

$ wavepeek property --waves path/to/dump.vcd --scope top --on 'posedge clk' --eval ready --capture deassert
@25ns deassert
```

Use this when you care about only one edge of the condition.

## Omit `--on` when any relevant input update should trigger evaluation

If `--eval` references signals, omitted `--on` falls back to wildcard tracking for those referenced inputs.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --eval "data == 8'h0f" --capture match
@10ns match
```

This is convenient when the property itself already tells `wavepeek` which signals matter.

## Bound the search window with `--from` and `--to`

Time bounds are inclusive. In transition modes, the baseline state is taken at `--from`, so moving the window can change which edges are visible.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --from 10ns --to 25ns --on 'posedge clk' --eval ready
@25ns deassert
```

In this window the property is already true at `10ns`, so there is no visible `assert` inside the selected range.

## Use JSON for scripts and agents

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on data --eval "data == 8'h0f" --capture match --json
{"$schema":"https://kleverhq.github.io/wavepeek/wavepeek_v1.json","command":"property","data":[{"time":"10ns","sample_time":"10ns","kind":"match"}],"diagnostics":[]}
```

Human output is for quick inspection. `--json` is the stable machine contract.

For long searches or incremental consumers, use `--jsonl`. It streams one JSON object per line: `begin`, one `item` per captured property row, optional `diagnostic` records, and a final `end` summary. Validate each line with `wavepeek schema --stream`, and require the final `end` record before treating the result as complete.

```text
$ wavepeek property --waves path/to/dump.vcd --scope top --on data --eval "data == 8'h0f" --capture match --jsonl
{"type":"begin","seq":0,"command":"property","$schema":"https://kleverhq.github.io/wavepeek/wavepeek-stream-v1.json"}
{"type":"item","seq":1,"command":"property","item":{"time":"10ns","sample_time":"10ns","kind":"match"}}
{"type":"end","seq":2,"command":"property","summary":{"status":"ok","items":1,"diagnostics":0,"truncated":false}}
```

## Trigger expressions you will actually use

Common `--on` patterns:

- `posedge clk`
- `negedge rst_n`
- `ready`
- `posedge clk iff rst_n`
- `* or posedge clk`

Full trigger and expression syntax is defined in `reference/expression-language`.

## Choose native or pre-edge value sampling

By default, `property` uses dump-native sampling: a row selected by `--on 'posedge clk'` evaluates `--eval` from values at the same dump timestamp as that edge.

For RTL and SVA-style debugging, `--sample-mode pre-edge` evaluates `--eval` from values immediately before the selected edge while keeping the reported row timestamp at the edge. Human output shows `sample@<time>` when the evaluation timestamp differs from the trigger timestamp:

```text
$ wavepeek property --waves path/to/dump.vcd --scope top \
    --on 'posedge clk' --eval valid --capture assert \
    --sample-mode pre-edge
@25ns sample@24999ps assert
```

`pre-edge` is accepted only with an explicit edge-only `--on`: `posedge`, `negedge`, or `edge`, optionally with `iff`. The trigger edge detection and any `iff` guard still use dump-native values at the edge timestamp; only the `--eval` value sampling moves to the pre-edge sample point. JSON and JSONL rows always include both `time` and `sample_time`; use `sample_time` for follow-up `value --at` checks.

Use this mode when `property` appears one clock ahead of a SystemVerilog assertion because a value is dumped after a nonblocking assignment at the same clock edge. See `troubleshooting/clock-edge-sampling` for diagrams and boundary behavior.

## Non-obvious behavior

- VCD and FST work in default builds. FSDB works only in binaries built with the `fsdb` Cargo feature and a local Verdi FSDB Reader SDK. FSDB `property` supports digital bit-vector/integral expressions, including raw event triggers when the FSDB contains event occurrences; unsupported real or string values fail with a `signal` error.
- No output is still success. It emits `WPK-W0003` and usually means no selected timestamp satisfied the requested capture mode.
- The default capture mode is `switch`, not `match`.
- `--sample-mode native` is the default and preserves historical behavior. `--sample-mode pre-edge` is opt-in and requires an explicit edge-only trigger.
- JSON and JSONL rows always include `sample_time`. In native mode it equals `time`; in pre-edge mode it is the timestamp whose values were evaluated.
- `property` prints only trigger/sample times and result kind. If you need payload values for a matching row, query them with `value --at <sample_time>`.
- With `--scope`, names inside `--on` and `--eval` must stay scope-relative. For example, `--scope top --on 'posedge top.clk'` is an error.
- If you omit `--on`, `wavepeek` must be able to infer tracked signals from `--eval`. A signal-free expression like `--eval 1` requires explicit `--on`.
