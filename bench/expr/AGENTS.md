# Expression Microbench Guide

This directory contains expression-engine microbenchmark helper scripts,
scenario manifests, and committed run artifacts across expression capabilities.

Rust Criterion targets for this area live at:

- `bench/expr/expr_parser.rs` (parser/tokenization scenarios)
- `bench/expr/expr_event_runtime.rs` (event-runtime scenarios)
- `bench/expr/expr_integral_boolean.rs` (integral-core logical/event scenarios)
- `bench/expr/expr_rich_types.rs` (rich-type logical/event + waveform-host scenarios)

All four targets are wired through `Cargo.toml` `[[bench]]` metadata and captured through
the shared `bench/expr/capture.py` + `bench/expr/compare.py` workflow.

## Parent Maps

- Performance map: `bench/AGENTS.md`
- Repository map: `AGENTS.md`

## Source of Truth

- Benchmark workflow and command contracts: `docs/DEVELOPMENT.md`
- Expression phase boundaries and acceptance: `docs/expression_roadmap.md`

## Child Maps

- Scenario manifests: `bench/expr/scenarios/AGENTS.md`
- Committed run artifacts: `bench/expr/runs/AGENTS.md`
