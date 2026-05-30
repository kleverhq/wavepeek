## Core Workflow

- `wavepeek` is a Rust CLI for deterministic `.vcd` and `.fst` waveform inspection.
- Development is container-first; run repository gates in the devcontainer/CI image.
- Development tasks are run through root `justfile` recipes, not `make`.
- Standard quality gate: `just ci`.
- Local pre-handoff gate: `just check`.
- Use repository-root `tmp/` for disposable scratch, logs, and ad hoc outputs, but never delete arbitrary existing files there because they may belong to the user or another agent.
- Treat binary waveform dumps such as `.fst` as binary data; inspect them through `wavepeek`, fixtures, or purpose-built tools rather than text-reading them directly.
- Do not bypass hooks unless the user explicitly requests it.
- Read the nearest applicable `AGENTS.md` before editing files; local breadcrumbs may contain extra rules and gotchas.

## Map

- `src/` — Rust source code and embedded docs runtime.
- `tests/` — integration tests, fixtures, and test helpers.
- `tools/` — helper automation after the migration from `scripts/`.
- `bench/` — end-to-end and expression benchmark harnesses.
- `.github/workflows/` — CI and release workflows.
- `.devcontainer/` — local and CI container setup.
- `docs/dev/` — maintainer workflow, quality, style, release, and architecture docs.
- `docs/tracker/` — backlog, roadmap, and branch-local WIP artifacts.
- `docs/public/` — embedded user documentation for `wavepeek docs`.
- `schema/` — canonical machine-output schema artifacts.
