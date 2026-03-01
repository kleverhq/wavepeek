# Make CLI Help Self-Descriptive and Uniform

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Today, users still need to read `docs/DESIGN.md` to understand critical command behavior, because command help text is uneven in depth and consistency. After this plan is implemented, every shipped command help page will stand on its own for day-to-day usage: it will explain command semantics, defaults, boundary rules, error categories, and output shape in one place.

This plan also standardizes help invocation ergonomics: `-h` and `--help` remain available everywhere, and both print the same detailed help output. A user should not have to remember which help flag gives "the real" contract. `wavepeek` with no arguments is treated as an implicit top-level help entry point and must match `wavepeek --help` byte-for-byte.

Observable outcome: running `wavepeek <command> -h` or `wavepeek <command> --help` yields identical, self-descriptive content, and integration tests enforce a reusable help-quality contract across all shipped commands.

## Non-Goals

This plan does not change runtime behavior of waveform commands (`info`, `scope`, `signal`, `at`, `change`, `when`, `schema`) beyond help rendering text/flag behavior.

This plan does not redesign output payload schemas or engine semantics.

This plan does not implement deferred runtime features (for example, `when` execution remains unimplemented; only help clarity changes).

This plan does not introduce command aliases or rename command surfaces.

## Progress

- [x] (2026-03-01 11:34Z) Mapped backlog acceptance criteria and identified all relevant code/docs/test anchors for CLI help behavior.
- [x] (2026-03-01 11:34Z) Confirmed current UX gap: top-level `-h/--help` are identical, but subcommands still differ (`-h` summary vs `--help` long help).
- [x] (2026-03-01 11:34Z) Drafted this execution plan with TDD-first milestones and explicit contract checks.
- [x] (2026-03-01 11:37Z) Incorporated review pass #1 feedback: embedded explicit per-command help contract, specified clap wiring strategy, and expanded command-by-command validation/closure mapping.
- [x] (2026-03-01 11:40Z) Incorporated independent review pass #2 feedback: locked one canonical clap implementation path, clarified no-args parity policy, and normalized error-guidance contract text.
- [x] (2026-03-01 13:07Z) Added new integration tests encoding no-args/top-level/subcommand help parity and command-level self-descriptive help markers (TDD-first: tests initially failed on legacy short-help behavior).
- [x] (2026-03-01 13:19Z) Implemented centralized clap builder wiring in `src/cli/mod.rs` so `-h` and `--help` both use long-help rendering and no-args invocation routes through synthetic `--help` parsing for byte-identical top-level output.
- [x] (2026-03-01 13:29Z) Expanded command `long_about` and flag docs across CLI modules to include semantics, defaults/requiredness, boundary rules, normalized error-guidance wording, and output-shape notes for all shipped commands.
- [x] (2026-03-01 13:34Z) Documented standalone-help principle in `docs/DEVELOPMENT.md` and `docs/DESIGN.md`; marked backlog item as completed and recorded unreleased changelog entries.
- [x] (2026-03-01 13:41Z) Ran targeted contract tests and full `cargo test` suite; all checks passed locally.
- [x] (2026-03-01 14:02Z) Ran repository quality gates `make check` and `make ci`; both completed successfully.
- [x] (2026-03-01 14:09Z) Completed mandatory review pass #1 (`review` subagent); no substantive issues reported.
- [x] (2026-03-01 14:13Z) Completed independent review pass #2 (`review` subagent); addressed one medium coverage finding by adding a guard test that keeps shipped-command help contract coverage synchronized with top-level command surface.
- [x] (2026-03-01 14:18Z) Re-ran `cargo test --test cli_contract`, `make check`, and `make ci` after review-driven fix; all checks stayed green.
- [x] (2026-03-01 14:31Z) Re-ran an additional fresh review after pass #2 fix; hardened the new command-surface parser against wrapped help-description lines and revalidated with `cargo test --test cli_contract`, `make check`, and `make ci`.
- [x] (2026-03-01 15:02Z) Applied follow-up wording pass per product feedback: removed inline JSON envelope field descriptions from command help, removed verbose per-command `error: args` hint sentences, and removed duplicated flag-default details already covered by argument docs.
- [x] (2026-03-01 15:07Z) Reworked command `Behavior` blocks to pull more non-duplicative semantics from `docs/DESIGN.md` (ordering guarantees, mode semantics, warnings, and deferred-runtime notes) while referring JSON contract readers to `wavepeek schema`.
- [x] (2026-03-01 15:14Z) Updated help-contract integration tests to assert the refined wording policy and reran `cargo test --test cli_contract`, `cargo test -q`, `make check`, and `make ci` successfully.
- [x] (2026-03-01 15:21Z) Removed inline JSON-envelope field details from waveform command `--json` argument help text (`src/cli/*.rs`) and added a negative guard test that fails if non-`schema` command help reintroduces inline envelope-field or parse-hint boilerplate.

## Surprises & Discoveries

- Observation: Backlog wording says `--help`/`--long-help`, but the current CLI does not expose a literal `--long-help` flag; it uses clap's built-in short-help (`-h`) versus long-help (`--help`) rendering behavior.
  Evidence: `src/cli/mod.rs` has no `--long-help` flag definition; runtime behavior shows `scope -h` summary and `scope --help` detailed output.

- Observation: The existing test suite verifies many isolated help fragments, but there is no single "all commands must be self-descriptive" contract test.
  Evidence: `tests/cli_contract.rs` contains focused assertions (for example unlimited literals, recursive flags) without a holistic rubric check.

- Observation: The project already has a natural home for the required design principle in `docs/DEVELOPMENT.md` under `CLI Design Constraints`.
  Evidence: `docs/DEVELOPMENT.md` section at line 201 defines cross-cutting CLI principles but currently lacks a standalone-help requirement.

- Observation: clap's `disable_help_flag` is a by-value builder API, so recursive mutation requires command ownership handoff rather than in-place field mutation.
  Evidence: compile error `E0507` when calling `command.disable_help_flag(true)` via `&mut clap::Command`; resolved by `std::mem::take(command)` + `mut_subcommands(...)` recursive rebuild path.

- Observation: custom global help arg without explicit `.help(...)` text renders a blank `-h, --help` description line.
  Evidence: `cargo run --quiet -- change --help` initially showed an empty help description row; adding `.help("Print help")` restored expected output clarity.

- Observation: static shipped-command lists in tests can drift silently as new subcommands are added.
  Evidence: independent review pass #2 flagged that parity checks iterated a hard-coded list; mitigated by adding a guard test that asserts `SHIPPED_COMMANDS` matches names listed in top-level `--help`.

- Observation: parsing top-level help command names by "first token on each non-empty line" is brittle because clap may wrap long command descriptions.
  Evidence: fresh post-fix review reported false-positive risk from wrapped continuation lines; mitigated by parsing only command rows with exact command-column indentation and stopping at `Options:` section.

- Observation: detailed per-command repeats of defaults and parse-hint boilerplate made help text noisy and harder to scan, even though the same facts were already present in argument-level docs and normalized error output.
  Evidence: follow-up product feedback requested removing duplicated defaults/requiredness and `error: args` hint boilerplate from command bodies, while keeping behavior-rich semantics.

- Observation: even after long-about cleanup, inline JSON envelope field details still leaked through `--json` argument doc comments.
  Evidence: new negative policy test failed on `info --help` until `--json` argument docs were updated to schema references.

## Decision Log

- Decision: Treat `-h` and `--help` as equivalent detailed-help entry points for all command scopes (top-level and subcommands).
  Rationale: This matches user expectation that any help invocation should provide full actionable guidance while preserving familiar flags.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Do not add a literal `--long-help` surface in this milestone.
  Rationale: The backlog intent is richer help content; introducing a new user-facing flag adds surface complexity without improving discoverability once `-h` and `--help` are unified.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Enforce help quality through integration tests in `tests/cli_contract.rs` (or a sibling integration test file) instead of adding a separate external script.
  Rationale: The repository already uses integration tests as CLI contract gates via `cargo test` and `make ci`; this keeps quality checks in the existing CI path.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Keep the recursive helper signature `fn disable_default_help_flags_recursively(command: &mut clap::Command)` and implement it via ownership round-trip (`std::mem::take` + `mut_subcommands`) to satisfy clap's by-value builder API.
  Rationale: This preserves the canonical function surface captured in this plan while using a compile-safe implementation that applies uniformly to every command node.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Prefer deterministic fragment assertions for help-quality tests instead of full-output snapshots.
  Rationale: Fragment checks enforce the required contract categories while minimizing brittleness from clap line-wrap formatting changes.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Retain explicit `SHIPPED_COMMANDS` test targeting, but add a synchronization guard against top-level help command discovery.
  Rationale: Keeping explicit command intent in tests preserves readability, while the guard test prevents silent drift when command surface changes.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Keep the guard parser intentionally minimal but robust by filtering to exact command-entry indentation and ignoring wrapped continuation rows.
  Rationale: This avoids clap wrapping noise while keeping the test implementation simple and deterministic.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Standardize command help body style around non-duplicative `Behavior` bullets: include semantics and runtime behavior from `docs/DESIGN.md`, avoid restating argument-table defaults/requiredness, and refer all JSON-shape specifics to `wavepeek schema`.
  Rationale: This keeps help self-descriptive for day-to-day use without repeating lower-level argument details or embedding schema field lists in every command.
  Date/Author: 2026-03-01 / OpenCode

- Decision: Add explicit negative style-policy assertions in integration tests for non-`schema` commands (no inline envelope field names, no repetitive `See 'wavepeek ... --help'.` boilerplate in long help).
  Rationale: Positive marker checks alone can miss style regressions; negative guards keep the new wording policy stable over future edits.
  Date/Author: 2026-03-01 / OpenCode

## Outcomes & Retrospective

Implementation outcome: all shipped commands now provide uniform detailed help for both `-h` and `--help`, and top-level no-args output is byte-identical to explicit `--help`.

The CLI help contract is now enforced by dedicated integration tests that cover parity and command-level self-descriptive markers. Documentation collateral (`docs/DEVELOPMENT.md`, `docs/DESIGN.md`, `docs/BACKLOG.md`, `CHANGELOG.md`) has been aligned so standalone help is treated as a core CLI design principle.

Retrospective: centralizing clap help wiring in one builder path removed divergent render paths and made parity properties easier to guarantee. Fragment-based help assertions provided stable quality checks while still capturing behavior-rich requirements. Independent reviewer feedback improved long-term reliability by adding explicit test coverage against command-surface drift, and follow-up product feedback refined help style toward behavior-first guidance with schema references instead of duplicated argument/envelope details.

## Context and Orientation

`wavepeek` CLI parsing and help text are centered in `src/cli/mod.rs`. Top-level command descriptions (`about` and `long_about`) live directly on the `Command` enum variants there. Flag-level help text is mostly sourced from doc comments in per-command args files: `src/cli/info.rs`, `src/cli/scope.rs`, `src/cli/signal.rs`, `src/cli/at.rs`, `src/cli/change.rs`, and `src/cli/when.rs`.

Help rendering is now unified through one clap builder path in `src/cli/mod.rs`, so `-h` and `--help` both render detailed help for top-level and shipped subcommands, and no-args invocation routes through the same render path as explicit `--help`. Parse errors and hints are normalized in `src/cli/mod.rs` (`normalize_clap_error`, `help_hint_for_rendered_clap_error`) and should stay stable.

Current contract tests are in `tests/cli_contract.rs`. They now include deterministic parity checks and a formal help-quality rubric for each shipped command. "Help-quality contract" in this plan means: each command help must include (1) command intent, (2) behavior semantics that matter at runtime (ordering, mode differences, warning/error cases, or implementation status), and (3) guidance that `--json` uses the schema contract exposed by `wavepeek schema`, without repeating JSON field-level envelope details in every command.

The required self-descriptive help contract for this plan is embedded here so implementation does not depend on external documents:

Normalized help-content rule for this milestone:

- For waveform commands (`info`, `scope`, `signal`, `at`, `change`, `when`), include behavior-oriented semantics and runtime notes; keep JSON guidance at the level of "contract is defined by `wavepeek schema`" rather than listing envelope field payloads.
- Do not restate defaults/requiredness already shown in argument help text unless a behavior cannot be understood without it.
- Keep global error-shape guidance (`error: <category>: <message>`) where useful, but avoid repetitive per-command parse-hint boilerplate.
- `info`: report metadata semantics and role as dump-bounds primer.
- `scope`: include deterministic traversal ordering, scope-kind coverage, tree/list mode behavior, and warning/error behavior for regex/limits.
- `signal`: include direct-vs-recursive semantics, deterministic recursive ordering, display-mode behavior (`--abs`), and regex/limit behavior notes.
- `at`: include name-resolution rules, ordering guarantees, time-alignment semantics, literal formatting, and fail-fast behavior.
- `change`: include inclusive range/baseline semantics, trigger sourcing, delta-row emission rules, empty-result warning behavior, and deferred `iff` runtime note.
- `when`: include intended evaluation semantics, qualifier behavior, parse-accepted `--max unlimited`, and explicit unimplemented runtime status.
- `schema`: include deterministic single-document semantics and source-of-truth status for all `--json` outputs.

This contract reuses wording intent from `docs/DESIGN.md` for alignment, but the implementation should be executable from this plan alone.

## Open Questions

No blocking questions remain for implementation. The one ambiguity from backlog wording (`--long-help`) is resolved by this plan via unified detailed output for existing `-h` and `--help` flags.

## Plan of Work

Milestone 1 locks expected behavior through failing tests before code changes (TDD). Extend CLI integration tests so they fail on current behavior for subcommand `-h` versus `--help` mismatch, and define reusable help-quality checks for all shipped commands. This milestone is complete when tests fail only because the code has not been updated yet, not because of unclear assertions.

Milestone 2 updates clap integration so help flags are ergonomically uniform. Keep both `-h` and `--help` available, but wire both to long-form help output. Implement this through one canonical builder path in `src/cli/mod.rs`: construct a command from `Cli::command()`, disable default help flags on every command node (root and all subcommands), add one global help arg (`-h` and `--help`) with `clap::ArgAction::HelpLong`, parse argv through this builder, and then construct `Cli` from matches for dispatch. Do not use mixed derive-plus-builder alternatives in this milestone. Preserve no-args behavior by routing no-args invocation through the same parser/render path as `--help` (synthetic `--help` argv), so byte-parity is guaranteed by construction. Keep version flag behavior unchanged.

Milestone 3 upgrades help content itself. Rework command `long_about` text and option-level docs to include semantics, boundary/range and mode behavior, warning/error notes, and schema-reference guidance using adapted normative language from `docs/DESIGN.md`. Keep wording concise and avoid duplicating defaults/requiredness that are already explicit in argument help text.

Milestone 4 aligns project documentation and closes the backlog acceptance loop. Add the standalone-help principle to the CLI design principles section in `docs/DEVELOPMENT.md` (and `docs/DESIGN.md` general conventions where appropriate), then update backlog/changelog entries to reflect completion once tests pass.

## Concrete Steps

Run all commands from `/workspaces/docs-verbose-help`.

1. Add failing tests first (new or expanded integration tests).

       cargo test --test cli_contract no_args_help_matches_long_help_output -- --exact
       cargo test --test cli_contract top_level_short_and_long_help_are_identical -- --exact
       cargo test --test cli_contract short_and_long_help_are_identical_for_shipped_commands -- --exact
       cargo test --test cli_contract subcommand_short_help_includes_long_help_contract_markers -- --exact
       cargo test --test cli_contract shipped_commands_help_is_self_descriptive -- --exact

   Expected before implementation: the shipped-command parity test fails on subcommands because `-h` currently prints summary help.

2. Implement unified help-flag behavior in `src/cli/mod.rs`.

   Implementation details for this step:

   - Add `fn build_cli_command() -> clap::Command` that starts from `Cli::command()`.
   - Add `fn disable_default_help_flags_recursively(command: &mut clap::Command)` and call it from `build_cli_command()` so root and all subcommands disable clap's built-in help flags before custom help wiring.
   - In `build_cli_command()`, add exactly one global help arg with `short = 'h'`, `long = "help"`, and `clap::ArgAction::HelpLong`.
   - Parse `std::env::args_os()` via `build_cli_command().try_get_matches_from(...)`, then construct `Cli` from matches (`Cli::from_arg_matches(...)`) for normal dispatch.
   - Replace separate no-args print path with synthetic `--help` argv routed through `build_cli_command()` so no-args output is rendered by the same long-help path as explicit `--help`.
   - Keep `disable_help_subcommand = true` unchanged.
   - Do not change version-flag behavior (`-V`, `--version`) or parse-error hint functions (`normalize_clap_error`, `help_hint_for_rendered_clap_error`).

       cargo test --test cli_contract short_and_long_help_are_identical_for_shipped_commands -- --exact
       cargo test --test cli_contract top_level_short_and_long_help_are_identical -- --exact
       cargo test --test cli_contract no_args_help_matches_long_help_output -- --exact
       cargo test --test cli_contract subcommand_short_help_includes_long_help_contract_markers -- --exact

   Expected after step 2: parity test passes for top-level and every shipped subcommand.

3. Expand and normalize help text across CLI modules (`src/cli/mod.rs` plus per-command args files).

       cargo test --test cli_contract shipped_commands_help_is_self_descriptive -- --exact

   Expected after step 3: self-descriptive help contract test passes for all shipped commands.

4. Update docs and backlog/changelog collateral.

       cargo test --test cli_contract unknown_top_level_flag_uses_global_help_hint -- --exact
       cargo test --test cli_contract unknown_flags_are_normalized_to_args_category -- --exact
       cargo test --test cli_contract waveform_commands_require_waves_flag -- --exact
       cargo test --test cli_contract
       cargo test -q

5. Run repository quality gates.

       make check
       make ci

   If running outside the required container, run inside devcontainer/CI image per repository policy, then rerun the same commands.

## Validation and Acceptance

Behavioral acceptance criteria:

- `wavepeek -h` output is byte-identical to `wavepeek --help`.
- `wavepeek` (no args) output is byte-identical to `wavepeek --help`.
- For each shipped subcommand (`info`, `scope`, `signal`, `at`, `change`, `when`, `schema`), `wavepeek <cmd> -h` output is byte-identical to `wavepeek <cmd> --help`.
- Help for each shipped command includes command semantics, default/requiredness guidance, boundary constraints, and output-shape notes.
- Error guidance remains stable: parse/runtime error categories continue to use `error: <category>:` format, and help hints still direct users to the correct command help entry.
- CI contract gate remains green via existing test path (`cargo test`, `make ci`).

Human verification transcript targets:

    wavepeek
    wavepeek -h
    wavepeek --help
    wavepeek info -h
    wavepeek info --help
    wavepeek scope -h
    wavepeek scope --help
    wavepeek signal -h
    wavepeek signal --help
    wavepeek at -h
    wavepeek at --help
    wavepeek change -h
    wavepeek change --help
    wavepeek when -h
    wavepeek when --help
    wavepeek schema -h
    wavepeek schema --help

For each pair above, outputs should be byte-identical and include the command's required contract fragments listed in `Context and Orientation`.

Representative expected evidence snippets during implementation:

    running 1 test
    test short_and_long_help_are_identical_for_shipped_commands ... FAILED

    running 1 test
    test short_and_long_help_are_identical_for_shipped_commands ... ok

    running 1 test
    test subcommand_short_help_includes_long_help_contract_markers ... ok

    error: args: unexpected argument '--wat' found See 'wavepeek info --help'.

Backlog closure mapping for `docs/BACKLOG.md` item `CLI help should be self-descriptive`:

- Backlog bullet "Improve `--help`/`--long-help`...": satisfied by parity tests (`top_level_short_and_long_help_are_identical`, `short_and_long_help_are_identical_for_shipped_commands`) plus expanded self-descriptive assertions.
- Backlog bullet "Reuse and adapt normative wording...": satisfied by help text updates in `src/cli/mod.rs` and per-command args docs, with wording aligned to the embedded contract in this plan.
- Backlog bullet "Add ... foundational CLI design principle": satisfied by doc update in `docs/DEVELOPMENT.md` (`CLI Design Constraints`) and corresponding `docs/DESIGN.md` general-conventions reinforcement.
- Backlog bullet "Close when all shipped commands pass a help-quality contract check": satisfied by green targeted self-descriptive integration tests for every shipped command in CI.

## Idempotence and Recovery

All test and documentation steps are repeatable. Re-running `cargo test` and `make` targets is safe and expected.

If clap help customization unexpectedly alters parse-error behavior, revert only the help-flag wiring commit and keep failing contract tests in place, then reintroduce help unification with a narrower approach.

If self-descriptive checks prove too brittle due to strict whole-string matching, switch to deterministic fragment assertions grouped by contract category (semantics/defaults/boundaries/errors/output-shape) while preserving broad coverage for every command.

## Artifacts and Notes

Primary files expected to change:

    src/cli/mod.rs
    src/cli/info.rs
    src/cli/scope.rs
    src/cli/signal.rs
    src/cli/at.rs
    src/cli/change.rs
    src/cli/when.rs
    tests/cli_contract.rs
    docs/DEVELOPMENT.md
    docs/DESIGN.md
    docs/BACKLOG.md
    CHANGELOG.md

Optional structure refinement: if `tests/cli_contract.rs` becomes too large, split help-quality assertions into `tests/cli_help_contract.rs` while keeping shared helpers under `tests/common/`.

## Interfaces and Dependencies

This plan keeps clap as the only CLI parser dependency (`clap` derive path already in use). The implementation should prefer declarative clap configuration over manual argument rewriting.

At completion, the CLI interface should preserve these invariants:

- Both `-h` and `--help` remain accepted at top-level and subcommand scope.
- Both help flags produce the same detailed output.
- Existing parse-error normalization helpers in `src/cli/mod.rs` continue to produce stable `error: args:` diagnostics and context-correct "See 'wavepeek ... --help'." hints.

Implement clap help-action wiring centrally in `src/cli/mod.rs` so command modules remain focused on semantic text, not parser mechanics.

Canonical clap wiring endpoint in `src/cli/mod.rs`:

    fn disable_default_help_flags_recursively(command: &mut clap::Command) {
        // Apply clap's disable-help-flag setting to this node.
        // Then recurse into every child subcommand and apply the same setting.
    }

    fn build_cli_command() -> clap::Command {
        let mut command = Cli::command();
        disable_default_help_flags_recursively(&mut command);
        command.arg(
            clap::Arg::new("help")
                .short('h')
                .long("help")
                .global(true)
                .action(clap::ArgAction::HelpLong),
        )
    }

Use this builder in parse path and in synthetic no-args `--help` path so all help entry points share one rendering contract.

Revision Note: 2026-03-01 / OpenCode - Initial ExecPlan created for backlog issue `CLI help should be self-descriptive`, incorporating requirement to keep `-h` and `--help` both available while making their output identical and self-descriptive.
Revision Note: 2026-03-01 / OpenCode - Incorporated review-pass #1 feedback by embedding per-command help contract requirements directly in this plan, specifying the clap help-flag wiring strategy, expanding per-command human verification steps, and adding a backlog-closure mapping.
Revision Note: 2026-03-01 / OpenCode - Incorporated independent review-pass #2 feedback by selecting a single builder-based clap wiring path, requiring `wavepeek` no-args parity with `--help`, and standardizing error-guidance expectations in help contracts.
Revision Note: 2026-03-01 / OpenCode - Completed implementation milestones, recorded clap builder implementation details discovered during coding, and updated outcomes with final validation/documentation closure evidence.
Revision Note: 2026-03-01 / OpenCode - Added post-review progress/results (both review passes), documented reviewer-driven test hardening for command-surface drift, and refreshed retrospective accordingly.
Revision Note: 2026-03-01 / OpenCode - Applied fresh post-fix review hardening for wrapped help-line parsing in the shipped-command guard test and recorded final re-validation evidence.
Revision Note: 2026-03-01 / OpenCode - Applied follow-up product feedback to shift command help text to behavior-first wording: removed duplicated argument defaults, removed per-command parse-hint boilerplate, and replaced inline JSON-envelope details with references to `wavepeek schema`.
Revision Note: 2026-03-01 / OpenCode - Extended follow-up with arg-level `--json` doc cleanup and negative integration guards to prevent reintroduction of inline envelope and parse-hint boilerplate text.
