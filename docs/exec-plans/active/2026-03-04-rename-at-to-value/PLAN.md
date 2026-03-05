# Rename `at` Surface to `value` and `--time` to `--at`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, users query point-in-time signal values with intent-first wording: `wavepeek value --waves <file> --at <time> --signals <names>`. The old `at` command and `--time` flag are retired as a deliberate breaking change. Runtime behavior stays equivalent to today: same sampling semantics, same ordering, same literal formatting, and same warning/error style, with only command/flag naming changed.

The change is observable in three ways. First, `wavepeek --help` lists `value` and no longer lists `at`. Second, `wavepeek value ... --json` returns the same payload structure as before with `command: "value"`. Third, migration behavior is explicit and test-covered: invoking legacy `wavepeek at` fails as an unrecognized subcommand and `wavepeek value --time ...` fails as an `error: args:` parse error with deterministic help hint.

## Non-Goals

This plan does not change point-in-time sampling semantics, signal resolution rules, deterministic ordering, value formatting, error-category conventions, or output boundedness policy. This plan does not introduce backward-compatible aliases for `at` or `--time`. This plan does not implement backlog items unrelated to this rename (for example `when`/`property` work, expression-evaluator completion, or streaming JSON mode).

## Progress

- [x] (2026-03-04 20:19Z) Located backlog requirement in `docs/BACKLOG.md` and captured explicit closure criteria (CLI/help/docs/schema/tests/changelog + migration integration coverage).
- [x] (2026-03-04 20:19Z) Mapped impacted code paths (`src/cli`, `src/engine`, `src/output.rs`, `schema/`, `tests/`, docs, and benchmark catalog) and common time-validation usage across point-sampling and range-sampling flows.
- [x] (2026-03-04 20:19Z) Chosen migration policy and recorded it: hard break (no compatibility alias), aligned with existing "legacy command is rejected" precedent in `tests/cli_contract.rs` and `docs/DESIGN.md`.
- [x] (2026-03-04 20:19Z) Drafted this executable plan with TDD-first sequencing, validation gates, and mandatory two-pass review closure.
- [x] (2026-03-04 20:24Z) Completed review pass #1 for this plan and tightened red-phase evidence, docs-audit scope, rename staging safety, and review-step executability wording.
- [x] (2026-03-04 20:25Z) Completed independent review pass #2 for this plan and removed duplicate test-file rename instruction that could break linear execution.
- [x] (2026-03-05 06:32Z) Revalidated plan after branch rebase onto updated `main`; updated stale assumptions that previously referenced `change` importing time helpers from `engine::at`.
- [x] (2026-03-05 06:34Z) Completed rebase-refresh review pass #1 and removed duplicated module-move instructions to keep linear stateless execution idempotent.
- [x] (2026-03-05 06:36Z) Addressed rebase-refresh pass #2 low findings by making red-phase test naming explicit and adding a mandatory final post-review `make check` + `make ci` gate.
- [ ] Implement TDD red phase for renamed CLI surface (`value`, `--at`) and migration assertions.
- [ ] Implement CLI/engine/schema/output rename and keep payload semantics equivalent.
- [ ] Update docs/changelog/benchmark catalog collateral to new naming.
- [ ] Pass validation gates (`make check`, `make ci`) and complete mandatory review pass #1 + independent pass #2.

## Surprises & Discoveries

- Observation: Recent refactor moved shared time parsing/alignment helpers into a dedicated engine module.
  Evidence: `src/engine/change.rs` imports time helpers from `crate::engine::time`, and `src/engine/at.rs` now uses the same `crate::engine::time` utilities.

- Observation: The benchmark E2E catalog hardcodes many `at` scenarios and `--time` flags; leaving this unchanged creates immediate drift in perf workflows after rename.
  Evidence: `bench/e2e/tests.json` contains repeated `"category": "at"`, `"at"`, and `"--time"` entries.

- Observation: Existing CLI contract tests enforce strict no-alias policy for prior command renames (`tree`, `modules`, `signals`, `changes`).
  Evidence: `tests/cli_contract.rs` explicitly asserts legacy subcommands are rejected without compatibility aliases.

## Decision Log

- Decision: Migration behavior is a hard break: remove `wavepeek at` and `--time`, do not add compatibility alias.
  Rationale: Repository precedent favors canonical singular command names without alias carryover, and backlog asks to define migration behavior explicitly.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Rename internal module/type surface from `at` to `value` (`src/cli/value.rs`, `src/engine/value.rs`, `CommandName::Value`, `CommandData::Value`) instead of only changing clap labels.
  Rationale: This avoids long-term semantic drift where source code says `at` while user-facing contracts say `value`, and it removes hidden coupling mistakes in future refactors.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Preserve data payload shape while renaming schema discriminator and definition labels (`atData` -> `valueData`).
  Rationale: User-visible JSON semantics remain equivalent while naming becomes consistent in schema artifacts and generated docs.
  Date/Author: 2026-03-04 / OpenCode

## Outcomes & Retrospective

Current status: planning complete, implementation not started.

Expected completion outcome: point-in-time query UX is intent-first (`value`/`--at`), migration behavior is explicit and deterministic, and all contracts/tests/docs/changelog are aligned without hidden legacy entry points.

Primary delivery risk to monitor during implementation: partial rename can leave stale user-facing strings (`at`, `--time`, `wavepeek at --help`) across code, schema, tests, and docs; this is handled by global contract-audit steps plus focused migration assertions.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI with a clear handoff between CLI parsing and engine execution. `src/cli/mod.rs` defines clap subcommands and converts parsed arguments into `crate::engine::Command`. `src/engine/mod.rs` dispatches command variants to command-specific runners. `src/output.rs` turns `CommandResult` into human text or JSON envelope output.

Today the point-in-time feature is implemented as `at`. CLI args live in `src/cli/at.rs` (`AtArgs` with `--time`). Runtime logic is in `src/engine/at.rs` and emits `CommandName::At` / `CommandData::At`. JSON schema contract in `schema/wavepeek.json` includes `"command": "at"` with `$defs.atData` and `if/then` conditioning. Integration coverage is concentrated in `tests/at_cli.rs` and top-level contract checks in `tests/cli_contract.rs`.

A few terms used in this plan:

- “Hard break” means old syntax is intentionally removed; calls fail deterministically instead of being auto-translated.
- “Command discriminator” means the JSON envelope `command` field used to identify the `data` shape.
- “Collateral” means non-runtime artifacts that must stay in sync with behavior: docs, schema, benchmark catalogs, changelog, and contract tests.

Primary files in scope:

- `src/cli/mod.rs`
- `src/cli/at.rs` (rename target: `src/cli/value.rs`)
- `src/engine/mod.rs`
- `src/engine/at.rs` (rename target: `src/engine/value.rs`)
- `src/engine/time.rs` (shared time-validation context; verify no rename-driven edits are needed)
- `src/output.rs`
- `schema/wavepeek.json`
- `tests/at_cli.rs` (rename target: `tests/value_cli.rs`)
- `tests/cli_contract.rs`
- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `README.md`
- `CHANGELOG.md`
- `bench/e2e/tests.json`

## Open Questions

No blocking questions remain for execution. This plan assumes hard-break migration is acceptable for current release stage and will be explicitly documented in help/changelog and enforced by integration tests.

## Plan of Work

Milestone 1 establishes contract-first failing tests for the new CLI surface and migration behavior. Add/adjust integration tests so they expect `value` and `--at`, then run targeted tests before code changes to capture a red phase.

Milestone 2 performs the runtime rename as one coherent atomic unit: CLI argument type, engine command variant, output rendering branch, schema discriminator/definitions, and parse-hint strings are updated together. This keeps compile and behavior drift low.

Milestone 3 updates collateral and user-facing documentation: design spec, roadmap naming, README command table, changelog migration notes, and perf scenario catalog entries.

Milestone 4 validates thoroughly and closes with mandatory review discipline: review pass #1, apply fixes, independent review pass #2 in fresh context, then final `make ci`.

### Concrete Steps

Run all commands from `/workspaces/fix-rename-at-value`.

1. Write failing tests first for the renamed command surface.

   - Rename `tests/at_cli.rs` to `tests/value_cli.rs` and update command invocations from `at` to `value`.
   - Rename flag usage and expectations from `--time` to `--at`.
   - Update JSON assertions from `"command": "at"` to `"command": "value"`.
   - Rename test function prefixes from `at_*` to `value_*` so exact red/green invocations target concrete tests in the renamed file.
   - In `tests/cli_contract.rs`, update `SHIPPED_COMMANDS`, help markers, and add explicit migration checks:
     - `wavepeek at` fails as unrecognized subcommand (`error: args:`).
     - `wavepeek value --time ...` fails with unexpected argument guidance and `See 'wavepeek value --help'.`

   Red-phase runs (expect at least one failure before implementation):

       cargo test --test value_cli value_human_output_with_scope_is_default -- --exact
       cargo test --test cli_contract legacy_at_command_is_rejected_without_alias -- --exact

   Expected red-phase signatures before implementation:

   - `value_cli` fails because `value` is not yet wired, with stderr containing `unrecognized subcommand 'value'`.
   - `cli_contract` fails on the new migration assertion path before renamed CLI surface is implemented.

2. Implement CLI and engine renames while preserving behavior.

   - Use `git mv` for module renames to preserve clear history: `src/cli/at.rs` -> `src/cli/value.rs` and `src/engine/at.rs` -> `src/engine/value.rs`. (`tests/at_cli.rs` is already renamed in step 1.)
   - In renamed `src/cli/value.rs`, rename `AtArgs` to `ValueArgs`, rename field `time` to `at`, and set clap long flag to `--at`.
   - In renamed `src/engine/value.rs`, rename types (`AtData`, `AtSignalValue`) and all `wavepeek at --help` parse hints to `wavepeek value --help`.
   - In `src/cli/mod.rs`, replace command variant `At` with `Value`, update `about`/`long_about` text, dispatch mapping, and parser tests.
   - In `src/engine/mod.rs`, replace `Command::At`/`CommandName::At`/`CommandData::At` with `Value` equivalents and ensure `as_str()` returns `"value"`.
   - In `src/output.rs`, rename render branches/tests to `CommandData::Value` and `crate::engine::value::*` types.

   Re-run targeted tests:

       cargo test --test value_cli
       cargo test --test cli_contract
       cargo test --test change_cli
       cargo test --test change_opt_equivalence

3. Update JSON schema and contracts.

   - In `schema/wavepeek.json`, replace command enum literal `"at"` with `"value"`.
   - Rename `$defs.atData` to `$defs.valueData` and update all references and conditional branches accordingly.
   - Regenerate/validate schema contract as required by project workflow.

   Run:

       make update-schema
       make check-schema

4. Update docs, roadmap, changelog, and benchmark catalog.

   - In `docs/DESIGN.md`, rename section `3.2.4 at` to `3.2.4 value`, switch syntax to `wavepeek value --at <time>`, and update examples/parameter tables.
   - Run a global docs audit for `wavepeek at` and `--time` across `docs/DESIGN.md` (including error examples), `docs/ROADMAP.md`, `README.md`, and `CHANGELOG.md`; update all live-contract references and intentionally keep mentions only in historical completed exec plans.
   - In `docs/ROADMAP.md`, replace `at` naming under Value Extraction milestone.
   - In `README.md`, change command table row from `at` to `value`.
   - In `CHANGELOG.md` Unreleased, add a clear migration note documenting hard break (`at`/`--time` removed, use `value`/`--at`).
   - In `bench/e2e/tests.json`, rename command tokens and categories so benchmark harness remains runnable with new CLI surface.

5. Commit atomic implementation + collateral units.

   Suggested commit split:

       git add -A src/cli src/engine src/output.rs tests
       git commit -m "rename point-in-time command surface to value"

       git add -A schema docs README.md CHANGELOG.md bench/e2e/tests.json
       git commit -m "align schema and docs with value command rename"

6. Run full validation gates.

       make check
       make ci

7. Mandatory review pass #1, then independent pass #2.

   - Load `ask-review` skill.
   - Run pass #1 and fix findings in a new commit.
   - Spawn a fresh reviewer session for pass #2 (do not resume pass #1 context), fix findings, and rerun fresh pass #2 if needed until clean.
   - For each review request, include a compact context packet with: scope summary, plan reference path, current diff/commits, focus files, tests run + results, and explicit assumptions (hard break policy).

8. Final post-review quality gate (required after last review fix commit).

       make check
       make ci

### Validation and Acceptance

Acceptance is complete only when all outcomes below are true simultaneously:

- `wavepeek value --waves <fixture> --at 10ns --scope top --signals clk,data` succeeds and matches previous `at` semantics for ordering and literal formatting.
- JSON mode uses envelope `command: "value"` and retains equivalent payload shape (`time` + ordered `signals[{path,value}]`).
- `wavepeek at ...` fails as an unrecognized subcommand with deterministic `error: args:` parse guidance.
- `wavepeek value --time ...` fails as an argument error with `See 'wavepeek value --help'.`
- Top-level and subcommand help contract tests pass with `value` replacing `at`.
- Schema validates and contains only the renamed discriminator/definition surface (`value`/`valueData`).
- `docs/DESIGN.md`, `docs/ROADMAP.md`, `README.md`, `CHANGELOG.md`, and `bench/e2e/tests.json` are aligned to `value`/`--at` naming.
- `make check` and `make ci` pass.
- Review pass #1 and independent pass #2 complete with no unresolved high/critical findings.

TDD acceptance requires explicit red/green evidence: at least one newly updated `value` test fails before implementation and passes after implementation in the same branch history.

### Idempotence and Recovery

All edits are text-only and safe to reapply. If compile failures appear mid-rename, recover by completing command/module-path rewiring first (`src/cli/mod.rs`, `src/engine/mod.rs`, `src/output.rs`) and then rerun targeted tests. Re-running `make update-schema`, `make check-schema`, `make check`, and `make ci` is safe and expected.

If review findings require follow-ups, apply fixes in new commits. Do not rewrite history.

### Artifacts and Notes

Expected modified/renamed artifacts:

- `src/cli/at.rs` -> `src/cli/value.rs`
- `src/engine/at.rs` -> `src/engine/value.rs`
- `tests/at_cli.rs` -> `tests/value_cli.rs`
- `src/cli/mod.rs`
- `src/engine/mod.rs`
- `src/engine/time.rs` (expected mostly unchanged; included for explicit shared-helper verification)
- `src/output.rs`
- `tests/cli_contract.rs`
- `schema/wavepeek.json`
- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `README.md`
- `CHANGELOG.md`
- `bench/e2e/tests.json`

Before closure, add concise evidence snippets to this section:

- One red-phase failing test excerpt.
- Matching green-phase passing test excerpt.
- `make ci` success signature excerpt.
- Final outputs from review pass #1 and pass #2.

### Interfaces and Dependencies

No new external dependencies are required.

The implementation must end with these canonical interfaces:

- `crate::cli::value::ValueArgs` with `waves`, `at`, `scope`, `signals`, `abs`, `json`.
- `crate::engine::Command::Value`, `crate::engine::CommandName::Value`, and `crate::engine::CommandData::Value`.
- `crate::engine::value::run(args: ValueArgs) -> Result<CommandResult, WavepeekError>`.
- Shared time-validation utilities remain centralized in `crate::engine::time`; renamed point-sampling command continues to consume that canonical path.

Contract invariants to preserve:

- Error shape remains `error: <category>: <message>`.
- Exit codes remain unchanged.
- Human/JSON payload semantics for point-in-time sampling remain equivalent aside from command/flag naming.
- No hidden or documented compatibility alias is introduced for retired `at` syntax.

Revision Note: 2026-03-04 / OpenCode - Created active ExecPlan for backlog item “Rename `at` command to `value` and `--time` to `--at`”, with explicit hard-break migration policy, TDD-first steps, and mandatory double-review closure.
Revision Note: 2026-03-04 / OpenCode - Updated after review pass #1 to remove tool-specific pseudocode, strengthen red-phase evidence criteria, require full live-doc audit for `wavepeek at`/`--time`, and harden commit staging for rename/delete safety.
Revision Note: 2026-03-04 / OpenCode - Updated after independent review pass #2 to remove duplicate `tests/at_cli.rs` rename instruction and keep steps linearly executable.
Revision Note: 2026-03-05 / OpenCode - Refreshed plan after rebase onto updated `main`: replaced stale `change -> engine::at` assumptions with current shared `engine::time` architecture and adjusted steps/interfaces accordingly.
Revision Note: 2026-03-05 / OpenCode - Addressed rebase-refresh review pass #1 by removing duplicated module-move steps in Milestone 2.
Revision Note: 2026-03-05 / OpenCode - Addressed rebase-refresh review pass #2 by clarifying test-function rename for exact red-phase execution and requiring post-review gate rerun.
