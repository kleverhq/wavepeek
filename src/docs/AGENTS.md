# Embedded Docs Runtime Guide

This directory contains the runtime loader and helpers for `wavepeek docs`.

## Parent Map

- Source code map: [`../AGENTS.md`](../AGENTS.md)

## Source of Truth

- Packaged Markdown source: [`../../docs/cli/`](../../docs/cli/)
- Completed implementation plan: [`../../docs/exec-plans/completed/2026-04-18-self-documenting-cli-docs/PLAN.md`](../../docs/exec-plans/completed/2026-04-18-self-documenting-cli-docs/PLAN.md)
- Cross-cutting output contracts: [`../../docs/design/contracts/machine_output.md`](../../docs/design/contracts/machine_output.md)

Keep metadata sourced from the embedded Markdown files rather than duplicated as
hand-maintained Rust literals.
