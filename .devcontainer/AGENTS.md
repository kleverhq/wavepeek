# Devcontainer Notes For Agents

This directory is designed so local development and CI share one foundation while paying different runtime costs.

## Why this layout exists
- `.devcontainer/Dockerfile` uses multi-stage targets to separate heavy tool builds, a shared base layer, and final `ci`/`dev` profiles. This keeps CI and local environments aligned while preserving layer reuse and rebuild speed.
- `.devcontainer/devcontainer.json` and `.devcontainer/.devcontainer.json` intentionally point to different targets in the same Dockerfile. Local sessions use `dev` (interactive + GUI tools), while automation uses `ci` (lean, predictable runtime) to avoid maintaining two drifting images.

## Non-obvious decisions
- The workspace mounts the repository parent into `/workspaces` (not just this repo) so sibling git worktrees work naturally during parallel branch workflows.
- OpenCode state is bind-mounted from the host and created in `initializeCommand` so auth/session state survives container recreation and first-run mount failures are avoided.
- Host networking is used because bridge networking often breaks routing in VPN-heavy environments.
- `postStartCommand: make bootstrap` runs on each start to re-converge tools/hooks after rebuilds and reopen flows, instead of assuming one-time setup remains valid.
- `safe.directory` is configured automatically so Git inside the container does not block the workspace as dubious when ownership/UID mapping differs.
- The dev profile forces X11 (`WINIT_UNIX_BACKEND=x11`) because this is the most reliable backend for waveform GUI tooling in common VS Code devcontainer setups.
- CI disables UID remapping (`updateRemoteUserUID: false`) to keep ephemeral runner behavior more reproducible.

## RTL fixture provisioning
- Large waveform fixtures are baked into the image at build time under `/opt/rtl-artifacts` by a dedicated Docker stage (`rtl_artifacts`).
- Fixture payload is controlled by `RTL_ARTIFACTS_VERSION` in `.devcontainer/Dockerfile` and is shared by both `ci` and `dev` targets through the common `base` stage.
- Test/runtime commands never download fixtures from the network; `make ci`/`make pre-commit` assert the local fixture payload is present.

## Bumping fixture version
1. Update `RTL_ARTIFACTS_VERSION` in `.devcontainer/Dockerfile` (`rtl_artifacts` stage).
2. Rebuild both container targets and run `make ci` + `make pre-commit` inside container to verify payload and tests.
