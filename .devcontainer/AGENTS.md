# Devcontainer Guidance

## Scope

This directory owns local and CI container definitions, fixture provisioning, and environment-contract helpers.

## Source of Truth

- Container workflow: `../docs/dev/environment.md`
- Quality gates: `../docs/dev/quality.md`
- Container configs and provisioning: `Dockerfile`, `devcontainer.json`, `devcontainer.ci.json`, `env_contract.sh`

## Local Guidance

- `Dockerfile` uses shared build stages with separate `ci` and `dev` targets; keep CI lean while preserving local GUI/tooling support.
- The workspace mounts the repository parent so sibling worktrees behave normally in parallel branch workflows.
- `initialize.sh` prepares host mount sources for OpenCode, Claude Code, Codex, Pi, and GitHub CLI state before container startup.
- Host networking is intentional for VPN-heavy environments.
- `postStartCommand: just dev-setup` reconverges tools and hooks after rebuilds or reopen flows.
- `env_contract.sh` is coupled to Codex setup/resume helpers. Use `../docs/dev/environment.md` and `../docs/dev/automation.md` for the current helper entrypoints, and update the helpers with these files when fixture or environment contracts change.
- The dev profile forces X11 for waveform GUI tooling; CI enables UID remapping so non-root build/test commands can write the workspace.
- GitHub Actions creates a transient `.devcontainer.json` symlink to `devcontainer.ci.json`; keep that workflow compatibility in mind when renaming configs.

## Safety

Large waveform fixtures are baked into the image by the `rtl_artifacts` stage. Runtime tests should not download them from the network. When bumping `WAVEPEEK_RTL_ARTIFACTS_VERSION`, rebuild both container targets and run `just ci` plus `just pre-commit` inside the container.
