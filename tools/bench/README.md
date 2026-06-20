# Benchmark Gate Helpers

Manual performance review is split across small Python helpers:

- `gate.py` clones two refs, builds both, prepares fixtures, runs captures in paired order, and calls comparison.
- `capture.py` captures benchmark artifacts for one ref or checkout.
- `compare.py` compares two capture directories.
- `common.py` contains shared command, JSON, git, and output helpers.

Use the root `justfile` entrypoints (`just bench-gate`, `just bench-capture`, and `just bench-compare`) as the stable interface.
