# Development Guide

This file captures the expected developer workflow and the code conventions we want to preserve.

## Container-First Workflow

Quality gates in this repository are container-first. Run development commands
inside the devcontainer/CI image so tooling, fixtures, and behavior are aligned
with CI.

- `make` targets enforce container execution via `WAVEPEEK_IN_CONTAINER=1`.
- Local interactive environment uses `.devcontainer/devcontainer.json` (`dev` target).
- Automation and CI use `.devcontainer/.devcontainer.json` (`ci` target).
- Large RTL fixtures are pre-provisioned under `/opt/rtl-artifacts` in image build,
  so tests do not fetch fixtures at runtime.

For rationale and non-obvious container decisions, see `.devcontainer/AGENTS.md`.

## Agent-Assisted Coding (OpenCode)

OpenCode is the primary agent toolchain for this repository.

- Runtime settings and command permissions: `opencode.json`
- Custom agents: `.opencode/agent/`
- Custom skills: `.opencode/skills/`
- Complex features/refactors: use `exec-plan`
- Implementation review: use `ask-review` with the `review` agent
- Periodic repository cleanup/simplification: use `repo-gc`

Operational policy:

- Keep agent-facing source of truth in `docs/` and `AGENTS.md` breadcrumbs.
- Do not store separate process documentation under `.opencode/`; keep only
  runnable agent/skill assets there.

## Build / Lint / Test

Use the `Makefile` targets when possible (they match CI and pre-commit hooks).

Common commands:

- Bootstrap dev env and install git hooks:
  - `make bootstrap`
- Format:
  - `make format`
  - `make format-check`
- Lint:
  - `make lint`
  - `make lint-fix` (applies clippy suggestions)
- Type/build checks:
  - `make check-build` (cargo check)
- Tests:
  - `make test`
- Run all pre-commit hooks locally:
  - `make pre-commit`
- Validate commit message (commit-msg hook runs this):
  - `make check-commit`
- One-shot local gate:
  - `make check` (format-check + clippy + check-schema + cargo check + commit msg check)
- Test-inclusive CI-parity gate:
  - `make ci` (format-check + clippy + check-schema + cargo test + cargo check)
- Cleanup:
  - `make clean`

Direct Cargo equivalents (useful when iterating):

- `cargo fmt` / `cargo fmt -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo check`
- `cargo test`
- `cargo build` / `cargo build --release`
- `cargo run -- <args>`

## CLI E2E Benchmark Harness

For reproducible CLI performance runs, use `bench/e2e/perf.py` (Python stdlib only, powered by `hyperfine`).

- List benchmark test catalog:
  - `python3 bench/e2e/perf.py list`
- Run benchmark matrix (or filtered subset) and generate run-local report:
  - `python3 bench/e2e/perf.py run --filter '^info_'`
- Regenerate report from existing run artifacts:
  - `python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/<run-id>`
- Compare revised run against golden run with regression threshold:
  - `python3 bench/e2e/perf.py compare --revised <dir> --golden <dir> --max-negative-delta-pct 5`

Benchmark test definitions live in `bench/e2e/tests.json` as a flat explicit list. Per-test `runs`/`warmup` values are configured there.

Set `WAVEPEEK_BIN` to choose the wavepeek binary for command composition.

`run --compare` and `report --compare` annotate deltas in `README.md` and add `🟢`/`🔴` markers when absolute delta is at least 3%.

Current harness mode runs real wavepeek commands for `info`, `at`, and `change`, writing one hyperfine JSON file per test (`<test_name>.json`) plus run-local `README.md`.

### Run A Single Test (Rust)

Prefer narrowing at the `cargo test` level rather than running everything.

- Run a single unit test by name (substring match):
  - `cargo test my_test_name`
- Run a single test with full module path:
  - `cargo test expr::parser::tests::parses_precedence`
- Run one integration test file in `tests/` (file stem):
  - `cargo test --test info_cli`
- Run a single test inside an integration test file:
  - `cargo test --test info_cli prints_json`
- Show output (stdout/stderr) for a failing test:
  - `cargo test my_test_name -- --nocapture`
- Re-run only ignored tests:
  - `cargo test -- --ignored`
- Run tests for a specific package (if/when this becomes a workspace):
  - `cargo test -p wavepeek my_test_name`

If you need to profile test runtime:

- `cargo test -- --test-threads=1` (more deterministic; slower)

## Pre-commit Hooks

Hooks are defined in `.pre-commit-config.yaml` and installed by `make bootstrap`.
Current hooks run (pre-commit): rustfmt, clippy, cargo check, cargo test.
Commit messages are validated (commit-msg) via `commitizen` (`cz check`).

Notes for agents:

- Do not bypass hooks (no `--no-verify`) unless the user explicitly asks.
- If you change formatting/lints, re-run `make check` (or at least the impacted
  targets) before proposing a commit.

## Changelog Policy

- The repository changelog lives in `CHANGELOG.md`.
- Changelog entries must follow the
  [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) format.
- Keep sections deterministic and human-scannable: use the standard buckets
  (`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`) and avoid
  ad-hoc headings.
- Add user-visible changes to `## [Unreleased]` as part of normal feature/fix
  work.
- During release prep, move `Unreleased` items into a versioned heading
  `## [X.Y.Z] - YYYY-MM-DD`, then create a new empty `Unreleased` section.
- Keep comparison links at the bottom updated for `Unreleased` and each release.

## Rust Code Style

### Formatting

- No manual alignment or stylistic diffs that fight rustfmt.
- Keep diffs minimal and mechanically consistent.
- Prefer explicit control flow over clever one-liners when it affects clarity.

### Imports

- Prefer explicit imports over glob imports (`use foo::*;`) except in `mod.rs`
  where it reduces noise.
- Group imports in the usual Rust style:
  - std
  - external crates
  - crate modules
- Avoid unused imports; clippy runs with `-D warnings`.

### Naming Conventions

- Modules, functions, locals: `snake_case`.
- Types, traits, enums: `PascalCase`.
- Constants: `SCREAMING_SNAKE_CASE`.
- CLI flags: long, self-documenting, kebab-case (via clap).

### Types / Ownership / Performance

- Prefer borrowing (`&str`, `&[T]`) at API boundaries; avoid cloning by default.
- Prefer `String`/`Vec` only when ownership is needed.
- Avoid `Box<dyn Error>` / dynamic error types in core code paths.
- Use `#[derive(Debug, Clone, PartialEq, Eq)]` where it meaningfully helps tests
  and error messages; don’t over-derive on large structs.

### Error Handling

- No panics in production paths. Avoid `unwrap()` / `expect()` except where it
  would indicate a programmer bug and is truly unreachable.
- Prefer typed error enums with `thiserror`.
- Avoid `anyhow` in the main binary; keep errors structured and predictable.
- Errors go to stderr; stdout is reserved for command output only.

Error output shape to preserve (keep stable and parseable):

  `error: <category>: <message>`

Also keep exit codes stable.

### Deterministic Output

Wavepeek is "LLM-first" and expects output stability:

- Sort output deterministically (depth-first + alpha where applicable).
- Avoid timestamps, random IDs, and hash map iteration order in user-facing
  output.
- Keep default outputs bounded (e.g., list commands use `--max`).
- Keep recursion bounded (`--max-depth`).

### CLI Design Constraints

- No positional arguments; all args are named flags.
- The waveform file flag is always `--waves`.
- Default output is human-readable; `--json` enables strict JSON envelope output with a stable `$schema` contract.
- Time values require explicit units; reject bare numbers.

### Testing Expectations

- Add focused unit tests for parsing/eval helpers and edge cases.
- Add integration tests for CLI behavior (exact stdout/stderr, exit codes).
- Assert determinism: same input => identical output.
- Prefer small fixtures; keep generated fixtures out of normal `cargo test` if
  they are heavy.
