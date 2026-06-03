# Codex Tools

This group owns Codex cloud setup and resume helpers. They are a manual projection of the devcontainer environment contract in `.devcontainer/env_contract.sh`.

Normal entrypoints:

    bash tools/codex/codex_setup.sh
    just codex-resume

First-time bootstrap may use the direct shell script because the environment may need to install or repair `just` before recipes are reliable. After bootstrap, prefer `just codex-resume` for maintenance checks.

This group also contains local FSDB environment and fixture helpers surfaced through `just check-fsdb-env`, `just prepare-fsdb-fixtures`, `just test-fsdb`, and FSDB benchmark recipes. Keep those helpers deterministic and free of proprietary Verdi payloads.
