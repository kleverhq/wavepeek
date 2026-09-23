# Development Environment

`wavepeek` is developed in one credentialless, command-line devcontainer. Local development, CI, release quality checks, and docs staging use `.devcontainer/devcontainer.json` and the final image in `.devcontainer/Dockerfile`.

## Host Entrypoint

Agents, Git identity and signing, credentials, commits, pushes, issues, and pull requests stay on the host. Run Cargo, Pre-commit, Commitizen, waveform tools, and repository gates in the container through root `./dev`:

```sh
./dev --install-hooks
./dev just check
git commit -m "chore: update development workflow"
git push
```

`./dev --install-hooks` explicitly installs reviewed `pre-commit`, `commit-msg`, and `./dev` copies under the current worktree's Git directory and activates them with worktree-local `core.hooksPath`. Each linked worktree therefore uses and updates its own hook copies. Installation is idempotent and refuses to replace another configured worktree hooks path. Branch switches do not alter the active copies.

`./dev` finds the enclosing Git worktree when called from any directory inside it, preserves that relative directory in the container, starts the worktree's container when needed, and passes command arguments, standard streams, signals, and exit status through unchanged. Each absolute worktree has its own runtime container and can use its own revision of the container definition. The image layers remain shared through Docker.

Use `./dev --exec-only COMMAND [ARG ...]` when a caller must use only an existing container. The installed Git hooks use this mode, so each worktree container must be started explicitly with a normal command such as `./dev true` before `git commit`. This mode never starts, builds, restarts, recreates, or removes a container.

If an existing container's linked-worktree Git mount or optional Verdi mount no longer matches the current host state, `./dev` refuses to use it and prints the explicit `devcontainer up --remove-existing-container` command needed to recreate it. The wrapper never deletes a container automatically.

## Container Contract

The image includes Rust and its WASM target, `wasm-bindgen`, Cargo tools, C/C++ compilers, Python, Material/Mike, Playwright's headless Chromium for Playground checks, actionlint, hooks, GitHub CLI, Icarus Verilog, waveform converters, benchmark tooling, and command-line Verdi integration. It does not include coding agents, Node.js, a nested Devcontainer CLI, Surfer, GUI forwarding, or local GitHub credential setup.

The workspace is mounted at `/workspaces/<worktree-name>`. For linked worktrees, `./dev` also mounts the Git common directory at the same absolute host and container path so Git follows the worktree's `.git` pointer correctly. No agent state, host GitHub configuration, or local token file is mounted.

Recipes in `justfile` require `WAVEPEEK_IN_CONTAINER=1`. Do not set it on the host to bypass the guard; use `./dev` instead.

Run `./dev just dev-setup` after creating or rebuilding the container to verify tool availability. It does not install or rewrite host hooks. Use `./dev just playground-build` for the standalone browser build. `playground-preview-build`, `playground-test`, and `playground-serve` compose the Playground at `/wavepeek/` with current documentation at `/wavepeek/latest/` for local checks and inspection.

## Fixture Location

Large RTL fixtures are baked into the image under `RTL_ARTIFACTS_DIR=/opt/rtl-artifacts`. That path is the only supported runtime fixture location.

Small source-backed integration fixtures are regenerated inside the repository with `./dev just prepare-waveform-fixtures`. Their checked-in sources live under `tests/fixtures/source/`; generated VCD/FST outputs live under ignored `tests/fixtures/generated/`.

The container environment contract lives in `.devcontainer/env_contract.sh`. Update it with container provisioning when fixture versions or layout change.

## Verdi / FSDB Development

When host `VERDI_HOME` is unset, `./dev` starts the container without `/opt/verdi`; FSDB gates report a skip and default VCD/FST development remains available.

When `VERDI_HOME` points to a valid Verdi installation containing the FSDB Reader SDK, `./dev` validates it and mounts only that directory at `/opt/verdi`. Reader library selection and absolute-path linking follow Ondas's FSDB contract. Invalid paths, incomplete SDKs, and unsupported Reader overrides are rejected before container startup. Use `./dev just check-fsdb-env` to distinguish available, skipped, and broken SDK states.

The full FSDB build, fixture, benchmark, and repository-safety contract lives in `fsdb.md`.

## Debug Mode

`DEBUG=1` enables maintainer-only internal diagnostics and hidden controls. Hidden controls are unstable implementation details and are not part of the public CLI contract, even when debug mode exposes them.

## Temporary Files

Use repository-root `tmp/` for scratch files, ad hoc logs, temporary benchmark captures, and other disposable working artifacts. It is ignored by Git and may be created freely.

Never globally clean `tmp/` or delete arbitrary existing files there. Other agents or the user may own them. If a temporary artifact needs review or must survive across sessions, move it intentionally into a tracked location such as `docs/wip/` and explain why.
