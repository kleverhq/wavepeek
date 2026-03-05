# Hard-Break Surface Rename to `property` and `--on` (No Runtime Delivery)

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, command naming becomes consistent and assertion-oriented without changing runtime maturity: users call `wavepeek property ...` instead of `wavepeek when ...`, and they use `--on` (not `--when`) for event triggers in both `property` and `change` surfaces. This is a hard break with no compatibility aliases.

This plan intentionally does not implement property runtime execution. `property` remains unimplemented and should continue returning deterministic `error: unimplemented:` status; only naming and collateral are migrated.

A user can verify the result by checking top-level help and argument parsing: `property` appears and `when` is gone, `change --on` works where `change --when` is rejected, `property --capture` is accepted by parser/help, and any `property` invocation still fails as unimplemented.

## Non-Goals

This plan does not implement `property` runtime behavior, expression evaluation, capture semantics execution, or any new waveform query engine logic. This plan does not change JSON payload schemas for successful command outputs. This plan does not implement deferred `iff logical_expr` execution. This plan does not add compatibility aliases for removed names.

## Progress

- [x] (2026-03-05 08:55Z) Reviewed backlog items for `when` -> `property`, `property --capture`, and trigger-flag consistency.
- [x] (2026-03-05 08:58Z) Mapped impacted code/docs/tests paths in `src/`, `tests/`, `docs/`, and `schema/`.
- [x] (2026-03-05 09:05Z) Updated backlog wording to capture hard-break policy and `--on` direction.
- [x] (2026-03-05 09:18Z) Authored initial active ExecPlan.
- [x] (2026-03-05 09:42Z) Completed review pass #1 on planning docs and addressed findings.
- [x] (2026-03-05 09:56Z) Completed independent review pass #2 on planning docs and addressed findings.
- [x] (2026-03-05 10:25Z) Scoped plan down per product clarification: rename and collateral only; `property` stays unimplemented.
- [x] (2026-03-05 10:48Z) Completed corrected-scope review pass #1; resolved medium findings on backlog path wording, capture default test coverage, and review runbook clarity.
- [x] (2026-03-05 10:56Z) Completed corrected-scope independent review pass #2 (fresh context); no high/medium findings remain.
- [x] (2026-03-05 11:17Z) Completed TDD red phase: updated rename-contract tests and captured expected failures (`unrecognized subcommand 'property'`, `unexpected argument '--on'`).
- [x] (2026-03-05 11:31Z) Renamed `change --when` surface to `change --on` across CLI/runtime/parser diagnostics with no semantic behavior drift.
- [x] (2026-03-05 11:35Z) Renamed `when` command surface to `property`, added `--capture` parser/help contract, and preserved deterministic unimplemented runtime status.
- [x] (2026-03-05 11:44Z) Updated collateral (`README.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, `CHANGELOG.md`, `docs/BACKLOG.md`) for rename-only delivery and explicit unimplemented `property` status.
- [x] (2026-03-05 11:58Z) Ran validation gates (`make check`, `make ci`) successfully before implementation review.
- [x] (2026-03-05 12:00Z) Completed mandatory review pass #1; addressed low findings by adding missing parse-contract tests for `property --eval` requiredness and empty `change --on` expression diagnostics.
- [x] (2026-03-05 12:02Z) Completed mandatory independent review pass #2 (fresh context); applied low-risk cleanup for residual parser/help wording and intentional `property` enum omission notes.
- [x] (2026-03-05 12:04Z) Re-ran full validation gates (`make check`, `make ci`) after review-fix commits; all checks green.

## Surprises & Discoveries

- Observation: `when` is already a parse-only/unimplemented command path, so command rename can be delivered without engine feature implementation.
  Evidence: `src/engine/when.rs` currently returns `WavepeekError::Unimplemented("`when` command execution is not implemented yet")`.

- Observation: Existing event-trigger runtime for `change` is mature and heavily test-locked, so flag rename must preserve semantics exactly.
  Evidence: `tests/change_cli.rs`, `tests/change_opt_equivalence.rs`, and `tests/change_vcd_fst_parity.rs` contain extensive `--when` contract assertions.

- Observation: Current schema excludes unimplemented command surfaces.
  Evidence: `schema/wavepeek.json` command enum contains `info|scope|signal|value|change`; no `when` entry exists today.

- Observation: Prior draft over-scoped into runtime delivery (`property` evaluator/capture execution), which conflicts with clarified product scope.
  Evidence: user clarification: "в плане должны быть только переименования ... Команда property всё также будет не реализована".

## Decision Log

- Decision: Execute this work as a hard break with no compatibility aliases.
  Rationale: Product request is explicit and consistent with prior repo rename policy (`tree/modules/signals/changes/at` removals without aliases).
  Date/Author: 2026-03-05 / OpenCode

- Decision: Limit this plan to naming and collateral migration; do not deliver runtime behavior.
  Rationale: Clarified scope requires that `property` remains intentionally unimplemented after rename.
  Date/Author: 2026-03-05 / OpenCode

- Decision: Introduce `property --capture` only as CLI contract surface in this phase.
  Rationale: Backlog requests flag introduction, while runtime semantics remain deferred.
  Date/Author: 2026-03-05 / OpenCode

- Decision: `property` runtime must return unimplemented before event-expression semantic parsing in this phase.
  Rationale: This avoids accidental partial-runtime behavior and keeps scope strictly rename-only.
  Date/Author: 2026-03-05 / OpenCode

- Decision: Keep schema command/data surface unchanged in this phase.
  Rationale: `property` remains unimplemented and does not produce successful JSON payloads requiring schema expansion.
  Date/Author: 2026-03-05 / OpenCode

- Decision: Preserve all current `change` runtime semantics and only rename flag wording from `--when` to `--on`.
  Rationale: This minimizes risk and keeps parity suites meaningful.
  Date/Author: 2026-03-05 / OpenCode

## Outcomes & Retrospective

Current status: implementation complete and validated.

Delivered outcome: user-facing CLI and docs now consistently use `property` and `--on`, legacy `when`/`--when` syntax is rejected without aliases, and `property` still returns deterministic `error: unimplemented:` status.

Validation evidence captured:
- Red phase failure examples: `help_lists_expected_subcommands` failed on missing `property`; `change_omitted_when_matches_explicit_wildcard` failed on unsupported `--on`; `property_cli` initially failed with `unrecognized subcommand 'property'`.
- Green phase: `cargo test --test property_cli`, `cargo test --test cli_contract`, `cargo test --test change_cli`, `cargo test --test change_opt_equivalence`, `cargo test --test change_vcd_fst_parity` all passed.
- Full gates: `make check` and `make ci` passed before review and again after review-fix commits.

Residual risk at completion: low (rename-only), mainly future drift risk if deferred `property` runtime implementation bypasses the locked parser/help contracts.

## Context and Orientation

`wavepeek` is a single-crate Rust CLI. Parsing and help contracts are in `src/cli/`. Runtime command dispatch is in `src/engine/mod.rs`. Output envelopes are in `src/output.rs`. Integration tests under `tests/` define the public CLI contract.

Today, `src/cli/when.rs` defines the legacy `when` surface (`--clk`, `--cond`, and qualifiers), while `src/engine/when.rs` returns `Unimplemented`. The implemented `change` flow is wired around `--when` through `src/cli/change.rs`, `src/engine/change.rs`, and `src/expr/parser.rs` diagnostics. Contract tests in `tests/cli_contract.rs` and change-focused suites currently assert the legacy naming, and collateral in `README.md`, `docs/DESIGN.md`, `docs/ROADMAP.md`, and `CHANGELOG.md` still contains `when`/`--when` wording.

In this plan, "rename-only rollout" means migrating CLI/help/tests/docs naming without adding engine behavior. "Collateral" means all non-runtime contract artifacts (tests, docs, changelog, help text) that must remain synchronized with the shipped surface. "TDD red phase" means first proving at least one targeted test fails before migration edits.

Expected touched paths are `src/cli/change.rs`, `src/cli/mod.rs`, `src/cli/when.rs` (renamed to `src/cli/property.rs`), `src/engine/change.rs`, `src/engine/mod.rs`, `src/engine/when.rs` (renamed to `src/engine/property.rs`), `src/expr/parser.rs`, `tests/cli_contract.rs`, `tests/change_cli.rs`, `tests/change_opt_equivalence.rs`, `tests/change_vcd_fst_parity.rs`, `tests/property_cli.rs` (new), `docs/DESIGN.md`, `docs/ROADMAP.md`, `README.md`, `CHANGELOG.md`, and `docs/BACKLOG.md`.

## Open Questions

No blocking questions remain. Scope is explicitly rename-only with unimplemented `property` runtime preserved.

## Plan of Work

Milestone 1 locks the future contract with failing tests first. The tests must prove hard-break behavior (`when`/`--when` rejected) and parse/help behavior (`property`, `--on`, `--capture`) before implementation edits.

Milestone 2 migrates `change` trigger flag naming to `--on` across CLI parsing, runtime wiring, and parser diagnostics, without changing event semantics.

Milestone 3 migrates command surface from `when` to `property` and adds `--capture` parsing/help while preserving unimplemented runtime status.

Milestone 4 updates collateral (`docs`, `README`, `CHANGELOG`, backlog wording) to match shipped rename-only behavior and explicitly document that `property` remains unimplemented.

Milestone 5 runs full validation and mandatory dual review closure.

### Concrete Steps

Run all commands from `/workspaces/fix-rename-when`.

1. TDD red phase for rename-only contracts.

   - Update `tests/cli_contract.rs`:
     - replace expected subcommand `when` with `property`;
     - keep unimplemented status checks, but for `property`;
     - assert `change --when` is rejected with `error: args:` and help hint;
     - assert legacy `when` command is rejected as unrecognized subcommand.
   - Update change integration suites (`tests/change_cli.rs`, `tests/change_opt_equivalence.rs`, `tests/change_vcd_fst_parity.rs`) to use `--on` instead of `--when` while preserving expected payloads.
   - Add `tests/property_cli.rs` for parser/help contract:
     - accepts `--on`, `--eval`, and `--capture=match|switch|assert|deassert`;
     - confirms omitted `--capture` defaults to `switch` in parse/help contract;
     - rejects legacy `when`-surface flags (`--clk`, `--cond`) on `property`;
     - rejects `--when` for `property`;
     - invocation fails as unimplemented after parse succeeds.

   Red-phase commands (expect at least one failure before code migration):

       cargo test --test cli_contract help_lists_expected_subcommands -- --exact
       cargo test --test change_cli change_omitted_on_matches_explicit_wildcard -- --exact
       cargo test --test property_cli property_defaults_capture_to_switch -- --exact
       cargo test --test property_cli property_accepts_capture_flag_but_is_unimplemented -- --exact

   Expected failure signatures before implementation:

       error: args: unrecognized subcommand 'property'
       ... FAILED

2. Implement `change --on` rename (no semantic changes).

   - In `src/cli/change.rs`, rename field `when` to `on` and clap flag to `--on`.
   - In `src/cli/mod.rs`, update change help text from `--when` to `--on`.
   - In `src/engine/change.rs`, consume `args.on` with same default wildcard behavior (`*`) and rename user-facing diagnostics from `--when` to `--on`.
   - In `src/expr/parser.rs`, update event-expression error wording/hints used by `change` so users see `--on` guidance.

   Run change-focused suites:

       cargo test --test change_cli
       cargo test --test change_opt_equivalence
       cargo test --test change_vcd_fst_parity

3. Rename `when` command surface to `property` and keep runtime unimplemented.

   - Rename modules:
     - `src/cli/when.rs` -> `src/cli/property.rs`
     - `src/engine/when.rs` -> `src/engine/property.rs`
   - In `src/cli/property.rs`, replace old `when`-specific flags with planned property surface (`--on`, `--eval`, `--capture`, waveform/scope/range/output controls as needed for docs consistency).
   - In `src/cli/mod.rs`, replace subcommand wiring/help from `when` to `property`, including top-level ordering and descriptive text.
   - In `src/engine/mod.rs`, replace `Command::When` path with `Command::Property`.
   - In `src/engine/property.rs`, keep unimplemented return path with updated message:

       "`property` command execution is not implemented yet"

   - Ensure `property` parser accepts `--capture` values but runtime still exits via unimplemented error.
   - Ensure `property` does not invoke event-expression semantic parsing in this phase; malformed `--on` text must still surface as unimplemented runtime status (after clap-level parsing succeeds).

   Run targeted suites:

       cargo test --test property_cli
       cargo test --test cli_contract

4. Collateral migration (rename-only, explicit unimplemented status).

   - Update `docs/DESIGN.md` command/flag references:
     - `change` uses `--on`;
     - planned command is `property` with `--on`, `--eval`, and `--capture` surface;
     - status remains unimplemented in this release.
   - Update `docs/ROADMAP.md` query-engine naming from `when` to `property`.
   - Update `README.md` command table (`property` planned/unimplemented, `change --on` wording).
   - Update `CHANGELOG.md` with breaking rename notes and explicit statement that `property` remains unimplemented.
   - Keep `schema/wavepeek.json` unchanged unless implementation creates successful `property --json` payloads (out of scope for this plan).
   - Run a collateral sweep to catch stale wording outside intentional historical references:

       rg --line-number '\\bwhen\\b|--when' docs/DESIGN.md docs/ROADMAP.md README.md CHANGELOG.md tests/cli_contract.rs

   - Allow only intentional historical mentions (for example, changelog history and explicit backlog tech-debt notes); all live contract/help wording must be `property`/`--on`.

5. Commit atomic units.

   Suggested split:

       git add tests/cli_contract.rs tests/change_cli.rs tests/change_opt_equivalence.rs tests/change_vcd_fst_parity.rs tests/property_cli.rs
       git commit -m "test(cli)!: lock property and --on rename contracts"

       git add src/cli/change.rs src/cli/mod.rs src/engine/change.rs src/expr/parser.rs
       git commit -m "refactor(change)!: rename trigger flag to --on"

       git add src/cli/property.rs src/engine/property.rs src/engine/mod.rs
       git commit -m "refactor(cli)!: rename when surface to property"

       git add docs/DESIGN.md docs/ROADMAP.md README.md CHANGELOG.md docs/BACKLOG.md
       git commit -m "docs!: align collateral with property and --on hard break"

6. Full validation gates.

       make check
       make ci

   Expected success signature:

       cargo fmt -- --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test -q
       ... completed successfully

7. Mandatory review cycle.

   - Run review pass #1 (`review` agent), apply valid fixes, commit.
   - Review pass #1 context packet must include: scope summary, changed file list, tests run/results, hard-break assumptions, and explicit "rename-only, property still unimplemented" note.

   - Run independent review pass #2 in fresh context, apply valid fixes, commit.
   - Pass #2 must use a fresh reviewer session (not resumed context) and must include a short delta of fixes applied after pass #1.

   - Re-run `make check` and `make ci` after final review-fix commit.

### Validation and Acceptance

Acceptance is complete only when all conditions below are true:

- Top-level help lists `property` and does not list `when`.
- `wavepeek change --on ...` behaves like prior `--when` behavior for equivalent inputs.
- `wavepeek change --when ...` fails deterministically with `error: args:` and help hint.
- `wavepeek property --when ...` fails deterministically with `error: args:` and help hint.
- Legacy `property` flags `--clk` and `--cond` are rejected as argument errors.
- Omitted `--capture` on `property` resolves to default `switch` in parser/help contract.
- `wavepeek property --waves tests/fixtures/hand/m2_core.vcd --scope top --on "posedge clk" --eval "1" --capture switch` parses and then fails as unimplemented (not as parse error).
- Unimplemented message is exactly ``error: unimplemented: `property` command execution is not implemented yet``.
- Docs (`docs/DESIGN.md`, `docs/ROADMAP.md`, `README.md`, `CHANGELOG.md`) consistently use rename-only wording and do not claim property runtime delivery.
- Invalid `property --on` text does not claim semantic parser errors in this phase; it still returns the unimplemented runtime status once clap parsing succeeds.
- `make check` and `make ci` pass after review fixes.
- Review pass #1 and independent review pass #2 are clean, or findings are fixed and rechecked.

TDD acceptance requirement: at least one updated rename-contract test fails before implementation and passes after implementation in the same branch history.

### Idempotence and Recovery

All steps are safe to rerun: they are deterministic file edits and test/gate runs. If compilation fails mid-rename, restore consistency first in `src/cli/mod.rs` and `src/engine/mod.rs` (module/variant wiring), then rerun targeted suites. If a test fails due to stale wording, update the matching help/error contract where the rename was intended and rerun.

If review reports regressions, apply follow-up commits; do not rewrite history.

### Artifacts and Notes

Expected change set for this plan: `src/cli/change.rs`, `src/cli/mod.rs`, `src/cli/when.rs` -> `src/cli/property.rs`, `src/engine/change.rs`, `src/engine/mod.rs`, `src/engine/when.rs` -> `src/engine/property.rs`, `src/expr/parser.rs`, `tests/cli_contract.rs`, `tests/change_cli.rs`, `tests/change_opt_equivalence.rs`, `tests/change_vcd_fst_parity.rs`, `tests/property_cli.rs` (new), `docs/DESIGN.md`, `docs/ROADMAP.md`, `README.md`, `CHANGELOG.md`, and `docs/BACKLOG.md`.

Record concise evidence before closure:

- Red-phase failure snippet from targeted tests.
- Green-phase snippets for `property_cli` and renamed `change` suites.
- `make ci` success snippet.
- Review pass #1 and #2 outcomes.

### Interfaces and Dependencies

No new dependencies are expected.

Target interfaces after implementation:

- `crate::cli::change::ChangeArgs` exposes `on` (not `when`).
- `crate::cli::property::PropertyArgs` exposes `on`, `eval`, and `capture` parser surface.
- `crate::engine::Command::Property` and `crate::engine::property::run` exist, with `run` intentionally returning unimplemented error in this phase.

Contract invariants:

- Error shape remains `error: <category>: <message>`.
- Exit-code mapping stays unchanged.
- This phase ships naming migration only and must not claim runtime property delivery.

Revision Note: 2026-03-05 / OpenCode - Created active ExecPlan for hard-break `property`/`--on` migration.
Revision Note: 2026-03-05 / OpenCode - Scope corrected after product clarification: only renames/collateral updates are in scope; `property` remains unimplemented.
Revision Note: 2026-03-05 / OpenCode - Incorporated corrected-scope review feedback: prose-first tightening, broader collateral sweep, explicit `property` parse-vs-unimplemented boundary, and additional rename-contract checks.
Revision Note: 2026-03-05 / OpenCode - Completed implementation with validation gates, dual independent review passes, and follow-up low-risk coverage/wording fixes.
