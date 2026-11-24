# Repository Tools

This group owns repository-maintenance helpers that are not part of the public `wavepeek` CLI.

Normal statistics entrypoint:

    python3 -B tools/repo/repo_stats.py

`repo_stats.py` prints stable line-count categories for source, tests, collateral helper code, JSON fixtures, and docs. `Total code` includes source, tests, benchmarks, and tools. `Total lines` adds JSON fixtures and Markdown docs while excluding disposable, generated, and branch-local tracking content.
