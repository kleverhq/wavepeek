---
id: commands/overview
title: Command overview
summary: Choose the right command family before opening exact command help.
section: commands
see_also:
  - commands/help
  - commands/docs
  - commands/skill
  - reference/command-model
  - reference/machine-output
---
# Command overview

Use this topic to choose the right command family. It is not the exact flag reference. For exact syntax, defaults, required flags, and examples, run `wavepeek help <command-path...>` or `wavepeek <command> --help`.

## Help and documentation

Use `help` when you want the detailed reference layer directly, especially for nested paths such as `wavepeek help docs show`.

Use `docs` when command help is not enough. It is the packaged narrative surface for command guidance, workflows, troubleshooting, topic discovery, and export.

## Helper commands

Use `schema` when a machine client needs the canonical JSON contract for `--json` outputs. It prints the schema artifact directly and does not require a waveform file.

Use `skill` when a coding agent needs the packaged Wavepeek skill Markdown from the installed build.

## Waveform inspection commands

Use `info` first when you need dump metadata before running time-based queries. It reports the dump time unit and normalized start/end bounds that other commands use.

Use `scope` to explore hierarchy structure. It is the stable way to discover scope paths before narrowing later queries to a smaller part of the design.

Use `signal` after `scope` to inspect the signals available in a selected scope. Recursive mode broadens that view into child scopes while preserving deterministic ordering.

Use `value` for exact point-in-time sampling. It is the most direct command when you already know the signal set and want one normalized timestamp.

Use `change` to inspect value transitions across a bounded time range. Trigger selection comes from `--on`; expression syntax lives in `reference/expression-language`.

Use `property` when you want to evaluate a logical expression on event-selected timestamps instead of printing raw signal snapshots. Capture modes control whether you keep every match or only state transitions such as asserts and deasserts.

## Which document is normative?

Use this overview to choose a command quickly. When exact flags matter, defer to generated help. When behavioral semantics matter, use the reference topics under `reference/*`. When exact JSON shapes matter, use `wavepeek schema`.
