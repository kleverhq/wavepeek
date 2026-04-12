# Design Documentation

## Overview

wavepeek is a command-line tool for RTL waveform inspection. It provides deterministic, machine-friendly output and a minimal set of primitives that compose into repeatable debug recipes.

The product exists to close the waveform access gap for automation. RTL debug usually depends on visually scanning dense temporal data in GUI viewers, but LLM agents, CI jobs, and post-simulation scripts need a textual, deterministic interface instead. wavepeek turns waveform dumps into bounded, composable command results that can be reasoned about, piped, and checked automatically.

The primary users are LLM-driven debugging workflows and other automation that need stable output contracts. Humans are still expected to use GUI viewers for open-ended interactive exploration, but wavepeek remains useful for scripting, repeatable queries, and compact inspections.

## Scope

### In Scope

- VCD and FST dump support
- Signal discovery, search, and hierarchy navigation
- Point-in-time value sampling and bounded time-range inspection
- Property checks over event-selected timestamps
- Stateless CLI execution with deterministic output

### Out of Scope

- GUI or TUI waveform viewing
- Real-time waveform streaming
- Live simulator connections
- Waveform diffing and comparison

### Future Considerations

- MCP-style agent integration
- Additional waveform formats such as FSDB, VPD, and WLF

## Design Principles

1. **LLM-first.** Output formats, command structure, and error messages should be easy for machine clients to consume reliably.
2. **Self-documenting I/O.** Commands should read like precise descriptions of intent. Human-readable output is the default user experience; strict machine output is opt-in with `--json`.
3. **Composable commands.** Each command should do one focused job well so automation can combine them into repeatable debug recipes.
4. **Deterministic output.** Identical inputs should produce identical observable outputs.
5. **Stable machine contracts.** JSON output is versioned through an explicit `$schema` URL, while human-readable output stays intentionally more flexible.
6. **Minimal footprint.** wavepeek stays stateless, fast to start, and free of background services.

## Document Map

Start here when you need the design overview and navigation map. The design corpus is intentionally split by ownership:

- `architecture.md` — internal engineering architecture, dependencies, execution strategy, and testing strategy.
- `open_questions.md` — unresolved design questions that are intentionally kept out of the stable contracts.
- `contracts/command_model.md` — **normative** cross-cutting command semantics such as time normalization, name resolution, bounded output, ordering, and output-mode rules.
- `contracts/machine_output.md` — **normative** stdout/stderr, JSON envelope, schema-linkage, warning, and exit-code contracts.
- `contracts/expression_lang.md` — **normative** expression-language syntax and semantics for `change --on` and `property --on` / `property --eval`.
- `reference/cli.md` — **derived** operator guide for the command families. It explains when to use each command but deliberately avoids duplicating flag tables.

The exact CLI surface is code-first. Command names, flags, defaults, requiredness, and help examples are authoritative in `src/cli/`, `wavepeek --help`, `wavepeek <command> --help`, and `wavepeek schema`. The documents under `contracts/` remain authoritative only for semantics that code alone does not express clearly enough.
