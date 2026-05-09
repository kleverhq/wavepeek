---
id: commands/change
title: Change command
summary: Inspect value transitions across a bounded time range.
section: commands
see_also:
  - commands/info
  - commands/scope
  - workflows/find-first-change
  - reference/command-model
  - reference/expression-language
---
# Change command

Use `change` when one timestamp is not enough and you need the moments when a small set of signals actually transitions.

`change` scans an inclusive time window, samples the signals from `--signals`, and prints a row only when at least one of those sampled values changed. By default, `--on` is `*`, which means "consider any change in the tracked signal set".

In practice, `change` is the command between `value` and `property`: it is more selective than sampling every cycle, but still shows raw signal snapshots instead of a derived pass/fail result.

`--on` is intentionally a SystemVerilog-style event-expression surface. Treat it as a practical CLI spelling of the same concepts you would use in `@(...)`: named events, `posedge`/`negedge`/`edge`, `*` for any tracked change, unions with `or` or `,`, and `iff` for gating. For the full shipped syntax and semantics, see `reference/expression-language`.

A rough mental model is this SystemVerilog-like pseudocode:

```systemverilog
logic initialized = 1'b0;
sample_t prev;

always @(<event from --on, or @(*) when omitted>) begin
  sample_t cur = sample(<signals from --signals>);
  if (initialized && (cur != prev))
    $display("@%0t ...", $time, cur);
  prev = cur;
  initialized = 1'b1;
end
```

That is only an intuition aid, not a normative definition: `wavepeek` runs over recorded dump timestamps, applies the selected inclusive `--from`/`--to` window, and initializes its baseline at `--from`.

For exact syntax and flags, run `wavepeek help change`.

## Start with a short window and a focused signal list

This is the fastest way to answer "what changed here?":

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1010000ps --to 1040000ps --max 10
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1030000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h1 trap=1'h0
@1040000ps cpu_state=8'h40 mem_valid=1'h0 mem_ready=1'h0 trap=1'h0
```

Use this as the default pattern when you already know the scope and just need the transition points.

## Trigger on one signal, but print several

`--on` decides when to sample. `--signals` decides what to print.

If you already know SystemVerilog event controls, read `--on` the same way: `mem_valid` means any change, `posedge mem_valid` means rising edges only, `*` means any change in the tracked set, and `iff` gates an event term without changing what gets printed.

If you care about every change of `mem_valid`, use the signal itself as the event:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1000000ps --to 1040000ps \
    --on mem_valid --max 10
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1040000ps cpu_state=8'h40 mem_valid=1'h0 mem_ready=1'h0 trap=1'h0
```

Named event `mem_valid` means any change of that signal, not only the rising edge.

## Keep only the edge you care about

If the deassert edge is noise, switch to an edge trigger:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1000000ps --to 1040000ps \
    --on "posedge mem_valid" --max 10
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
```

This is usually the cleanest way to inspect request starts, handshake assertions, enables, and state-entry pulses.

## Sample on clock edges only while a condition is true

When combinational chatter is irrelevant, gate sampling with `iff`:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1010000ps --to 1040000ps \
    --on "posedge clk iff mem_valid" --max 10
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1030000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h1 trap=1'h0
```

This means: sample on `posedge clk`, but only on cycles where `mem_valid` is true.

## Use scope-relative names or full canonical paths

With `--scope`, short names stay readable. Without it, pass canonical paths directly:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --signals testbench.uut.cpu_state,testbench.uut.mem_valid,testbench.uut.mem_ready,testbench.uut.trap \
    --from 0ps --to 20000ps --max 20
@10000ps testbench.uut.cpu_state=8'h40 testbench.uut.mem_valid=1'h0 testbench.uut.mem_ready=1'h0 testbench.uut.trap=1'h0
```

If you like scoped input but still want canonical names in human output, add `--abs`:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 0ps --to 20000ps --abs
@10000ps testbench.uut.cpu_state=8'h40 testbench.uut.mem_valid=1'h0 testbench.uut.mem_ready=1'h0 testbench.uut.trap=1'h0
```

## Use JSON for scripts and agents

`--json` keeps canonical paths and moves warnings into the payload:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1010000ps --to 1040000ps --json
{"$schema":"https://raw.githubusercontent.com/kleverhq/wavepeek/v0.4.0/schema/wavepeek.json","command":"change","data":[{"time":"1020000ps","signals":[{"path":"testbench.uut.cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","value":"1'h1"},{"path":"testbench.uut.mem_ready","value":"1'h0"},{"path":"testbench.uut.trap","value":"1'h0"}]},{"time":"1030000ps","signals":[{"path":"testbench.uut.cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","value":"1'h1"},{"path":"testbench.uut.mem_ready","value":"1'h1"},{"path":"testbench.uut.trap","value":"1'h0"}]},{"time":"1040000ps","signals":[{"path":"testbench.uut.cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","value":"1'h0"},{"path":"testbench.uut.mem_ready","value":"1'h0"},{"path":"testbench.uut.trap","value":"1'h0"}]}],"warnings":[]}
```

## Watch for bounded-output warnings

If `--max` truncates the result, the command still succeeds and warns:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1000000ps --to 11000000ps \
    --on "posedge clk" --max 3
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1030000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h1 trap=1'h0
@1040000ps cpu_state=8'h40 mem_valid=1'h0 mem_ready=1'h0 trap=1'h0
warning: truncated output to 3 entries (use --max to increase limit)
```

If you disable the limit intentionally, that is also reported:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1010000ps --to 1040000ps \
    --on "posedge clk" --max unlimited
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1030000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h1 trap=1'h0
@1040000ps cpu_state=8'h40 mem_valid=1'h0 mem_ready=1'h0 trap=1'h0
warning: limit disabled: --max=unlimited
```

## Non-obvious behavior

- `--from` is inclusive for selection, but it also initializes the baseline state. `change` does not emit a row exactly at `--from`; if you need the boundary value itself, use `value`.
- `--on` does not guarantee a row by itself. A trigger can fire, but `change` still suppresses the row if none of the requested `--signals` changed.
- In scoped mode, use scope-relative names in `--signals` and `--on`. Without `--scope`, use canonical full paths.
- Empty output is valid. If the query is well-formed but nothing matched, the command succeeds and warns:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 0ps --to 20000ps \
    --on "posedge mem_valid" --max 20
warning: no signal changes found in selected time range
```

When a query keeps coming back empty, widen one dimension at a time: start with the time window, then the trigger, then the signal list.