# Development Environment

`wavepeek` is developed container-first. The devcontainer and CI image keep Rust, Python helpers, actionlint, fixtures, and git hooks aligned so contributors do not debug two subtly different universes, which is traditionally where the gremlins live.

## Containers

Local interactive work uses `.devcontainer/devcontainer.json`, which targets the `dev` stage in `.devcontainer/Dockerfile`. CI and release automation use `.devcontainer/devcontainer.ci.json`, which targets the leaner `ci` stage from the same Dockerfile.

Recipes in `justfile` require `WAVEPEEK_IN_CONTAINER=1`. Set it only inside a wavepeek-managed devcontainer or CI image; outside the container, install or enter the proper environment instead of bypassing the guard.

Run `just dev-setup` after opening or rebuilding the devcontainer. It verifies tool availability and installs the pre-commit and commit-msg hooks.

## Codex Cloud Setup

For first-time Codex bootstrap, run `bash scripts/codex_setup.sh` until the helper-tool migration in this branch moves that entrypoint under `tools/codex/`. This direct script path exists because the first bootstrap may need to install or repair tools before `just` recipes are safe to assume. After the environment has `just`, use `just codex-resume` for maintenance after cache resume.

Codex setup may populate the cache fallback for RTL fixtures, then `just` propagates the resolved path to runtime and test processes as `WAVEPEEK_RTL_ARTIFACTS_DIR`.

## Fixture Location Resolution

Large RTL fixtures are baked into the devcontainer and CI image under `/opt/rtl-artifacts`. Runtime path resolution prefers `WAVEPEEK_RTL_ARTIFACTS_DIR`, then `RTL_ARTIFACTS_DIR`, then `/opt/rtl-artifacts`, and finally `~/.cache/wavepeek/rtl-artifacts`.

The shared environment contract lives in `.devcontainer/env_contract.sh` and `.devcontainer/resolve_rtl_artifacts_dir.sh`. Update those files, container provisioning, and Codex helper behavior together when fixture versions or layout change.

## Debug Mode

`DEBUG=1` enables maintainer-only internal diagnostics and hidden controls. Hidden controls are unstable implementation details and are not part of the public CLI contract, even when debug mode exposes them.

## Temporary Files

Use repository-root `tmp/` for scratch files, ad hoc logs, temporary benchmark captures, and other disposable working artifacts. It is ignored by git and may be created freely.

Never globally clean `tmp/` or delete arbitrary existing files there. Other agents or the user may own them. If a temporary artifact needs review or must survive across sessions, move it intentionally into a tracked location such as `docs/tracker/wip/` and explain why.
