# Development Environment

Development, CI, release checks, and docs use `.devcontainer/devcontainer.json` and its public image. Proprietary tools belong in a private configuration.

## Host Entrypoint

Agents, credentials, and Git operations run on the host. Run Cargo, Pre-commit, Commitizen, waveform tools, and quality gates through root `./dev`:

```sh
./dev --install-hooks
./dev just check
git commit -m "chore: update development workflow"
git push
```

Install hooks before the first commit in each worktree and after launcher changes. Installation copies `pre-commit`, `commit-msg`, and `./dev` into the worktree's Git directory and sets its `core.hooksPath`. It is idempotent and refuses to replace another hooks path. Branch switches retain the installed copies.

`./dev` starts one container per worktree and configuration, preserving the working directory, arguments, streams, signals, and exit status. Public and private containers can coexist.

`./dev --exec-only COMMAND [ARG ...]` requires a running container and never changes its lifecycle. Git hooks use this mode; start the selected container with `./dev true` before committing.

## Configuration

`WAVEPEEK_DEV_CONFIG` selects a file inside the worktree, relative to its root. It defaults to `.devcontainer/devcontainer.json`. Persist a private selection for commands and hooks in ignored root `.env`:

```dotenv
WAVEPEEK_DEV_CONFIG=.devcontainer.local/devcontainer.json
```

The host sources `.env` as trusted Bash; an explicit host `WAVEPEEK_DEV_CONFIG` takes precedence. Profiles define container mounts and environment.

JSON or required workspace/Git mount changes invalidate the selected container; public containers also reject extra bind mounts. Dockerfile, image, and expanded host-variable changes require manual recreation:

```sh
./dev --recreate true
```

Recreation replaces only the selected container. Keep persistent data in the checkout or named volumes.

## Container Contract

The public image provides Rust/WASM, C/C++, Python, documentation, browser, waveform, and quality tools; see `.devcontainer/Dockerfile`. It includes Verdi wrappers but no proprietary SDK, licenses, or default `VERDI_HOME`. Agents, credentials, and GUI tools stay on the host.

The workspace mounts at `/workspaces/<worktree-name>`. Linked worktrees also mount the Git common directory at its host path. Host agent state and credentials are never mounted.

Recipes in `justfile` require `WAVEPEEK_IN_CONTAINER=1`. Do not set it on the host to bypass the guard; use `./dev` instead.

Run `./dev just dev-setup` after container creation or rebuild to verify tools. It does not install hooks. `playground-build` builds the standalone browser app; `playground-preview-build`, `playground-test`, and `playground-serve` combine `/wavepeek/` with documentation at `/wavepeek/latest/`.

## Fixture Location

Selected [Ondas fixtures](https://github.com/kleverhq/ondas-fixtures) are baked into the image at `ONDAS_FIXTURES_DIR=/opt/ondas-fixtures`. Each FST input lives at `fst/<fixture-id>/waveform.fst` beside its `fixture.json` metadata. The Docker build uses the pinned upstream installer to download only the declared fixtures and verify their sizes and SHA-256 checksums. Fixture directories are writable by the container user for derived VCD/FSDB files.

`./dev just prepare-waveform-fixtures` regenerates small fixtures from `tests/fixtures/source/` into ignored `tests/fixtures/generated/`.

Update `.devcontainer/env_contract.sh` and provisioning together when fixture versions or layout change.

## Private Containers and Verdi / FSDB Development

The public configuration ignores host SDK variables; FSDB gates skip. Private profiles explicitly configure installations and licenses.

Copy the public profile into ignored `.devcontainer.local/`:

```sh
mkdir -p .devcontainer.local
cp .devcontainer/devcontainer.json .devcontainer.local/devcontainer.json
```

For host-installed Verdi, change `build.dockerfile` to `../.devcontainer/Dockerfile` and set these fields:

```json
{
    "containerEnv": {
        "WAVEPEEK_IN_CONTAINER": "1",
        "VERDI_HOME": "/opt/verdi"
    },
    "mounts": [
        "type=bind,source=${localEnv:VERDI_HOME},target=/opt/verdi,readonly"
    ]
}
```

Export the host `VERDI_HOME` and select the profile:

```sh
WAVEPEEK_DEV_CONFIG=.devcontainer.local/devcontainer.json ./dev just check-fsdb-env
WAVEPEEK_DEV_CONFIG=.devcontainer/devcontainer.json ./dev just check
```

For a Docker volume, use `type=volume,source=eda-tools,target=/opt/eda,readonly` and set container `VERDI_HOME` to the installation within it. For an existing private image, replace `build` with `image`. Preserve the workspace, user, and `WAVEPEEK_IN_CONTAINER=1` settings.

To derive a private image, tag the public build on the host:

```sh
devcontainer build --workspace-folder "$PWD" --config .devcontainer/devcontainer.json --image-name wavepeek-dev:local
```

Set private `build.context` to `.` and `build.dockerfile` to `Dockerfile`. Start `.devcontainer.local/Dockerfile` with:

```dockerfile
FROM wavepeek-dev:local
```

Add private provisioning and a `.dockerignore` for that build context. Keep credentials and proprietary payloads outside Git and public images; the public build excludes `.env` and `.devcontainer.local/`.

See [fsdb.md](fsdb.md) for SDK requirements and FSDB gates.

## Debug Mode

`DEBUG=1` enables maintainer diagnostics and unstable hidden controls outside the public CLI contract.

## Temporary Files

Use ignored root `tmp/` for scratch files and logs. Never clean it globally or delete others' files. Move artifacts requiring tracked review or handoff into `docs/wip/` with a reason.
