---
id: commands/overview
title: Command overview
description: Choose the right command family before opening exact command help.
section: commands
see_also:
  - commands/help
  - commands/docs
  - commands/skill
  - reference/command-model
  - reference/machine-output
  - reference/waveform-performance
---
# Command overview

Use this topic to choose the right command family. It is not the exact flag reference. For exact syntax, defaults, required flags, and examples, run `wavepeek help <command-path...>` or `wavepeek <command> --help`.

## Help and documentation

Use `help` when you want the detailed reference layer directly, especially for nested paths such as `wavepeek help docs show`.

Use `docs` when command help is not enough. It is the packaged narrative surface for command guidance, workflows, troubleshooting, topic discovery, and export.

## Helper commands

Use `schema` when a machine client needs the canonical JSON contract for `--json` outputs, `schema --stream` for per-record JSONL contracts, or `schema --input` for structured JSON input documents. It prints the selected schema artifact directly and does not require a waveform file.

Use `skill` when a coding agent needs the packaged Wavepeek skill Markdown from the installed build.

## Waveform inspection commands

Use `info` first when you need dump metadata before running time-based queries. It reports the dump time unit and normalized start/end bounds that other commands use.

Use `scope` to explore hierarchy structure. It is the stable way to discover scope paths before narrowing later queries to a smaller part of the design.

Use `signal` after `scope` to inspect the signals available in a selected scope. Recursive mode broadens that view into child scopes while preserving deterministic ordering.

Use `value` for exact point sampling. It is the most direct command when you already know the signal set and want one or more explicit normalized timestamps.

Use `change` to inspect value transitions across a bounded time range. Trigger selection comes from `--on`; expression syntax lives in `reference/expression-language`.

Use `property` when you want to evaluate a logical expression on event-selected timestamps instead of printing raw signal snapshots. Capture modes control whether you keep every match or only state transitions such as asserts and deasserts.

Use `extract` commands when you want one row per matching synchronous event with ordered payload values sampled at the pre-edge point. `extract apb` covers APB3, APB4, and APB5 Setup, waited Access, and completed Access states from Arm IHI 0024E Issue E. `extract axi` covers AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channel transfers. `extract generic` covers custom handshakes, FIFO pushes and pops, and other transfer-like rows.

When choosing between VCD, FST, and FSDB input or diagnosing unexpectedly slow queries, use `reference/waveform-performance` for format-level performance guidance.

## Which document is normative?

Use this overview to choose a command quickly. When exact flags matter, defer to generated help. When behavioral semantics matter, use the reference topics under `reference/*`. When exact JSON shapes matter, use `wavepeek schema` for envelope output, `wavepeek schema --stream` for JSONL stream records, and `wavepeek schema --input` for structured input documents.
