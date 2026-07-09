# Testing Guide

Tests should prove the public command and machine-output contracts, not just exercise code paths. The stable behavior references are `docs/public/reference/command-model.md`, `docs/public/reference/machine-output.md`, and `docs/public/reference/expression-language.md`.

## Test Levels

Rust unit tests live next to implementation code under `src/` and cover parsing, evaluation helpers, time handling, value formatting, and edge cases. Integration tests under `tests/` invoke the CLI and assert stdout, stderr, exit codes, diagnostics, JSON payloads, and deterministic ordering.

Auxiliary Python tests cover repository tooling, docs-site helpers, and benchmark harnesses. Run them with `just test-aux`; individual suites may also be run with `python3 -B -m unittest ...` while iterating.

## Fixtures

Ordinary reusable waveform fixtures are source-backed. Store their Verilog sources under `tests/fixtures/source/`, declare expected outputs in `tests/fixtures/waveform_policy.json`, and regenerate ignored dumps under `tests/fixtures/generated/` with `just prepare-waveform-fixtures`. The generator uses Icarus Verilog for VCD output and `vcd2fst` for derived FST output.

Checked-in dumps under `tests/fixtures/hand/` are reserved for cases where raw VCD syntax or metadata is the contract, such as event variables, real values, missing initial values, explicit scope kinds, or same-timestamp update ordering. Every checked-in `.vcd` or `.fst` in that directory must have a reason in `tests/fixtures/waveform_policy.json`. Do not commit generated `.fsdb` fixtures.

Large representative `.fst` and related RTL artifacts are provisioned by the devcontainer and CI image, not downloaded during tests. Fixture path resolution is documented in `environment.md` and enforced by `just test`, `just ci`, and `just pre-commit`.

Optional FSDB tests require Verdi. `just test-fsdb` prepares only the generated FSDB fixtures derived from test VCD fixtures and runs `tests/fsdb_cli.rs` with `--features fsdb`; RTL benchmark FSDB artifacts are prepared by the benchmark recipes instead. `fsdb.md` owns the detailed SDK and fixture contract.

Do not read `.fst` or `.fsdb` dumps as text. Treat binary waveform dumps as binary data and inspect them through `wavepeek`, fixture helpers, Verdi tools, or binary-safe metadata commands.

## Snapshots and Manifests

Use Insta snapshots only when rendered diagnostic text, notes, or source echo are part of the contract. Routine negative cases should prefer structured assertions for error layer, code, span, stdout, stderr, and exit status.

Command fixture manifests live under `tests/fixtures/cli/`. Expression manifests live under `tests/fixtures/expr/` and are validated by the fixture-contract tests. Keep manifests small, explicit, and tied to durable command behavior.

## Targeted Versus Full Runs

During tight loops, run the narrowest useful test: a single Cargo test, a focused integration test file, or a helper unittest. Before handoff, run at least `just check`; before merge or when changing behavior, run `just ci`. If a gate cannot run because of environment constraints, record the exact command and failure instead of pretending the fog is a passing test suite.
