# Clocks and sampling

This page explains event and trigger selection for inspection commands, including synchronous and asynchronous sampling.

`wavepeek` reads values recorded at discrete timestamps in a waveform dump. Commands such as `change`, `property`, and `extract generic` select events first, then read or evaluate values at a sample point associated with each event. The event timestamp and the sample point are not always the same.

Three options control this process:
- `--from` and `--to` set the inclusive time window.
- `--on` selects events within the window.
- `--sample-mode` controls where values are read relative to an event.

## `--on` as a SystemVerilog event control

An `--on` expression has the same role as the expression inside a SystemVerilog event control:

```systemverilog
always @(posedge clk iff rst_n) begin
  // inspect or process values
end
```

In `wavepeek`, omit `@(...)`:

```text
--on "posedge clk iff rst_n"
```

Each matching event runs the command-specific equivalent of the block body. `change` reads the signals in `--signals`. `property` evaluates `--eval` and applies its capture mode. `extract generic` evaluates `--when` and, when the condition is true, emits `--payload`.

For example:

```text
wavepeek change --waves dump.vcd --scope top.dut --signals state,valid,data --on "posedge clk iff rst_n"
```

This command inspects `state`, `valid`, and `data` at every rising edge of `clk` where `rst_n` is true.

The event expression supports the usual forms:

```text
--on '*'
--on req
--on 'posedge clk'
--on 'negedge rst_n'
--on 'posedge clk iff enable'
--on 'posedge a or negedge b'
```

A plain signal name selects any change of that signal. `posedge`, `negedge`, and `edge` select specific transitions. `or` and `,` form a union, while `iff` gates one event term.

The wildcard `*` selects any change in the command's tracked signal set. For `change`, the tracked set is `--signals`. For `property`, it consists of the signals referenced by `--eval`. `extract generic` is a synchronous, edge-sampled command, so it does not accept wildcard or plain-signal triggers.

See [Event expressions](event-expressions.md) for the full language contract.

## Event time and sample time

Every result row has two timestamps:

- `time` is the timestamp selected by `--on`.
- `sample_time` is the point where values were read or evaluated.

They are equal in native mode (`--sample-mode native`):

```text
time == sample_time
```

In pre-edge mode (`--sample-mode pre-edge`), the result still belongs to the clock edge, but its values come from immediately before that edge:

```text
sample_time = time - 1 raw tick
```

`change`, `property`, and extraction skip a pre-edge event when its preceding tick is before
the dump start. The previous candidate timestamp is not the sample point.

Human output includes both timestamps when they differ:

```text
@25ns sample@24999ps state=3'h2 valid=1'h1
```

JSON and JSONL expose the same distinction through `time` and `sample_time`.

## Asynchronous logic

Asynchronous inspection usually selects a signal change rather than an owning clock edge. Native mode reads the waveform snapshot at the event timestamp:

```text
wavepeek change --waves dump.vcd --scope top.dut --signals req,ack,state --on req --sample-mode native
```

This command samples whenever `req` changes. To inspect every change in the requested signal set, use:

```text
--on '*' --sample-mode native
```

You can also select one asynchronous edge:

```text
--on 'posedge req' --sample-mode native
```

Native mode answers the question, "What values does the dump contain at this event timestamp?"

A waveform dump usually stores one settled value per signal at each timestamp, not the simulator's full scheduling-region history. When several signals update at the same simulation time, native mode sees the values recorded at that timestamp. It cannot recover an ordering that the dump did not preserve.

Wildcard, plain-signal, and mixed signal/edge triggers require native mode.

## Synchronous logic

A clock edge and the data updates caused by that edge may appear at the same dump timestamp. This makes native sampling easy to misread for clocked RTL.

Consider this code:

```systemverilog
always_ff @(posedge clk) begin
  data <= next_data;
end

assert property (@(posedge clk) data == 8'haa);
```

The RTL and the assertion use values sampled before nonblocking assignments from the same edge take effect. The waveform snapshot at the edge timestamp may already contain the updated `data`. Native sampling can therefore make the transition appear one cycle earlier than it does in the assertion, clocked process, or simulator log.

With `--sample-mode native`, the clock edge and same-time data update are observed together:

```text
time        0ns      5ns      10ns     15ns
clk         0        1        0        1
data        00       aa       aa       aa
                      ^
                      posedge clk and data update in the dump
```

The `pre-edge` mode keeps trigger detection at the edge timestamp, but samples displayed or evaluated values just before that edge:

```text
time        0ns      5ns      10ns     15ns
clk         0        1        0        1
data        00       aa       aa       aa
                      ^                 ^
                      edge              next posedge

pre-edge sample before 5ns sees data=00
pre-edge sample before 15ns sees data=aa
```

At `5ns`, native mode sees `data == 8'haa`, while pre-edge mode sees `data == 8'h00`. Both results still refer to the clock event at `5ns`.

Because this is the common interpretation for clocked queries, `change` and `property` default to `--sample-mode pre-edge`. For example, this expression would match at 15ns on the pseudo-waveform above:

```text
wavepeek property --waves dump.vcd --scope top.dut --on 'posedge clk' --eval "data == 8'haa"
```
Pre-edge mode detects the requested edge at timestamp `t`, then reads or evaluates values at the representable waveform point immediately before `t`. This is close to SystemVerilog sampled-value semantics and does not depend on whether the dump snapshot at `t` already includes same-edge updates.

## What moves to the pre-edge point

Pre-edge mode reads these values at `sample_time`:

- signals requested by `change --signals`;
- the expression passed to `property --eval`;
- the `extract generic --when` expression and `--payload` signals.

Event detection remains at the native event timestamp. This includes edge detection in `--on`, evaluation of an `iff` guard, and the row's `time` value.

For example:

```text
--on 'posedge clk iff rst_n'
```

The command detects the edge and checks `rst_n` at the event timestamp. It then reads or evaluates the command values at `sample_time`, immediately before the edge.

Pre-edge mode accepts only `--on` expressions made of explicit edge events, with optional `iff` gates. Wildcard, plain-signal, and mixed triggers do not define a single unambiguous clock boundary, so they require native mode.

`extract generic` always uses pre-edge sampling and has no `--sample-mode` option. It accepts only edge-based `--on` expressions.

## Choosing the sample mode

Use native mode for questions about the values stored in the dump and for asynchronous triggers:

```text
--on '*' --sample-mode native
--on req --sample-mode native
--on 'posedge req' --sample-mode native
```

Use pre-edge mode for clocked queries:

```text
--on 'posedge clk'
--on 'negedge clk iff enable'
```

In short, use `native` for "what changed in the dump?" and `pre-edge` for "what did the clocked logic sample on this edge?" Use native mode with a clock edge only when you deliberately want the post-update snapshot stored at the edge timestamp.

## Time boundaries

`--from` and `--to` bound event timestamps, not sample timestamps. Both bounds are inclusive:

```text
--from 10ns --to 20ns
```

An event at exactly `10ns` or `20ns` can be selected. In pre-edge mode, an event at `--from` may have a `sample_time` before `--from` because the window applies to the event's `time`.

If no representable point exists before an edge, `wavepeek` cannot sample that event in pre-edge mode. It skips the event instead of substituting native values.

A query therefore has two separate decisions. `--on` decides when the command runs, much like the event control of an `always` block. The sample mode decides whether the command sees values at the event timestamp or immediately before an edge. That distinction lets the same event syntax cover asynchronous dump inspection and synchronous sampling.
