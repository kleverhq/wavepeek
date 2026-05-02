# Expression Microbench Guide

This directory contains expression-engine microbenchmark Rust targets, the
explicit suite catalog, the unified performance harness, and committed run
artifacts.

Rust Criterion targets for this area live at:

- `expr_syntax.rs` (lexer/parser scenarios)
- `expr_logical.rs` (standalone logical bind/eval scenarios)
- `expr_event.rs` (standalone event bind/match scenarios)
- `expr_waveform_host.rs` (waveform-backed metadata scenarios)

All four targets are wired through `../../Cargo.toml` `[[bench]]` metadata. The
workflow is owned by `suites.json` plus `perf.py`, which produce one grouped run
directory such as `runs/baseline/`.

## Parent Maps

- Performance map: `../AGENTS.md`
- Repository map: `../../AGENTS.md`

## Source of Truth

- Benchmark workflow and command contracts: `../../docs/DEVELOPMENT.md`
- Expression semantics and shipped rollout context: `../../docs/design/contracts/expression_lang.md`, `../../docs/ROADMAP.md`

## Run Artifacts

Committed exported runs live under `runs/`. The durable committed state is
`runs/baseline/`; other run directories are temporary local or review artifacts
unless a later plan explicitly promotes them. Run directories should keep
namespaced exported `*.raw.csv`, one machine-readable `summary.json`, and a
human-readable `README.md`. Scenario ordering and schema are stable;
generation-time provenance fields can vary from capture to capture.
