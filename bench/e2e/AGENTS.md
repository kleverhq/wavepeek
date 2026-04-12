# Perf E2E Guide

This directory contains end-to-end CLI performance scenarios and supporting
harness code.

## Parent Maps

- Performance map: `bench/AGENTS.md`
- Repository map: `AGENTS.md`

## Source of Truth

- Build/lint/test workflow: `docs/DEVELOPMENT.md` (Build / Lint / Test)
- Targeted test execution patterns: `docs/DEVELOPMENT.md` (Run A Single Test (Rust))
- CLI behavior contracts under test: `docs/design/reference/cli.md`, `docs/design/contracts/command_model.md`, `docs/design/contracts/machine_output.md`

Run scenarios in the devcontainer/CI environment to keep fixture availability
and runtime behavior aligned with project gates.
