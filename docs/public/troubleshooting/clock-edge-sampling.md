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

`wavepeek` reads recorded waveform dumps. A dump timestamp contains the values recorded at that simulation time. In default `native` mode, `wavepeek` evaluates and displays values at the same timestamp as the trigger edge.

RTL and concurrent SystemVerilog assertions often use sampled values. A common clocked property samples values before the clocking event and observes nonblocking assignment updates on the following clock, not on the same edge where the dump value changed.

## Native sampling

With native sampling, the clock edge and same-time data update are observed together:

```text
time        0ns      5ns      10ns     15ns
clk         0        1        0        1
data        00       aa       aa       aa
                      ^
                      posedge clk and data update in the dump

wavepeek property --on 'posedge clk' --eval "data == 8'haa"
                      ^ match at 5ns
```

This is the default for compatibility. It is useful when you want to inspect exactly what is recorded at each dump timestamp.

## Pre-edge sampling

`--sample-mode pre-edge` keeps trigger detection at the edge timestamp, but samples displayed or evaluated values just before that edge:

```text
time        0ns      5ns      10ns     15ns
clk         0        1        0        1
data        00       aa       aa       aa
                      ^                 ^
                      edge              next posedge

pre-edge sample before 5ns sees data=00
pre-edge sample before 15ns sees data=aa

wavepeek property --on 'posedge clk' --eval "data == 8'haa" --sample-mode pre-edge
                                        ^ match at 15ns
```

The row timestamp remains the trigger edge timestamp. Only the sampled values move to the pre-edge query point.

## What changes and what does not

`pre-edge` affects:

- values printed by `change --signals`;
- values evaluated by `property --eval`.

`pre-edge` does not affect:

- edge detection in `--on`;
- `iff` guards in `--on`;
- row timestamps.

For example, `--on 'posedge clk iff rst_n' --sample-mode pre-edge` still detects `posedge clk` and evaluates `rst_n` for the `iff` guard at the native edge timestamp. The property expression or printed signal values use the pre-edge sample point.

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

The restriction keeps the mode tied to edge-triggered RTL-style sampling. Use the default `native` mode for wildcard, plain-signal, or mixed triggers.

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

## Which mode to choose

Use `native` when you want the literal dump value at each selected timestamp or when your trigger is wildcard/plain-signal based.

Use `pre-edge` when debugging clocked RTL or SVA-like checks and a same-edge nonblocking assignment update makes `wavepeek` output look one clock early.
