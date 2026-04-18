# CLI Reference

This file is a thin operator guide to the shipped command families. It is intentionally derived documentation, not the authoritative flag-by-flag contract.

For exact command syntax, defaults, requiredness, and help examples, use:

- `wavepeek -h`
- `wavepeek --help`
- `wavepeek help <command-path...>`
- `wavepeek <command> --help`
- [`src/cli/`](../../../src/cli/)
- `wavepeek schema` for the `--json` contract

For normative semantics that code alone does not explain clearly enough, use [`../contracts/documentation_surface.md`](../contracts/documentation_surface.md), [`../contracts/command_model.md`](../contracts/command_model.md), [`../contracts/machine_output.md`](../contracts/machine_output.md), and [`../contracts/expression_lang.md`](../contracts/expression_lang.md).

## `help`

Use `help` when you want the detailed reference layer directly, especially for
nested paths such as `wavepeek help docs show`.

## `docs`

Use `docs` when command reference is not enough. It is the packaged narrative
surface for concepts, workflows, troubleshooting, topic discovery, export, and
the shipped agent skill.

## `schema`

Use `schema` when a machine client needs the canonical JSON contract for `--json` outputs. It prints the schema artifact directly and does not require a waveform file.

## `info`

Use `info` first when you need dump metadata before running time-based queries. It reports the dump time unit and the normalized start and end bounds that other commands use.

## `scope`

Use `scope` to explore hierarchy structure. It is the stable way to discover scope paths before narrowing later queries to a smaller part of the design.

## `signal`

Use `signal` after `scope` to inspect the signals available in a selected scope. Recursive mode broadens that view into child scopes while preserving deterministic ordering.

## `value`

Use `value` for exact point-in-time sampling. It is the most direct command when you already know the signal set and want one normalized timestamp.

## `change`

Use `change` to inspect value transitions across a bounded time range. Trigger selection comes from `--on`, and the expression language for that trigger lives in [`../contracts/expression_lang.md`](../contracts/expression_lang.md).

## `property`

Use `property` when you want to evaluate a logical expression on event-selected timestamps instead of printing raw signal snapshots. Capture modes control whether you keep every match or only state transitions such as asserts and deasserts.

## Which Document Is Normative?

Use this guide to choose the right command family quickly. When exact flags matter, defer to help text and [`src/cli/`](../../../src/cli/). When behavioral semantics matter, defer to the contracts under [`../contracts/`](../contracts/).
