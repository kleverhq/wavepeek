# Performance Guide

This directory hosts performance work: benchmarks, measurement harnesses, and
repeatable performance scenarios.

## Parent Map

- Repository map: `../AGENTS.md`

## Source of Truth

- Container-first workflow and commands: `../docs/DEVELOPMENT.md`
- Runtime and output contracts to preserve while optimizing: `../docs/design/contracts/command_model.md`, `../docs/design/contracts/machine_output.md`

## Areas

- CLI end-to-end performance scenarios: `e2e/`
- Expression microbenchmark harness and runs: `expr/AGENTS.md`

Run performance scenarios in the devcontainer/CI environment to keep fixture
availability and runtime behavior aligned with project gates.
