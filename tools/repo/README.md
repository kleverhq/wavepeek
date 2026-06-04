# Repository Tools

This group owns repository-maintenance helpers that are not part of the public `wavepeek` CLI.

Normal entrypoint:

    python3 -B tools/repo/repo_stats.py

`repo_stats.py` prints stable line-count categories for source, tests, collateral helper code, fixtures, docs, and tracked WIP artifacts.

Optional one-shot host setup for devcontainer GitHub auth:

    bash tools/repo/setup_github_env.sh <github-token>

`setup_github_env.sh` writes `~/.config/wavepeek/github.empty.env`, `github.maintainer.env`, and an active `github.env` symlink for trusted maintainer work. If `~/.config/wavepeek` already exists and is not empty, the helper exits and leaves manual edits to the maintainer.
