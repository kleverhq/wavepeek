# Embedded Topics Guide

This directory contains the Markdown topic corpus embedded into `wavepeek docs`.

## Parent Map

- CLI docs map: `../AGENTS.md`

## Source of Truth

- Topic metadata and runtime behavior: `../../design/contracts/documentation_surface.md`
- Exact command reference: `../../../src/cli/`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`

## Topic Areas

- Concept topics: `concepts/`; explain semantics and operator guidance without duplicating flag tables.
- Narrative command companions: `commands/`; complement help with caveats, workflows, and context, but do not become a second authoritative flag reference.
- Workflow topics: `workflows/`; show repeatable task recipes that build on command help.
- Troubleshooting topics: `troubleshooting/`; explain why valid commands produce surprising results and how to recover safely.

## Related Contracts

- Cross-cutting command semantics: `../../design/contracts/command_model.md`
- Runtime error and warning behavior: `../../design/contracts/machine_output.md`
- Expression semantics for workflow topics: `../../design/contracts/expression_lang.md`

Keep topic IDs stable, slash-separated, and narrative-focused.
