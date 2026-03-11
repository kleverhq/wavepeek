# Expression Microbench Guide

This directory contains expression-engine microbenchmark helper scripts,
scenario manifests, and committed run artifacts across expression phases.

Rust Criterion targets for this area live at:

- `bench/expr/expr_c1.rs` (parser/tokenization scenarios)
- `bench/expr/expr_c2.rs` (event-runtime scenarios)

Both are wired through `Cargo.toml` `[[bench]]` metadata and captured through
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
