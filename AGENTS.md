# Agent Guide (wavepeek)

This file is for coding agents operating in this repository. It captures the
expected developer workflow and the code conventions we want to preserve.

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
- One-shot quality gate:
  - `make check` (format-check + clippy + cargo check + commit msg check)
- Cleanup:
  - `make clean`

Direct Cargo equivalents (useful when iterating):

- `cargo fmt` / `cargo fmt -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo check`
- `cargo test`
- `cargo build` / `cargo build --release`
- `cargo run -- <args>`

### Run A Single Test (Rust)

Prefer narrowing at the `cargo test` level rather than running everything.

- Run a single unit test by name (substring match):
  - `cargo test my_test_name`
- Run a single test with full module path:
  - `cargo test expr::parser::tests::parses_precedence`
- Run one integration test file in `tests/` (file stem):
  - `cargo test --test cli_info`
- Run a single test inside an integration test file:
  - `cargo test --test cli_info prints_json`
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

## Code Style (Rust)

The project goal is a deterministic, machine-friendly CLI for waveform
inspection. The PRD in `.memory-bank/PRD.md` is the source of truth for CLI
behavior, output stability, and error semantics.

### Formatting

- Use `rustfmt` (no manual alignment or stylistic diffs that fight rustfmt).
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

### Error Handling (Important)

Per the PRD:

- No panics in production paths. Avoid `unwrap()` / `expect()` except where it
  would indicate a programmer bug and is truly unreachable.
- Prefer typed error enums (planned: `WavepeekError` in `src/error.rs`) with
  `thiserror`.
- Avoid `anyhow` in the main binary; keep errors structured and predictable.
- Errors go to stderr; stdout is reserved for command output only.

Error output shape to preserve (keep stable and parseable):

  `error: <category>: <message>`

Also keep exit codes stable (PRD suggests 0=ok, 1=user error, 2=file/parse).

### Deterministic Output

Wavepeek is "LLM-first" and expects output stability:

- Sort output deterministically (depth-first + alpha where applicable).
- Avoid timestamps, random IDs, and hash map iteration order in user-facing
  output.
- Keep default outputs bounded (e.g., list commands use `--max`).
- Keep recursion bounded (`--max-depth`).

### CLI Design Constraints (From PRD)

- No positional arguments; all args are named flags.
- The waveform file flag is always `--waves`.
- Default output: JSON envelope with stable schema version; `--human` enables human-friendly output without a strict contract.
- Time values require explicit units; reject bare numbers.

### Testing Expectations

- Add focused unit tests for parsing/eval helpers and edge cases.
- Add integration tests for CLI behavior (exact stdout/stderr, exit codes).
- Assert determinism: same input => identical output.
- Prefer small fixtures; keep generated fixtures out of normal `cargo test` if
  they are heavy.

## Repo References

- Developer workflow entrypoints: `Makefile`, `.pre-commit-config.yaml`.
- Product/architecture requirements: `.memory-bank/PRD.md`.
- Release process and checklist: `.memory-bank/RELEASE.md`.
- Changelog: `CHANGELOG.md` (Keep a Changelog format).
- Agent workflow docs (OpenCode skills): `.opencode/README.md`.
