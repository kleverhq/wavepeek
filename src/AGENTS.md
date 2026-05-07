# Source Code Guide

This directory contains implementation code.

## Parent Map

- Repository agent map: `../AGENTS.md`

## Source of Truth

- Development workflow and coding conventions: `../docs/DEVELOPMENT.md`
- Public docs entrypoint: `../docs/public/intro.md`
- Cross-cutting command and output contracts: `../docs/public/reference/command-model.md`, `../docs/public/reference/machine-output.md`
- Expression semantics for `change` and `property`: `../docs/public/reference/expression-language.md`

## Embedded Docs Runtime

- Runtime loader and helpers for `wavepeek docs` live under `docs/`.
- Packaged Markdown topic source: `../docs/public/`
- Packaged skill source: `../docs/skills/wavepeek.md`
- Completed implementation plan for the original docs runtime: `../docs/exec-plans/completed/2026-04-18-self-documenting-cli-docs/PLAN.md`
- Keep metadata sourced from embedded Markdown files rather than duplicated as hand-maintained Rust literals.

If guidance conflicts with local context, follow the docs above.
