---
id: commands/docs
title: Docs command
description: Browse packaged topics, search local documentation, and export Markdown.
section: commands
see_also:
  - intro
  - commands/help
  - commands/skill
  - reference/command-model
  - reference/machine-output
---
# Docs command

Use `wavepeek docs` when command help is not enough. The docs command is the installed narrative documentation surface. It is version-matched to the binary you are running and works offline.

For exact syntax and flags, run:

    wavepeek docs --help

## Topic discovery

Run `wavepeek docs topics` to list packaged topic IDs and descriptions. Topic IDs are stable slash-separated names such as `commands/change` or `reference/command-model`.

Run `wavepeek docs show <topic>` to print one topic body as Markdown. Front matter is removed from display output. Add `--description` when you only need the stored description text.

Run `wavepeek docs search <query>` when you do not know the exact topic ID.

## Search behavior

Docs search is deterministic and case-insensitive. Queries are plain text, not regular expressions. They are split on whitespace. Search covers topic ID, title, description, Markdown headings, and Markdown body text.

Matches that cover more distinct query tokens rank ahead of weaker matches. Remaining ties use structural match strength and then topic ID order, so repeated searches with the same installed binary produce stable results.

## JSON support

Only two docs subcommands support stable JSON output:

- `wavepeek docs topics --json`
- `wavepeek docs search <query> --json`

Both use the standard JSON success envelope described by `reference/machine-output` and the exact schema printed by `wavepeek schema`.

Other docs subcommands are human or Markdown surfaces. `docs show --json` and `docs export --json` fail as argument errors instead of silently changing output mode.

## Export behavior

Run `wavepeek docs export <out-dir>` when you need the authored Markdown topic corpus on disk. Export writes one Markdown file per public topic and a deterministic `manifest.json`.

Export does not write the packaged agent skill. Use `wavepeek skill` when you need the skill text.

Export protects existing files. It can populate a missing or empty directory. A non-empty unmanaged directory is rejected. `--force` may replace an empty directory or a previously managed export root with a recognized manifest version. A managed export root is a directory with a `manifest.json` whose `kind` is `wavepeek-docs-export` and whose `export_format_version` is recognized by this binary.
