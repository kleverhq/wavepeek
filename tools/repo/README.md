# Repository Tools

This group owns repository-maintenance helpers that are not part of the public `wavepeek` CLI.

Normal entrypoint:

    python3 -B tools/repo/repo_stats.py

`repo_stats.py` prints stable line-count categories for source, tests, collateral helper code, fixtures, docs, and tracked WIP artifacts.

Optional one-shot host setup for devcontainer GitHub auth:

    bash tools/repo/setup_github_env.sh

`setup_github_env.sh` prompts for a token, writes `~/.config/wavepeek-dev/github.empty.env`, `github.maintainer.env`, and an active `github.env` symlink for trusted maintainer work. The helper tolerates unrelated managed state in `~/.config/wavepeek-dev` and the default empty GitHub env files, but refuses to overwrite existing maintainer or non-default active GitHub env files.
