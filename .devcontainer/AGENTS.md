# Devcontainer Guidance

## Scope

This directory owns local and CI container definitions, fixture provisioning, and environment-contract helpers.

## Source of Truth

- Container workflow: `../docs/dev/environment.md`
- Optional GitHub auth: `../docs/dev/github-auth.md`
- Quality gates: `../docs/dev/quality.md`
- Container configs and provisioning: `Dockerfile`, `devcontainer.json`, `devcontainer.ci.json`, `initialize.sh`, `setup-github-auth.sh`, `env_contract.sh`

## Local Guidance

- `Dockerfile` uses shared build stages with separate `ci` and `dev` targets; keep CI lean while preserving local GUI/tooling support.
- The workspace mounts the repository parent so sibling worktrees behave normally in parallel branch workflows.
- `initialize.sh` prepares host mount sources under `~/.config/wavepeek-dev` for Claude Code, Codex, Pi, Verdi, and the optional wavepeek GitHub env-file before container startup.
- Do not mount host `~/.config/gh` into the devcontainer; optional GitHub auth uses `~/.config/wavepeek-dev/github.env` and is documented in `../docs/dev/github-auth.md`.
- For external PR review, follow `../docs/dev/github-auth.md`; switching `github.env` to empty does not protect a readable host token file from PR-controlled `initializeCommand`.
- `verdi-tool-wrapper.sh` exposes selected Verdi FSDB utilities on `PATH` and invokes their launchers with bash for compatibility.
- Host networking is intentional for VPN-heavy environments.
- `postStartCommand: just dev-setup` reconverges tools and hooks after rebuilds or reopen flows.
- `env_contract.sh` is coupled to Codex setup/resume helpers. Use `../docs/dev/environment.md` and `../docs/dev/automation.md` for the current helper entrypoints, and update the helpers with these files when fixture or environment contracts change.
- The dev profile forces X11 for waveform GUI tooling; CI enables UID remapping so non-root build/test commands can write the workspace.
- GitHub Actions creates a transient `.devcontainer.json` symlink to `devcontainer.ci.json`; keep that workflow compatibility in mind when renaming configs.
- Inline shell or Python in devcontainer configs and devcontainer-backed workflow `runCmd` blocks is acceptable only for short glue up to 5 logical lines. Longer logic belongs in a separate testable helper under `../tools/`.

## Safety

Do not store PATs in repository files, `.git/config`, breadcrumb files, logs, or shell history. Large waveform fixtures are baked into the image by the `rtl_artifacts` stage. Runtime tests should not download them from the network. When bumping `WAVEPEEK_RTL_ARTIFACTS_VERSION`, rebuild both container targets and run `just ci` plus `just pre-commit` inside the container.
