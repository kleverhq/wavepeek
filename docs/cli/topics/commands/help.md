---
id: commands/help
title: Help command
summary: Jump from compact syntax help to full command reference without leaving the binary.
section: commands
see_also:
  - intro
  - commands/docs
---
# Help command

Use `wavepeek help` when you want the detailed reference form directly.

## Top-level alias

`wavepeek help` prints the same long help as `wavepeek --help`.

## Nested paths

`wavepeek help <command-path...>` follows nested command paths. This is useful
for commands such as `wavepeek help docs show` where the path is longer than one
token.
