# Expression Scenario Manifest Guide

This directory stores committed benchmark scenario manifests for expression
microbenchmark capabilities.

## Parent Maps

- Expression microbench map: `bench/expr/AGENTS.md`
- Performance map: `bench/AGENTS.md`

## Source of Truth

- Capture workflow: `bench/expr/capture.py`
- Compare workflow: `bench/expr/compare.py`
- Benchmark policy: `docs/DEVELOPMENT.md`

Each manifest declares a stable scenario-set identity, the expected Criterion
bench target, and the exact benchmark IDs that must be exported.
