# Add `--max` to `wavepeek property`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Before this change, users could ask `wavepeek property` to capture every property match or transition in a waveform range, but they could not bound the number of rows. Dense trigger streams or `--capture match` could therefore flood a terminal, a JSON client, or an agent context window. This change adds `--max` with the same bounded-output contract used by `change`: the default limit is 50 rows, `--max unlimited` deliberately disables truncation with warning `WPK-W0001`, `--max 0` is an argument error, and truncation emits warning `WPK-W0002`. A user can see the behavior by running `wavepeek property --waves <dump> --eval <expr> --capture match --max 1 --json` and observing one data row plus a truncation diagnostic.

## Non-Goals

This change does not alter expression parsing, event trigger semantics, sample timing, or the schema shape of a property row. It does not add a payload-value feature to `property`; users still follow up with `value --at <sample_time>` when they need signal values. It does not change the `change`, `scope`, `signal`, or `value` command contracts except where shared documentation mentions that `property` is now a bounded command.

## Progress

- [x] (2026-06-23T09:58:02Z) Created branch `feat/property-max-limit` from `main`.
- [x] (2026-06-23T09:58:02Z) Read issue #38 and confirmed the requested behavior: add `property --max`, default bounded output, `unlimited` warning, zero-limit argument error, truncation warning, JSONL `summary.truncated`, docs, and tests.
- [x] (2026-06-23T09:58:02Z) Inspected the existing `change` implementation and tests to identify the reusable limit contract.
- [x] (2026-06-23T09:58:02Z) Created this execution plan.
- [x] (2026-06-23T10:09:02Z) Committed this execution plan as the first branch milestone (`7226a42`).
- [x] (2026-06-23T10:09:02Z) Ran a focused read-only review of this execution plan and incorporated the findings.
- [x] (2026-06-23T10:17:16Z) Implemented the CLI and engine changes for `property --max`.
- [x] (2026-06-23T10:17:16Z) Added and updated integration/unit tests for CLI parsing, human output, JSON output, JSONL output, and argument errors.
- [x] (2026-06-23T10:17:16Z) Updated public docs and changelog.
- [x] (2026-06-23T10:17:16Z) Ran formatting and focused tests during implementation.
- [x] (2026-06-23T10:26:27Z) Ran read-only implementation review, fixed low-severity docs findings, and recorded that the code lane had no substantive findings.
- [x] (2026-06-23T10:37:40Z) Ran a final independent control review pass with no substantive findings.
- [x] (2026-06-23T10:37:40Z) Ran the repository pre-handoff gate `just check` successfully.
- [ ] Commit the implementation.
- [x] (2026-06-23T10:38:32Z) Explicitly justified retaining this branch-local WIP plan for PR review because the user requested a committed ExecPlan; the PR body should note it can be removed before merge if the maintainer prefers.
- [ ] Push the branch and open a pull request linked to issue #38.

## Surprises & Discoveries

- Observation: `property` already has a streaming JSONL entrypoint with a row sink, but it currently always ends with `writer.end(false)` regardless of whether any future limit truncates output.
  Evidence: `src/engine/property.rs` has `run_jsonl` calling `writer.end(false)` after diagnostics.
- Observation: Generic JSONL adaptation in `src/output.rs` already derives summary truncation from warning code `WPK-W0002`, but `property` bypasses that adapter in streaming mode through `engine::run_jsonl`.
  Evidence: `src/cli/mod.rs` dispatches `--jsonl` directly to `engine::run_jsonl`; `src/output.rs::write_jsonl_result` computes truncation only for non-streaming adapter paths.
- Observation: `change` is the best contract template because it uses `LimitArg`, defaults `--max` to `50`, rejects zero, warns on `unlimited`, stops before emitting row number `limit + 1`, and reports JSONL truncation through `writer.end(outcome.stats.truncated)`.
  Evidence: `src/cli/change.rs` defines `pub max: LimitArg` with default `50`; `src/engine/change.rs::run_with_sink` maps the flag into `Option<usize>` and emits diagnostics.
- Observation: The new `max` field also affects unit-test struct literals because `PropertyArgs` is constructed directly in `src/engine/property.rs` tests.
  Evidence: The focused plan review found `PropertyArgs { ... }` literals around `src/engine/property.rs:573`; the implementation now sets `max: LimitArg::Unlimited` in those evaluation-focused tests.
- Observation: Focused tests confirm JSON, human, and JSONL truncation behavior before full-gate review.
  Evidence: `cargo test --test property_cli property_` passed 22 tests; `cargo test --test jsonl_cli property_jsonl` passed 2 tests; property help contract tests passed.
- Observation: Implementation review found no code issues; docs review found two stale or narrow wording issues.
  Evidence: Code review returned "No substantive findings"; docs review asked to broaden `property` docs from "matching rows" to "captured rows" and to update the plan purpose from future-tense wording.
- Observation: The final independent control review found no substantive findings after those wording fixes.
  Evidence: Control review returned "No substantive findings".
- Observation: The repository pre-handoff gate passed with FSDB tooling available.
  Evidence: `just check` completed successfully, including `cargo fmt -- --check`, clippy for normal and FSDB targets, docs publish check, commit-message check, and FSDB smoke tests.
- Observation: The WIP plan is intentionally retained in the PR despite normal cleanup guidance.
  Evidence: The user explicitly requested a committed ExecPlan as part of this branch workflow.

## Decision Log

- Decision: Use the existing `crate::cli::limits::LimitArg` type for `property --max`.
  Rationale: It already parses positive integers plus the literal `unlimited`; sharing it keeps CLI behavior consistent with `scope`, `signal`, and `change`.
  Date/Author: 2026-06-23 / Grin
- Decision: Use a default property limit of `50`.
  Rationale: Issue #38 asks for the default to likely match `change`, and `change` already documents and tests `50` as the bounded-output default.
  Date/Author: 2026-06-23 / Grin
- Decision: Detect truncation during the property evaluation loop before emitting a row once `emitted == limit`.
  Rationale: This matches `change` semantics: exactly `limit` rows are emitted, then the command records that additional matching output existed and stops evaluating.
  Date/Author: 2026-06-23 / Grin
- Decision: Keep warning order as disabled-limit first, empty-result second, truncation after evaluation.
  Rationale: `change` pushes the `WPK-W0001` disabled-limit diagnostic before executing the search, then appends empty/truncation diagnostics based on the result. Mirroring this makes automation behavior predictable.
  Date/Author: 2026-06-23 / Grin

## Outcomes & Retrospective

Implementation is present in the working tree but not yet committed. The current code adds `property --max`, enforces the default 50-row limit, emits `WPK-W0001` and `WPK-W0002` in the expected modes, and reports JSONL truncation through the end summary. Focused tests pass, implementation review and control review are clean, and `just check` passes. The remaining work is commit, push, and PR creation.

## Context and Orientation

`wavepeek` is a Rust CLI. The command-line argument structs live under `src/cli/`, while command execution lives under `src/engine/`. `property` evaluates a logical expression from `--eval` at timestamps selected by an event expression from `--on`. A property row is represented by `src/engine/property.rs::PropertyCaptureRow`, with `time`, `sample_time`, and `kind`. `time` is the event timestamp; `sample_time` is the timestamp where values were evaluated. In native sampling those are the same; in pre-edge sampling, `sample_time` can be just before the event.

A bounded-output flag is a command-line option that limits how many user-visible rows are printed. In this repository the shared parser type is `src/cli/limits.rs::LimitArg`. `LimitArg::Numeric(n)` means the command should emit at most `n` rows. `LimitArg::Unlimited` means the user explicitly opted out of truncation. The repository uses stable warning diagnostics for bounded output: `WPK-W0001` means a limit was disabled, `WPK-W0002` means output was truncated, and `WPK-W0003` means a valid query returned no rows.

The key files for this change are:

- `src/cli/property.rs`, which defines `PropertyArgs` and must gain a `max: LimitArg` field under output options.
- `src/engine/property.rs`, which runs property evaluation, emits rows through `PropertyRowSink`, and must enforce the limit for both collecting and streaming sinks.
- `src/cli/mod.rs`, whose tests verify that clap parsing and dispatch preserve property arguments.
- `tests/property_cli.rs`, which covers property CLI behavior in human and JSON modes.
- `tests/jsonl_cli.rs`, which covers streaming JSONL behavior and validates each record against `schema/wavepeek-stream-v1.json`.
- `tests/cli_contract.rs`, which locks command help fragments.
- `docs/public/commands/property.md`, which explains property workflows.
- `docs/public/reference/command-model.md` and `docs/public/reference/machine-output.md`, which define cross-cutting bounded-output and diagnostic semantics.
- `CHANGELOG.md`, which records user-visible changes under `Unreleased`.

`property --jsonl` is important because it does not use the generic `src/output.rs::write_jsonl_result` adapter. `src/cli/mod.rs` dispatches JSONL waveform commands directly to `engine::run_jsonl`, and `src/engine/property.rs::run_jsonl` writes rows as they are found. Therefore the property engine itself must pass the real truncation value into `writer.end(...)`.

## Open Questions

No open product questions remain. The issue asks for consistency with `change`, and this plan resolves the default as `50`.

## Plan of Work

First, update `src/cli/property.rs`. Add `use crate::cli::limits::LimitArg;` near the sampling import. Add a `pub max: LimitArg` field to `PropertyArgs` in the Output options group. The field should use the same clap annotation and wording as `change`, except the help text should refer to property capture rows rather than snapshot rows. The intended help text is `Maximum number of property rows (`unlimited` disables truncation, value must be > 0)`, with `#[arg(long, default_value = "50", help_heading = "Output options")]`.

Second, update `src/engine/property.rs`. Import `LimitArg`. Extend `PropertyRunStats` with `truncated: bool`. At the start of `run_with_sink`, convert `args.max` into `max_entries: Option<usize>`: reject `LimitArg::Numeric(0)` with `WavepeekError::Args("--max must be greater than 0.".to_string())`, return `Some(value)` for numeric values, and `None` for unlimited. Initialize `diagnostics` with `WPK-W0001` when `args.max.is_unlimited()`.

Third, enforce the limit at every point where a property row would be emitted. In both `CaptureMode::Match` and transition capture branches, after deciding a row qualifies and before calling `sink.emit(...)`, check `if let Some(limit) = max_entries && emitted == limit`. If true, set `truncated = true` and break out of the candidate loop without emitting another row. This preserves exactly `limit` emitted rows and proves there was another row available. After the loop, append `WPK-W0003` only if `emitted == 0`. Append `WPK-W0002` if `max_entries` is `Some(max)` and `truncated` is true, using the message `truncated output to {max} entries (use --max to increase limit)`. Return `PropertyRunStats { emitted, truncated }`.

Fourth, update JSONL completion. In `src/engine/property.rs::run_jsonl`, replace `writer.end(false)` with `writer.end(outcome.stats.truncated)`. The non-streaming JSON and human paths do not need a special summary flag; the diagnostics already carry truncation in those modes.

Fifth, update tests. In `src/cli/mod.rs` unit tests, expand `property_dispatch_parses_capture_default` or add a neighboring test so it asserts the default `args.max == LimitArg::Numeric(50)` and explicit `--max unlimited` parses as `LimitArg::Unlimited`. In `src/engine/property.rs` unit tests, update every `PropertyArgs { ... }` literal to set `max`, usually `LimitArg::Unlimited` when the test is about evaluation semantics and not row limiting. Import `LimitArg` in that test module if needed. In `tests/cli_contract.rs`, update property help assertions so `property --help` documents `--max` and the `unlimited` literal. In `tests/property_cli.rs`, add a dense property fixture that emits more than 50 match rows, then test: default JSON emits 50 rows plus `WPK-W0002`; `--max 1` emits one human/JSON row plus truncation warning; `--max unlimited` emits more than 50 rows plus `WPK-W0001`; and `--max 0` fails as an args error. In `tests/jsonl_cli.rs`, add or extend a property JSONL test so `--max 1 --jsonl` emits one item, a `WPK-W0002` diagnostic record, and an `end` summary with `truncated: true`.

Sixth, update documentation. In `docs/public/commands/property.md`, explain that property output is bounded by default, show a short `--max` truncation example, and mention `--max unlimited` in the non-obvious behavior list. In `docs/public/reference/command-model.md`, include `property` among commands whose event-row output is bounded by count limits if the existing text needs a direct example. In `docs/public/reference/machine-output.md`, no schema change is expected because diagnostics and JSONL truncation summary already exist; update only if wording currently implies truncation examples are `change`-only. Add a concise `CHANGELOG.md` entry under `Unreleased / Added`.

Seventh, run validation. Start with focused commands while iterating, then run `just check` before handoff. If `just check` fails due to environment rather than code, capture the exact failure and run the closest available focused tests, but do not pretend the full gate passed.

## Concrete Steps

Run all commands from repository root `/workspaces/wavepeek`.

Create and inspect the plan:

    git switch -c feat/property-max-limit
    git status --short --branch

Expected branch output:

    ## feat/property-max-limit

After editing code, run focused formatting and tests:

    cargo fmt --check
    cargo test property_dispatch_parses_capture_default
    cargo test --test property_cli property_default_max_is_50_with_truncation_warning
    cargo test --test property_cli property_max_one_truncates_in_human_and_json_modes
    cargo test --test property_cli property_unlimited_max_disables_truncation_and_emits_warning
    cargo test --test property_cli property_rejects_zero_max
    cargo test --test jsonl_cli property_jsonl_reports_truncation_in_summary
    cargo test --test cli_contract property_help_uses_aligned_summary_behavior_and_grouped_option_docs
    cargo test --test cli_contract help_documents_unlimited_limit_literals_for_all_affected_commands

Before handoff, run the repository gate:

    just check

If the gate passes, expect a final success line from the recipe and no uncommitted formatting changes. If the gate fails, inspect the first failing command, fix the underlying cause, and rerun the failing command before rerunning `just check`.

Commit the plan and implementation with conventional commits. Suggested commit messages are:

    docs(tracker): plan property max limit
    feat(property): add max output limit

Before pushing, decide the fate of this WIP plan. Local guidance says `docs/tracker/wip/` artifacts should normally be removed before merge unless the maintainer wants them for handoff. If the branch is ready for PR review and the implemented tests/docs are self-explanatory, remove `docs/tracker/wip/property-max-limit.md` in the final implementation commit. If the maintainer wants the plan preserved during review, keep it in the PR but mention that it should be dropped before merge.

Push and open the PR:

    git push -u origin feat/property-max-limit
    gh pr create --repo kleverhq/wavepeek --base main --head feat/property-max-limit --title "feat(property): add --max output limit" --body "Closes #38\n\n## Summary\n- Add bounded `property --max` output with default 50-row limit\n- Emit disabled-limit and truncation diagnostics across human, JSON, and JSONL modes\n- Update docs and tests for the property limit contract\n\n## Tests\n- `just check`"

## Validation and Acceptance

A successful implementation must satisfy these observable behaviors:

Running `wavepeek property --waves <dense.vcd> --scope top --eval sig --capture match --json` on a fixture with more than 50 matching rows emits exactly 50 JSON data rows and a diagnostic:

    {"kind":"warning","code":"WPK-W0002","message":"truncated output to 50 entries (use --max to increase limit)"}

Running the same query with `--max 1` in human mode emits one row on stdout and this warning on stderr:

    warning[WPK-W0002]: truncated output to 1 entries (use --max to increase limit)

Running the same query with `--max unlimited --json` emits all rows and this warning:

    {"kind":"warning","code":"WPK-W0001","message":"limit disabled: --max=unlimited"}

Running with `--max 0` fails before opening or evaluating the waveform, with empty stdout, exit code `1`, and stderr beginning with:

    fatal: args: --max must be greater than 0.

Running `wavepeek property ... --max 1 --jsonl` emits one `item`, one `diagnostic` with code `WPK-W0002`, and a final `end` record whose summary contains `"truncated":true`.

`wavepeek property --help` must show `--max <MAX>` in Output options and mention the `unlimited` literal. The docs topic `commands/property` must explain how to use the limit without duplicating the full generated flag table.

## Idempotence and Recovery

All code and documentation edits are plain text and safe to repeat. If a test fixture helper writes a temporary VCD, it uses `tempfile::NamedTempFile` and does not leave persistent artifacts. Use repository-root `tmp/` only for disposable logs if needed, and do not delete arbitrary files there. If an edit goes wrong, use `git diff` to inspect it and revert only the affected file or hunk. Do not bypass hooks; if a commit hook fails, fix the cause and commit again.

## Artifacts and Notes

Issue #38 request summary:

    Add a `--max` limit to `property`, aligned with existing bounded-output semantics used by `change`: default bounded output, `--max unlimited` warning `WPK-W0001`, `--max 0` args error, truncation warning `WPK-W0002`, JSONL `summary.truncated: true`, docs and tests.

Current property JSONL behavior before this change:

    src/engine/property.rs::run_jsonl writes diagnostics, then calls writer.end(false).

Change template from `change`:

    --max default: 50
    --max unlimited diagnostic: limit disabled: --max=unlimited
    truncation diagnostic: truncated output to N entries (use --max to increase limit)

## Interfaces and Dependencies

The implementation must use these existing interfaces:

`src/cli/limits.rs::LimitArg`:

    pub enum LimitArg {
        Numeric(usize),
        Unlimited,
    }

`src/diagnostic.rs::WarningDiagnosticCode`:

    LimitDisabled -> WPK-W0001
    OutputTruncated -> WPK-W0002
    EmptyResult -> WPK-W0003

`src/engine/property.rs::PropertyRunStats` must end with at least:

    struct PropertyRunStats {
        emitted: usize,
        truncated: bool,
    }

`src/engine/property.rs::run_with_sink` must accept `PropertyArgs`, enforce `args.max`, and return `PropertyCommandOutcome` containing diagnostics and stats. `src/engine/property.rs::run_jsonl` must call:

    writer.end(outcome.stats.truncated)

Revision note: Initial plan created on 2026-06-23 for issue #38 so implementation can proceed from a self-contained specification.
Revision note: Updated on 2026-06-23 after read-only plan review to mark the plan commit/review complete, mention internal `PropertyArgs` test literals, and document WIP-plan cleanup before merge.
Revision note: Updated on 2026-06-23 after implementation and focused tests to record completed code, docs, and test work before implementation review.
Revision note: Updated on 2026-06-23 after implementation review to record the clean code lane and docs wording fixes.
Revision note: Updated on 2026-06-23 after the final control review and `just check` gate passed.
Revision note: Updated on 2026-06-23 to record the rationale for retaining the branch-local WIP plan in the PR.
