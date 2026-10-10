## Core Workflow

- `wavepeek` is a Rust CLI for deterministic `.vcd` and `.fst` waveform inspection.
- Agents, Git, signing, credentials, pushes, issues, and pull requests run on the host. Cargo, Pre-commit, Commitizen, waveform tools, and quality gates run in the devcontainer through root `./dev`.
- Each worktree and selected configuration has its own container. Start it explicitly before host `git commit`; hooks never start or rebuild containers.
- After creating or entering a new worktree, run `./dev --install-hooks` before the first commit. The command is idempotent and configures hooks only for that worktree.
- Development tasks are run through root `justfile` recipes.
- Standard quality gate: `just ci`.
- Local pre-handoff gate: `just check`.
- Use repository-root `tmp/` for disposable scratch, logs, and ad hoc outputs, but never delete arbitrary existing files there because they may belong to the user or another agent.
- Treat binary waveform dumps such as `.fst` as binary data; inspect them through `wavepeek`, fixtures, or purpose-built tools rather than text-reading them directly.
- Do not bypass hooks unless the user explicitly requests it.
- Read the nearest applicable `AGENTS.md` before editing files; local breadcrumbs may contain extra rules and gotchas.

## Writing Style

- Keep Markdown, comments, breadcrumbs, execution plans, and PR text concise, minimal, precise, and neutral.
- Describe current behavior and durable rationale directly. Do not frame current docs around retrospective bugs or removed behavior unless the history is necessary for migration or troubleshooting.
- Avoid jokes, sarcasm, editorial asides, and colorful phrasing in repository artifacts.

## Development

Maintainer workflow lives under `docs/`:

- `docs/environment.md` for the shared devcontainer, host entrypoint, fixtures, and `tmp/`.
- `docs/quality.md` for `just check`, `just ci`, coverage, and hooks.
- `docs/testing.md` for test strategy and fixtures.
- `docs/style.md` for Rust, CLI, output, and docs conventions.
- `docs/benchmarking.md` for manual performance gate and E2E benchmark workflows.
- `docs/automation.md` for CI, `justfile`, pre-commit, and helper tools.
- `docs/git.md`, `docs/changelog.md`, and `docs/release.md` for contribution hygiene and releases.
- `docs/architecture.md` for internal module boundaries.

## Map

- `src/` — Rust source code and embedded skill runtime.
- `tests/` — integration tests, fixtures, and test helpers.
- `tools/` — helper automation used by `just` recipes and workflows.
- `bench/` — end-to-end benchmark harnesses.
- `.github/workflows/` — CI and release workflows.
- `.devcontainer/` — shared development and automation container setup.
- `docs/` — maintainer workflow, quality, style, release, backlog, and roadmap docs, with branch-local artifacts under `docs/wip/`.
- `skills/wavepeek/` — canonical source for the packaged Wavepeek skill.
