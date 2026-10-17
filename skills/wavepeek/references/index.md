# Overview

`wavepeek` is a command-line tool for RTL waveform inspection. It provides deterministic, machine-friendly output and a small set of primitives that compose into repeatable debug recipes.

The primary use cases are LLM-driven debugging workflows and other automation. For open-ended interactive exploration, humans may still prefer GUI viewers, but `wavepeek` is useful for scripting, repeatable queries, and compact inspections.

`wavepeek` is a stateless CLI. It starts on demand and does not require a background service. It is not a GUI or TUI waveform viewer. It does not provide real-time waveform streaming, live simulator connections, or waveform comparison.

The main usage flow is simple:

- Provide a VCD/FST/FSDB file or SHM database directory.
- Specify an inspection command and its arguments.
- Get results in human-readable, JSON, or JSONL format.
- Analyze the output and repeat.

For example, this command returns the value of the `top.data` signal at the `10ns` time point:

```text
$ wavepeek value --waves dump.vcd --at 10ns --signals top.data
@10ns top.data=8'h0f
```

To run `wavepeek` on your machine, see [Quickstart](quickstart.md) or try it in your browser with [Playground](https://kleverhq.github.io/wavepeek).

## Concepts

- [Commands](commands.md) - command model and common conventions.
- [Waveform formats](waveforms.md) - formats, performance, and optional native readers.
- [Paths, signals and scopes](paths.md) - canonical paths, relative paths, and bit projections.
- [Time units and windows](timeunits.md) - time tokens and query boundaries.
- [Clocks and sampling](sampling.md) - event times, sample times, and pre-edge sampling.
- [Boolean conditions](predicates.md) - Boolean expressions over waveform values.

## Usage

- [Explore dump](explore-dump.md) - get dump bounds, navigate the hierarchy, and search signals.
- [Inspect values](inspect-values.md) - sample values at explicit points and get a table of changes.
- [Evaluate properties](evaluate-properties.md) - evaluate Boolean expressions and check whether a property holds.
- [Extract transfers](extract-transfers.md) - find transfers and get payload data under handshakes and valid/ready strobes.
- [Extract AMBA AXI](extract-axi.md) - map signals, get a table of transfers, and use AXI profiles.
- [Extract AMBA AXI-Stream](extract-axis.md) - map signals, get a table of transfers, and use AXI-Stream profiles.
- [Extract AMBA AHB](extract-ahb.md) - map signals, get a table of phase events, and use AHB profiles.
- [Extract AMBA APB](extract-apb.md) - map signals, get a table of phase events, and use APB profiles.
- [Extract AMBA ATB](extract-atb.md) - map signals, get a table of events, and use ATB profiles.

## Reference

- [CLI help reference](cli-reference.md) - complete CLI help reference.
- [Machine output format](machine-output.md) - JSON, JSONL, diagnostics, and exit codes.
- [Event expression language](event-expressions.md) - event expression language contract.
- [Boolean expression language](boolean-expressions.md) - Boolean expression language contract.
