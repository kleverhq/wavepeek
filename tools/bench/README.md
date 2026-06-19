# Benchmark Gate Helpers

`gate.py` owns the manual performance gate used during release review. It clones explicit source refs into `tmp/bench-gate/`, captures benchmark artifacts outside git, and compares two captures with configured regression thresholds.

Use the root `justfile` entrypoints (`just bench-gate`, `just bench-capture`, and `just bench-compare`) as the stable interface.
