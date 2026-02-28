# Add Recursive `signal` Listing with Depth Bounds

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Today `wavepeek signal` lists only signals that are directly declared in one scope. Users cannot ask for nested signals under that scope, which makes hierarchy-heavy designs tedious to inspect and hard to script.

After this change, users will be able to run `wavepeek signal --recursive` to include nested scopes and optionally cap recursion with `--max-depth <n>`. The command remains deterministic, bounded by `--max`, and backward compatible by default (no recursion unless explicitly requested).

The user-visible success case is simple: with a scope that has child scopes, `wavepeek signal --recursive` returns additional nested signal rows in stable order; `--max-depth 0` returns the same signal set as current non-recursive mode.

## Non-Goals

This plan does not change JSON envelope shape or schema definitions for `signal` (`name`, `path`, `kind`, `width` stay unchanged).

This plan does not implement `unlimited` limit literals for `--max` or `--max-depth`; that is a separate backlog task.

This plan does not redesign the `scope` command traversal contracts.

This plan does not include performance optimization work beyond preserving existing bounded behavior and deterministic ordering.

This plan does not introduce a strict performance regression gate for the new `signal` recursive scenarios in CI on day one; benchmark entries are added for observability and trend tracking.

## Progress

- [x] (2026-02-28 09:20Z) Mapped backlog requirements and current `signal` contract in `docs/BACKLOG.md` and `docs/DESIGN.md`.
- [x] (2026-02-28 09:34Z) Traced implementation flow across CLI parser, engine runtime, waveform adapter, output renderer, and current tests.
- [x] (2026-02-28 09:47Z) Resolved non-critical ambiguities for depth semantics, ordering policy, and `--max-depth` validation behavior.
- [x] (2026-02-28 10:05Z) Drafted this executable implementation plan with TDD-first milestones and validation protocol.
- [ ] Implement Milestone 1 (tests-first contract capture for recursive/non-recursive behavior).
- [ ] Implement Milestone 2 (CLI and engine behavior changes for `--recursive` and `--max-depth`).
- [ ] Implement Milestone 3 (waveform traversal API for deterministic recursive signal collection).
- [ ] Implement Milestone 4 (docs/help updates and contract collateral alignment).
- [ ] Add `signal` recursive E2E perf scenarios to `bench/e2e/tests.json` and capture a baseline run on SCR1 fixture.
- [ ] Complete review pass #1 findings and commit fixes.
- [ ] Complete fresh independent review pass #2 and close residual findings.
- [ ] Run final quality gates (`make check`, `make ci`) and update `CHANGELOG.md` (`Unreleased -> Added`) for the new CLI flags.

## Surprises & Discoveries

- Observation: `signal` human rendering currently has only two display modes (`name` or absolute `path`) and no intermediate relative-path display mode.
  Evidence: `src/output.rs` uses `signal_display_name(entry, abs)` where non-`--abs` always returns `entry.name`.

- Observation: current `signal` command help explicitly states non-recursive behavior and will become contractually stale once recursive mode is added.
  Evidence: `src/cli/mod.rs` `Signal` long help contains "Listing is non-recursive".

- Observation: existing integration fixtures already contain at least one nested level for `signal` recursion checks (`top.cpu.*`, `top.mem.*`), but not deeper than one level for robust `--max-depth` matrix tests.
  Evidence: `tests/fixtures/hand/m2_core.vcd` declares signals in `top`, `top.cpu`, and `top.mem`.

- Observation: `scope` command semantics already treat `--max-depth 0` as "current/root level only", giving a strong precedent for recursive depth semantics in `signal`.
  Evidence: `tests/scope_cli.rs` test `scope_respects_max_depth` expects only depth `0` entries.

- Observation: current E2E perf catalog has no `signal` category scenarios yet, so post-feature traversal cost changes would be invisible in benchmark reports.
  Evidence: `bench/e2e/tests.json` includes `info`/`at`/`change` categories and no `signal_*` entries.

## Decision Log

- Decision: Define `signal --recursive --max-depth 0` as listing only signals directly in `--scope`.
  Rationale: Matches existing recursive depth convention in `scope`, keeps behavior intuitive, and guarantees parity test against non-recursive mode.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Treat `--max-depth` as valid only when `--recursive` is enabled; return `error: args:` otherwise.
  Rationale: Avoids silent no-op flags and keeps CLI intent explicit.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Use deterministic depth-first scope traversal with lexicographically sorted child scopes; within each visited scope, sort signals by `(name, path)`.
  Rationale: Reuses existing deterministic conventions and maps directly to backlog wording about deterministic traversal and ordering.
  Date/Author: 2026-02-28 / OpenCode

- Decision: In human mode and recursive listing, render signal display as path relative to `--scope` (for example `clk`, `cpu.valid`, `mem.ready`), while `--abs` still renders canonical absolute paths.
  Rationale: Satisfies backlog requirement while preserving current `--abs` behavior.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Keep JSON payload unchanged (`path` remains canonical absolute), and add any new display-only field with `#[serde(skip_serializing)]` if needed.
  Rationale: Preserves machine contract and schema stability.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Apply TDD for this feature: add failing tests first at CLI integration and waveform unit levels, then implement minimal production changes.
  Rationale: The feature is contract-heavy and has clear observable behavior, making TDD feasible and lower-risk.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Keep `--filter` semantics unchanged in recursive mode: regex applies to signal `name` only, not to relative display path.
  Rationale: Preserves established behavior and avoids hidden contract break for existing users.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Add three SCR1-focused E2E perf tests for recursive `signal` listing: full recursive list from top scope, recursive list filtered by `valid`, and depth-limited recursive list.
  Rationale: Gives immediate coverage for the new command shape and captures the main cost dimensions (full traversal, regex filtering, and depth pruning) with one stable fixture family.
  Date/Author: 2026-02-28 / OpenCode

## Outcomes & Retrospective

Planning outcome: requirements from backlog were converted into explicit acceptance criteria, non-critical ambiguities were resolved into concrete defaults, and implementation is split into testable milestones.

Implementation outcome is pending. Expected end state is a backward-compatible `signal` command that supports recursive listing with depth control and stable output contracts in both human and JSON modes.

## Context and Orientation

The `signal` execution path is currently:

`src/cli/signal.rs` (flag parsing) -> `src/engine/mod.rs` (dispatch) -> `src/engine/signal.rs` (`run`) -> `src/waveform/mod.rs` (`signals_in_scope`) -> `src/output.rs` (human or JSON rendering).

Important terms used in this plan:

Canonical absolute signal path means the full dot-separated path stored in waveform hierarchy, for example `top.cpu.valid`. This is the value that must remain in JSON `data[*].path`.

Relative display path means a human-facing path with the selected `--scope` prefix removed, for example scope `top` plus signal `top.cpu.valid` displays as `cpu.valid`.

Recursion depth means how many child-scope edges below `--scope` are allowed during traversal. Depth `0` means only the selected scope itself. Depth `1` includes direct child scopes, and so on.

Current code behavior relevant to this feature:

- `src/cli/signal.rs` has no recursive/depth flags yet.
- `src/engine/signal.rs` always calls non-recursive `Waveform::signals_in_scope`.
- `src/waveform/mod.rs` already has deterministic scope DFS machinery used by `scope`, but signal lookup is only per-scope.
- `src/output.rs` uses `SignalEntry.name` for human non-abs output and `SignalEntry.path` for `--abs`.
- `tests/signal_cli.rs` already covers JSON shape, sorting determinism, `--max` warning, regex filtering, and scope errors for non-recursive behavior.

## Open Questions

No blocking questions remain for implementation.

The plan intentionally resolves prior ambiguities through the Decision Log so implementation can proceed autonomously without user interruption.

## Plan of Work

Milestone 1 captures expected behavior as failing tests before production edits. This milestone establishes contracts for recursive inclusion, depth bounding, relative-vs-absolute human rendering, JSON path stability, and invalid flag combinations. This milestone must include a deep test fixture with at least one grandchild scope so `--max-depth` values `0`, `1`, and `2` are independently provable. At the end of this milestone, tests should fail only because implementation is not yet updated.

Milestone 2 extends CLI argument parsing and engine orchestration. This includes adding `--recursive` and `--max-depth`, validating argument combinations, and preparing human display fields without changing JSON payload shape.

Milestone 3 introduces waveform-layer recursive signal collection under a selected scope with deterministic ordering and depth limits. This milestone keeps non-recursive behavior untouched and explicitly verifies recursive/non-recursive parity at depth `0`.

Milestone 4 updates help and docs to keep shipped contracts aligned (`src/cli/mod.rs`, `docs/DESIGN.md`, and user-facing examples where needed).

Milestone 5 adds E2E performance coverage for recursive `signal` in the existing benchmark harness (`bench/e2e/perf.py` and `bench/e2e/tests.json`), captures a baseline run on SCR1, and then runs full project gates and review protocol.

### Concrete Steps

Run all commands from `/workspaces/feat-signal-recursive`.

1. Add or update tests first (expected to fail before implementation).

   First, add a dedicated fixture with depth > 1 (required, not optional):

       tests/fixtures/hand/signal_recursive_depth.vcd

   The fixture must include this minimum hierarchy shape:

       top
       top.cpu
       top.cpu.core

   with at least one signal in each visited scope so `--max-depth 0/1/2` produce distinct result sets.

   Then add explicit tests with stable names and run them with exact matching:

       cargo test --test signal_cli signal_recursive_human_mode_uses_relative_paths_from_scope -- --exact
       cargo test --test signal_cli signal_recursive_abs_mode_uses_canonical_paths -- --exact
       cargo test --test signal_cli signal_recursive_max_depth_zero_matches_non_recursive -- --exact
       cargo test --test signal_cli signal_recursive_max_depth_respects_grandchild_boundary -- --exact
       cargo test --test signal_cli signal_max_depth_requires_recursive_flag -- --exact
       cargo test --test signal_cli signal_recursive_filter_and_max_preserve_truncation_warning -- --exact
       cargo test --test signal_cli signal_recursive_filter_matches_name_not_relative_path -- --exact
       cargo test --test signal_cli signal_recursive_json_output_is_bit_for_bit_deterministic_across_runs -- --exact
       cargo test waveform::tests::recursive_signals_in_scope_respect_depth_boundaries -- --exact
       cargo test waveform::tests::recursive_signals_in_scope_are_deterministic_depth_first -- --exact
       cargo test cli::tests::signal_dispatch_keeps_recursive_and_max_depth_args -- --exact
       cargo test --test cli_contract signal_help_documents_recursive_and_max_depth_flags -- --exact

   Before implementation, each new test above must fail for the expected reason (missing flag, missing traversal, wrong display, or wrong validation). After implementation, all of them must pass.

   For every exact test command, verify cargo output includes `running 1 test` to avoid false-green runs caused by empty filter matches.

   Add/adjust tests in these files:

   - `tests/signal_cli.rs`
   - `src/waveform/mod.rs` (unit tests in `mod tests`)
   - `src/cli/mod.rs` (parser-forwarding test)
   - `tests/cli_contract.rs` (if help wording assertions are affected)

2. Implement CLI argument model and validation:

   - Update `src/cli/signal.rs` to include:

          pub recursive: bool
          pub max_depth: Option<usize>

   - Introduce one shared default-depth constant (single source of truth), for example in `src/engine/signal.rs`:

         const DEFAULT_RECURSIVE_MAX_DEPTH: usize = 5;

   - In `src/engine/signal.rs`, enforce:

         if args.max_depth.is_some() && !args.recursive {
             return Err(WavepeekError::Args("--max-depth requires --recursive. See 'wavepeek signal --help'.".to_string()));
         }

     and compute effective depth in recursive mode:

          let max_depth = args.max_depth.unwrap_or(DEFAULT_RECURSIVE_MAX_DEPTH);

3. Implement recursive traversal in waveform adapter:

   - Keep `signals_in_scope` behavior unchanged for non-recursive path.
   - Add a recursive collection API in `src/waveform/mod.rs`, for example:

         pub fn signals_in_scope_recursive(&self, scope_path: &str, max_depth: usize) -> Result<Vec<SignalEntry>, WavepeekError>

   - Reuse deterministic scope ordering (`sort_scope_refs`) and per-scope signal sort `(name, path)`.

4. Implement display behavior while preserving JSON shape:

   - Add a human-only display field to `src/engine/signal.rs::SignalEntry`:

         #[serde(skip_serializing)]
         pub display: String

   - In non-recursive mode set `display = name`.
   - In recursive mode set `display` to path relative to `--scope`.
   - Update `src/output.rs` signal renderer to use `display` when `!signals_abs`.

5. Update command help and product docs:

   - `src/cli/mod.rs` `Signal` long help must describe recursive behavior, depth semantics, and defaults.
   - `docs/DESIGN.md` section `3.2.3 signal` must include new flags and behavior bullets.
   - `README.md` quick-start/examples must include one recursive usage example because a new public flag is introduced.

6. Run focused functional validation:

       cargo test --test signal_cli
       cargo test waveform::tests::signals_in_scope_are_sorted_and_preserve_parser_var_type_aliases
       cargo test waveform::tests::missing_scope_returns_scope_category_error
       cargo test --test cli_contract

7. Add recursive `signal` E2E perf scenarios to `bench/e2e/tests.json`.

   Add these entries:

        {
          "name": "signal_scr1_top_recursive_all_json",
          "category": "signal",
          "runs": 10,
          "warmup": 2,
          "command": [
            "{wavepeek_bin}",
            "signal",
            "--waves", "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
            "--scope", "TOP",
            "--recursive",
            "--max", "200000",
            "--json"
          ],
          "meta": {
            "waves": "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
            "scope": "TOP",
            "filter": ".*",
            "recursive": true,
            "max_depth": "default"
          }
        }

        {
          "name": "signal_scr1_top_recursive_filter_valid_json",
          "category": "signal",
          "runs": 10,
          "warmup": 2,
          "command": [
            "{wavepeek_bin}",
            "signal",
            "--waves", "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
            "--scope", "TOP",
            "--recursive",
            "--filter", "(?i).*valid.*",
            "--max", "200000",
            "--json"
          ],
          "meta": {
            "waves": "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
            "scope": "TOP",
            "filter": "(?i).*valid.*",
            "recursive": true,
            "max_depth": "default"
          }
        }

        {
          "name": "signal_scr1_top_recursive_depth2_json",
          "category": "signal",
          "runs": 10,
          "warmup": 2,
          "command": [
            "{wavepeek_bin}",
            "signal",
            "--waves", "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
            "--scope", "TOP",
            "--recursive",
            "--max-depth", "2",
            "--max", "200000",
            "--json"
          ],
          "meta": {
            "waves": "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst",
            "scope": "TOP",
            "filter": ".*",
            "recursive": true,
            "max_depth": 2
          }
        }

8. Validate benchmark catalog and capture baseline perf run for new tests.

       python3 bench/e2e/perf.py list --filter '^signal_scr1_top_recursive_'
       cargo build --release
       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/signal-recursive-baseline --filter '^signal_scr1_top_recursive_'
       python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/signal-recursive-baseline

9. Run full quality validation and review protocol.

       make check
       make ci

10. Run review protocol:

   - Load `ask-review` skill and run review pass #1 via `review` subagent.
   - Provide in prompt: scope summary, changed files, test commands executed, test results, and known decisions from this plan.
   - Fix findings and re-run affected tests.
   - Start a fresh `review` session (new context) for pass #2.
   - Pass criterion: no new high/medium findings, or all findings resolved with rerun evidence.
   - Resolve any remaining findings and re-run affected tests plus final gate.

### Validation and Acceptance

Acceptance is behavior-based and must be observable from CLI output and tests:

- Without `--recursive`, `wavepeek signal` output is unchanged relative to current contract for the same fixture and flags.
- With `--recursive`, nested scope signals appear in deterministic order.
- With `--recursive --max-depth 0`, result set matches non-recursive result set for the same scope/filter.
- Human output in recursive mode defaults to relative paths from `--scope`.
- Human output with `--abs` always uses canonical absolute paths.
- JSON output in recursive mode keeps canonical absolute `path` values and existing envelope shape.
- `--max` truncation behavior and warning channel routing stay unchanged.
- `--max-depth` without `--recursive` is rejected as `error: args:` with help hint.
- E2E perf harness includes recursive `signal` scenarios so future regressions are observable in benchmark reports.

Reference CLI checks (expected fragments):

    wavepeek signal --waves tests/fixtures/hand/m2_core.vcd --scope top --recursive
    stdout contains: "cpu.valid kind=wire width=1"
    stdout contains: "mem.ready kind=wire width=1"

    wavepeek signal --waves tests/fixtures/hand/m2_core.vcd --scope top --recursive --abs
    stdout contains: "top.cpu.valid kind=wire width=1"
    stdout contains: "top.mem.ready kind=wire width=1"

    wavepeek signal --waves tests/fixtures/hand/signal_recursive_depth.vcd --scope top --recursive --max-depth 0 --json
    data must match non-recursive `wavepeek signal --waves ... --scope top --json`

    wavepeek signal --waves tests/fixtures/hand/m2_core.vcd --scope top --max-depth 1
    stderr starts with: "error: args: --max-depth requires --recursive"

Suggested integration assertions to include in `tests/signal_cli.rs`:

- recursive includes `cpu.valid` and `mem.ready` for scope `top` on `m2_core.vcd`.
- recursive `--abs` includes `top.cpu.valid` and `top.mem.ready`.
- recursive JSON rows still contain canonical absolute `path`.
- non-recursive vs recursive depth-0 parity check (same `data`, same warnings behavior under same `--max`).
- depth matrix check on `signal_recursive_depth.vcd`: `--max-depth 0`, `1`, and `2` each produce the expected inclusion boundary.
- invalid `--max-depth` usage without `--recursive` returns args error.
- `--recursive + --filter + --max` still emits truncation warning after filtering when bounded result is exceeded.
- recursive `--filter` is validated against `name` semantics (not relative display path) by a dedicated assertion.
- recursive output determinism is verified end-to-end (`run1.stdout == run2.stdout`, same for `stderr`).
- `wavepeek signal --help` explicitly documents `--recursive` and `--max-depth`, including the requirement relation.
- `bench/e2e/perf.py list --filter '^signal_scr1_top_recursive_'` returns exactly the three added benchmark names.

### Idempotence and Recovery

All edits in this plan are additive or local refactors and are safe to repeat.

If recursive traversal tests fail due to ordering drift, keep behavior deterministic by sorting child scopes and per-scope signals before attempting larger refactors.

If display-path logic introduces regressions, keep canonical `path` untouched and limit fixes to human-only `display` derivation.

If help/docs assertions fail after wording updates, update tests and docs together in one pass so contract and tests stay aligned.

If full gates are expensive during iteration, run focused tests first, then finish with `make check` and `make ci` before handoff.

Backlog tracking note: do not edit `docs/BACKLOG.md` during implementation commits for this feature branch. Backlog curation is handled when the feature is merged/released.

### Artifacts and Notes

Files expected to change during implementation:

    src/cli/signal.rs
    src/cli/mod.rs
    src/engine/signal.rs
    src/output.rs
    src/waveform/mod.rs
    tests/signal_cli.rs
    tests/cli_contract.rs
    tests/fixtures/hand/signal_recursive_depth.vcd
    docs/DESIGN.md
    README.md
    CHANGELOG.md
    bench/e2e/tests.json

The deep fixture is mandatory for this plan and should stay minimal (small hierarchy, explicit nested scopes, deterministic signal names).

### Interfaces and Dependencies

No new third-party dependencies are required.

Target interface updates:

In `src/cli/signal.rs`, extend `SignalArgs` with recursive controls:

    pub recursive: bool,
    pub max_depth: Option<usize>,

In `src/engine/signal.rs`, extend `SignalEntry` with human-only display text:

    #[serde(skip_serializing)]
    pub display: String,

In `src/waveform/mod.rs`, add recursive retrieval API while preserving current non-recursive API:

    pub fn signals_in_scope_recursive(
        &self,
        scope_path: &str,
        max_depth: usize,
    ) -> Result<Vec<SignalEntry>, WavepeekError>

Behavioral invariants to preserve:

- Scope lookup failures remain `WavepeekError::Scope`.
- Signal sorting remains deterministic.
- JSON schema compatibility remains intact (no required-field changes).
- Output remains bounded by `--max` with existing warning text pattern.

Revision Note: 2026-02-28 / OpenCode - Initial ExecPlan for backlog task "Add recursive signal listing (`signal --recursive`, `--max-depth`)" with explicit depth semantics, deterministic ordering policy, TDD-first milestones, and mandatory dual-review execution protocol.
Revision Note: 2026-02-28 / OpenCode - Incorporated review pass #1 findings by making deep-fixture depth validation mandatory, replacing ambiguous test filters with exact test commands, adding concrete expected CLI output fragments, formalizing review execution criteria, and removing subjective backlog-update wording.
Revision Note: 2026-02-28 / OpenCode - Incorporated independent review pass #2 finding by hardening exact-test commands with explicit test targets, adding a mandatory `running 1 test` check against empty-filter false positives, introducing a shared default-depth constant to avoid contract drift, and adding explicit tests for recursive filter semantics, recursive determinism, and signal help coverage.
Revision Note: 2026-02-28 / OpenCode - Added Milestone 5 benchmark coverage with three SCR1 recursive-signal E2E perf scenarios (`all`, `filter valid`, `max-depth 2`) plus catalog validation and baseline run capture commands.
