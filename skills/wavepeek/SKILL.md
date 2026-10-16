---
name: wavepeek
description: Use the wavepeek CLI for deterministic VCD, FST, FSDB, or SHM waveform inspection, including metadata and hierarchy discovery, timestamped values and transitions, Boolean properties, generic synchronous events, AMBA protocol extraction, and JSON or JSONL post-processing.
---

# Wavepeek

Use `wavepeek` for deterministic, scriptable questions about RTL waveform dumps. Treat VCD, FST, and FSDB files and SHM database directories as waveform inputs to the CLI, not as text or binary files to inspect directly.

Wavepeek is a stateless command-line tool. Each invocation opens one dump, runs one query, writes human-readable, JSON, or JSONL output, and exits. It is useful for focused debug, repeatable analysis, and agent workflows. It is not a GUI waveform viewer, a live simulator connection, or a waveform comparison service.

## Core workflow

For an unfamiliar dump, use this sequence and stop as soon as the required context is known:

1. Run `info` to learn the dump bounds and native time unit.
2. Run `scope` only if the hierarchy path is unknown.
3. Run `signal` only if the exact signal names, widths, or types are unknown.
4. Choose one inspection command that matches the question.
5. Start with a narrow time window, focused signal set, and bounded output.
6. Inspect diagnostics and completeness metadata before drawing an exhaustive conclusion.

Do not rediscover metadata, scopes, or signals that are already known from a previous result.

Choose the inspection command by the shape of the answer:

- `value`: values at one or more explicit timestamps.
- `change`: event-aligned signal rows or sparse changes in a time window.
- `property`: timestamps where a Boolean expression matches or changes truth state.
- `extract generic`: synchronous events selected by a predicate, with payload values on the same row.
- `extract axi`, `axistream`, `ahb`, `apb`, or `atb`: standard protocol transfer or phase events.

## Practical command shapes

### Explore an unknown dump

```text
wavepeek info --waves dump.fst
wavepeek scope --waves dump.fst --tree --max-depth 2 --max 30
wavepeek signal --waves dump.fst --scope tb.dut --filter '.*(clk|reset|state).*'
```

A typical `info` result is:

```text
time_unit: 1ns
time_start: 0ns
time_end: 250000ns
```

Use `--recursive`, a small `--max-depth`, and `--abs` with `signal` only when the signal may be below the selected scope and canonical paths are needed.

### Sample known signals

```text
wavepeek value --waves dump.fst \
  --scope tb.dut.cpu \
  --at 100ns,120ns \
  --signals state,pc,instr_valid
```

Human output is compact and uses normalized SystemVerilog-style values:

```text
@120ns state=4'h3 pc=32'h00001040 instr_valid=1'h1
```

Request related signals and timestamps together instead of starting many small processes.

### Inspect asynchronous changes

```text
wavepeek change --waves dump.fst \
  --scope tb.dut.cpu \
  --from 100ns --to 160ns \
  --on '*' --sample-mode native \
  --signals req,ack,state --max 20
```

This answers what changed in the dump. Narrow the trigger, range, or signal list before increasing the limit.

### Inspect clocked behavior

```text
wavepeek change --waves dump.fst \
  --scope tb.dut.cpu \
  --from 100ns --to 200ns \
  --on 'posedge clk iff reset_n' \
  --signals state,req,ack \
  --row-mode sparse --max 20
```

Edge-only `change` and `property` queries default to pre-edge sampling. Their rows can therefore have different `time` and `sample_time` values. Edge detection and an `iff` guard are evaluated at the event timestamp; displayed signals and `--eval` are sampled at the pre-edge `sample_time`.

### Find or count a condition

```text
wavepeek property --waves dump.fst \
  --scope tb.dut.cpu \
  --on 'posedge clk iff reset_n' \
  --eval "state == 4'h7" \
  --capture match --max 1
```

Use `--capture match --max 1` for the first match. For an exact count without rows:

```text
wavepeek property --waves dump.fst \
  --scope tb.dut.bus \
  --on 'posedge clk iff reset_n' \
  --eval 'valid && ready' \
  --capture match --summary --max unlimited
```

### Extract a custom synchronous event

```text
wavepeek extract generic --waves dump.fst \
  --scope tb.dut.queue \
  --on 'posedge clk iff reset_n' \
  --when 'valid && ready' \
  --payload data,last
```

`extract generic` always uses pre-edge sampling. Use a JSON source file when several event types have different clocks, predicates, or payloads.

### Extract protocol events

```text
wavepeek extract axi --waves dump.fst \
  --scope tb.dut.axi_m \
  --profile axi4 \
  --map aclk=clk \
  --map aresetn=reset_n \
  --include '^m_axi_(aw|w|b|ar|r)' \
  --from 10us --to 11us
```

A protocol extractor emits accepted channel transfers or protocol phase events. It does not automatically reconstruct higher-level transactions, bursts, or packets. Check the resolved mappings before interpreting empty output.

## Human and machine output

Human-readable output is the default and is suitable for focused inspection. Use `--json` when a script needs one complete result and `--jsonl` when rows should be consumed incrementally.

For example:

```text
wavepeek value --waves dump.fst \
  --scope tb.dut.cpu --at 120ns --signals state --json
```

produces a single envelope shaped like:

```json
{"type":"result","command":"value","context":{"scope":"tb.dut.cpu"},"data":[{"time":"120ns","signals":[{"path":"tb.dut.cpu.state","relative_path":"state","value":"4'h3"}]}],"diagnostics":[]}
```

Commands with `--max` also report:

```json
{"complete":false,"returned":50,"limit":50,"total":null}
```

`complete: false` means the returned rows do not cover the selected result set. `--max unlimited` scans the complete selected set, but use it only for a narrow query, `--summary`, or machine processing where the full stream is required.

A successful JSONL stream starts with `begin`, contains zero or more `data` and `diagnostic` records, and ends with `end`. Treat a nonzero process exit, a `fatal` record, or a missing final `end` as incomplete input.

## Safety and inquiry discipline

- Never inspect `.fst`, `.fsdb`, or SHM components with generic text or binary tools. Avoid reading raw `.vcd` directly; dumps can be large and timing-sensitive.
- Use `info` before inventing timestamps for an unfamiliar dump. Explicit time tokens require an integer plus `zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, or `s`; bare numbers and decimal tokens are invalid Wavepeek input.
- Keep queries bounded with scopes, filters, signal lists, time windows, hierarchy depth, and row limits. Prefer narrowing a query over dumping more rows.
- With `--scope`, ordinary query names may be relative to that scope or canonical paths inside it. Without `--scope`, use canonical dump-derived paths. Protocol mappings are relative to their selected scope.
- Use `[n:n]` for one projected bit. `[n]` remains ordinary waveform path syntax.
- Write integral literals in SystemVerilog form, such as `8'hff`, `64'h10`, or `4'b10xz`. C-style `0xff` and `0b1010` literals are invalid.
- For synchronous logic, put the clock or edge in `--on` and the tested condition in `--eval` or `--when`. Write `--on 'posedge clk'`, not `@(...)`.
- Use native sampling for asynchronous or dump-state questions. Use pre-edge sampling for what clocked RTL sampled. Event `time` and value `sample_time` can differ.
- When following a `change`, `property`, or extraction row with `value`, query `sample_time` to reproduce the sampled values unless same-edge native state is intentional.
- Use dense `change` for event-aligned raw signal rows, `property` for derived Boolean results, and `extract` when payload values or protocol semantics belong on the same event.
- Select one concrete protocol interface per invocation. Keep auto-mapping includes interface-specific and add explicit `--map` entries for irregular names.
- Do not infer transaction completion from one AXI channel row, one APB Setup row, or one AHB address row. Use the required completion channel or phase, or a bundled scoreboard example.
- In JSON, inspect `diagnostics` and `summary`. In JSONL, also require the terminal `end` record. An empty result is trustworthy only after mappings, scope, time range, and completeness are checked.
- FST is usually the best format for repeated scripted queries. Large VCD input can be expensive, and every FSDB process pays native reader setup cost.

## Bundled examples

The examples demonstrate common processing above Wavepeek's machine output:

- [Output diff](examples/output_diff/README.md): finds the first or all divergent row blocks between two JSON or JSONL results.
- [AXI scoreboard](examples/axi_scoreboard/README.md): joins AR/R and AW/W/B channel transfers into AXI4 read and write transactions.
- [AHB scoreboard](examples/ahb_scoreboard/README.md): joins address, stall, and data-completion phases into AHB transfers.
- [APB scoreboard](examples/apb_scoreboard/README.md): joins Setup, waited Access, and completion events into APB transactions.

Each scoreboard accepts stdin or a file. A typical file workflow is:

```text
wavepeek extract axi ... --max unlimited --jsonl > axi.jsonl
python3 examples/axi_scoreboard/axi_scoreboard.py axi.jsonl
```

Replace `...` with the mapped command for one interface; the linked README contains a complete invocation. These scripts are compact reconstruction examples, not protocol checkers.

## Bundled helper scripts

Run helpers from the extracted skill directory:

```text
$ python3 scripts/time_convert.py 1.5ns
1500ps

$ python3 scripts/time_convert.py 1500ps --to ns
1.5ns

$ python3 scripts/time_math.py 10ns + 250ps
10250ps
```

`time_convert.py` performs exact conversion across `zs` through `s`. Without `--to`, it emits the largest exact integer token suitable for Wavepeek. `time_math.py` adds or subtracts values with different units without floating-point arithmetic.

## Reference map

Read only the files needed for the current question.

### Start here

- [Overview](references/index.md): purpose, operating model, and the full documentation entry points.
- [Quickstart](references/quickstart.md): installation, optional FSDB/SHM builds, skill extraction, and first run.

### Concepts

- [Commands](references/commands.md): command selection, output modes, limits, summaries, and deterministic ordering.
- [Waveform formats](references/waveforms.md): VCD, FST, FSDB, SHM support, performance, and repeated-query costs.
- [Paths, signals and scopes](references/paths.md): canonical and scoped names, bit projections, and ambiguous FSDB paths.
- [Time units and windows](references/timeunits.md): accepted time tokens, inclusive windows, alignment errors, and helper scripts.
- [Clocks and sampling](references/sampling.md): event expressions, native and pre-edge modes, `time`, `sample_time`, and boundary behavior.
- [Boolean conditions](references/predicates.md): common predicate forms and SystemVerilog literal syntax.

### Practical usage

- [Explore dump](references/explore-dump.md): `info`, hierarchy navigation, and signal discovery.
- [Inspect values](references/inspect-values.md): point sampling, asynchronous changes, clocked tables, and clock periods.
- [Evaluate properties](references/evaluate-properties.md): first matches, truth transitions, exact counts, and asynchronous predicates.
- [Extract transfers](references/extract-transfers.md): generic handshakes, payloads, and multi-source extraction.
- [Extract AMBA AXI](references/extract-axi.md): profiles, mapping, channel rows, and AXI limitations.
- [Extract AMBA AXI-Stream](references/extract-axis.md): stream beats, payloads, and implicit-high `TREADY`.
- [Extract AMBA AHB](references/extract-ahb.md): address/data pipeline events, stalls, and initial data-phase context.
- [Extract AMBA APB](references/extract-apb.md): Setup, Access waits, completions, and `PREADY` modes.
- [Extract AMBA ATB](references/extract-atb.md): transfer, flush, and synchronization-request events.

### Exact contracts

- [CLI help reference](references/cli-reference.md): generated command syntax, options, defaults, profiles, and mappings.
- [Machine output format](references/machine-output.md): normative JSON/JSONL shapes, diagnostics, summaries, fatal records, and exit codes.
- [Event expression language](references/event-expressions.md): event forms, edge semantics, unions, `iff`, precedence, and grammar.
- [Boolean expression language](references/boolean-expressions.md): operand types, casts, conversions, operators, precedence, and grammar.

## Final checks

Before reporting a waveform conclusion:

- confirm the dump, scope, signal paths, clock, and queried interval;
- confirm the sampling mode matches the question and use `sample_time` consistently;
- inspect mappings before trusting protocol results;
- when a summary is present, confirm `summary.complete` is true before claiming exhaustive coverage;
- inspect diagnostics and do not treat truncated or fatal machine output as complete;
- identify the channel or phase that proves protocol completion.
