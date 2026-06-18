# Simplify DEBUG=1 Waveform Debug Trace

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek` maintainers need a low-friction way to ask for timing breadcrumbs when a waveform command is slow. After this change, running a waveform command with `DEBUG=1` writes simple debug event lines to stderr while leaving the normal stdout payload unchanged. Each debug line is one JSON object with `kind: "debug"`, a short `message`, a monotonic `timestamp_ns`, and a free-form `details` object. A maintainer can compare neighboring timestamps to estimate where time went.

The intended behavior can be observed with a command such as:

    DEBUG=1 cargo run -- value --waves tests/fixtures/hand/m2_core.vcd --at 10ns --signals top.clk --json

Stdout should still be the usual JSON envelope. Stderr should contain JSON debug event lines before or around normal fatal/stderr output, for example:

    {"kind":"debug","message":"backend.open.start","timestamp_ns":1234,"details":{"command":"value"}}
    {"kind":"debug","message":"backend.open.done","timestamp_ns":2345,"details":{"command":"value","backend":"wellen","format":"vcd"}}

## Non-Goals

This change does not add a public CLI flag. It reuses `DEBUG=1`, which already unlocks maintainer-only `change --tune-*` controls. This change does not introduce a logging or tracing framework. It does not add debug diagnostics to the successful JSON envelope and does not extend the `diagnostics` schema with debug codes. It does not create a mini benchmarking abstraction that wraps command execution. It does not hide debug events on fatal errors: a debug print followed by a fatal print is acceptable because they are independent sequential stderr events. It does not expose signal values, waveform file paths, or signal names in debug details.

## Progress

- [x] (2026-06-17T20:45:25Z) Confirmed the repository starts clean on branch `feat/perf-diag` and issue #22 requests opt-in backend performance diagnostics.
- [x] (2026-06-17T20:45:25Z) Confirmed `DEBUG=1` previously only allowed hidden `change --tune-*` controls and did not print debug output.
- [x] (2026-06-17T23:12:00Z) Opened PR #31 with an initial implementation using `PerfDiagnostics`, debug codes, schema changes, and buffered diagnostics in the command envelope.
- [x] (2026-06-18T11:20:41Z) Recorded maintainer review: keep the unrelated Git helper environment fix because hooks need it in this worktree; simplify docs; remove debug opcodes/codes; replace invasive wrappers with direct debug event prints; do not force-push.
- [x] (2026-06-18T11:24:00Z) Commit this revised ExecPlan before code changes.
- [x] (2026-06-18T11:45:00Z) Replace the current `PerfDiagnostics`/schema-envelope implementation with a direct stderr debug trace implementation.
- [x] (2026-06-18T11:45:00Z) Restore command execution code close to `main` and add only local `debug.event(...)` lines around existing steps.
- [x] (2026-06-18T11:45:00Z) Simplify public docs and schema/tests to remove debug diagnostic codes/details from the successful JSON envelope contract.
- [x] (2026-06-18T11:55:00Z) Run focused tests, `cargo test --quiet`, and `just check`; all passed.
- [ ] Commit implementation changes on top of the existing PR branch, run review, apply fixes, push, and update PR #31.

## Surprises & Discoveries

- Observation: `DEBUG=1` exists but originally printed nothing by itself.
  Evidence: the original runtime `DEBUG` check in `src/cli/mod.rs::is_debug_mode_enabled` only gated hidden `change --tune-*` flags. Explicit stderr output was diagnostics in `src/output.rs` and fatal errors in `src/main.rs`.

- Observation: Pre-commit auxiliary tests failed during `git commit` because temporary-repository Git commands inherited hook-local `GIT_*` environment variables.
  Evidence: `git commit` initially failed in `tools/docs/test_publish_docs.py`, then `tools/release/test_publish_crate.py`, then `tools/repo/test_setup_github_auth.py`; direct test runs passed outside the hook. Clearing local Git environment variables fixed those tests and was committed as `test(tools): isolate git helper environment`. The maintainer accepted keeping this fix in the PR because otherwise quality gates require bypassing hooks in this worktree.

- Observation: The first implementation was too invasive.
  Evidence: it wrapped command logic with `perf.time_phase(...)`, rewrote substantial parts of `change`, `property`, and `value`, added D-code schema contracts, and buffered debug diagnostics into successful command envelopes. Maintainer review requested a simpler event trace made of local prints.

- Observation: The simpler implementation can preserve the existing successful-envelope schema.
  Evidence: after restoring schema-related files to the pre-feature shape and adding direct stderr debug events, `just check-schema`, `cargo test --test schema_cli --quiet`, and `just check` passed.

- Observation: Tune-mode tests that set `DEBUG=1` now receive stderr debug JSON lines.
  Evidence: `tests/change_vcd_fst_parity.rs` and `tests/change_cli.rs` needed assertions changed from empty stderr to well-formed debug stderr or a debug substring.

## Decision Log

- Decision: Keep `DEBUG=1` as the only switch for this feature.
  Rationale: The feature is primarily maintainer diagnostics and user-provided performance reports. Reusing the existing debug mode avoids adding another public CLI option.
  Date/Author: 2026-06-17 / Grin with maintainer confirmation.

- Decision: Debug events are written directly to stderr as JSON lines, not buffered into `CommandResult::diagnostics`.
  Rationale: Maintainer review clarified that debug prints and fatal prints are independent sequential events. Direct stderr events are simpler, less invasive, and work even before a command succeeds.
  Date/Author: 2026-06-18 / Grin with maintainer correction.

- Decision: The debug event shape is `kind`, `message`, `timestamp_ns`, and `details`.
  Rationale: This gives scripts enough structure without inventing a debug opcode catalog. `details` is a free object so future debug events can evolve without schema churn.
  Date/Author: 2026-06-18 / Grin with maintainer confirmation.

- Decision: Do not use `WPK-D####` codes or code-specific `details` schemas.
  Rationale: The first implementation over-contracted maintainer debug output. Debug messages are not part of the stable successful-envelope diagnostic taxonomy.
  Date/Author: 2026-06-18 / Grin with maintainer confirmation.

- Decision: Do not introduce a logging framework in this slice.
  Rationale: The required behavior is command-local JSON stderr breadcrumbs under `DEBUG=1`. A global `log` or `tracing` subscriber would add dependency and test complexity without solving a current problem.
  Date/Author: 2026-06-18 / Grin.

- Decision: Keep the Git helper environment isolation commit in this PR.
  Rationale: In this worktree, hooks use local Git environment variables and aux tests create temporary Git repositories. Without the isolation fix, quality gates fail and commits require bypassing hooks.
  Date/Author: 2026-06-18 / Grin with maintainer confirmation.

- Decision: Do not force-push PR #31.
  Rationale: The maintainer requested additive fix commits on top of the existing branch history.
  Date/Author: 2026-06-18 / Grin with maintainer confirmation.

## Outcomes & Retrospective

The implementation has been simplified in the working tree. It preserves the Git helper test fix, removes the debug code/schema/envelope approach, and adds direct `DEBUG=1` stderr JSON event traces with minimal command-code churn. Focused tests, `cargo test --quiet`, and `just check` passed. Review, any follow-up fixes, push, and PR update remain.

## Context and Orientation

The repository is a Rust CLI named `wavepeek`. Waveform commands inspect VCD, FST, and optionally FSDB waveform files. The CLI layer in `src/cli/` parses command arguments and dispatches to command functions in `src/engine/`. Command functions open waveform files through `src/waveform/mod.rs`, then build `CommandResult`. `src/output.rs` prints either human output or the normal JSON envelope.

`CommandResult::diagnostics` is for successful command diagnostics in the stdout JSON envelope or human stderr diagnostics. The revised design does not use it for debug events. Debug events are direct stderr lines emitted at the point where the event occurs. Because they are direct stderr lines, they may appear before a later fatal error; this is intended.

The existing first PR implementation added `src/perf_diag.rs`, `DiagnosticKind::Debug`, `DebugDiagnosticCode`, debug schema definitions, and wrapper-style timing in engine modules. The revised implementation should remove those pieces or revert them to the pre-feature shape unless they are still needed for direct stderr events. Backend and format names remain useful for debug event details and can stay as small facade helpers if they are added with minimal risk.

A debug event is a single JSON object written to stderr when `DEBUG=1`. `timestamp_ns` is a monotonic elapsed nanosecond count since the trace object was created for the command, not wall-clock Unix time. This makes it useful for calculating deltas between adjacent events. `details` must not contain signal values, signal names, or waveform file paths.

## Open Questions

There are no remaining product questions. Implementation should use direct stderr JSON events, no codes, no buffering, no force-push, and minimal command-code changes.

## Plan of Work

First, update this ExecPlan and commit it so the changed design is durable before editing code. This is the current step.

Second, remove the envelope-based debug diagnostics. Restore `src/diagnostic.rs`, `src/output.rs`, `schema/wavepeek_v1.json`, `tools/schema/check_schema_contract.py`, and schema tests toward the previous `info`/`warning`/`error` successful-envelope contract. Keep only a concise public docs note in `docs/public/reference/machine-output.md` that `DEBUG=1` may write debug JSON lines to stderr, with the generic event shape. Do not document specific event names or commands.

Third, replace `src/perf_diag.rs` with a small `src/debug_trace.rs` module. The module should check `DEBUG=1`, store an `Instant` only when enabled, and expose an `event` method that lazily builds `details` only when enabled and writes one JSON line to stderr. The line must contain `kind: "debug"`, `message`, `timestamp_ns`, and `details`. It should add `details.command` automatically when missing. It should use existing `serde` and `serde_json`; no new logging dependency.

Fourth, reduce engine changes. Start from the original command flow from `main` for `info`, `scope`, `signal`, `value`, `change`, and `property`, then add local event calls before and after major existing steps. For example, add `backend.open.start` before `Waveform::open`, `backend.open.done` after it succeeds, `metadata.load.done` after metadata is loaded, and command-specific done events after existing loops finish. Avoid wrapping large closures. Avoid moving logic around merely to measure it. Add small backend context helpers only if needed for `backend.open.done` details.

Fifth, update tests. Remove assertions that debug diagnostics appear in the stdout JSON envelope. Add tests that run a small waveform command with `DEBUG=1`, assert stdout still has the normal payload, parse stderr lines as JSON debug events, and verify the event shape. Add a fatal-path test that shows a debug line may precede the existing fatal line without changing the fatal line itself. Keep the Git helper environment tests. Update or remove schema tests that referenced debug codes.

Sixth, run focused tests and `just check`. Commit the implementation as a follow-up fix commit on top of the PR branch. Run read-only review lanes focused on code/tests and docs/contract. Apply fixes in the main session, rerun tests, commit fixes, push to `origin feat/perf-diag`, and let PR #31 update normally.

## Concrete Steps

Work from `/workspaces/feat-perf-diag`.

1. Commit this revised plan:

    git add docs/tracker/wip/perf-debug-diagnostics-execplan.md
    git commit -m "docs(tracker): revise debug trace plan"

2. Restore or simplify the previous debug-diagnostics contract changes. Useful commands are:

    git checkout main -- src/diagnostic.rs src/output.rs schema/wavepeek_v1.json tools/schema/check_schema_contract.py tests/schema_cli.rs

   Then reapply only the minimal docs note in `docs/public/reference/machine-output.md`.

3. Replace `src/perf_diag.rs` with `src/debug_trace.rs`, update `src/lib.rs`, and add direct `debug.event(...)` calls in engine modules.

4. Run focused validation:

    cargo fmt --check
    cargo test --test value_cli debug --quiet
    cargo test --test schema_cli --quiet
    cargo test --test change_opt_equivalence --quiet
    just check-schema

5. Run the full local gate:

    just check

6. Commit implementation fixes:

    git add src docs/public/reference/machine-output.md schema/wavepeek_v1.json tools/schema/check_schema_contract.py tests
    git commit -m "fix(debug): simplify waveform debug trace"

7. Request review lanes and iterate. Then push:

    git push

## Validation and Acceptance

Without `DEBUG=1`, waveform commands must preserve existing stdout and stderr. With `DEBUG=1`, waveform commands may write JSON debug event lines to stderr. If `--json` is also used, stdout remains the normal command envelope and debug events still go to stderr rather than into `diagnostics`.

Each debug stderr line must parse as JSON and contain:

    {
      "kind": "debug",
      "message": "backend.open.done",
      "timestamp_ns": 123456,
      "details": {
        "command": "value"
      }
    }

`timestamp_ns` is nondeterministic and tests must only assert that it is an integer. `details` is a free object and tests should not assert a fixed full set of fields. Tests must assert no signal value strings appear in debug event details.

A fatal command under `DEBUG=1` may print debug events before the normal fatal line. The fatal line itself must keep the existing `fatal: <category>: <message>` format.

`just check` must pass before pushing.

## Idempotence and Recovery

All changes are ordinary source, docs, schema, and tests. If schema tests fail, confirm that debug events are not being added to the successful JSON envelope contract; the current major schema should not need debug-code definitions. If direct debug printing breaks tests that expected empty stderr under `DEBUG=1`, update only those tests where `DEBUG=1` is intentional. If a command refactor grows beyond adding local event calls, stop and simplify; the point of this revision is to reduce risk, not move more control flow around.

## Artifacts and Notes

Existing PR history on the branch before this revision:

    c97e34d test(tools): isolate git helper environment
    a21e939 docs(tracker): plan debug performance diagnostics
    bb93b7e feat(debug): add waveform performance diagnostics
    e755f18 fix(debug): tighten performance diagnostic schema
    3d9a791 docs(tracker): record perf diagnostics review
    427fe5a docs(tracker): record perf diagnostics PR

The Git helper environment fix remains intentionally in the branch. The subsequent implementation commits should simplify or revert the heavy debug diagnostics implementation without force-pushing.

## Interfaces and Dependencies

The direct trace helper should look like this conceptually:

    pub(crate) struct DebugTrace { ... }

    impl DebugTrace {
        pub(crate) fn for_command(command: CommandName) -> Self;
        pub(crate) fn event(&self, message: &'static str, details: impl FnOnce() -> serde_json::Value);
    }

`DebugTrace::event` should be lazy: when `DEBUG` is not exactly `1`, it must return before constructing details. It should write one JSON object to stderr when enabled. It should not return diagnostics to callers and should not change command result types.

Plan revision note, 2026-06-17: Created the initial self-contained execution plan after maintainer confirmation of the debug-mode diagnostics design.

Plan revision note, 2026-06-17: Updated progress and discoveries after implementing debug performance diagnostics and passing `just check`.

Plan revision note, 2026-06-17: Recorded review findings and the follow-up fixes after the first review pass.

Plan revision note, 2026-06-17: Recorded the clean final control review before push and PR creation.

Plan revision note, 2026-06-17: Marked push and PR creation complete after opening pull request #31.

Plan revision note, 2026-06-18: Replanned the feature after maintainer review requested direct stderr debug traces instead of an envelope-based performance diagnostics framework.

Plan revision note, 2026-06-18: Recorded completion of the simplified direct stderr debug trace implementation and successful validation before committing the implementation.
