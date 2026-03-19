# Expression Run Artifacts Guide

This directory stores committed exported expression microbenchmark runs.

## Parent Maps

- Expression microbench map: `bench/expr/AGENTS.md`
- Performance map: `bench/AGENTS.md`

## Source of Truth

- Microbenchmark catalog and harness: `bench/expr/suites.json`, `bench/expr/perf.py`
- Repository benchmark policy: `docs/DEVELOPMENT.md`

The durable committed state in this directory is `baseline/` plus this map.
Other run directories are temporary local or review artifacts unless a later
plan explicitly promotes them.

Run directories should keep deterministic exported `*.raw.csv`, `summary.json`,
and a human-readable `README.md`.
