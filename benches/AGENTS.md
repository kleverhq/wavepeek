# Bench Target Guide

This directory contains Rust `cargo bench` targets used for internal
microbenchmarks.

## Parent Maps

- Repository map: `AGENTS.md`

## Source of Truth

- Benchmark workflow and quality gates: `docs/DEVELOPMENT.md`
- Expression-phase scope and boundaries: `docs/expression_roadmap.md`

Keep benchmark targets deterministic and focused on internal parser/evaluator
surfaces, not end-to-end CLI flows.
