# Source Code Guide

This directory contains implementation code.

## Parent Map

- Repository agent map: `../AGENTS.md`

## Source of Truth

- Development workflow and coding conventions: `../docs/DEVELOPMENT.md`
- Product design entrypoint: `../docs/design/index.md`
- Cross-cutting command and output contracts: `../docs/design/contracts/command_model.md`, `../docs/design/contracts/machine_output.md`
- Expression semantics for `change` and `property`: `../docs/design/contracts/expression_lang.md`

## Embedded Docs Runtime

- Runtime loader and helpers for `wavepeek docs` live under `docs/`.
- Packaged Markdown source: `../docs/cli/`
- Completed implementation plan: `../docs/exec-plans/completed/2026-04-18-self-documenting-cli-docs/PLAN.md`
- Keep metadata sourced from embedded Markdown files rather than duplicated as hand-maintained Rust literals.

If guidance conflicts with local context, follow the docs above.
