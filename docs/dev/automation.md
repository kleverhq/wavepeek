# Automation Guide

Repository automation is exposed through the root `justfile`. Prefer invoking `just` recipes instead of calling helper scripts directly; recipes encode container guards, fixture checks, environment variables, and the command sequences CI uses.

## Main Entrypoints

- `just dev-setup` prepares the local devcontainer and installs hooks.
- `just codex-setup` and `just codex-resume` prepare Codex cloud sessions.
- `just check`, `just ci`, and `just pre-commit` are the main quality gates.
- `just update-schema` refreshes the current major artifact such as `schema/wavepeek_v0.json`; `just check-schema` validates it.
- `just docs-site-build`, `just docs-site-check`, `just docs-site-stage-deploy`, `just docs-site-push-staged`, and `just docs-site-check-deploy` own GitHub Pages docs preparation, publication, and deployed endpoint verification.
- `just bench-e2e-run`, `just bench-e2e-update-baseline`, `just bench-expr-run`, and `just bench-expr-update-baseline` own default benchmark flows.
- `just check-fsdb-env`, `just test-fsdb`, `just lint-fsdb`, and `just bench-e2e-fsdb-smoke-commit` own optional Verdi/FSDB flows; see `fsdb.md` for the full contract.

## Devcontainer Lifecycle Helpers

`.devcontainer/initialize.sh` runs on the host before container creation and prepares bind-mount and env-file prerequisites. `.devcontainer/setup-github-auth.sh` runs inside the container from `postCreateCommand` and configures optional repo-local GitHub auth. `tools/repo/setup_github_env.sh` is the optional one-shot host bootstrap for a clean GitHub auth env directory.

When changing those helpers, keep `.devcontainer/devcontainer.json`, `environment.md`, and `github-auth.md` aligned. Do not duplicate the GitHub-auth runbook here; that way lies the traditional swamp of stale docs with confident headings.

## Workflows and Hooks

GitHub Actions workflows live under `.github/workflows/`. The release workflow validates tag/version agreement, runs `just ci` in the CI devcontainer, packages the crate, extracts release notes from `CHANGELOG.md`, publishes to crates.io, creates the GitHub Release, and then dispatches docs publication. The docs workflow is manual-only, uses trusted tooling from `main`, stages the `gh-pages` update without persisted contents-write checkout credentials, pushes only after verifying the staged bundle in a separate job, and deploys the verified tree through GitHub Pages Actions rather than relying on a branch-push Pages build.

Pre-commit configuration lives in `.pre-commit-config.yaml`. Hooks should stay deterministic, non-interactive, and wired through `just` where possible.

## Helper Tool Layout

Helper implementation code belongs in grouped root `tools/` directories with short READMEs and local tests when applicable. The stable interface remains the `just` recipe or workflow step, not an undocumented helper path. Keep helper stdout/stderr stable and return explicit non-zero exits on failure. Docs-site helpers live under `tools/docs/`; `prepare_mkdocs.py` generates MkDocs staging input from `wavepeek docs export`, `publish_docs.py` separates local check, staged deploy, and push-only verification, and `check_deploy.py` verifies published Pages state after deployment.

During path moves, update `justfile`, affected workflow YAML, hooks, docs, and helper tests in the same change. Nothing says reliability like a helper script confidently living at yesterday's address.
