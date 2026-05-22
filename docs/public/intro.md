---
id: intro
title: Introduction
summary: Start here to understand wavepeek, its documentation, and the next help layer to use.
section: intro
see_also:
  - commands/overview
  - commands/help
  - commands/docs
  - commands/skill
  - reference/command-model
  - reference/machine-output
---
# Introduction

`wavepeek` is a command-line tool for RTL waveform inspection. It provides deterministic, machine-friendly output and a small set of primitives that compose into repeatable debug recipes.

The product exists to close the waveform access gap for automation. RTL debug usually depends on visually scanning dense temporal data in GUI viewers, but LLM agents, CI jobs, and post-simulation scripts need a textual, deterministic interface instead. `wavepeek` turns waveform dumps into bounded, composable command results that can be reasoned about, piped, and checked automatically.

The primary users are LLM-driven debugging workflows and other automation that need stable output contracts. Humans are still expected to use GUI viewers for open-ended interactive exploration, but `wavepeek` is useful for scripting, repeatable queries, and compact inspections.

## Scope

Default `wavepeek` builds support VCD and FST waveform dumps, hierarchy and signal discovery, point-in-time value sampling, bounded time-range inspection, property checks over event-selected timestamps, and stateless CLI execution with deterministic output. FSDB requires installing with the Cargo feature `fsdb` and the Synopsys Verdi FSDB Reader SDK.

`wavepeek` is not a GUI or TUI waveform viewer. It does not provide real-time waveform streaming, live simulator connections, or waveform diffing and comparison.

## What to expect

`wavepeek` is designed around a few user-visible guarantees:

1. **Machine-friendly output.** Command output, command structure, and error messages should be easy for automation and agents to consume reliably.
2. **Human by default, JSON when requested.** Human-readable output is the default user experience. Stable machine-readable output is opt-in with `--json` where supported.
3. **Composable commands.** Each command does one focused job so scripts and agents can combine commands into repeatable debug recipes.
4. **Deterministic output.** Identical inputs should produce identical observable output.
5. **Stable machine contracts.** JSON output is versioned through an explicit `$schema` URL, while human-readable output stays intentionally more flexible.
6. **Minimal footprint.** `wavepeek` is stateless, fast to start, and does not require a background service.

## Documentation map

The installed documentation is organized by topic type:

- `commands/*` topics help you choose a command family and find the right help layer. They do not replace exact command help.
- `workflows/*` topics show repeatable task recipes.
- `troubleshooting/*` topics explain surprising but valid results and recovery steps.
- `reference/*` topics define stable semantics and contracts such as the command model, machine output, and expression language.

For exact command syntax, defaults, required flags, and examples, use generated help from the installed binary rather than these narrative topics.

## Getting help

Use progressive disclosure when you need help:

- `wavepeek -h` gives compact top-level lookup help.
- `wavepeek --help` gives detailed top-level reference help.
- `wavepeek help <command-path...>` gives detailed help for a top-level or nested command, such as `wavepeek help docs show`.
- `wavepeek docs --help` explains the local documentation command family.
- `wavepeek docs topics` lists packaged topic IDs and summaries.
- `wavepeek docs search <query>` searches topics when you do not know the exact ID.
- `wavepeek skill` prints the packaged agent skill Markdown from the installed build.
