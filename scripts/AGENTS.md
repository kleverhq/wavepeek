# Scripts Guide

This directory stores repository helper scripts that are usually invoked by CI
workflows or root `../justfile` recipes.

## Parent Map

- Repository map: `../AGENTS.md`

## Source of Truth

- Developer workflow and quality gates: `../docs/DEVELOPMENT.md`
- CI and release automation entrypoints: `../.github/workflows/`
- Script entrypoints and contracts: `../justfile`

Keep scripts deterministic and CI-safe: avoid interactive prompts, keep
stdout/stderr stable, and return explicit non-zero exits on failure.
