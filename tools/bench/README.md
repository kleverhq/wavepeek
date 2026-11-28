# Benchmark Gate Helpers

Manual performance review is split across small Python helpers:

- `gate.py` builds two refs and captures them through the E2E runner's labeled round-robin schedule. The public release profile requires VCD and FST. The private release profile adds FSDB, SHM, WLF, and VWDB.
- `capture.py` captures benchmark artifacts for one binary ref using current benchmark scripts, catalogs, and fixtures.
- `compare.py` checks run manifests and same-format payloads and timing. It applies explicit metadata exceptions to cross-format comparisons. When median timing is the only failure, it runs best-sample confirmation for the failed tests.
- `common.py` contains shared command, JSON, git, and output helpers.

Use `just bench-release-gate` for release checks. Use `just bench-gate`, `just bench-capture`, and `just bench-compare` for diagnostic runs. See `docs/benchmarking.md`.
