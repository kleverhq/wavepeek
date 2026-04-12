# Expression Microbench Guide

This directory contains expression-engine microbenchmark Rust targets, the
explicit suite catalog, the unified performance harness, and committed run
artifacts.

Rust Criterion targets for this area live at:

- `bench/expr/expr_syntax.rs` (lexer/parser scenarios)
- `bench/expr/expr_logical.rs` (standalone logical bind/eval scenarios)
- `bench/expr/expr_event.rs` (standalone event bind/match scenarios)
- `bench/expr/expr_waveform_host.rs` (waveform-backed metadata scenarios)

All four targets are wired through `Cargo.toml` `[[bench]]` metadata. The
workflow is owned by `bench/expr/suites.json` plus `bench/expr/perf.py`, which
produce one grouped run directory such as `bench/expr/runs/baseline/`.

## Parent Maps

- Performance map: `bench/AGENTS.md`
- Repository map: `AGENTS.md`

## Source of Truth

- Benchmark workflow and command contracts: `docs/DEVELOPMENT.md`
- Expression semantics and shipped rollout context: `docs/design/contracts/expression_lang.md`, `docs/ROADMAP.md`

## Child Maps

- Committed run artifacts: `bench/expr/runs/AGENTS.md`
