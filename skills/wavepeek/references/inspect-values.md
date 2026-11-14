# Inspect values

Use `value` for snapshots at known times and `change` for event-aligned tables over a time range.

## What was the value at a specific time?

Sample related signals together:

```text
$ wavepeek value --waves dump.fst \
    --scope tb.dut.cpu \
    --at 120ns \
    --signals state,pc,instr_valid
@120ns state=4'h3 pc=32'h00001040 instr_valid=1'h1
```

Several points can be requested in one command:

```text
wavepeek value --waves dump.fst \
  --scope tb.dut.cpu \
  --at 100ns,110ns --at 120ns \
  --signals state --signals pc
```

Snapshots follow the requested time order, including repeated and unsorted times.
Each snapshot preserves signal order, repeated operands, and alias paths. A point
uses the final recorded state at that tick, holding the last observation on quiet
ticks. If a selected persistent signal has no observation at or before any requested point,
the command fails; a recorded X remains a value.

Use a static projection when only part of a flat vector matters:

```text
wavepeek value --waves dump.fst --scope tb.dut.cpu \
  --at 120ns --signals 'status[7:4],status[0:0]'
```

## What changed in this time range?

Start with a short window and a focused signal list:

```text
$ wavepeek change --waves dump.fst \
    --scope tb.dut.cpu \
    --from 100ns --to 160ns \
    --on '*' --sample-mode native \
    --signals req --signals ack,state --max 20
@110ns req=1'h1 ack=1'h0 state=4'h2
@130ns req=1'h1 ack=1'h1 state=4'h3
@150ns req=1'h0 ack=1'h0 state=4'h0
```

This uses dump-native timestamps and answers "what changed in the file?" If the result is noisy, narrow the window, replace `*` with a specific signal or edge, or reduce the signal list.

## What did the design see on each clock edge?

Use the owning clock as the trigger. Pre-edge sampling is the default for edge-only triggers:

```text
wavepeek change --waves dump.fst \
  --scope tb.dut.cpu \
  --from 100ns --to 200ns \
  --on 'posedge clk iff reset_n' \
  --signals state,req,ack \
  --row-mode sparse --max 20
```

`--row-mode sparse` suppresses rows where none of the requested values changed between selected clock edges. Remove it when every selected cycle matters.

## What is the clock period?

Inspect a few native clock transitions:

```text
$ wavepeek change --waves dump.fst \
    --scope tb.dut.cpu \
    --to 100ns --on '*' --sample-mode native \
    --signals clk --max 6
@10ns clk=1'h1
@15ns clk=1'h0
@20ns clk=1'h1
```

The example has a 10 ns period: consecutive rising edges are at 10 ns and 20 ns.

When following a `change`, `property`, or `extract` result with `value`, query the row's `sample_time` if you want the same sampled values. See [Clocks and sampling](sampling.md) for the difference between `time` and `sample_time`.
