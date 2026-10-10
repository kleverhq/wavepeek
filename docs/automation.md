# Automation Guide

Repository automation is exposed through the root `justfile`. Prefer invoking `just` recipes instead of calling helper scripts directly; recipes encode container guards, fixture checks, environment variables, and the command sequences CI uses.

## Main Entrypoints

Run container commands from the host through `./dev`.

- `./dev just dev-setup` verifies the development container; host `./dev --install-hooks` explicitly installs reviewed hook copies.
- `./dev just check`, `./dev just ci`, and `./dev just pre-commit` are the main quality gates.
- `just prepare-waveform-fixtures` regenerates ignored VCD/FST fixtures under `tests/fixtures/generated/` from `tests/fixtures/source/` and documented hand-derived outputs.
- `just playground-build` builds the current browser app; `just playground-preview-build`, `just playground-test`, and `just playground-serve` compose it with current documentation under `/wavepeek/latest/` for local checks and inspection.
- `just docs-site-build`, `just docs-site-check`, `just docs-site-stage-deploy`, `just docs-site-push-staged`, and `just docs-site-check-deploy` own GitHub Pages documentation and current-Playground publication.
- `just bench-gate`, `just bench-capture`, and `just bench-compare` own manual performance review; generated benchmark runs are ignored and are not committed baselines.
- `just check-fsdb-env`, `just test-fsdb`, and `just lint-fsdb` own optional Verdi/FSDB flows; see `fsdb.md` for the full contract.

## Devcontainer Lifecycle

The root `./dev` wrapper is the host lifecycle entrypoint. It selects one runtime container per absolute Git worktree and configuration, supplies the linked-worktree Git mount, and then executes the requested command. `WAVEPEEK_DEV_CONFIG` selects optional private profiles; mounts and vendor environment belong to those profiles. Agents and Git remote operations stay on the host; repository tools run in the container. Keep `./dev` aligned with `.devcontainer/devcontainer.json` and `environment.md`.

GitHub Actions uses `.devcontainer/devcontainer.json` directly. Workflows use job-scoped runner authentication when required; the development container does not provision local credentials.

## Workflows and Hooks

GitHub Actions workflows live under `.github/workflows/`. Pre-merge CI runs for pushes to `main` and pull requests targeting `main` or `dev*` branches. The release workflow validates stable `vX.Y.Z` tag/version agreement, runs `just ci` and `cargo package --locked` in the shared devcontainer, uses `cargo-dist` to build VCD/FST binary archives, installers, checksums, and attestations, creates the GitHub Release, then dispatches docs and crates.io publication on the default branch. The docs workflow is manual-only, uses trusted tooling from `main`, builds the current browser Playground from the release tag, downloads installer assets from the created GitHub Release, stages the `gh-pages` update without persisted contents-write checkout credentials, pushes only after verifying the staged bundle in a separate job, and deploys the verified tree through GitHub Pages Actions rather than relying on a branch-push Pages build. Mike accumulates documentation versions; only a promoted latest release replaces the non-versioned root Playground. The crate publication workflow is manual-only, uses trusted tooling from `main`, checks out release source through `refs/tags/<tag>`, and treats already-published crates.io versions as a successful no-op.

Pre-commit configuration lives in `.pre-commit-config.yaml`, and the reviewed host dispatcher lives at `tools/repo/git-hook`. `./dev --install-hooks` copies the dispatcher and `./dev` into the current worktree's Git directory; hooks then use only `./dev --exec-only` and require that worktree's container to be running. Hooks should stay deterministic, non-interactive, and wired through `just` where possible.

## Helper Tool Layout

Helper implementation code belongs in grouped root `tools/` directories with short READMEs and local tests when applicable. The stable interface remains the `just` recipe or workflow step, not an undocumented helper path. Keep helper stdout/stderr stable and return explicit non-zero exits on failure. Waveform fixture generation lives under `tools/waveform/`. Benchmark gate helpers live under `tools/bench/`. Docs-site helpers live under `tools/docs/`; `prepare_mkdocs.py` stages versioned packaged references, `prepare_playground.py` stages the current root app, `check_playground.py` checks the composed local site plus browser parity/privacy, `publish_docs.py` separates local check, staged deploy, and push-only verification, and `check_deploy.py` verifies published Pages state plus one browser/WASM smoke command after deployment.

During path moves, update `justfile`, affected workflow YAML, hooks, docs, and helper tests in the same change.
