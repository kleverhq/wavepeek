# Deliver Hard-Break Migration to `property --on` and `change --on`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, waveform property checks use one assertion-oriented command surface: `wavepeek property --on "<event_expr>" --eval "<logical_expr>"`. The legacy `when` command is removed, and `change` uses `--on` instead of `--when` for event triggers. This is an intentional hard break: no compatibility aliases are kept.

Users can verify the outcome by checking that top-level help lists `property` (and not `when`), that `change --on` preserves existing trigger semantics, that legacy `--when` is rejected for both commands, and that `property --capture=match|switch|assert|deassert` produces deterministic human and JSON outputs.

## Non-Goals

This plan does not introduce compatibility aliases for `when` command or `--when` flags. This plan does not add temporal operators beyond the current MVP boolean expression scope. This plan does not add JSON streaming (`--jsonl`) or redesign unrelated commands. This plan does not change existing `change` runtime semantics beyond trigger-flag naming and related diagnostics.

## Progress

- [x] (2026-03-05 08:55Z) Reviewed backlog requirements in `docs/BACKLOG.md` and captured user override: use `--on` (not `--when`) for `property` and `change` with hard-break migration policy.
- [x] (2026-03-05 08:58Z) Mapped current implementation and contract gaps across `src/cli`, `src/engine`, `src/expr`, `tests/`, `docs/`, and `schema/`.
- [x] (2026-03-05 09:05Z) Updated backlog wording so planned scope explicitly targets `property --on`, `change --on`, and no compatibility aliases.
- [x] (2026-03-05 09:18Z) Authored this executable plan with TDD-first milestones, validation gates, commit boundaries, and mandatory double-review workflow.
- [x] (2026-03-05 09:42Z) Completed review pass #1 for this planning diff; resolved findings on default omitted-`--on` semantics, baseline determinism, and self-contained verification details.
- [x] (2026-03-05 09:56Z) Completed independent review pass #2 in fresh context; aligned `match` baseline semantics with transition modes and finalized backlog closure wording for `change --on`.
- [ ] Implement TDD red phase for CLI contract migration (`property` surface, `--on`, hard-break rejection of legacy `when`/`--when`).
- [ ] Implement `change --on` rename end-to-end in CLI, parser diagnostics, engine wiring, and tests.
- [ ] Implement `property` command runtime path and `--capture` modes, replacing `when` surface end-to-end.
- [ ] Complete expression parser/evaluator runtime needed for `property --eval` execution and deterministic capture decisions.
- [ ] Align JSON schema, docs, changelog, and backlog closure with shipped behavior.
- [ ] Run final gates (`make check`, `make ci`) after implementation and after last review-fix commit.
- [ ] Complete mandatory review pass #1 (`review` agent), apply fixes, and commit.
- [ ] Complete mandatory independent review pass #2 in fresh context, apply fixes if needed, and commit.

## Surprises & Discoveries

- Observation: `when` command exists only as CLI surface; runtime is still unimplemented.
  Evidence: `src/engine/when.rs` returns `WavepeekError::Unimplemented("`when` command execution is not implemented yet")`.

- Observation: Logical-expression runtime for property-like evaluation does not exist yet.
  Evidence: `src/expr/eval.rs` returns `WavepeekError::Unimplemented("expression evaluation is not implemented yet")`, and `src/expr/parser.rs` currently stores raw `--cond` source text instead of a parsed AST.

- Observation: Event-expression parser and `change` diagnostics are tightly coupled to `--when` wording today.
  Evidence: `src/expr/parser.rs` emits `invalid --when expression` and `src/engine/change.rs` validates event-name failures with `invalid --when expression: ...`.

- Observation: Existing integration coverage heavily locks current `change --when` contracts.
  Evidence: `tests/change_cli.rs`, `tests/change_opt_equivalence.rs`, and `tests/change_vcd_fst_parity.rs` contain many explicit `--when` invocations and message assertions.

## Decision Log

- Decision: Treat migration as a hard break with no compatibility options.
  Rationale: User explicitly requested no compatibility aliases; repository precedent already enforces alias-free renames (`tree`, `modules`, `signals`, `changes`, `at`).
  Date/Author: 2026-03-05 / OpenCode

- Decision: Standardize event-trigger flag spelling on `--on` for both `property` and `change`.
  Rationale: One trigger keyword reduces cognitive load and avoids command-specific dialect drift.
  Date/Author: 2026-03-05 / OpenCode

- Decision: Implement `property` by replacing `when` command surface and runtime modules, not by adding a parallel alias.
  Rationale: Hard-break policy and long-term maintainability both favor one canonical command name.
  Date/Author: 2026-03-05 / OpenCode

- Decision: Keep `--capture=switch` as default and preserve backlog semantics exactly (`match`, `switch`, `assert`, `deassert`).
  Rationale: Backlog defines default signal-to-noise target and deterministic semantics for each mode.
  Date/Author: 2026-03-05 / OpenCode

- Decision: When `--on` is omitted for `property`, treat wildcard `*` as "any change among signals referenced by `--eval`"; if `--eval` references no signals, fail with `error: args:` and require explicit `--on`.
  Rationale: This preserves backlog intent (event-driven default tied to evaluated signals) and avoids accidental global wildcard scans on constant expressions.
  Date/Author: 2026-03-05 / OpenCode

- Decision: For all capture modes, process candidate timestamps strictly after baseline checkpoint (`--from` or dump start). For transition modes (`switch`/`assert`/`deassert`), initialize previous boolean state at baseline and cast unknown to false.
  Rationale: Explicit baseline rules keep `match` and transition modes deterministic and prevent first-sample ambiguity across fixtures and engines.
  Date/Author: 2026-03-05 / OpenCode

- Decision: Keep staged `iff` runtime behavior unchanged in this migration (still explicit deferred-runtime error), but rename user-facing flag references to `--on`.
  Rationale: This plan focuses on command/flag migration and property delivery; full `iff` execution remains separately tracked debt.
  Date/Author: 2026-03-05 / OpenCode

## Outcomes & Retrospective

Current status: planning complete; implementation not started.

Expected completion outcome: users will have one event-driven property command (`property`) with deterministic capture modes and one consistent trigger flag (`--on`) across `property` and `change`, while legacy `when`/`--when` syntax is rejected deterministically.

Primary delivery risk: the migration touches parser diagnostics, CLI help contracts, schema discriminators, and many integration tests at once. This is mitigated by TDD-first sequencing, small atomic commits, and mandatory two-pass review.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI. Command parsing lives under `src/cli/`, runtime logic under `src/engine/`, expression parsing/evaluation under `src/expr/`, and output envelopes in `src/output.rs`. JSON schema is tracked as a canonical artifact at `schema/wavepeek.json`.

In this repository, an "event expression" means the trigger language already used by `change` (`*`, named signal, `posedge`/`negedge`/`edge`, unions with `or`/`,`). A "capture mode" means how property outcomes are emitted per candidate event timestamp:

- `match`: emit each candidate event where `--eval` is true.
- `switch`: emit state transitions (`assert` for `0->1`, `deassert` for `1->0`).
- `assert`: emit only `0->1` transitions.
- `deassert`: emit only `1->0` transitions.

Additional terms used in this plan:

- "TDD red phase" means writing/updating tests first and proving at least one targeted test fails before implementation.
- "Discriminator mapping" means JSON schema and runtime binding between envelope `command` value and the required `data` shape.
- "Parity suite" means tests that assert behavior equivalence after refactor/rename (here: `change --on` must match previous `--when` semantics).

Current state relevant to this work:

- `src/cli/when.rs` and `src/engine/when.rs` define a planned-but-unimplemented command path.
- `src/cli/change.rs`, `src/engine/change.rs`, and `src/expr/parser.rs` are currently wired around `--when` wording.
- `src/engine/mod.rs` and `src/output.rs` have no `property` command/data variants yet.
- `schema/wavepeek.json` command enum currently includes `info|scope|signal|value|change` only.
- `tests/cli_contract.rs` currently expects top-level `when` and `change --when` help markers.
- `docs/DESIGN.md`, `docs/ROADMAP.md`, `README.md`, and `CHANGELOG.md` contain `when`/`--when` references that must be reconciled with the hard break once implementation lands.

Primary files in scope:

- `src/cli/mod.rs`
- `src/cli/change.rs`
- `src/cli/when.rs` (rename target: `src/cli/property.rs`)
- `src/engine/mod.rs`
- `src/engine/change.rs`
- `src/engine/when.rs` (rename target: `src/engine/property.rs`)
- `src/expr/mod.rs`
- `src/expr/parser.rs`
- `src/expr/eval.rs`
- `src/output.rs`
- `schema/wavepeek.json`
- `tests/cli_contract.rs`
- `tests/change_cli.rs`
- `tests/change_opt_equivalence.rs`
- `tests/change_vcd_fst_parity.rs`
- `tests/property_cli.rs` (new)
- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `README.md`
- `CHANGELOG.md`
- `docs/BACKLOG.md`

## Open Questions

No blocking questions remain. Scope, naming, and migration policy are explicit and can be implemented directly.

## Plan of Work

Milestone 1 is contract-first TDD red phase. Update integration tests to assert the future surface (`property`, `--on`, hard-break rejection of legacy syntax) before runtime changes are applied, then run targeted test commands and capture failing evidence.

Milestone 2 migrates `change` trigger naming from `--when` to `--on` without changing event semantics. This includes clap flags, help text, parser diagnostics, engine wiring, and all `change`-related integration suites.

Milestone 3 replaces the `when` command path with `property` and implements runtime behavior required by backlog scope: event-driven candidate timestamps via `--on`, boolean evaluation via `--eval`, and deterministic capture semantics via `--capture` modes, including explicit baseline rules.

Milestone 4 aligns schema and collateral. Update JSON schema discriminator/data definitions for `property`, update docs/changelog to reflect hard break, and close completed backlog items once implementation is validated.

Milestone 5 enforces quality and review closure. Run repository quality gates, execute mandatory review pass #1, apply fixes, then run an independent fresh review pass #2 and resolve remaining findings.

### Concrete Steps

Run all commands from `/workspaces/fix-rename-when`.

1. TDD red phase for CLI migration and property contract tests.

   - In `tests/cli_contract.rs`, replace `when` expectations with `property`, replace `change --when` help fragments with `change --on`, and add explicit hard-break checks:
     - `wavepeek when ...` fails as unrecognized subcommand.
     - `wavepeek change --when ...` fails as unexpected argument with `See 'wavepeek change --help'.`
     - `wavepeek property --when ...` fails as unexpected argument with `See 'wavepeek property --help'.`
   - Update `tests/change_cli.rs`, `tests/change_opt_equivalence.rs`, and `tests/change_vcd_fst_parity.rs` invocations from `--when` to `--on` while keeping semantic expectations unchanged.
   - Add `tests/property_cli.rs` with parser/help/runtime contract tests for `--on`, `--eval`, and `--capture` modes.
    - In `tests/property_cli.rs`, add explicit defaults tests:
      - omitted `--on` with signal-bearing `--eval` tracks changes among `--eval` signals;
      - omitted `--on` with signal-free `--eval` (for example `"1"`) fails with deterministic `error: args:` guidance.
    - In `tests/property_cli.rs`, add baseline tests for all capture modes at range baseline (`--from`):
      - `match` confirms candidate evaluation starts strictly after baseline;
      - `switch|assert|deassert` confirm previous-state initialization at baseline;
      - include unknown (`x`/`z`) casting checks.

   Red-phase runs (expect failure before implementation):

       cargo test --test cli_contract help_lists_expected_subcommands -- --exact
       cargo test --test change_cli change_omitted_on_matches_explicit_wildcard -- --exact
       cargo test --test property_cli property_default_capture_is_switch -- --exact

   Expected red-phase signatures before implementation:

       error: args: unrecognized subcommand 'property'
       ... FAILED

2. Implement `change --on` rename plumbing.

   - In `src/cli/change.rs`, rename field `when` to `on` and clap long flag to `--on`.
   - In `src/cli/mod.rs`, update `change` long help wording from `--when` to `--on`.
   - In `src/engine/change.rs`, consume `args.on` (default `*`) and rename all user-facing diagnostics from `--when` to `--on`.
   - In `src/expr/parser.rs` and `src/expr/mod.rs`, update event-expression parse error wording and helper hints so both `change` and `property` can reuse the same parser without stale `--when` phrasing.

   Re-run change-focused tests:

       cargo test --test change_cli
       cargo test --test change_opt_equivalence
       cargo test --test change_vcd_fst_parity

3. Implement `property` command and runtime semantics.

   - Rename modules with history preservation:
     - `src/cli/when.rs` -> `src/cli/property.rs`
     - `src/engine/when.rs` -> `src/engine/property.rs`
    - Replace `WhenArgs` with `PropertyArgs` in `src/cli/property.rs` and wire subcommand rename in `src/cli/mod.rs`.
    - In `src/engine/mod.rs`, replace `Command::When` path with `Command::Property`, add `CommandName::Property`, and add `CommandData::Property` payload type.
    - Implement `src/engine/property.rs` end-to-end:
      - resolve requested scope and time window deterministically;
      - parse event expression from `--on`; when omitted, synthesize tracked wildcard from `--eval` signal references;
      - parse/evaluate logical expression from `--eval` at candidate timestamps;
      - initialize previous evaluation state at baseline (`--from` or dump start), cast unknown to false, then apply `--capture` semantics and emit deterministic records for candidate timestamps strictly greater than baseline.
    - Implement expression runtime support needed by property in `src/expr/parser.rs`, `src/expr/mod.rs`, and `src/expr/eval.rs` (replace current placeholder-only path with executable parser/evaluator compatible with current MVP boolean operators).
    - Update `src/output.rs` human/JSON rendering for `CommandData::Property`.

   Property-focused tests:

       cargo test --test property_cli
       cargo test --test cli_contract

4. Schema and collateral alignment.

   - Update `schema/wavepeek.json`:
     - add `"property"` command discriminator;
     - add `$defs.propertyData` (or equivalent canonical name) with deterministic shape for capture records;
     - ensure `allOf` discriminator mapping is complete.
   - Run schema regeneration/verification targets as needed.
   - Update live contracts and migration notes in `docs/DESIGN.md`, `docs/ROADMAP.md`, `README.md`, and `CHANGELOG.md`.
   - Update `docs/BACKLOG.md` by removing or closing completed items after implementation acceptance.

   Run:

       make update-schema
       make check-schema

   Expected schema-check signature:

       ... check_schema_contract.py
       (no error output, exit code 0)

5. Commit atomic units.

   Recommended commit split:

       git add -A tests/cli_contract.rs tests/change_cli.rs tests/change_opt_equivalence.rs tests/change_vcd_fst_parity.rs tests/property_cli.rs
       git commit -m "test(cli)!: lock property and --on migration contracts"

       git add -A src/cli/change.rs src/cli/mod.rs src/engine/change.rs src/expr/mod.rs src/expr/parser.rs
       git commit -m "refactor(change)!: rename trigger flag from --when to --on"

       git add -A src/cli/property.rs src/engine/property.rs src/engine/mod.rs src/expr/eval.rs src/output.rs
       git commit -m "feat(property)!: replace when command with property capture modes"

       git add -A schema/wavepeek.json docs/DESIGN.md docs/ROADMAP.md README.md CHANGELOG.md docs/BACKLOG.md
       git commit -m "docs(schema)!: align contracts with property and --on"

6. Full validation gates.

       make check
       make ci

   Expected gate signature:

       cargo fmt -- --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test -q
       ... completed successfully

7. Mandatory review pass #1, then independent pass #2.

   - Load `ask-review` skill.
   - Run review pass #1 on the full migration diff; fix valid findings in new commit(s).
   - Run a fresh review pass #2 in a new reviewer session; fix findings in new commit(s) and re-run pass #2 until clean.

8. Final post-review gate and plan lifecycle closure.

       make check
       make ci

   - Record final evidence in this plan.
   - Move plan directory from `docs/exec-plans/active/` to `docs/exec-plans/completed/` when fully done.

### Validation and Acceptance

Acceptance is complete only when all outcomes below are simultaneously true:

- `wavepeek --help` lists `property` and does not list `when`.
- `wavepeek property --waves tests/fixtures/hand/m2_core.vcd --scope top --on "posedge clk" --eval "data == 0x0f"` executes and emits deterministic records in human format `@<time> <state>` (for example `@10ns assert`) and equivalent JSON records.
- Omitted `--on` with signal-bearing `--eval` uses tracked-signal wildcard semantics; omitted `--on` with signal-free `--eval` fails deterministically with `error: args:`.
- All capture modes evaluate candidate timestamps strictly greater than baseline (`--from` or dump start): `--capture=match` emits true evaluations at those candidates, `--capture=switch` emits transition records (`assert`/`deassert`), and `--capture=assert|deassert` emit only their respective edges.
- Transition-mode first-sample behavior is deterministic: baseline state is taken at `--from` (or dump start), unknown values cast to false, and transition decisions use that initialized baseline state.
- `wavepeek change --on ...` preserves current event semantics and output parity (excluding renamed flag wording).
- Legacy syntax is rejected deterministically: `wavepeek when ...`, `wavepeek change --when ...`, and `wavepeek property --when ...` all fail with stable `error: args:` guidance.
- JSON schema and runtime output agree (`make check-schema` passes), including `command: "property"`, valid property data shape, and no legacy `when` discriminator.
- `make check` and `make ci` pass after final review fixes.
- Review pass #1 and independent review pass #2 are clean, or all findings are resolved and rechecked.

TDD acceptance requirement: at least one newly introduced migration/property test fails before implementation and passes after implementation in the same branch history.

### Idempotence and Recovery

All planned steps are file edits, test runs, and schema regeneration; they are safe to re-run. If migration work temporarily breaks compilation (common during command/module renames), recover by completing `src/cli/mod.rs` and `src/engine/mod.rs` wiring first, then re-run targeted tests.

If parser/evaluator rollout exposes semantic mismatches, keep `property` tests as the contract source of truth, fix evaluator behavior, and re-run targeted suites before broad gates. If review finds regressions, apply follow-up commits; do not rewrite history.

### Artifacts and Notes

Expected modified and new artifacts:

- `src/cli/change.rs`
- `src/cli/mod.rs`
- `src/cli/when.rs` -> `src/cli/property.rs`
- `src/engine/change.rs`
- `src/engine/mod.rs`
- `src/engine/when.rs` -> `src/engine/property.rs`
- `src/expr/mod.rs`
- `src/expr/parser.rs`
- `src/expr/eval.rs`
- `src/output.rs`
- `schema/wavepeek.json`
- `tests/cli_contract.rs`
- `tests/change_cli.rs`
- `tests/change_opt_equivalence.rs`
- `tests/change_vcd_fst_parity.rs`
- `tests/property_cli.rs` (new)
- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `README.md`
- `CHANGELOG.md`
- `docs/BACKLOG.md`

Before closure, record concise evidence snippets in this section:

- Red-phase failing test excerpt (at least one explicit `FAILED` signature).
- Green-phase passing excerpt for `property_cli` and `change` parity suites.
- `make ci` success excerpt.
- Review pass #1 and independent pass #2 outcomes.

### Interfaces and Dependencies

No new external dependencies are expected.

The implementation should end with these canonical interfaces:

- `crate::cli::property::PropertyArgs` includes waveform input, optional range/scope, `on`, `eval`, `capture`, output controls, and bounded-result controls consistent with repository conventions.
- `crate::engine::Command::Property`, `crate::engine::CommandName::Property`, and `crate::engine::CommandData::Property` are wired through dispatch and output.
- `crate::engine::property::run(args: PropertyArgs) -> Result<CommandResult, WavepeekError>` executes end-to-end.
- Event-expression parsing is reused for both `change --on` and `property --on` without stale `--when` diagnostics.
- Expression parsing/evaluation APIs in `crate::expr` are executable (no `Unimplemented` placeholder path for property runtime).

Contract invariants to preserve:

- Error format remains `error: <category>: <message>`.
- Exit-code mapping stays unchanged.
- `change` data shape remains stable; only trigger-flag naming and related docs/help/errors change.

Revision Note: 2026-03-05 / OpenCode - Created active ExecPlan from backlog items for `when`->`property`, `property --capture`, and user-mandated hard-break migration to `--on` for both `property` and `change`.
Revision Note: 2026-03-05 / OpenCode - Revised plan after review: made default omitted-`--on` semantics explicit, defined baseline behavior for `switch/assert/deassert`, clarified transitional backlog wording for `change --when` to `--on`, and added concrete expected-output signatures for stateless verification.
Revision Note: 2026-03-05 / OpenCode - Applied independent review follow-ups: synchronized baseline rule for `match` with other capture modes (strictly after baseline), added concrete fixture-based acceptance example, and aligned tech-debt closure wording to `change --on`.
Revision Note: 2026-03-05 / OpenCode - Recorded dual review completion in `Progress` for planning-stage quality gate traceability.
