# Development Guide

This file captures the expected developer workflow and the code conventions we want to preserve.

## Container-First Workflow

Quality gates in this repository are container-first. Run development commands
inside the devcontainer/CI image so tooling, fixtures, and behavior are aligned
with CI.

- `make` targets enforce container execution via `WAVEPEEK_IN_CONTAINER=1`.
- Local interactive environment uses `.devcontainer/devcontainer.json` (`dev` target).
- Automation and CI use `.devcontainer/devcontainer.ci.json` (`ci` target).
- Large RTL fixtures are pre-provisioned under `/opt/rtl-artifacts` in image build,
  so tests do not fetch fixtures at runtime.

For rationale and non-obvious container decisions, see `.devcontainer/AGENTS.md`.

## Debug Mode

Debug mode is a repository-wide internal contract enabled only with `DEBUG=1`.

- `DEBUG=1` is intended for maintainers and CI diagnostics, not normal user flows.
- In debug mode, hidden internal controls may become available across commands.
- Hidden controls remain unstable implementation details and are not part of the
  public CLI contract, even when they are enabled.

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
  - `make test-aux`
- Run all pre-commit hooks locally:
  - `make pre-commit`
- Validate commit message (commit-msg hook runs this):
  - `make check-commit`
- One-shot local gate:
  - `make check` (format-check + clippy + check-schema + cargo check + commit msg check)
- Test-inclusive CI-parity gate:
  - `make ci` (format-check + clippy + check-schema + cargo test + auxiliary Python unit tests + cargo check)
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

This harness is intentionally scoped to end-to-end CLI command timing. Do not
use it for any internal microbenchmarks.

- List benchmark test catalog:
  - `python3 bench/e2e/perf.py list`
- Run benchmark matrix (or filtered subset) and generate run-local report:
  - `python3 bench/e2e/perf.py run --filter '^info_'`
- Resume an existing run directory and execute only missing tests:
  - `python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/<run-id> --missing-only`
- Override wavepeek timeout cap (default is 300 seconds per invocation):
  - `python3 bench/e2e/perf.py run --wavepeek-timeout-seconds 600`
- Regenerate report from existing run artifacts:
  - `python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/<run-id>`
- Compare revised run against golden run with regression threshold:
  - `python3 bench/e2e/perf.py compare --revised <dir> --golden <dir> --max-negative-delta-pct 5`

Benchmark test definitions live in `bench/e2e/tests.json` as a flat explicit list. Per-test `runs`/`warmup` values are configured there.

For focused optimization campaigns, keep dedicated run directories under `bench/e2e/runs/` (for example, `change-stateless-golden`, `change-stateless-m2`, `change-stateless-m3`, `change-stateless-m4`, `change-stateless-final-matrix`) and compare against either the campaign golden run or shared `bench/e2e/runs/baseline` as appropriate.

Set `WAVEPEEK_BIN` to choose the wavepeek binary for command composition.

`run --verbose` prints the resolved run directory at startup so resume/retry flows are explicit in detailed logs. Default mode stays quiet and ends with a concise success line.

Each benchmark run writes two per-test artifacts plus a run-level report:

- `<test_name>.hyperfine.json` for timing metrics.
- `<test_name>.wavepeek.json` for functional payload (`data` + `warnings`), or `{}` when functional capture hit timeout cap.
- `README.md` with grouped metrics and compare status.

`run --compare` and `report --compare` annotate timing deltas in `README.md`, add `🟢`/`🔴` markers when absolute delta is at least 3%, and include a functional parity marker (`✅` match, `✅E` match with empty data, `⚠️D` data mismatch, `⏱T` timeout artifact, `?` missing counterpart).

`compare` is a blocking gate for matched tests: it exits with code `1` for mean or median timing threshold violations, functional `data` mismatch, or missing/invalid `<test_name>.wavepeek.json` artifacts. Empty timeout artifacts (`{}`) are treated as non-blocking timeout signals and are reported as warnings. `warnings` are ignored for functional parity to avoid false regressions from warning text churn during refactors. Tests present only on one side are reported as warnings and do not fail compare.

## Expression Microbenchmarks

For expression-engine microbenchmarks, use `bench/expr/perf.py` (Python stdlib
only) together with the explicit suite catalog in `bench/expr/suites.json`.
The functional Rust bench targets are `expr_syntax`, `expr_logical`,
`expr_event`, and `expr_waveform_host`.

- List the available expression suites:
  - `python3 bench/expr/perf.py list`
- Validate the four bench targets in test mode:
  - `cargo test --bench expr_syntax --bench expr_logical --bench expr_event --bench expr_waveform_host`
- Capture one aggregated run directory and grouped report in a fresh location:
  - `python3 bench/expr/perf.py run --run-dir bench/expr/runs/<run-id>`
- Resume the same run directory only when the catalog fingerprint and selected-suite set still match exactly:
  - `python3 bench/expr/perf.py run --run-dir bench/expr/runs/<run-id> --missing-only`
- Regenerate `README.md` from an existing run directory:
  - `python3 bench/expr/perf.py report --run-dir bench/expr/runs/baseline`
- Compare a revised run against the maintained baseline with an explicit timing threshold:
  - `python3 bench/expr/perf.py compare --revised <dir> --golden bench/expr/runs/baseline --max-negative-delta-pct 15 --require-matching-metadata <key> [<key> ...]`
- For same-commit verification or other provenance-sensitive checks, require the matching metadata keys explicitly:
  - `python3 bench/expr/perf.py compare --revised <dir> --golden bench/expr/runs/baseline --max-negative-delta-pct 10 --require-matching-metadata source_commit worktree_state cargo_version rustc_version criterion_version environment_note`

`bench/expr/perf.py` runs the selected `cargo bench --bench <target>` commands,
consumes Criterion `raw.csv` artifacts from `target/criterion`, exports
namespaced `*.raw.csv` files into one run directory, and writes one aggregated
`summary.json` + `README.md`. `run --run-dir <dir>` requires `<dir>` to be empty
unless `--missing-only` is passed.

Convenience targets mirror the new workflow:

- `make bench-expr-update-baseline`
- `make bench-expr-run`

Use a fresh `<run-id>` directory for ad hoc local captures. Refresh the committed
baseline through `make bench-expr-update-baseline` so the replace-in-place flow
stays guarded behind one explicit target.

## Pre-commit Hooks

Hooks are defined in `.pre-commit-config.yaml` and installed by `make bootstrap`.
Current hooks run (pre-commit): rustfmt, clippy, cargo check, schema contract, cargo test, auxiliary Python unit tests, and the bench/e2e smoke run+compare catalog.
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
- Treat published version sections as immutable historical release records. Do
  not retroactively rewrite shipped scope or behavior in old release sections.
- If a factual correction is needed after a release, record it in
  `## [Unreleased]` or a later release instead of rewriting the old shipped
  section.
- During release prep, move `Unreleased` items into a versioned heading
  `## [X.Y.Z] - YYYY-MM-DD`, then create a new empty `Unreleased` section.
- GitHub Release notes are published from the matching version section in
  `CHANGELOG.md`; do not rely on auto-generated GitHub notes.
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
- CLI help must be standalone and uniform: `wavepeek`, `wavepeek -h`, and `wavepeek --help` must print the same top-level contract, and every shipped subcommand must make `-h` byte-identical to `--help` while documenting semantics, defaults/requiredness, boundary rules, error categories, and output shape.
- Time values require explicit units; reject bare numbers.

### Testing Expectations

- Add focused unit tests for parsing/eval helpers and edge cases.
- Add integration tests for CLI behavior (exact stdout/stderr, exit codes).
- Assert determinism: same input => identical output.
- Prefer small fixtures; keep generated fixtures out of normal `cargo test` if
  they are heavy.
