# Self-Documenting CLI Docs Proposal

## Status

- Status: accepted and implemented
- Scope retained here: historical staging note only

This file was the temporary proposal used to define the layered-help and
embedded-docs target before implementation. It is no longer a normative
contract.

Use these canonical sources instead:

- [`design/contracts/documentation_surface.md`](design/contracts/documentation_surface.md) for layered-help semantics, embedded docs topics, search, export, and packaged skill ownership.
- [`design/reference/cli.md`](design/reference/cli.md) for the thin operator guide.
- [`../src/cli/`](../src/cli/) plus generated help (`wavepeek -h`, `wavepeek --help`, `wavepeek help <command-path...>`, `wavepeek docs --help`) for the exact CLI surface.
- [`../schema/wavepeek.json`](../schema/wavepeek.json) and `wavepeek schema` for precise stable JSON shapes.

The historical design and implementation trail remains in the active exec plan
at [`exec-plans/active/2026-04-18-self-documenting-cli-docs/PLAN.md`](exec-plans/active/2026-04-18-self-documenting-cli-docs/PLAN.md).
