# Deduplicate Time Validation Between `at` and `change`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, both `wavepeek at` and `wavepeek change` will rely on one canonical time-validation path for parsing time tokens, enforcing dump bounds, checking dump-precision alignment, and converting to raw timestamps. This removes drift risk between commands and makes future time-contract changes safer.

A user can verify the outcome by running targeted CLI tests for both commands and observing parity on accepted and rejected time tokens (including invalid token, missing units, out-of-bounds, misalignment, and range handling), while command-specific help hints and argument labels remain unchanged.

## Non-Goals

This plan does not change waveform sampling semantics, event-expression behavior, output schema, or human output formatting. This plan does not alter `change --when` runtime behavior, `iff` status, or engine-dispatch heuristics. This plan does not expand accepted time syntax (for example, decimals remain invalid).

## Progress

- [x] (2026-03-04 20:03Z) Read backlog/design context and confirmed close criteria from `docs/BACKLOG.md` and `docs/DESIGN.md`.
- [x] (2026-03-04 20:03Z) Mapped existing duplication in `src/engine/at.rs` and `src/engine/change.rs`, including tests that currently cover only part of parity.
- [x] (2026-03-04 20:03Z) Drafted this ExecPlan with explicit TDD red/green steps, review gates, and commit boundaries.
- [x] (2026-03-04 20:22Z) Added parity-focused integration tests in `tests/change_cli.rs` and captured TDD red/green evidence with `change_misaligned_time_includes_help_hint`.
- [x] (2026-03-04 20:22Z) Introduced `src/engine/time.rs` and migrated both `src/engine/at.rs` and `src/engine/change.rs` to shared validation/context helpers.
- [x] (2026-03-04 20:22Z) Ran targeted tests (`at_cli`, `change_cli`, `engine::time::tests`) and full gate (`make ci`) with all checks passing.
- [x] (2026-03-04 20:25Z) Completed mandatory review pass #1 with `review` agent; no severity-tagged findings.
- [x] (2026-03-04 20:25Z) Completed mandatory independent review pass #2 with fresh `review` session; no severity-tagged findings.
- [x] (2026-03-04 20:22Z) Closed the backlog item by removing the completed entry from `docs/BACKLOG.md`.
- [x] (2026-03-04 20:25Z) Finalized plan lifecycle notes and prepared plan move from active to completed.

## Surprises & Discoveries

- Observation: `change` already imports core time primitives from `at`, but not the full validation path.
  Evidence: `src/engine/change.rs` imports `parse_time_token`, `as_zeptoseconds`, and related helpers from `src/engine/at.rs` while still implementing separate `parse_bound_time` validation logic.

- Observation: `change` reparses dump bounds inside `parse_bound_time` even though bounds are already parsed in `run`.
  Evidence: `src/engine/change.rs` parses `metadata.time_start`/`metadata.time_end` in both `run` and `parse_bound_time`.

- Observation: `change` misalignment errors previously omitted the command help hint even though adjacent time-token errors included it.
  Evidence: TDD red-phase output for `change_misaligned_time_includes_help_hint` showed stderr without `See 'wavepeek change --help'.`.

## Decision Log

- Decision: Create a dedicated canonical module `src/engine/time.rs` rather than keeping shared logic in `at.rs`.
  Rationale: `at` is a command module, while time parsing/validation is cross-command infrastructure. A dedicated module reduces coupling and clarifies ownership.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Keep command-specific error wording and help hints in command-layer wrappers while sharing validation mechanics.
  Rationale: Contracts intentionally differ in wording (`wavepeek at --help` versus `wavepeek change --help`, and `--from`/`--to` labels). Shared mechanics must not erase these distinctions.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Extend the `change` misalignment error text to include `See 'wavepeek change --help'.`.
  Rationale: This keeps command-local guidance consistent with other `change` time-token errors and allowed writing a meaningful red/green parity test.
  Date/Author: 2026-03-04 / OpenCode

## Outcomes & Retrospective

Current status: complete.

Target outcome: backlog item "Duplicated time parsing/alignment logic in `at` and `change`" can be closed because validation mechanics are canonicalized and parity tests exist for accepted/rejected token behavior.

Delivered outcome: shared validation mechanics now live in `src/engine/time.rs`, both commands consume the canonical path, parity-focused `change` tests are in place, and the backlog debt item was removed.

Known residual risk: none identified by two independent review passes. Future risk is limited to intentional message-contract changes in either command layer; integration tests now catch this class of drift.

## Context and Orientation

`wavepeek` is a Rust CLI with command runtime code under `src/engine/`. The `at` command reads values at one timestamp; the `change` command computes value deltas over a time window.

In this repository, a "time token" is a string with an integer plus unit suffix (`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`). "Dump precision" means the waveform `time_unit`. "Alignment" means a requested timestamp must be representable as an integer multiple of that precision.

Current state relevant to this task:

- Canonical time mechanics now live in `src/engine/time.rs` (`ParsedTime`, `TimeUnit`, metadata parsing context, and token-to-raw validation helpers).
- `src/engine/at.rs` and `src/engine/change.rs` both call the shared helper path and map `TimeValidationError` into command-specific `error: args:` messages.
- `tests/change_cli.rs` now includes parity-focused checks for decimal rejection, out-of-bounds rejection, inclusive-bound acceptance, and misalignment help-hint presence.
- `docs/BACKLOG.md` no longer contains the duplicated-time-validation debt item because this implementation is now in place.

Files expected to change:

- `src/engine/mod.rs`
- `src/engine/time.rs` (new)
- `src/engine/at.rs`
- `src/engine/change.rs`
- `tests/change_cli.rs`
- `docs/BACKLOG.md`
- `docs/exec-plans/completed/2026-03-04-time-validation-dedup/PLAN.md`

## Open Questions

No blocking questions remain. The close criteria are explicit in backlog text and can be implemented without product-level ambiguity.

## Plan of Work

Milestone 1 is a TDD red phase for parity coverage. Add integration tests that express parity-relevant acceptance and rejection behavior across `at` and `change`, then run targeted tests and capture at least one failing assertion before implementation.

Milestone 2 implements canonical shared validation. Introduce `src/engine/time.rs` with shared time primitives plus reusable metadata/time-bound validation helpers, then migrate `at` and `change` to call this canonical path. Keep all command-specific error messages and help hints stable.

Milestone 3 is validation and closure. Run targeted tests and repository gate(s), then update backlog and move the ExecPlan to completed state once both review passes are clean.

Milestone 4 enforces mandatory review discipline. Run review pass #1, fix findings, commit fixes if needed, then run an independent review pass #2 with fresh context and resolve anything remaining.

### Concrete Steps

Run all commands from `/workspaces/fix-time-parsing-duplicated`.

1. TDD red phase.

   - Add/extend tests in `tests/change_cli.rs` and `tests/at_cli.rs` for parity-relevant cases:
     - rejected decimal token for `change` (`--from 1.5ns`)
     - rejected out-of-bounds token for `change` (`--from 11ns` against fixture bounds)
     - accepted inclusive lower/upper bounds for `change` (matching `at` contract style)
     - retained missing-units and misalignment fragments in `change`
   - Run targeted tests and capture at least one failing test output before code changes:

       cargo test --test change_cli
       cargo test --test at_cli

2. Implement canonical time-validation module.

   - Add `src/engine/time.rs` with:
     - `ParsedTime`, `TimeUnit`, `parse_time_token`, `as_zeptoseconds`, `ensure_non_zero_dump_tick`, `format_raw_timestamp`
     - parsed metadata context and helper(s) to validate a requested token into raw timestamp with shared mechanics.
   - Update `src/engine/mod.rs` to expose `time` module.
   - Update `src/engine/at.rs` to import shared helpers from `crate::engine::time` and replace inline bounds/alignment/raw conversion with shared helper usage.
   - Update `src/engine/change.rs` to import shared helpers from `crate::engine::time` and replace `parse_bound_time` internals to use canonical helper/context instead of reparsing metadata.

3. TDD green phase and broader validation.

   - Re-run targeted tests:

       cargo test --test at_cli
       cargo test --test change_cli

   - Run full quality gate:

       make ci

4. Commit atomic implementation unit.

       git add src/engine/mod.rs src/engine/time.rs src/engine/at.rs src/engine/change.rs tests/change_cli.rs docs/BACKLOG.md docs/exec-plans/completed/2026-03-04-time-validation-dedup/PLAN.md
       git commit -m "refactor(engine): share time validation across at and change"

5. Mandatory review pass #1.

   - Load `ask-review` skill and run review agent on current diff.
   - Apply fixes and commit if findings are valid.

6. Mandatory independent review pass #2.

   - Run a fresh review request (new context).
   - Apply fixes and commit if findings are valid.

7. Finalize plan lifecycle.

   - Move this plan directory from `docs/exec-plans/active/` to `docs/exec-plans/completed/` after final clean validation.
   - Add final retrospective notes and revision note.

### Validation and Acceptance

Acceptance is complete only when all of the following are true:

- `at` and `change` use one canonical validation utility path for parse/convert/bounds/alignment/raw-range mechanics.
- `at` and `change` retain command-appropriate error fragments and help hints.
- Regression tests cover parity-relevant accepted and rejected token behavior for both commands.
- `make ci` passes.
- Review pass #1 and fresh review pass #2 are both clean, or all findings are fixed and rechecked.
- Backlog item at `docs/BACKLOG.md` is removed/closed.

TDD requirement: at least one new/adjusted test fails before implementation and passes after implementation, with evidence recorded in this plan.

### Idempotence and Recovery

All edits are text/code changes and are safe to reapply. Re-running test commands is idempotent. If refactor introduces message drift, recover by restoring command-layer formatting strings in `src/engine/at.rs` or `src/engine/change.rs` while keeping shared validation logic intact.

If review findings require adjustments, apply them in follow-up commits; do not rewrite history.

### Artifacts and Notes

Recorded evidence so far:

- TDD red-phase (`cargo test --test change_cli change_misaligned_time_includes_help_hint -- --exact`, before implementation):

      test change_misaligned_time_includes_help_hint ... FAILED
      Unexpected stderr, failed var.contains(See 'wavepeek change --help'.)
      var: error: args: time '15ps' cannot be represented exactly in dump precision '1ns'.

- TDD green-phase (`cargo test --test change_cli change_misaligned_time_includes_help_hint -- --exact`, after implementation):

      running 1 test
      test change_misaligned_time_includes_help_hint ... ok
      test result: ok. 1 passed; 0 failed

- Validation gate (`make ci`):

      cargo fmt -- --check
      cargo clippy --all-targets --all-features -- -D warnings
      cargo test -q
      python3 -m unittest discover -s bench/e2e -p "test_*.py"
      cargo check
      ... all checks passed

Review closure evidence:

- Review pass #1 (`review` agent): clean, no High/Medium/Low findings.
- Review pass #2 (`review` agent, fresh context): clean, no High/Medium/Low findings.

### Interfaces and Dependencies

The canonical shared API in `src/engine/time.rs` will be internal to `crate::engine` (`pub(crate)` visibility) and consumed by `crate::engine::at` and `crate::engine::change`.

The shared helper should return structured validation output (for example, raw timestamp plus parsed context) so command modules can preserve their own message text. This avoids copy/paste validation while keeping user-facing contracts stable.

Revision note (2026-03-04 20:03Z): Initial plan created from backlog item analysis and repository state.
Revision note (2026-03-04 20:22Z): Updated progress after TDD red/green, shared-module migration, backlog closure, and successful `make ci` run.
Revision note (2026-03-04 20:25Z): Recorded both mandatory review passes as clean and marked plan complete.
