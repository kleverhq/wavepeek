# Design Docs Guide

This directory is the canonical entrypoint for product design documentation.

## Parent Map

- Documentation map: `../AGENTS.md`

## Source of Truth

- Design entrypoint and document map: `index.md`
- Internal architecture and testing strategy: `architecture.md`
- Open design questions, backlog, and tech debt: `../BACKLOG.md`
- Derived operator-facing CLI guide: `reference/cli.md`
- Documentation/help semantics for reference docs: `contracts/documentation_surface.md`
- Exact CLI surface for reference docs: `../../src/cli/`, `wavepeek -h`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`
- Machine-readable output contract: `../../schema/wavepeek.json` and `wavepeek schema`

## Child Maps

- Normative semantics contracts: `contracts/AGENTS.md`
