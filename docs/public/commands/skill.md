---
id: commands/skill
title: Skill command
summary: Print the packaged agent skill Markdown from the installed build.
section: commands
see_also:
  - intro
  - commands/help
  - commands/docs
  - reference/command-model
---
# Skill command

Use `wavepeek skill` when a coding agent needs the packaged Wavepeek skill Markdown from the installed binary.

The command prints the authored skill text verbatim. That keeps agent setup version-matched to the build you are running.

For exact syntax, run:

    wavepeek help skill

## When to use it

Use `skill` when you want to install or inspect the compact agent entrypoint without exporting the full docs corpus.

The packaged skill is intentionally short. It routes agents back to generated help, `wavepeek docs topics`, and the stable reference topics instead of duplicating the whole command contract.

## Output behavior

`wavepeek skill` is a human or Markdown surface. It does not support `--json`.

If you need broader offline guidance, use `wavepeek docs` for topic discovery, topic display, search, and export.
