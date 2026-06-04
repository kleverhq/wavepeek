## Core Workflow

- `wavepeek` is a Rust CLI for deterministic `.vcd` and `.fst` waveform inspection.
- Development is container-first; run repository gates in the devcontainer/CI image.
- Development tasks are run through root `justfile` recipes.
- Standard quality gate: `just ci`.
- Local pre-handoff gate: `just check`.
- Use repository-root `tmp/` for disposable scratch, logs, and ad hoc outputs, but never delete arbitrary existing files there because they may belong to the user or another agent.
- Treat binary waveform dumps such as `.fst` as binary data; inspect them through `wavepeek`, fixtures, or purpose-built tools rather than text-reading them directly.
- Do not bypass hooks unless the user explicitly requests it.
- Read the nearest applicable `AGENTS.md` before editing files; local breadcrumbs may contain extra rules and gotchas.

## Development

Maintainer workflow lives under `docs/dev/`:

- `docs/dev/environment.md` for devcontainer, CI image, Codex, fixtures, and `tmp/`.
- `docs/dev/github-auth.md` for optional repo-scoped GitHub auth in the devcontainer.
- `docs/dev/quality.md` for `just check`, `just ci`, coverage, and hooks.
- `docs/dev/testing.md` for test strategy and fixtures.
- `docs/dev/style.md` for Rust, CLI, output, and docs conventions.
- `docs/dev/benchmarking.md` for E2E and expression benchmark workflows.
- `docs/dev/automation.md` for CI, `justfile`, pre-commit, and helper tools.
- `docs/dev/git.md`, `docs/dev/changelog.md`, and `docs/dev/release.md` for contribution hygiene and releases.
- `docs/dev/architecture.md` for internal module boundaries.

## Map

- `src/` — Rust source code and embedded docs runtime.
- `tests/` — integration tests, fixtures, and test helpers.
- `tools/` — helper automation used by `just` recipes and workflows.
- `bench/` — end-to-end and expression benchmark harnesses.
- `.github/workflows/` — CI and release workflows.
- `.devcontainer/` — local and CI container setup.
- `docs/dev/` — maintainer workflow, quality, style, release, and architecture docs.
- `docs/tracker/` — backlog, roadmap, and branch-local WIP artifacts.
- `docs/public/` — embedded user documentation for `wavepeek docs`.
- `schema/` — canonical machine-output schema artifacts.
