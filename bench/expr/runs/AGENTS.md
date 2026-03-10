# Parser Run Artifacts Guide

This directory stores committed exported parser microbenchmark runs.

## Parent Maps

- Parser microbench map: `bench/expr/AGENTS.md`
- Performance map: `bench/AGENTS.md`

## Source of Truth

- Microbenchmark capture/compare workflow: `bench/expr/capture.py`, `bench/expr/compare.py`
- Repository benchmark policy: `docs/DEVELOPMENT.md`

Run directories should keep deterministic exported `*.raw.csv`, `summary.json`,
and a human-readable `README.md`.
