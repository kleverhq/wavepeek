# Testing Guide

Tests should prove the public command and machine-output contracts, not just exercise code paths. The stable behavior references are `docs/public/reference/command-model.md`, `docs/public/reference/machine-output.md`, and `docs/public/reference/expression-language.md`.

## Test Levels

Rust unit tests live next to implementation code under `src/` and cover parsing, evaluation helpers, time handling, value formatting, and edge cases. Integration tests under `tests/` invoke the CLI and assert stdout, stderr, exit codes, warnings, JSON payloads, and deterministic ordering.

Auxiliary Python tests cover repository tooling and benchmark harnesses. Run them with `just test-aux`; individual suites may also be run with `python3 -B -m unittest ...` while iterating.

## Fixtures

Prefer small hand-written `.vcd` fixtures for edge cases. Large representative `.fst` and related RTL artifacts are provisioned by the devcontainer and CI image, not downloaded during tests. Fixture path resolution is documented in `environment.md` and enforced by `just test`, `just ci`, and `just pre-commit`.

Do not read `.fst` dumps as text. Treat binary waveform dumps as binary data and inspect them through `wavepeek`, fixture helpers, or purpose-built tools.

## Snapshots and Manifests

Use Insta snapshots only when rendered diagnostic text, notes, or source echo are part of the contract. Routine negative cases should prefer structured assertions for error layer, code, span, stdout, stderr, and exit status.

Command fixture manifests live under `tests/fixtures/cli/`. Expression manifests live under `tests/fixtures/expr/` and are validated by the fixture-contract tests. Keep manifests small, explicit, and tied to durable command behavior.

## Targeted Versus Full Runs

During tight loops, run the narrowest useful test: a single Cargo test, a focused integration test file, or a helper unittest. Before handoff, run at least `just check`; before merge or when changing behavior, run `just ci`. If a gate cannot run because of environment constraints, record the exact command and failure instead of pretending the fog is a passing test suite.
