# Development Environment

`wavepeek` uses a public, credentialless command-line devcontainer for development, CI, release quality checks, and docs staging. Its configuration is `.devcontainer/devcontainer.json`, built from `.devcontainer/Dockerfile`. Optional private configurations supply proprietary tools independently.

## Host Entrypoint

Agents, Git identity and signing, credentials, commits, pushes, issues, and pull requests stay on the host. Run Cargo, Pre-commit, Commitizen, waveform tools, and repository gates in the container through root `./dev`:

```sh
./dev --install-hooks
./dev just check
git commit -m "chore: update development workflow"
git push
```

`./dev --install-hooks` explicitly installs reviewed `pre-commit`, `commit-msg`, and `./dev` copies under the current worktree's Git directory and activates them with worktree-local `core.hooksPath`. Each linked worktree therefore uses and updates its own hook copies. Installation is idempotent and refuses to replace another configured worktree hooks path. Branch switches do not alter the active copies.

`./dev` finds the enclosing Git worktree when called from any directory inside it, preserves that relative directory in the container, starts the selected container when needed, and passes command arguments, standard streams, signals, and exit status through unchanged. Each absolute worktree and configuration pair has its own runtime container. Public and private containers can coexist; image layers remain shared through Docker.

Use `./dev --exec-only COMMAND [ARG ...]` when a caller must use only an existing container. The installed Git hooks use this mode, so each worktree container must be started explicitly with a normal command such as `./dev true` before `git commit`. This mode never starts, builds, restarts, recreates, or removes a container.

`WAVEPEEK_DEV_CONFIG` selects a configuration relative to the worktree root and must resolve inside it. The default is `.devcontainer/devcontainer.json`. An optional root `.env` supplies a persistent selection for commands and installed Git hooks:

```dotenv
WAVEPEEK_DEV_CONFIG=.devcontainer.local/devcontainer.json
```

The host sources `.env` as trusted Bash configuration. An explicit host `WAVEPEEK_DEV_CONFIG` overrides its selection. Only values selected by the profile's mounts, `containerEnv`, or `remoteEnv` are passed into the container.

If the selected JSON or required workspace/Git mounts change, `./dev` rejects the existing container. Public containers also reject additional bind mounts. Recreate only the selected container explicitly:

```sh
./dev --recreate true
```

Dockerfile/image changes and changes to expanded host environment values also require explicit recreation; the launcher does not fingerprint them. Keep persistent data in the checkout or named volumes. `--exec-only` never recreates containers. Containers created by older launcher revisions remain untouched and are not selected by the new profile labels. Reinstall reviewed hook copies with `./dev --install-hooks` after changing the launcher.

## Container Contract

The public image includes Rust and its WASM target, `wasm-bindgen`, Cargo tools, C/C++ compilers, Python, Material/Mike, Playwright's headless Chromium for Playground checks, actionlint, hooks, GitHub CLI, Icarus Verilog, waveform converters, benchmark tooling, and project-owned Verdi launcher wrappers. It contains no proprietary SDK, licenses, or default `VERDI_HOME`. It does not include coding agents, Node.js, a nested Devcontainer CLI, Surfer, GUI forwarding, or local GitHub credential setup.

The workspace is mounted at `/workspaces/<worktree-name>`. For linked worktrees, `./dev` also mounts the Git common directory at the same absolute host and container path so Git follows the worktree's `.git` pointer correctly. No agent state, host GitHub configuration, or local token file is mounted.

Recipes in `justfile` require `WAVEPEEK_IN_CONTAINER=1`. Do not set it on the host to bypass the guard; use `./dev` instead.

Run `./dev just dev-setup` after creating or rebuilding the container to verify tool availability. It does not install or rewrite host hooks. Use `./dev just playground-build` for the standalone browser build. `playground-preview-build`, `playground-test`, and `playground-serve` compose the Playground at `/wavepeek/` with current documentation at `/wavepeek/latest/` for local checks and inspection.

## Fixture Location

Large RTL fixtures are baked into the image under `RTL_ARTIFACTS_DIR=/opt/rtl-artifacts`. That path is the only supported runtime fixture location.

Small source-backed integration fixtures are regenerated inside the repository with `./dev just prepare-waveform-fixtures`. Their checked-in sources live under `tests/fixtures/source/`; generated VCD/FST outputs live under ignored `tests/fixtures/generated/`.

The container environment contract lives in `.devcontainer/env_contract.sh`. Update it with container provisioning when fixture versions or layout change.

## Private Containers and Verdi / FSDB Development

The public configuration ignores host SDK variables and never mounts Verdi. Its FSDB gates report a skip. The launcher does not discover EDA installations, validate host SDKs, or configure licenses.

Keep machine-specific configuration in ignored `.devcontainer.local/`. For a host installation, create `.devcontainer.local/devcontainer.json`:

```json
{
    "name": "wavepeek private",
    "build": {
        "context": "..",
        "dockerfile": "../.devcontainer/Dockerfile",
        "options": ["--network=host"]
    },
    "containerEnv": {
        "WAVEPEEK_IN_CONTAINER": "1",
        "VERDI_HOME": "/opt/verdi"
    },
    "mounts": [
        "type=bind,source=${localEnv:VERDI_HOME},target=/opt/verdi,readonly"
    ],
    "runArgs": ["--network=host"],
    "workspaceFolder": "/workspaces/${localWorkspaceFolderBasename}",
    "workspaceMount": "source=${localWorkspaceFolder},target=/workspaces/${localWorkspaceFolderBasename},type=bind",
    "remoteUser": "ubuntu",
    "updateRemoteUserUID": true,
    "postCreateCommand": "git config --global --add safe.directory \"$PWD\""
}
```

Set the host `VERDI_HOME` used by this explicit bind mount, then select the profile:

```sh
WAVEPEEK_DEV_CONFIG=.devcontainer.local/devcontainer.json ./dev just check-fsdb-env
WAVEPEEK_DEV_CONFIG=.devcontainer/devcontainer.json ./dev just check
```

For tools installed in a Docker volume, replace the bind with `type=volume,source=eda-tools,target=/opt/eda,readonly` and set the container's `VERDI_HOME` to the installation path inside that volume. A profile can also use an existing private `image` instead of `build`, with no SDK mount. Keep the workspace layout, user, and `WAVEPEEK_IN_CONTAINER=1` contract.

To derive a private Dockerfile from the public image, build and tag it on the host:

```sh
devcontainer build --workspace-folder "$PWD" --config .devcontainer/devcontainer.json --image-name wavepeek-dev:local
```

Use `.devcontainer.local/Dockerfile` as the private profile's `build.dockerfile`, with `build.context` set to `.`:

```dockerfile
ARG WAVEPEEK_PUBLIC_IMAGE=wavepeek-dev:local
FROM ${WAVEPEEK_PUBLIC_IMAGE}
```

Add private provisioning there as needed. The public Docker context excludes `.env` and `.devcontainer.local/`; a private build context needs its own `.dockerignore` for credentials and unrelated inputs. Keep proprietary payloads and infrastructure details outside Git and public images.

SDK checks run inside the selected environment. Use `./dev just check-fsdb-env` to distinguish available, skipped, and broken SDK states. Reader library selection and absolute-path linking follow Ondas's FSDB contract.

The full FSDB build, fixture, benchmark, and repository-safety contract lives in `fsdb.md`.

## Debug Mode

`DEBUG=1` enables maintainer-only internal diagnostics and hidden controls. Hidden controls are unstable implementation details and are not part of the public CLI contract, even when debug mode exposes them.

## Temporary Files

Use repository-root `tmp/` for scratch files, ad hoc logs, temporary benchmark captures, and other disposable working artifacts. It is ignored by Git and may be created freely.

Never globally clean `tmp/` or delete arbitrary existing files there. Other agents or the user may own them. If a temporary artifact needs review or must survive across sessions, move it intentionally into a tracked location such as `docs/wip/` and explain why.
