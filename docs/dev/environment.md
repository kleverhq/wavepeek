# Development Environment

`wavepeek` is developed container-first. The devcontainer and CI image keep Rust, Python helpers, actionlint, fixtures, and git hooks aligned so contributors do not debug two subtly different universes, which is traditionally where the gremlins live.

## Containers

Local interactive work uses `.devcontainer/devcontainer.json`, which targets the `dev` stage in `.devcontainer/Dockerfile`. CI and release automation use `.devcontainer/devcontainer.ci.json`, which targets the leaner `ci` stage from the same Dockerfile.

Recipes in `justfile` require `WAVEPEEK_IN_CONTAINER=1`. Set it only inside a wavepeek-managed devcontainer or CI image; outside the container, install or enter the proper environment instead of bypassing the guard.

Run `just dev-setup` after opening or rebuilding the devcontainer. It verifies tool availability and installs the pre-commit and commit-msg hooks.

The devcontainer may prepare host-side mount sources under `~/.cache/wavepeek`, including the Verdi mount source at `~/.cache/wavepeek/verdi`. Treat that directory as wavepeek-managed cache state: do not place manual installs or durable files there.

## Codex Cloud Setup

For first-time Codex bootstrap, run `bash tools/codex/codex_setup.sh`. This direct script path exists because the first bootstrap may need to install or repair tools before `just` recipes are safe to assume. After the environment has `just`, use `just codex-resume` for maintenance after cache resume.

Codex setup uses the same RTL fixture location as the devcontainer and may populate missing fixtures under `RTL_ARTIFACTS_DIR`.

## Fixture Location

Large RTL fixtures are baked into the devcontainer and CI image under `RTL_ARTIFACTS_DIR`, which the container sets to `/opt/rtl-artifacts`. That path is the only supported runtime fixture location.

The shared environment contract lives in `.devcontainer/env_contract.sh`. Update that file, container provisioning, and Codex helper behavior together when fixture versions or layout change.

## Verdi / FSDB Development

FSDB work is optional and local-only unless a task explicitly says otherwise. The devcontainer sets `VERDI_HOME=/opt/verdi`, usually backed by the host-managed cache mount prepared from the host `VERDI_HOME`. Use `just check-fsdb-env` to distinguish available, skipped, and broken SDK states.

The full FSDB build, fixture, benchmark, and repository-safety contract lives in `fsdb.md`.

## Debug Mode

`DEBUG=1` enables maintainer-only internal diagnostics and hidden controls. Hidden controls are unstable implementation details and are not part of the public CLI contract, even when debug mode exposes them.

## Temporary Files

Use repository-root `tmp/` for scratch files, ad hoc logs, temporary benchmark captures, and other disposable working artifacts. It is ignored by git and may be created freely.

Never globally clean `tmp/` or delete arbitrary existing files there. Other agents or the user may own them. If a temporary artifact needs review or must survive across sessions, move it intentionally into a tracked location such as `docs/tracker/wip/` and explain why.
