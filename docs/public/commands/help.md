---
id: commands/help
title: Help command
description: Move from compact lookup help to detailed command help without leaving the binary.
section: commands
see_also:
  - intro
  - commands/docs
  - commands/overview
  - commands/skill
---
# Help command

`wavepeek` uses progressive disclosure: start with a compact reminder, then ask for more detail only when you need it.

## Help layers

Use `wavepeek -h` for compact top-level lookup help. This layer is intentionally short and points you to the deeper layers.

Use `wavepeek --help` for detailed top-level reference help. This layer explains general conventions and lists the available command families.

Use `wavepeek help <command-path...>` for detailed help on a specific command or nested command. For example:

    wavepeek help change
    wavepeek help docs show
    wavepeek help skill

You can also ask a command directly for detailed help with `wavepeek <command> --help` or `wavepeek <command-path...> --help`.

## Where narrative docs fit

Generated help is the authority for exact syntax, flags, defaults, and required arguments. Use `wavepeek docs` when you need narrative guidance, workflows, troubleshooting, or stable semantic reference topics.

Use `wavepeek skill` when you need the packaged agent skill Markdown from the installed build.
