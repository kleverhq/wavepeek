# Finish `change` Auto-Dispatcher Tail Work

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

Users already get sub-second `wavepeek change` runtime in the current `dev-baseline` campaign, but the remaining slow tail is concentrated in a small, repeatable set of auto-dispatch choices. After this plan is implemented, those tail cases should use a better internal engine automatically, without changing CLI behavior or output contract. This is not a "lower one constant" change; dispatcher routing should be based on expected work (trigger shape plus estimated candidate work), while keeping safety gates for tiny workloads where switching engines adds overhead. The result should be visible in the benchmark report: the top slow `change` rows in `bench/e2e/runs/dev-baseline/README.md` should drop materially, while correctness and parity tests remain unchanged.

## Non-Goals

This plan does not change command-line flags, output formatting, warning text, JSON envelope, or benchmark harness semantics. This plan does not add persistent caching, daemons, environment toggles, or new runtime dependencies. This plan does not optimize `at`, `info`, `scope`, `signal`, `when`, or schema generation.

## Progress

- [x] (2026-03-03 21:02Z) Loaded `exec-plan` skill and captured repository orientation for `change` dispatch path (`src/engine/change.rs`, `src/cli/change.rs`, `src/waveform/mod.rs`) plus benchmark anchor `bench/e2e/runs/dev-baseline/README.md`.
- [x] (2026-03-03 21:11Z) Reproduced dispatcher-mode evidence with direct mode forcing (`auto`, `pre-fusion`, `edge-fast`, `fused`) on representative tail tests; confirmed large gaps are dispatch-driven, not contract-driven.
- [x] (2026-03-03 21:25Z) Drafted this standalone implementation plan with concrete commands, acceptance gates, and rollback/retry instructions using `bench/e2e/runs/dev-baseline` as the baseline.
- [x] (2026-03-03 21:39Z) Refined plan for execution clarity after review: fixed TDD step ordering, added explicit target improvement thresholds, and defined dispatch terminology for novice implementers.
- [x] (2026-03-03 21:51Z) Clarified dispatcher strategy after user review: route by expected work and trigger shape, not by blunt threshold lowering; added explicit low-work canary expectations.
- [x] (2026-03-03 19:10Z) Implemented auto-dispatch heuristic updates in `src/engine/change.rs` with explicit work-estimate helpers and expanded `auto_engine_mode_*` unit tests (including red/green TDD evidence).
- [x] (2026-03-03 19:31Z) Added integration-level auto/forced parity guards in `tests/change_opt_equivalence.rs` for dense and sparse AnyTracked/edge profiles, asserting `data` + `warnings` equality.
- [x] (2026-03-03 19:52Z) Ran correctness gates: `cargo test auto_engine_mode`, `cargo test --test change_opt_equivalence`, `cargo test --test change_cli`, `cargo test --test change_vcd_fst_parity`, `make check`, and `make ci`.
- [x] (2026-03-03 20:03Z) Captured benchmark evidence in `bench/e2e/runs/change-dispatcher-tail-final` and `bench/e2e/runs/change-dispatcher-matrix-final`; focused compare passes after outlier reruns, broad `^change_` compare still fails (see discoveries), so plan remains active.
- [x] (2026-03-03 20:04Z) Re-ran focused target metrics to confirm named closure rows are still >30% improved and low-work canaries remain within guardrails.

## Surprises & Discoveries

- Observation: The current long tail is dominated by `signal_count=10` `change` tests, not by `signal_count=100` tests.
  Evidence: `bench/e2e/runs/dev-baseline/README.md` top rows include `change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk` at about `0.943s`, while many `signal_count=100` rows are around `0.24-0.42s`.

- Observation: For multiple heavy `signal_count=10` cases, forcing `fused` or `edge-fast` gives multi-x wins over `auto`, which means the remaining issue is dispatcher selection, not missing engine capability.
  Evidence: direct local measurements show roughly `0.93s` (`auto`) vs `~0.22s` (`edge-fast`) and `~0.15s` (`fused`) for `change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk`.

- Observation: The `edge-fast` path already has an internal workload gate and automatic fallback to `pre-fusion`, so broadening edge auto-dispatch can stay low risk.
  Evidence: `src/engine/change.rs` `run_edge_fast` checks `EDGE_FAST_MIN_WORK` and returns `run_prefusion(...)` when work is below threshold.

- Observation: The baseline directory rename (`dev-eval` -> `dev-baseline`) preserved files, but `README.md` still shows old run metadata strings.
  Evidence: `bench/e2e/runs/dev-baseline/README.md` header still names run `dev-eval` and prints old run directory text.

- Observation: For clustered/dense edge-trigger tails, forced `fused` is consistently faster than forced `edge-fast`, while preserving output parity.
  Evidence: local hyperfine checks on chipyard clustered/dualrocket `signals_10 window_32us trigger_posedge_clk` show `fused` around `0.15-0.17s` vs `edge-fast-force` around `0.22-0.23s` and `auto/pre-fusion` around `0.56-0.94s` before rerouting.

- Observation: Broad benchmark stability is noisy and bimodal on several rows (especially clustered chipyard and some scr1 short-window rows), which can swing compare outcomes by more than the `5%` gate.
  Evidence: repeated reruns of the same row alternate between low `~0.24s` and high `~0.29s` clusters in hyperfine artifacts under identical commands.

- Observation: Broad `^change_` compare currently remains red even after focused wins and targeted reruns.
  Evidence: latest failing set includes (among others) `change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_any`, `change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_signal`, and `change_scr1_coremark_imem_axi_1sig_to_1000ps` with negative deltas beyond `-5%`.

## Decision Log

- Decision: Focus this plan on auto-dispatch logic only, without changing engine semantics.
  Rationale: Existing forced-mode parity tests already lock behavior; dispatch-only work gives the highest performance leverage with the least product risk.
  Date/Author: 2026-03-03 / OpenCode

- Decision: Use `bench/e2e/runs/dev-baseline` as the sole golden reference for this plan, even before baseline refresh.
  Rationale: The user explicitly requested this and it keeps all acceptance checks anchored to one comparable run.
  Date/Author: 2026-03-03 / OpenCode

- Decision: Keep broad perf acceptance as two layers: focused tail filter and full `^change_` filter.
  Rationale: Focused checks prove the target win, while broad checks prevent accidental regressions outside the target set.
  Date/Author: 2026-03-03 / OpenCode

- Decision: Replace "single count cutoff" thinking with workload-aware dispatch guidance.
  Rationale: A lower global signal-count threshold can help some tail cases but harms low-work windows; expected-work gates preserve wins while avoiding unnecessary engine-switch overhead.
  Date/Author: 2026-03-03 / OpenCode

- Decision: Route edge-only auto workloads to `fused` for medium/high estimated work, and reserve auto `edge-fast` only for ultra-high estimated edge work.
  Rationale: Measurement showed `fused` outperforming `edge-fast` on the key clustered/dualrocket edge tails while preserving parity; keeping an ultra-high `edge-fast` branch avoids dead-path drift and retains coverage for extreme edge workloads.
  Date/Author: 2026-03-03 / OpenCode

## Outcomes & Retrospective

Current status: implementation complete for dispatcher + tests + focused evidence, but plan is intentionally kept active because broad `^change_` compare still reports regressions beyond the `5%` gate.

What worked: correctness parity stayed intact across unit/integration/project gates, and focused tail targets materially improved (named closure rows exceed `30%` by large margins in current artifacts).

What did not close: broad matrix compare remains unstable/regressive in this environment; several rows drift above the allowed negative delta even after reruns, so closure criteria are not met yet.

Next iteration scope: tune/validate broad-distribution stability without sacrificing focused tail wins, then rerun broad compare until `info: compare: all checks passed` is reproducible.

## Context and Orientation

The user-facing command is `wavepeek change`. It has internal engine modes that are hidden from normal users but available for testing and diagnostics: `pre-fusion`, `fused`, and `edge-fast`, plus auto mode. Auto mode is selected in `src/engine/change.rs` by `select_engine_mode(...)`.

In this repository, a "candidate timestamp" means a dump timestamp where at least one relevant signal changed and the command may need to evaluate trigger and delta logic. "Dispatch" means choosing which internal engine executes the same command contract. "Tail" means the slowest remaining benchmark rows after previous optimizations.

In this plan, "AnyTracked-only terms" means the trigger expression is exactly wildcard change tracking (`--when "*"`) with no unioned edge or named terms. "Edge-only terms" means every trigger term is one of `posedge <sig>`, `negedge <sig>`, or `edge <sig>`. "Dense fixture" means a test window where many candidate timestamps are present and trigger checks happen often. "Sparse fixture" means the opposite: only a few candidate timestamps in the selected window.

Relevant files:

- `src/cli/change.rs`: hidden internal flags (`--internal-change-engine`, `--internal-change-candidates`, `--internal-change-edge-fast-force`) used for forced-mode testing.
- `src/engine/change.rs`: auto-dispatch logic (`select_engine_mode`) and engine implementations (`run_prefusion`, `run_fused`, `run_edge_fast`).
- `src/waveform/mod.rs`: signal loading, candidate collection, and offset/decode helpers consumed by all engines.
- `tests/change_opt_equivalence.rs`: forced-mode parity tests that compare output equality between engines.
- `tests/change_vcd_fst_parity.rs`: cross-format behavior guard.
- `bench/e2e/tests.json`: benchmark matrix definition.
- `bench/e2e/perf.py`: benchmark runner and compare gate used for evidence and regression detection.
- `bench/e2e/runs/dev-baseline/`: baseline artifacts for this plan.

## Open Questions

No product-level open questions are blocking implementation. The remaining choices are technical threshold tuning choices that this plan resolves with measurements.

## Plan of Work

Milestone 1 tightens dispatcher unit behavior with test-first changes in `src/engine/change.rs`. First, add failing unit tests that encode the desired auto-routing behavior for representative trigger shapes and signal-count bands, including edge-only terms and AnyTracked-only terms below the old `32` threshold. Then update `select_engine_mode(...)` and any small helper predicates so auto mode routes by workload estimate (for example candidate-count-informed work) and trigger shape, not by one global signal-count cutoff. Keep all explicit forced modes unchanged.

Routing target matrix for this plan (required behavior for new unit tests):

- AnyTracked-only terms with high estimated work must route to `fused`.
- Edge-only terms with medium/high estimated work should route to `fused`; only ultra-high estimated edge work should route to `edge-fast` (runtime fallback inside `run_edge_fast` remains preserved).
- Low estimated work (including tiny windows and sparse candidate ranges) must stay on `pre-fusion` to avoid engine-switch overhead.

Use one explicit helper in dispatcher code for the work estimate so tests can target it predictably; avoid scattering magic constants across multiple branches.

Milestone 2 extends integration parity checks in `tests/change_opt_equivalence.rs` so the new auto routes are covered by behavior tests, not only unit tests. Add cases that exercise auto mode against forced-mode outputs on dense and sparse fixtures for both edge and non-edge trigger shapes. These tests must assert equality of both `data` and `warnings` and should reuse the helper pattern already present in this file.

Milestone 3 validates correctness and performance together. Run targeted tests first, then project gates, then benchmark runs against `bench/e2e/runs/dev-baseline`. The first benchmark pass should focus on known tail rows; the second pass should run full `^change_` to catch collateral regressions. If broad compare fails while focused compare passes, iterate dispatch thresholds without changing output semantics.

Milestone 4 finalizes documentation and release readiness. Update this plan's living sections with exact results, including any regressions encountered and the final selected thresholds. If acceptance criteria are met, move plan to `completed`; if not, keep active and document the next iteration scope explicitly.

### Concrete Steps

Run all commands from `/workspaces/perf-change`.

1. Add unit tests for auto-dispatch decisions before implementation edits.

   Add new tests in `src/engine/change.rs` first, with explicit names that describe the intended new behavior, for example:

   - `auto_engine_mode_uses_fused_for_mid_any_tracked_only`
   - `auto_engine_mode_uses_edge_fast_for_mid_edge_only_terms`
   - `auto_engine_mode_keeps_prefusion_for_low_work_any_tracked_only`

   Then run:

       cargo test auto_engine_mode

   Expected before dispatcher edits: at least one newly added high-work routing test fails because current auto mode still routes those cases to `pre-fusion`.

2. Implement dispatcher updates in `src/engine/change.rs` and re-run unit tests.

       cargo test auto_engine_mode

   Expected after edits: all `auto_engine_mode*` tests pass.

3. Add integration parity cases in `tests/change_opt_equivalence.rs` and run targeted integration suites.

       cargo test --test change_opt_equivalence
       cargo test --test change_cli
       cargo test --test change_vcd_fst_parity

4. Run project quality gates.

       make check
       make ci

5. Build release binary for benchmark evidence.

       cargo build --release

6. Capture focused tail benchmark evidence versus dev baseline.

       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-dispatcher-tail-final --filter '^change_.*signals_10_.*(trigger_any|trigger_posedge_clk)$' --compare bench/e2e/runs/dev-baseline
       python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-dispatcher-tail-final --golden bench/e2e/runs/dev-baseline --max-negative-delta-pct 5

7. Capture broad `change` regression evidence versus dev baseline.

       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-dispatcher-matrix-final --filter '^change_' --compare bench/e2e/runs/dev-baseline
       python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-dispatcher-matrix-final --golden bench/e2e/runs/dev-baseline --max-negative-delta-pct 5

8. Regenerate report markdown for both run directories (idempotent, useful after reruns).

       python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/change-dispatcher-tail-final --compare bench/e2e/runs/dev-baseline
       python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/change-dispatcher-matrix-final --compare bench/e2e/runs/dev-baseline

9. Compute explicit win percentages for the named tail targets and record them in this plan.

       python3 - <<'PY'
       import json
       from pathlib import Path

       base = Path('bench/e2e/runs/dev-baseline')
       rev = Path('bench/e2e/runs/change-dispatcher-tail-final')
       targets = [
           'change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk',
           'change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any',
           'change_picorv32_signals_10_window_32us_trigger_any',
       ]
       for t in targets:
           b = json.loads((base / f'{t}.hyperfine.json').read_text())['results'][0]['mean']
           r = json.loads((rev / f'{t}.hyperfine.json').read_text())['results'][0]['mean']
           delta = ((float(b) - float(r)) / float(b)) * 100.0
           print(f'{t}: improvement_pct={delta:.2f}')
       PY

10. Verify low-work canary rows remain stable (no broad "over-dispatch" side effect).

       python3 - <<'PY'
       import json
       from pathlib import Path

       base = Path('bench/e2e/runs/dev-baseline')
       rev = Path('bench/e2e/runs/change-dispatcher-matrix-final')
       canaries = [
           'change_scr1_signals_1_window_2ns_trigger_any',
           'change_scr1_signals_1_window_4ns_trigger_any',
           'change_scr1_signals_10_window_2ns_trigger_any',
       ]
       for t in canaries:
           b = json.loads((base / f'{t}.hyperfine.json').read_text())['results'][0]['mean']
           r = json.loads((rev / f'{t}.hyperfine.json').read_text())['results'][0]['mean']
           delta = ((float(b) - float(r)) / float(b)) * 100.0
           print(f'{t}: improvement_pct={delta:.2f}')
           if delta < -5.0:
               raise SystemExit(f'canary regression exceeds 5% for {t}: {delta:.2f}%')
       PY

### Validation and Acceptance

Correctness acceptance is satisfied only when these all hold together: unit tests in `src/engine/change.rs` for auto-dispatch pass, `tests/change_opt_equivalence.rs` passes with added auto-route coverage, and both `tests/change_cli.rs` and `tests/change_vcd_fst_parity.rs` remain green without output drift.

Performance acceptance is satisfied only when both focused and broad evidence pass:

- Focused tail run (`change-dispatcher-tail-final`) must show clear reductions in the known top tail rows from `dev-baseline`, especially the `signals_10` chipyard and picorv32 cases that currently sit around `0.59-0.94s`.
- Required closure gate for this plan: each named target below must improve by at least `30%` versus `dev-baseline` (measured by hyperfine mean):
  - `change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk`
  - `change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any`
  - `change_picorv32_signals_10_window_32us_trigger_any`
- Focused compare command must pass `--max-negative-delta-pct 5`.
- Broad `^change_` compare command must also pass `--max-negative-delta-pct 5`.
- Low-work canary expectation: these rows should not regress by more than `5%` against `dev-baseline`:
  - `change_scr1_signals_1_window_2ns_trigger_any`
  - `change_scr1_signals_1_window_4ns_trigger_any`
  - `change_scr1_signals_10_window_2ns_trigger_any`

Expected pass signature for both compare commands:

    info: compare: all checks passed

TDD acceptance is satisfied only when the newly added `auto_engine_mode_*` tests fail before dispatcher edits and pass after dispatcher edits in the same branch history.
TDD acceptance must include at least one low-work routing test that asserts auto mode remains `pre-fusion` for an AnyTracked low-work profile.

If focused wins are present but broad compare fails, do not close the plan. Keep the safest dispatcher changes, record failing rows in `Surprises & Discoveries`, and iterate thresholds in a follow-up milestone within this same plan.

### Idempotence and Recovery

All benchmark commands are idempotent when reusing the same `--run-dir`: artifacts are overwritten deterministically and `report` can be rerun at any time. If a benchmark run is interrupted, rerun the same command with `--missing-only` to fill missing artifacts safely.

Dispatcher edits are isolated to `src/engine/change.rs`, so rollback is straightforward: revert only the dispatcher-selection hunk while keeping tests that demonstrate expected behavior, then rerun targeted suites to confirm baseline parity.

### Artifacts and Notes

Expected code/test edits for implementation:

- `src/engine/change.rs`
- `tests/change_opt_equivalence.rs`

Expected evidence artifacts after execution:

- `bench/e2e/runs/change-dispatcher-tail-final/README.md`
- `bench/e2e/runs/change-dispatcher-matrix-final/README.md`

Representative current evidence (pre-implementation, from local mode-forcing diagnostics):

    change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_posedge_clk
      auto            ~0.93s
      edge-fast-force ~0.22s
      fused           ~0.15s

    change_chipyard_dualrocketconfig_dhrystone_signals_10_window_32us_trigger_any
      auto            ~0.93s
      fused           ~0.15s

These diagnostics justify dispatcher work as the primary remaining lever.

### Interfaces and Dependencies

No new dependencies are allowed.

Implementation must keep these interfaces and invariants intact:

- `crate::engine::change::run` public behavior and `CommandResult` contract remain unchanged.
- Hidden CLI flags in `crate::cli::change::ChangeArgs` continue to force explicit modes exactly as today.
- `crate::waveform` APIs remain stateless per invocation; no environment-variable toggles or cross-run caches are introduced.
- `docs/DESIGN.md` `change` command semantics remain true: strict predecessor delta behavior, same warning text, same JSON envelope shape.

Revision Note: 2026-03-03 / OpenCode - Created a standalone active ExecPlan focused on finishing `change` auto-dispatch tuning using `bench/e2e/runs/dev-baseline` as the starting baseline per user request.
Revision Note: 2026-03-03 / OpenCode - Updated plan after review with corrected red/green test ordering, explicit quantitative closure gates for named tail targets, and additional plain-language definitions for dispatch terms.
Revision Note: 2026-03-03 / OpenCode - Clarified that dispatcher work is workload-aware routing (not simple threshold lowering), and added explicit low-work canary checks to preserve the purpose of safety gates.
Revision Note: 2026-03-03 / OpenCode - Implemented dispatcher/test milestones, recorded focused benchmark wins and broad-compare instability, and updated routing guidance to reflect measured edge-only behavior (`fused` default, `edge-fast` for ultra-high work only).
