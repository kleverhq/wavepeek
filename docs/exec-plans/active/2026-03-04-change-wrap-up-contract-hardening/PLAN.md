# Wrap Up `change` Performance Work Without Leaking Internal Controls

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, `wavepeek change` keeps the recent performance gains while reducing accidental contract leakage from internal tuning controls. Internal override flags are renamed to a shorter `--perf-*` family, documented in code for maintainers, and gated behind a single debug contract: `DEBUG=1`. In parallel, architecture documentation explains why the multi-engine and heuristic dispatcher exists, changelog messaging reflects the public story (major speedup with large dumps/windows), and development docs define debug-mode behavior as a repository-wide contract.

The result is observable in four places: CLI behavior rejects internal overrides unless `DEBUG=1` is set, `docs/DEVELOPMENT.md` has a new debug-mode contract section, `docs/DESIGN.md` contains a clear high-level section on `change` execution architecture, and `CHANGELOG.md` highlights measured performance outcomes (around 0.5s on large-window scenarios) without exposing internal flags as a user-facing feature.

## Non-Goals

This plan does not change `change` output schema, event semantics, warning texts, or dispatch algorithm thresholds. This plan does not update benchmark golden baseline artifacts under `bench/e2e/runs/dev-baseline/` (explicitly out of scope per user request). This plan does not rewrite historical completed exec plans except when absolutely required for link integrity.

## Progress

- [x] (2026-03-04 06:05Z) Mapped current internal override surface in `src/cli/change.rs`, `src/engine/change.rs`, and tests; confirmed three hidden `--internal-change-*` flags and no hidden subcommands.
- [x] (2026-03-04 06:05Z) Collected current doc/changelog references and selected target update points in `docs/DESIGN.md` section 5 and `CHANGELOG.md` Unreleased entries.
- [x] (2026-03-04 06:05Z) Drafted this executable wrap-up plan with TDD-first milestones, commit boundaries, and mandatory two-pass review gates.
- [x] (2026-03-04 06:12Z) Ran review pass #1 on this plan and tightened commit/review/TDD evidence requirements based on findings.
- [x] (2026-03-04 06:12Z) Ran independent review pass #2 on this plan and resolved remaining executability/documentation-trail gaps.
- [x] (2026-03-04 06:18Z) Incorporated user clarification to remove dual-mode ambiguity: debug contract is now `DEBUG=1` only, with explicit DEVELOPMENT.md update requirements.
- [x] (2026-03-04 06:31Z) Implemented CLI/internal-override hardening: renamed hidden flags to `--perf-*`, added centralized `DEBUG=1` gate, and kept hidden help behavior unchanged.
- [x] (2026-03-04 06:33Z) Updated tests/contracts for new flag names and debug-gated behavior, including explicit-default gating and legacy-flag rejection coverage.
- [x] (2026-03-04 06:34Z) Updated architecture/development/changelog collateral and passed `make check` after targeted test reruns.
- [x] (2026-03-04 06:34Z) Completed review pass #1, addressed suggested test-coverage gaps, and re-ran targeted validation.
- [x] (2026-03-04 06:35Z) Completed independent review pass #2 with fresh context; no remaining severity-tagged findings.
- [x] (2026-03-04 06:36Z) Passed final verification gate `make ci` and captured closure artifacts in this plan.

## Surprises & Discoveries

- Observation: Internal override controls are currently hidden but still directly invokable, and they are used by integration tests as regular CLI args.
  Evidence: `src/cli/change.rs` defines hidden flags; `tests/change_opt_equivalence.rs` and `tests/change_vcd_fst_parity.rs` pass them in every forced-mode invocation.

- Observation: Public release notes currently expose one hidden override flag explicitly.
  Evidence: `CHANGELOG.md` Unreleased `Changed` entry mentions ``--internal-change-engine edge-fast``.

- Observation: Technical architecture currently describes performance only at a broad level and does not explain the multi-engine dispatcher model that now exists.
  Evidence: `docs/DESIGN.md` has command-level behavior notes and generic performance principles, but no dedicated architecture subsection for `change` engine selection and heuristics.

## Decision Log

- Decision: Use the new internal flag family `--perf-engine`, `--perf-candidates`, and `--perf-edge-fast-force`, and remove the old `--internal-change-*` names.
  Rationale: The old names duplicate command context (`change`) and are unnecessarily verbose; removing old names prevents accidental long-term dependence.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Gate all `--perf-*` overrides behind a single debug contract: `DEBUG=1` environment variable.
  Rationale: One debug switch avoids ambiguity and keeps operator guidance simple while preserving internal access for engineers and CI diagnostics.
  Date/Author: 2026-03-04 / OpenCode

- Decision: When users try `--perf-*` without debug mode, return an explicit `error: args:` message instead of pretending the flags do not exist.
  Rationale: Clear user feedback is safer and more maintainable than ambiguous parse failures; the message should explain exactly how to intentionally opt in.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Keep `--perf-*` hidden from help output and document debug-mode access only in development docs and inline internal-flag docstrings.
  Rationale: Internal controls remain non-public while still understandable for maintainers editing the code.
  Date/Author: 2026-03-04 / OpenCode

## Outcomes & Retrospective

Current status: complete. All planned implementation, collateral updates, two-pass review, and `make ci` validation succeeded.

Expected completion outcome: internal tuning surface becomes intentionally unstable/private-by-default while preserving current optimized behavior and parity guarantees.

Known residual risk to monitor during implementation: forced-mode tests may initially fail until all helpers are migrated to `--perf-*` plus debug opt-in; this is expected and should be handled inside the TDD flow.

## Context and Orientation

`wavepeek` is a single-process stateless CLI. The `change` command currently has multiple execution engines in `src/engine/change.rs` (`pre-fusion`, `fused`, `edge-fast`, plus `auto` dispatcher). Internal CLI override args are defined in `src/cli/change.rs` and fed into engine selection/candidate collection in `src/engine/change.rs`.

In this plan, “internal override” means a hidden CLI flag that changes engine/candidate mode regardless of normal auto-dispatch logic. “Debug mode” means process-level opt-in via environment variable `DEBUG=1` that allows such overrides for diagnostics. “Public contract leakage” means users can discover or script against internal knobs as if they were stable product features.

Key files to modify:

- `src/cli/change.rs` for renamed hidden flags and maintainer-facing docstrings.
- `src/cli/mod.rs` for centralized debug-mode detection (`DEBUG=1`) and debug gate enforcement before dispatch.
- `src/engine/change.rs` for field/type rename plumbing from `internal_*` to `perf_*` without behavioral drift.
- `tests/change_opt_equivalence.rs` and `tests/change_vcd_fst_parity.rs` for forced-mode helper migration.
- `tests/cli_contract.rs` and/or `tests/change_cli.rs` for explicit debug-gate behavior coverage and hidden-help checks.
- `docs/DEVELOPMENT.md` for repository-wide debug contract and high-level side-effect statement.
- `docs/DESIGN.md` for high-level architecture rationale of multi-engine `change` execution.
- `CHANGELOG.md` for release-note cleanup and performance messaging.

## Open Questions

No blocking product-level questions remain. This plan assumes debug mode is enabled only via `DEBUG=1` and this contract is documented once in `docs/DEVELOPMENT.md` for all commands.

## Plan of Work

Milestone 1 introduces a failing-test-first contract for internal override privacy. Start by updating/adding tests so they assert that `--perf-*` overrides require `DEBUG=1` and old `--internal-change-*` names are no longer accepted. Then implement CLI and engine wiring so internal overrides work only when `DEBUG=1` is set and are rejected otherwise with a clear `args` error. Keep help output unchanged for normal users (hidden options remain hidden).

Milestone 2 updates architecture and release collateral. Add a focused technical-architecture subsection in `docs/DESIGN.md` explaining the multi-engine `change` model, dispatcher heuristics at a high level, and why this complexity exists (performance on large windows while preserving contract-equivalent behavior). Add a new debug section in `docs/DEVELOPMENT.md` that defines `DEBUG=1` as the single debug switch and describes only high-level side effects (for now: hidden internal controls become available, without listing them). Update `CHANGELOG.md` Unreleased entries to remove hidden-flag callouts and replace them with user-facing performance outcomes.

Milestone 3 performs consistency sweep and validation. Search for stale references, especially old `--internal-change-*` names in active code/docs. Exclude benchmark baseline artifact edits per request. Run targeted tests, then full quality gates.

Milestone 4 closes with mandatory review discipline and commits. Request review pass #1, resolve findings and commit fixes. Then request a fresh review pass #2 with new context; resolve/commit again if needed. Only close once both passes are clean.

### Concrete Steps

Run all commands from `/workspaces/perf-change`.

1. Write failing tests first for the new override contract.

   - Update forced-mode test helpers to use new flag names and explicit debug opt-in.
   - Add or extend tests that verify:
     - `change` with `--perf-engine` (or other `--perf-*`) fails without debug mode and prints a clear `error: args:` hint.
     - same invocation succeeds with `DEBUG=1`.
     - old `--internal-change-*` invocations fail with exit code `1` and `error: args:` parse guidance.
     - `change --help` does not expose `--perf-*`.

    Run:

        cargo test --test change_opt_equivalence
        cargo test --test change_vcd_fst_parity
        cargo test --test change_cli
        cargo test --test cli_contract

   Expected before implementation is complete: at least one new debug-gate assertion fails.
   Capture a short red-phase excerpt (failing test name and failure line) in this plan's `Artifacts and Notes` section before code implementation, then capture green-phase pass output after implementation.

2. Implement CLI/engine changes.

   - In `src/cli/change.rs`, rename internal args to `--perf-*`, keep `hide = true`, and add concise docstrings that these are unstable internal controls available only when `DEBUG=1` is set.
   - In `src/cli/mod.rs`, add centralized debug detection based on `DEBUG=1`; enforce gate before dispatch when any `--perf-*` override is explicitly used.
   - In `src/engine/change.rs`, rename argument/type usage from `internal_*` to `perf_*` and keep mode semantics unchanged.

   Re-run targeted tests:

       cargo test --test change_opt_equivalence
       cargo test --test change_vcd_fst_parity
       cargo test --test change_cli
       cargo test --test cli_contract

3. Commit atomic implementation unit.

       git add src/cli/change.rs src/cli/mod.rs src/engine/change.rs tests/change_opt_equivalence.rs tests/change_vcd_fst_parity.rs tests/change_cli.rs tests/cli_contract.rs
       git commit -m "harden internal change tuning flags behind debug opt-in"

4. Update architecture/changelog collateral.

   - In `docs/DESIGN.md`, add a dedicated technical architecture subsection for `change` execution engines and dispatch heuristics rationale, and add short cross-links from existing high-level/module sections.
   - In `docs/DEVELOPMENT.md`, add `## Debug Mode` section describing that debug mode is enabled via `DEBUG=1` and, at high level, enables hidden internal controls across commands without making them public contract.
   - In `CHANGELOG.md`, remove hidden-flag wording and add user-visible performance outcome wording (include benchmark context around ~0.5s for large dumps/windows).
   - Run a consistency grep and update remaining stale references in active contract docs/tests (excluding historical completed plans unless a direct contradiction appears).

   Run:

       cargo test --test cli_contract
       make check

5. Commit collateral unit.

       git status --short
       git add docs/DESIGN.md docs/DEVELOPMENT.md CHANGELOG.md tests/change_cli.rs tests/cli_contract.rs
       git status --short
       git commit -m "document change engine architecture and update perf release notes"

6. Mandatory review pass #1.

   - Load `ask-review` skill.
   - Invoke review pass #1 with a fresh request that includes changed file list, commands already run (`cargo test ...`, `make check`), and known risk focus (`DEBUG=1` gate semantics, hidden flags, docs/changelog wording).

        Task(description="review pass 1", subagent_type="review", prompt="Review current branch diff for wrap-up hardening of change internal controls. Focus on CLI gating correctness for DEBUG=1, hidden-flag contract, docs/changelog consistency, and missing tests. Return severity-tagged findings only.")

   - Apply fixes and commit if findings are valid.

   Suggested fix commit message (if needed):

       git commit -m "address review findings for change wrap-up hardening"

7. Mandatory independent review pass #2.

   - Invoke review pass #2 as a new fresh request (do not resume pass #1 context) on the updated diff after pass #1 fixes.

        Task(description="review pass 2", subagent_type="review", prompt="Independent second review of current branch diff for wrap-up hardening of change internal controls. Treat prior review as unknown; find remaining defects/regressions/docs mismatches.")

   - Apply and commit any remaining fixes.

   Closure rule: no unresolved high/critical findings in either pass. If pass #2 finds issues, fix and rerun a fresh pass #2 until clean.

   Suggested fix commit message (if needed):

       git commit -m "resolve second-pass review findings for change wrap-up"

8. Final verification gate.

       make ci

   Expected success signature:

       make ci
       ...
       Finished `dev` profile ...
       test result: ok.

### Validation and Acceptance

Acceptance is met only when all of the following are true together:

- Internal override flags are renamed to `--perf-*`.
- Invoking any old `--internal-change-*` flag fails with exit code `1` and `error: args:` parse guidance.
- `--perf-*` usage without debug mode fails with explicit `error: args:` guidance.
- `--perf-*` usage with `DEBUG=1` succeeds and preserves forced-mode parity tests.
- `wavepeek change --help` does not expose `--perf-*`.
- `docs/DEVELOPMENT.md` documents `DEBUG=1` as the single debug-mode contract and states high-level side effects only (hidden internal controls become available).
- `docs/DESIGN.md` clearly explains the multi-engine `change` architecture and why heuristics exist.
- `CHANGELOG.md` no longer advertises hidden flags and instead states the user-visible performance improvement narrative.
- Validation commands pass: targeted tests, `make check`, and final `make ci`.
- Review pass #1 and fresh review pass #2 both complete with no unresolved high/critical findings.

TDD acceptance requires evidence that at least one new debug-gate test fails before implementation and passes after implementation in the same branch history.

### Idempotence and Recovery

All edits are text-only and idempotent. Re-running tests or quality gates is safe. If a partial implementation breaks parsing, recover by reverting only the in-progress hunk in `src/cli/mod.rs` or `src/cli/change.rs`, then rerun targeted tests before continuing.

If review fixes are needed, apply them in separate commits without rewriting history. Do not amend commits unless required by hook-modified files in the immediately previous local commit.

### Artifacts and Notes

Expected modified artifacts:

- `src/cli/change.rs`
- `src/cli/mod.rs`
- `src/engine/change.rs`
- `tests/change_opt_equivalence.rs`
- `tests/change_vcd_fst_parity.rs`
- `tests/change_cli.rs`
- `tests/cli_contract.rs`
- `docs/DESIGN.md`
- `docs/DEVELOPMENT.md`
- `CHANGELOG.md`

Performance note for changelog wording should be sourced from benchmark evidence already captured in repository runs (for example, `bench/e2e/runs/dev-baseline` and post-optimization run reports), but this plan intentionally does not modify baseline artifacts.

Before closure, include two short command excerpts here:

- TDD red-phase: one failing assertion from step 1 before implementation.
- TDD green-phase: passing output after implementation for the same test target.
- Review evidence: short excerpts from pass #1 and pass #2 outputs showing final clean status (or showing fixes were applied and re-reviewed).

Recorded excerpts:

- TDD red-phase (`cargo test --test change_cli change_perf_overrides_require_debug_mode -- --exact`, before implementation):

      test change_perf_overrides_require_debug_mode ... FAILED
      Unexpected stderr, failed var.contains(--perf-*)
      var: error: args: unexpected argument '--perf-engine' found See 'wavepeek change --help'.

- TDD green-phase (`cargo test --test change_cli change_perf_overrides_require_debug_mode -- --exact`, after implementation):

      running 1 test
      test change_perf_overrides_require_debug_mode ... ok
      test result: ok. 1 passed; 0 failed

- Review evidence:

  - Pass #1 initial output: `No significant implementation defects found ...` with test-gap recommendations for explicit-default and per-flag gate assertions.
  - Pass #1 follow-up output after fixes: `No findings.`
  - Independent pass #2 output: `No severity-tagged findings in the reviewed scope`.

### Interfaces and Dependencies

No new third-party dependencies are allowed.

The implementation must preserve these interfaces and invariants:

- `crate::engine::change::run` output behavior remains contract-equivalent for identical user-facing inputs.
- `change` command JSON/human payload semantics remain unchanged.
- Internal override controls remain hidden and explicitly non-contractual.
- CLI error shape remains `error: <category>: <message>` with exit code `1` for args misuse.
- Existing benchmark baseline files remain untouched.

Revision Note: 2026-03-04 / OpenCode - Created active wrap-up ExecPlan for internal-flag hardening, architecture/changelog updates, collateral sweep, and mandatory two-pass review with atomic commits.
Revision Note: 2026-03-04 / OpenCode - Updated plan after review pass #1 to remove ambiguous acceptance wording, make TDD evidence explicit, and tighten collateral commit scope.
Revision Note: 2026-03-04 / OpenCode - Updated plan after independent review pass #2 to replace placeholder staging with executable commands, add explicit review invocations, and document review evidence requirements.
Revision Note: 2026-03-04 / OpenCode - Incorporated user direction to use only `DEBUG=1` (no `--debug`), and added explicit `docs/DEVELOPMENT.md` debug-contract update requirements plus internal-flag docstring wording expectations.
