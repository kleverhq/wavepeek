# Benchmark Gate Helpers

Manual performance review is split across small Python helpers:

- `gate.py` clones two refs, builds both binaries, captures them through the current E2E runner's labeled round-robin schedule, and calls comparison.
- `capture.py` captures benchmark artifacts for one binary ref using current benchmark scripts, catalogs, and fixtures.
- `compare.py` compares two capture directories, applies the explicit cross-format metadata ignore list, and, when same-format median timing is the only failure with no functional mismatches, missing/invalid artifacts, or timeout warnings, runs best-sample confirmation for the failed tests.
- `common.py` contains shared command, JSON, git, and output helpers.

Use the root `justfile` entrypoints (`just bench-gate`, `just bench-capture`, and `just bench-compare`) as the stable interface.
