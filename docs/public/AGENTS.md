# Public Docs Guide

This directory contains the Markdown topic corpus embedded into `wavepeek docs`.

## Parent Map

- Documentation map: `../AGENTS.md`

## Source of Truth

- Public introduction and topic map: `intro.md`
- Command-family guidance: `commands/`
- Stable semantics references: `reference/`
- Documentation maintenance workflow and topic metadata rules: `../DEVELOPMENT.md`
- Exact command reference: `../../src/cli/`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`

## Child Maps

- Command topics: `commands/AGENTS.md`
- Reference topics: `reference/AGENTS.md`

Keep topic IDs stable, slash-separated, and user-facing. Do not duplicate exact flag tables from generated help.
