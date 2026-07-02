---
id: troubleshooting/clock-edge-sampling
title: Clock-edge sampling and one-cycle mismatches
description: Explain native and pre-edge sampling for RTL and SVA-style clocked checks.
section: troubleshooting
see_also:
  - commands/change
  - commands/property
  - reference/expression-language
---
# Clock-edge sampling and one-cycle mismatches

Use this topic when `change --on 'posedge clk'` or `property --on 'posedge clk'` appears one clock different from an RTL assertion, simulator log, or mental model.

`wavepeek` reads recorded waveform dumps. A dump timestamp contains the values recorded at that simulation time. For edge-only triggers, the default `pre-edge` mode evaluates and displays values from just before the trigger edge while keeping the row timestamp at the edge. Use `--sample-mode native` when you want the values recorded at the selected dump timestamp itself.

RTL and concurrent SystemVerilog assertions often use sampled values. A common clocked property samples values before the clocking event and observes nonblocking assignment updates on the following clock, not on the same edge where the dump value changed.

## Native sampling

With `--sample-mode native`, the clock edge and same-time data update are observed together:

```text
time        0ns      5ns      10ns     15ns
clk         0        1        0        1
data        00       aa       aa       aa
                      ^
                      posedge clk and data update in the dump

wavepeek property --on 'posedge clk' --eval "data == 8'haa" --sample-mode native
                      ^ match at 5ns
```

Native mode is useful when you want to inspect exactly what is recorded at each dump timestamp. It is also required for wildcard, plain-signal, and mixed triggers.

## Pre-edge sampling

The default `pre-edge` mode keeps trigger detection at the edge timestamp, but samples displayed or evaluated values just before that edge:

```text
time        0ns      5ns      10ns     15ns
clk         0        1        0        1
data        00       aa       aa       aa
                      ^                 ^
                      edge              next posedge

pre-edge sample before 5ns sees data=00
pre-edge sample before 15ns sees data=aa

wavepeek property --on 'posedge clk' --eval "data == 8'haa"
                                        ^ match at 15ns
```

The row `time` remains the trigger edge timestamp. The row `sample_time` is the pre-edge query point whose values were printed or evaluated. Human output shows that distinction as `sample@<time>`:

```text
@15ns sample@14ns match
```

Use `sample_time`, not `time`, for follow-up `value --at` payload inspection.

## What changes and what does not

`pre-edge` affects:

- values printed by `change --signals`;
- values evaluated by `property --eval`.

`pre-edge` does not affect:

- edge detection in `--on`;
- `iff` guards in `--on`;
- the trigger timestamp stored in row `time`.

`pre-edge` does add a separate `sample_time` to JSON and JSONL rows. In native mode `sample_time == time`; in pre-edge mode `sample_time` is immediately before the trigger edge.

For example, `--on 'posedge clk iff rst_n' --sample-mode pre-edge` still detects `posedge clk` and evaluates `rst_n` for the `iff` guard at the native edge timestamp. The property expression or printed signal values use the pre-edge sample point recorded as `sample_time`.

## Accepted triggers

`pre-edge` requires an explicit edge-only `--on` expression. These are valid:

```text
--on 'posedge clk' --sample-mode pre-edge
--on 'negedge rst_n' --sample-mode pre-edge
--on 'edge clk' --sample-mode pre-edge
--on 'posedge clk iff rst_n' --sample-mode pre-edge
```

These are rejected:

```text
--sample-mode pre-edge
--on '*' --sample-mode pre-edge
--on data --sample-mode pre-edge
--on 'data or posedge clk' --sample-mode pre-edge
```

The restriction keeps the mode tied to edge-triggered RTL-style sampling. Use `--sample-mode native` for wildcard, plain-signal, or mixed triggers.

## Range boundaries

Time bounds are still inclusive. Transition captures and `change` also keep their normal baseline at `--from`.

If a trigger occurs exactly at `--from`, its pre-edge query point may be before the selected window:

```text
--from 5ns

time        0ns      5ns      10ns     15ns
clk         0        1        0        1
data        00       aa       aa       aa
             ^        ^
             |        trigger at range start
             pre-edge sample is before --from
```

For transition modes, that pre-window sample does not replace the `--from` baseline. The baseline remains the value sampled at the range start. This prevents a boundary edge from manufacturing an extra `assert`, `deassert`, or `change` row using a value outside the requested range.

If there is no representable query point before a trigger, for example at the first timestamp in the dump, `pre-edge` skips value evaluation for that trigger. This can produce an empty result even though the edge itself was present.

## Back-to-back handshakes

For a ready/valid interface, consecutive accepted beats can keep `ready && valid` high across multiple cycles. Use `property --capture match` to report every selected clock edge where the handshake is true:

```text
$ wavepeek property --waves path/to/dump.vcd --scope top \
    --on 'posedge clk' --eval 'ready && valid' \
    --capture match --sample-mode pre-edge --json
{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-output-v2.1.json",
  "command": "property",
  "data": [
    {"time":"100ns","sample_time":"99999ps","kind":"match"},
    {"time":"110ns","sample_time":"109999ps","kind":"match"}
  ],
  "diagnostics": []
}
```

If you then inspect payload with `value`, use each row's `sample_time`:

```text
$ wavepeek value --waves path/to/dump.vcd --scope top \
    --at 99999ps --signals ready,valid,data
```

Using `value --at 100ns` asks for native dump values at the trigger edge. For nonblocking-assignment style RTL dumps, that can show the next beat's payload, which is precisely the mismatch `pre-edge` is meant to avoid.

## Which mode to choose

Use `native` when you want the literal dump value at each selected timestamp or when your trigger is wildcard/plain-signal based.

Use the default `pre-edge` mode when debugging clocked RTL or SVA-like checks and a same-edge nonblocking assignment update makes `wavepeek` output look one clock early.
