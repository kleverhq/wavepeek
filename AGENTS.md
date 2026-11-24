## Core Workflow

- `wavepeek` is a Rust CLI for deterministic `.vcd` and `.fst` waveform inspection.
- Agents, Git, signing, credentials, pushes, issues, and pull requests run on the host. Cargo, waveform tools, and quality gates run in the devcontainer through root `./dev`.
- Each worktree and selected configuration has its own container.
- Development tasks are run through root `justfile` recipes.
- Standard quality gate: `just ci`.
- Local pre-handoff gate: `just check`.
- Use ignored repository-root `tmp/` for generated tool outputs and logs. Never clean it globally or delete files owned by others.
- Treat binary waveform dumps such as `.fst` as binary data; inspect them through `wavepeek`, fixtures, or purpose-built tools rather than text-reading them directly.
- Run the local native SDK gates in `docs/quality.md` before opening a pull request.
- Read the nearest applicable `AGENTS.md` before editing files; local breadcrumbs may contain extra rules and gotchas.

## Writing Style

- Keep Markdown, comments, breadcrumbs, and PR text concise, minimal, precise, and neutral.
- Describe current behavior and durable rationale directly. Do not frame current docs around retrospective bugs or removed behavior unless the history is necessary for migration or troubleshooting.
- Avoid jokes, sarcasm, editorial asides, and colorful phrasing in repository artifacts.

## Development

Maintainer workflow lives under `docs/`:

- `docs/environment.md` for the shared devcontainer, host entrypoint, fixtures, and `tmp/`.
- `docs/quality.md` for `just check`, `just ci`, and native SDK gates.
- `docs/testing.md` for test strategy and fixtures.
- `docs/style.md` for Rust, CLI, output, and docs conventions.
- `docs/benchmarking.md` for manual performance gate and E2E benchmark workflows.
- `docs/automation.md` for CI, `justfile`, and helper tools.
- `docs/git.md`, `docs/changelog.md`, and `docs/release.md` for contribution hygiene and releases.
- `docs/architecture.md` for internal module boundaries.

## Map

- `src/` — Rust source code and embedded skill runtime.
- `tests/` — integration tests, fixtures, and test helpers.
- `tools/` — helper automation used by `just` recipes and workflows.
- `bench/` — end-to-end benchmark harnesses.
- `.github/workflows/` — CI and release workflows.
- `.devcontainer/` — shared development and automation container setup.
- `docs/` — maintainer workflow, quality, style, release, backlog, and roadmap docs.
- `skills/wavepeek/` — canonical source for the packaged Wavepeek skill.
