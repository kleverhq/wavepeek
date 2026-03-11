# Parser Microbench Guide

This directory contains parser/tokenization microbenchmark helper scripts and
committed run artifacts for expression-engine phases.

The Rust Criterion target for this area lives at `bench/expr/expr_c1.rs` and is
wired through `Cargo.toml` `[[bench]]` metadata.

## Parent Maps

- Performance map: `bench/AGENTS.md`
- Repository map: `AGENTS.md`

## Source of Truth

- Benchmark workflow and command contracts: `docs/DEVELOPMENT.md`
- Expression phase boundaries and acceptance: `docs/expression_roadmap.md`

## Child Maps

- Committed run artifacts: `bench/expr/runs/AGENTS.md`
