# Add explicit `unlimited` limit values for `--max` and `--max-depth`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Users currently need to guess and pass very large numbers when they want unbounded output. After this change, users can ask for unbounded behavior explicitly with `unlimited` on limit flags. This makes automation scripts clearer and removes the magic-number workaround.

The user-visible contract after implementation is: `--max unlimited` disables row-count truncation, `--max-depth unlimited` disables recursion-depth truncation, `--max 0` remains invalid, `--max-depth 0` keeps its current meaning (root/current scope only), and human/JSON modes both emit explicit warnings when any limit is set to `unlimited`.

## Non-Goals

This plan does not redesign command names, output schema shape, or unrelated command semantics. It does not implement the `when` runtime engine (the command remains unimplemented). It does not change existing deterministic ordering, scope/signal resolution rules, or truncation text for bounded runs.

## Progress

- [x] (2026-02-28 11:05Z) Backlog requirement and impacted code paths were mapped from docs, CLI arg structs, engine modules, and integration tests.
- [x] (2026-02-28 11:25Z) Initial ExecPlan drafted with explicit warning text, shared limit-type strategy, and TDD-first rollout.
- [x] (2026-02-28 11:40Z) Review pass #1 findings incorporated: full help/contract coverage across affected commands, explicit warning ordering, and novice-oriented term definitions.
- [x] (2026-02-28 11:48Z) Independent review pass #2 findings incorporated: added mandatory dual-unlimited warning-order test case.
- [x] (2026-02-28 12:35Z) Added failing TDD integration/contract tests for `scope`, `signal`, `change`, and `when` (`--max unlimited` parse path) and observed pre-implementation failures.
- [x] (2026-02-28 13:15Z) Implemented shared CLI `LimitArg` parsing, wired all affected arg structs, and added CLI dispatch unit coverage for unlimited literals.
- [x] (2026-02-28 13:45Z) Implemented runtime bounded/unlimited behavior in `scope`/`signal`/`change`, optional depth bounds in waveform traversal APIs, and deterministic unlimited warning ordering before legacy warnings.
- [x] (2026-02-28 14:05Z) Updated `docs/DESIGN.md`, `CHANGELOG.md`, and `docs/BACKLOG.md` for delivered unlimited-limit contracts and backlog closure.
- [x] (2026-02-28 14:20Z) Ran full repository quality gates successfully: `make check` and `make ci`.
- [x] (2026-02-28 14:40Z) Completed mandatory review pass #1 (`review` agent): no substantive defects, plus optional coverage suggestions implemented in follow-up test commit `6be1cda`.
- [x] (2026-02-28 14:55Z) Completed mandatory independent review pass #2 (fresh `review` agent): fixed low-severity `when --help` placeholder mismatch (`--max <LIMIT>`), no remaining substantive findings.

## Surprises & Discoveries

- Observation: `when` already exposes `--max` in CLI args even though runtime is unimplemented.
  Evidence: `src/cli/when.rs` defines `max`, while `src/engine/when.rs` always returns `Unimplemented`.

- Observation: `--max 0` validation exists for `scope`, `signal`, and `change`, but there is no explicit integration test for `scope --max 0` and `signal --max 0` yet.
  Evidence: existing checks in `src/engine/scope.rs` and `src/engine/signal.rs`; missing matching test cases in `tests/scope_cli.rs` and `tests/signal_cli.rs`.

- Observation: preserving warning ordering across commands required injecting unlimited warnings before pre-existing warning branches (notably in `change`, where empty-result warning previously came first).
  Evidence: `tests/scope_cli.rs` and `tests/signal_cli.rs` now assert unlimited-warning precedence with truncation warnings; `tests/change_cli.rs` asserts unlimited warning parity with existing warning flows.

- Observation: clap `value_name` placeholders can drift from parser capabilities when argument types evolve; `when --help` still showed `--max <N>` until explicitly updated.
  Evidence: second review pass flagged help/contract mismatch; fixed in `src/cli/when.rs` by switching to `value_name = "LIMIT"` and covered in `tests/cli_contract.rs`.

## Decision Log

- Decision: Introduce one shared CLI limit enum with numeric or `unlimited`, and keep per-command semantic validation in engine.
  Rationale: This minimizes clap wiring duplication and preserves existing `error: args:` behavior where `--max 0` is rejected by runtime logic.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Emit one deterministic warning per unlimited flag, in stable order (`--max` first, then `--max-depth`).
  Rationale: Backlog requires warning parity whenever any unlimited flag is used; per-flag warnings make mixed combinations unambiguous and testable.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Keep warning text explicit and machine-friendly: `limit disabled: --max=unlimited` and `limit disabled: --max-depth=unlimited`.
  Rationale: These strings are short, deterministic, and clearly explain which limit changed behavior.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Represent unlimited depth in waveform traversal APIs as `Option<usize>` (`None` = unlimited).
  Rationale: This preserves existing bounded semantics (including depth `0`) while minimizing branching across scope/signal engines and keeping deterministic traversal order unchanged.
  Date/Author: 2026-02-28 / OpenCode

## Outcomes & Retrospective

Plan-authoring outcome: implementation scope was fully mapped and broken into atomic, testable steps with clear acceptance behavior and warning text contracts.

Implementation outcome: CLI parsing, runtime behavior, tests, and docs/backlog/changelog collateral are updated for explicit `unlimited` limits across affected commands. Full quality gates pass (`make check`, `make ci`). Both mandatory review passes completed with no substantive remaining findings.

## Context and Orientation

The CLI entrypoint is `src/cli/mod.rs`, which dispatches subcommands into engine modules. Limit flags are now typed via shared `src/cli/limits.rs::LimitArg` in command-specific arg structs: `src/cli/scope.rs`, `src/cli/signal.rs`, `src/cli/change.rs`, and `src/cli/when.rs`. Runtime validation for `--max > 0` remains in `src/engine/scope.rs`, `src/engine/signal.rs`, and `src/engine/change.rs`.

Depth-bounded traversal is implemented in `src/waveform/mod.rs` by recursive helpers that now accept `Option<usize>` bounds and stop when `depth == max_depth` for bounded runs. This is why `--max-depth 0` continues to return only root/current-scope level. `signal` additionally enforces that `--max-depth` is valid only together with `--recursive`.

Terminology used in this plan is defined here for a new contributor. TDD means "test-driven development": add or update tests first so they fail, then change code until they pass. clap is the Rust command-line parser library used by this repository in `src/cli/*`. Deterministic DFS means deterministic depth-first search order: parents before children, with sibling ordering stable across runs.

Warnings flow through `CommandResult.warnings` and are rendered as stderr lines in human mode and as JSON `warnings` in envelope mode by `src/output.rs`. Existing truncation warning contracts must remain unchanged for bounded runs.

The feature request source of truth is `docs/BACKLOG.md` entry "Add explicit `unlimited` values for limit flags (`--max`, `--max-depth`)". Product contract text to update lives in `docs/DESIGN.md` sections for `scope`, `signal`, `change`, and `when`.

## Open Questions

No blocking open questions remain. This plan fixes the previously ambiguous warning wording by defining exact strings in the Decision Log and Validation sections.

## Plan of Work

Milestone 1 introduces failing tests and contract checks before production code changes (TDD, meaning tests are written to fail before implementation). Add integration cases that prove `unlimited` is accepted, that mixed bounded/unbounded combinations work, and that warnings appear in both human and JSON modes with exact text. Add missing negative tests for `scope --max 0` and `signal --max 0` to protect existing behavior while refactoring types.

Milestone 2 adds shared limit parsing in the CLI layer. Create a small reusable limit type under `src/cli/` that parses either a non-negative integer or the literal `unlimited`. Wire it into all affected commands so clap (the command-line parser used by this binary) accepts `unlimited` for `--max` and `--max-depth` where applicable. Update CLI dispatch tests in `src/cli/mod.rs` to validate parsed values.

Milestone 3 updates engine behavior. Convert parsed limits into effective bounded/unbounded behavior. Keep existing `--max 0` runtime rejection with current error category and help hints. Preserve `--max-depth 0` semantics, and support `--max-depth unlimited` in recursive traversals by using an unbounded traversal path in waveform helpers.

Milestone 4 completes collateral and closes quality gates. Update `docs/DESIGN.md` contracts, refresh changelog notes in `CHANGELOG.md`, and mark backlog entry complete in `docs/BACKLOG.md`. Run repository checks and integration tests to prove no contract regressions.

### Concrete Steps

Run all commands from `/workspaces/feat-unlimited-max`.

1. Add failing tests first.

    - Extend `tests/scope_cli.rs` with cases for:
      - `--max unlimited` returns full fixture rows and includes warning `limit disabled: --max=unlimited`.
      - `--max-depth unlimited` preserves full depth and includes warning `limit disabled: --max-depth=unlimited`.
      - `--max unlimited --max-depth unlimited` emits both unlimited warnings in exact order in JSON and human modes.
      - `--max 0` returns `error: args:`.
    - Extend `tests/signal_cli.rs` with cases for:
      - `--max unlimited` warning parity in JSON/human.
      - `--recursive --max-depth unlimited` support and warning.
      - `--max 0` returns `error: args:`.
      - mixed combo `--max unlimited --max-depth 1` and `--max 2 --max-depth unlimited`.
    - Extend `tests/change_cli.rs` with cases for:
      - `--max unlimited` removes truncation and adds warning.
      - `--max 0` remains invalid.
    - Extend `tests/cli_contract.rs` help checks so all affected commands document `unlimited` correctly:
      - `scope --help` mentions `--max` and `--max-depth` unlimited support.
      - `signal --help` mentions `--max` and `--max-depth` unlimited support.
      - `change --help` mentions `--max unlimited` support.
      - `when --help` mentions `--max unlimited` support even though runtime is unimplemented.
    - Add a contract test that `wavepeek when --max unlimited ...` passes clap argument parsing and then fails with the existing unimplemented runtime error (not an args parse error).

    Execute targeted test commands and confirm new tests fail before implementation:

        cargo test --test scope_cli
        cargo test --test signal_cli
        cargo test --test change_cli
        cargo test --test cli_contract

2. Implement shared CLI limit parsing.

    - Add `src/cli/limits.rs` with a shared enum and parser for integer-or-`unlimited` values.
    - Register the new module from `src/cli/mod.rs`.
    - Update arg structs in:
      - `src/cli/scope.rs`
      - `src/cli/signal.rs`
      - `src/cli/change.rs`
      - `src/cli/when.rs`
    - Update unit tests in `src/cli/mod.rs` that assert parsed fields.

3. Implement engine and waveform support.

    - Update `src/engine/scope.rs`, `src/engine/signal.rs`, and `src/engine/change.rs` to consume the new limit type.
    - Add deterministic unlimited warnings to `CommandResult.warnings` in these engines.
    - Update `src/waveform/mod.rs` traversal helpers to accept optional depth bound (`None` means unlimited).
    - Ensure no changes to ordering and existing bounded truncation messages.

4. Update docs and release collateral.

    - Update command contracts/examples in `docs/DESIGN.md`.
    - Add unreleased note to `CHANGELOG.md`.
    - Mark backlog item as completed in `docs/BACKLOG.md`.

5. Run full verification.

        make check
        make ci

### Validation and Acceptance

Acceptance is behavior-based and must be observable via CLI and tests.

For `scope`, `signal`, and `change`, `--max unlimited` must disable truncation by count limit. JSON output must include warning `limit disabled: --max=unlimited`; human mode must print the same warning text to stderr prefixed by `warning:`.

For `scope` and recursive `signal`, `--max-depth unlimited` must disable depth truncation. JSON output must include warning `limit disabled: --max-depth=unlimited`; human mode must print the same warning text to stderr prefixed by `warning:`.

Warning order must be fully deterministic. Emit unlimited warnings first in this fixed order: `limit disabled: --max=unlimited`, then `limit disabled: --max-depth=unlimited` when applicable. Emit existing legacy warnings after unlimited warnings, preserving each command's current legacy ordering.

`--max 0` must remain invalid with `error: args:` for implemented commands (`scope`, `signal`, `change`). `--max-depth 0` behavior must remain unchanged (root/current scope only). `signal --max-depth <...>` without `--recursive` must remain `error: args:`.

`when --max unlimited` must parse successfully and still fail with existing unimplemented runtime error, proving the new parser does not break current command status.

Add combination tests where unlimited and legacy warnings can coexist (for example bounded `--max` with unlimited `--max-depth` on a fixture that still truncates by `--max`) and assert exact warning order in both JSON and human modes.

### Idempotence and Recovery

All edits are additive/refactoring-safe and can be re-applied without destructive operations. If a test assertion is too strict due to fixture variability, adjust only the assertion granularity (for example row count or warning presence) without weakening contract checks for warning text, error category, and deterministic ordering.

If traversal refactor introduces regression risk, keep old helper behavior behind tests while migrating to optional depth bound in small steps. Re-run targeted suites after each file change before proceeding.

### Artifacts and Notes

Expected examples after implementation:

    $ wavepeek scope --waves fixtures/m2_core.vcd --max unlimited --json
    {"command":"scope","warnings":["limit disabled: --max=unlimited"], ... }

    $ wavepeek signal --waves fixtures/signal_recursive_depth.vcd --scope top --recursive --max-depth unlimited
    ...
    warning: limit disabled: --max-depth=unlimited

    $ wavepeek scope --waves fixtures/signal_recursive_depth.vcd --max 1 --max-depth unlimited --json
    {"warnings":["limit disabled: --max-depth=unlimited","truncated output to 1 entries (use --max to increase limit)"], ... }

    $ wavepeek scope --waves fixtures/signal_recursive_depth.vcd --max unlimited --max-depth unlimited --json
    {"warnings":["limit disabled: --max=unlimited","limit disabled: --max-depth=unlimited"], ... }

    $ wavepeek change --waves fixtures/change_many_events.vcd --signals top.sig --max 0
    error: args: --max must be greater than 0.

### Interfaces and Dependencies

Define a shared type in `src/cli/limits.rs`:

    pub enum LimitArg {
        Numeric(usize),
        Unlimited,
    }

Provide parsing and helpers in the same module:

    impl std::str::FromStr for LimitArg { ... }
    impl LimitArg {
        pub fn is_unlimited(&self) -> bool { ... }
        pub fn numeric(&self) -> Option<usize> { ... }
    }

Command arg struct fields must use this type:

- `ScopeArgs.max: LimitArg`
- `ScopeArgs.max_depth: LimitArg`
- `SignalArgs.max: LimitArg`
- `SignalArgs.max_depth: Option<LimitArg>`
- `ChangeArgs.max: LimitArg`
- `WhenArgs.max: Option<LimitArg>`

Waveform traversal helpers should accept optional depth bounds:

    pub fn scopes_depth_first(&self, max_depth: Option<usize>) -> Vec<ScopeEntry>
    pub fn signals_in_scope_recursive(&self, scope_path: &str, max_depth: Option<usize>) -> Result<Vec<SignalEntry>, WavepeekError>

Do not add new third-party crates. Use existing `clap`, engine error types, and warning pipeline.

Plan revision note: initial plan created for backlog item `Add explicit unlimited values for limit flags (--max, --max-depth)`; then revised after two review passes to cover `change`/`when` help-contract parity, define deterministic ordering against legacy warnings, add novice term definitions, and require an explicit dual-unlimited warning-order test.

Plan revision note (implementation update): marked milestones 1-4 complete with timestamps, recorded waveform/API representation decision (`Option<usize>` for unlimited depth), captured warning-ordering implementation discovery, and logged successful `make check`/`make ci` execution before mandatory double-review passes.

Plan revision note (review completion update): recorded both completed review passes, documented and fixed the `when --help` limit placeholder mismatch, and updated implementation outcome status to fully complete.
