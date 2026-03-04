# Eliminate `at` Performance Anomaly on High-Activity Signal Sets

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Users currently observe a counterintuitive result: `wavepeek at` for one timestamp can be much slower than `wavepeek change` over a time window on the same dataset class. In baseline evidence, `at_picorv32_signals_1000` is about `2.11s`, while top slow `change` rows are about `0.53-0.59s`. This is surprising because `at` asks for one point in time and should be the lower-cost operation.

After this plan is implemented, `at` keeps the exact existing contract (same output order, same duplicate behavior, same errors/warnings, same JSON shape) but removes redundant internal work in the sampling path. The result is visible by rerunning the `at` benchmark subset: `at_picorv32_signals_1000` should drop materially and no correctness tests should change.

Expected performance payoff for the target anomaly (after mandatory attribution step, not by assumption):

- conservative: `2.0x-2.8x` speedup from removing duplicate-amplified resolve/load/decode work;
- target: `>=3.5x` speedup (`2.11s -> <=0.60s`), making `at_picorv32_signals_1000` no slower than the current `change` long-tail rows;
- stretch: `~4.0x` (`<=0.53s`) so `at` clearly beats current slowest `change` baseline rows.

## Non-Goals

This plan does not rename commands or flags (`at`/`--time` remain unchanged).

This plan does not alter `at` output semantics, including duplicate signal order preservation and fail-fast behavior.

This plan does not modify `change` logic, dispatcher heuristics, or `--when` semantics.

This plan does not introduce persistent caching, background daemons, or cross-invocation state.

This plan does not replace the benchmark harness. `bench/e2e/perf.py` remains the source of truth for perf evidence.

## Progress

- [x] (2026-03-04 10:30Z) Loaded `exec-plan` skill and mapped canonical backlog requirement in `docs/BACKLOG.md` for `at` anomaly closure criteria.
- [x] (2026-03-04 10:30Z) Collected baseline evidence and context: `at_picorv32_signals_1000` (`2.111695s`) versus top `change` tail (`~0.53-0.59s`) from `bench/e2e/runs/baseline/README.md`.
- [x] (2026-03-04 10:30Z) Quantified signal-list duplication in benchmark catalog: `at_picorv32_signals_1000` has `1000` requested tokens but only `419` unique paths.
- [x] (2026-03-04 10:30Z) Drafted this executable implementation plan with explicit milestones, acceptance gates, and rollback strategy.
- [x] (2026-03-04 10:34Z) Completed review pass #1 on the plan and incorporated fixes: made duplication a tested hypothesis (not assumed root cause), fixed acceptance command paths, and replaced pseudo-command review steps with explicit workflow instructions.
- [x] (2026-03-04 10:37Z) Completed independent review pass #2 on the plan after follow-up fixes (matrix compare gate + plan-file commit staging); no new substantive findings.
- [x] (2026-03-04 12:05Z) Executed Milestone 1 baseline capture (`at-anomaly-golden`, `at-anomaly-golden-hot`) and attribution probe artifacts.
- [x] (2026-03-04 12:20Z) Executed Milestone 2 TDD red/green for duplicate-preserving projection (`duplicate_projection_deduplicates_paths_and_tracks_requested_order` failed before implementation, then passed).
- [x] (2026-03-04 12:40Z) Executed Milestone 3 implementation in `src/waveform/mod.rs`: duplicate-aware projection/reconstruction and per-call load dedup in `ensure_signals_loaded`.
- [x] (2026-03-04 13:30Z) Executed Milestone 4 pivot: added FST-only streaming point-sampling path with guarded heuristic after attribution showed duplicates were not dominant.
- [x] (2026-03-04 14:15Z) Executed Milestone 5 validation (`at` correctness suites, `make check`, `make ci`) and benchmark runs (`at-anomaly-final`, `at-anomaly-matrix`).
- [x] (2026-03-04 15:05Z) Completed Milestone 6 collateral update (`CHANGELOG.md`) and recorded hard-gate miss rationale + required follow-up milestone in this plan.
- [x] (2026-03-04 15:15Z) Completed mandatory review pass #1; fixed fallback + heuristic findings and committed follow-up patch.
- [x] (2026-03-04 15:25Z) Completed mandatory independent review pass #2 from fresh context; no substantive findings.
- [x] (2026-03-04 16:20Z) Simplified implementation per user direction: removed `at` streaming point-sampling path and threshold heuristics, kept duplicate-preserving projection + per-call ref dedup.
- [x] (2026-03-04 16:35Z) Re-ran validation after simplification (`make check`, `make ci`, targeted `at`/`change` suites); all gates passed.
- [x] (2026-03-04 16:45Z) Re-ran mandatory review cycle for simplification: pass #1 clean, independent pass #2 flagged change-path loader regression risk, fixed by restoring unconditional FST multi-threaded load in shared loader path, then re-check clean.

## Surprises & Discoveries

- Observation: The backlog anomaly is real and currently severe: a small (13M) picorv32 FST case is the slowest `at` row.
  Evidence: `bench/e2e/runs/baseline/README.md` lists `at_picorv32_signals_1000 = 2.111695s`, slower than larger chipyard `at` rows (`~0.91-0.94s`).

- Observation: The target benchmark input has heavy duplicate request tokens, which is a strong optimization hypothesis but not yet proof of primary root cause.
  Evidence: `bench/e2e/tests.json` + script measurement: `at_picorv32_signals_1000` has `1000` tokens, `419` unique, `581` duplicates.

- Observation: `at` path currently routes through `Waveform::sample_signals_at_time`, which resolves/loads/decodes per requested token order and then returns values in that same order.
  Evidence: `src/engine/at.rs` calls `sample_signals_at_time` with full `canonical_paths`; `src/waveform/mod.rs` `sample_signals_at_time` currently resolves and samples directly from that vector.

- Observation: `ensure_signals_loaded` currently filters against previously loaded global state only, so duplicate signal refs inside one call can still enter `to_load` repeatedly.
  Evidence: `src/waveform/mod.rs` `ensure_signals_loaded` builds `to_load` from input refs before extending `loaded_signals`.

- Observation: Attribution probe disproved duplicate-token dominance for the anomaly case.
  Evidence: `at_picorv32_signals_1000_raw` vs `..._unique419` means in `bench/e2e/runs/at-anomaly-golden-hot/*.hyperfine.json` were both ~`2.03-2.05s` (delta far below 30% pivot threshold).

- Observation: Runtime is overwhelmingly dominated by signal-load work in `sample_resolved_optional`, while decode/format are negligible.
  Evidence: instrumented stage timings during implementation showed `load ~2.01s`, `decode ~0.0003s` for `resolved=419` at the anomaly query point.

- Observation: After removing streaming/heuristic branches for simplicity, anomaly performance returns close to baseline and still misses closure targets.
  Evidence: `bench/e2e/runs/at-anomaly-simplified-mt/README.md` reports `at_picorv32_signals_1000 = 2.094s` (`speedup_x=1.008` vs baseline `2.111695s`), with `faster=False` against baseline change tail (`0.591289s`).

## Decision Log

- Decision: Keep optimization contract-preserving and treat duplicate ordering as a hard invariant.
  Rationale: `tests/at_cli.rs` and waveform tests explicitly lock duplicate-order behavior; product requirement says semantics must remain unchanged.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Treat duplicate-amplified work as a first hypothesis, then enforce a mandatory attribution gate before locking implementation scope.
  Rationale: `docs/BACKLOG.md` states duplicates are not the primary known contributor; we must measure attribution before committing to one optimization path.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Use two-tier perf closure: hard target (`>=3.5x`) plus matrix regression guard (`<=5%` negatives).
  Rationale: We need to satisfy the user expectation that `at` should not be slower than equivalent heavy `change` rows while preventing collateral regressions.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Keep duplicate-preserving projection and per-call load dedup as mandatory parity-safe baseline optimization even after attribution showed limited payoff.
  Rationale: These changes remove known redundant work, are covered by explicit tests, and preserve external `at` contract semantics.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Add a guarded FST streaming point-sampling path only for mid-size unique-signal queries with sparse timestamp stride characteristics.
  Rationale: Unconditional stream path regressed chipyard `signals_1000`; guarded heuristic preserved matrix guard while improving the anomaly row.
  Date/Author: 2026-03-04 / OpenCode

- Decision: Roll back `at` streaming point-sampling and threshold heuristics to reduce complexity/entropy, while retaining low-risk duplicate-preserving projection and per-call dedup.
  Rationale: User requested simplification; measured ROI of heuristic path was insufficient for added complexity and did not close hard gate.
  Date/Author: 2026-03-04 / OpenCode

## Outcomes & Retrospective

Current status: implementation and validation complete for Milestones 1-5; hard performance gate is not met.

Delivered outcome: contract-preserving simplification landed with `at` now using the straightforward random-access sampling path plus duplicate-preserving projection/per-call dedup; experimental streaming heuristics were removed.

Measured closure status: `at_picorv32_signals_1000` is `2.094s` in simplified run (`speedup_x=1.008` from baseline `2.111695s`), which does not satisfy hard target (`>=3.5x`) or cross-command sanity (`faster=False` against baseline change tail).

Dominant residual cost: signal loading/materialization in `sample_resolved_optional` remains the hotspot.

Required follow-up milestone (plan remains open): design and implement a dedicated point-sampling backend for FST that can obtain value-at-or-before-time without full history materialization for all selected signals.

## Context and Orientation

`wavepeek at` is implemented in `src/engine/at.rs`. It parses/validates `--time`, resolves requested signal paths, and asks `Waveform` to sample values at one raw timestamp. The waveform adapter lives in `src/waveform/mod.rs`.

The relevant runtime path today is:

1. `src/engine/at.rs::run` builds `canonical_paths` from requested tokens and calls `Waveform::sample_signals_at_time`.
2. `src/waveform/mod.rs::sample_signals_at_time` resolves paths to signal refs, then calls `sample_resolved_optional`.
3. `sample_resolved_optional` computes one time-table index, ensures signals are loaded, then decodes each resolved signal at that index.

Important terms used in this plan:

- Duplicate-preserving projection: an internal mapping from original requested order (which may contain duplicates) to a deduplicated unique-signal list used for expensive backend work. Final output is reconstructed back into original order.
- Time-table index: the integer index in waveform timestamp table corresponding to `floor(query_time_raw)`. The value at this index is used to sample “latest value at or before time”.
- Contract parity: identical externally visible behavior for `at` (stdout/stderr/JSON/errors), including duplicate entries, canonical path fields, and fail-fast rules.

Key files for this plan:

- `docs/BACKLOG.md` (anomaly requirement and close condition).
- `bench/e2e/tests.json` and `bench/e2e/perf.py` (perf scenarios and harness).
- `bench/e2e/runs/baseline/README.md` (current baseline evidence).
- `src/engine/at.rs` (at runtime and time parsing contracts).
- `src/waveform/mod.rs` (signal resolution, loading, point sampling internals).
- `tests/at_cli.rs` (integration behavior, including duplicate-order contract).
- `CHANGELOG.md` (user-visible release note for delivered perf fix).

## Open Questions

No product-level blockers remain.

Technical question to resolve during Milestone 3 if hard perf gate is not reached after duplicate-aware projection:

- whether to add a second optimization pass for `at` that minimizes allocation/formatting overhead in result materialization (still contract-preserving);
- whether to introduce a dedicated benchmark-only micro-measurement helper under `bench/e2e/` for stage-level timing attribution.

## Plan of Work

Milestone 1 establishes reproducible baseline evidence dedicated to the anomaly and captures request-shape diagnostics (duplicate ratio, unique-count distribution). It also runs a mandatory attribution probe (original 1000-token list vs deduplicated unique list) so optimization direction is data-driven and aligned with the backlog statement about likely root cause.

Milestone 2 applies TDD for new internal behavior. First, add tests that define duplicate-preserving projection and deduplicated loading behavior. These tests must fail before implementation (red phase), then pass after changes (green phase). Existing `at` integration tests remain the contract guard.

Milestone 3 implements the optimization. The core idea is to perform expensive waveform operations only once per unique signal path while preserving the output vector shape/order expected by existing contracts. This includes deduplicating per-call load requests in `ensure_signals_loaded` and reconstructing sampled results back to requested order.

Milestone 4 is a pivot milestone. If Milestone 3 gains are below hard target, add a second optimization pass that addresses remaining dominant cost (load/decode/allocation) identified by attribution evidence while preserving contract parity.

Milestone 5 validates correctness and performance. Run targeted tests, full quality gates, then benchmark compares against both plan-local golden and shared baseline. Close only when hard performance and regression gates are satisfied.

Milestone 6 updates collateral (`CHANGELOG.md`) with measured user-visible improvement, then executes mandatory two-pass review discipline.

### Concrete Steps

Run all commands from `/workspaces/perf-at-anomaly`.

1. Capture plan-local golden run for all `at` rows and focused anomaly subset.

       cargo build --release
       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/at-anomaly-golden --filter '^at_' --compare bench/e2e/runs/baseline
       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/at-anomaly-golden-hot --filter '^at_.*signals_1000$' --compare bench/e2e/runs/baseline
       python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/at-anomaly-golden --compare bench/e2e/runs/baseline

   Run attribution probe (duplicate contribution check) before implementation:

       python3 - <<'PY'
       import json
       import subprocess
       from pathlib import Path

       tests = json.loads(Path('bench/e2e/tests.json').read_text())['tests']
       target = next(t for t in tests if t['name'] == 'at_picorv32_signals_1000')
       cmd = target['command']
       sig_idx = cmd.index('--signals') + 1
       time_idx = cmd.index('--time') + 1
       all_signals = [s.strip() for s in cmd[sig_idx].split(',') if s.strip()]

       seen = set()
       unique = []
       for s in all_signals:
           if s not in seen:
               seen.add(s)
               unique.append(s)

       waves = cmd[cmd.index('--waves') + 1]
       time_token = cmd[time_idx]

       def run_case(name, signals):
           args = [
               './target/release/wavepeek', 'at',
               '--waves', waves,
               '--signals', ','.join(signals),
               '--time', time_token,
               '--json',
           ]
           out = subprocess.check_output(['hyperfine', '--runs', '6', '--warmup', '3', '--export-json', f'bench/e2e/runs/at-anomaly-golden-hot/{name}.hyperfine.json', '--command-name', name, ' '.join(args)], text=True)
           print(out)

       run_case('at_picorv32_signals_1000_raw', all_signals)
       run_case('at_picorv32_signals_1000_unique419', unique)
       print(f'raw_count={len(all_signals)} unique_count={len(unique)}')
       PY

   Attribution pivot rule:

   - If raw-vs-unique delta is small (`<30%`), duplicates are not dominant; Milestone 4 second pass is mandatory.
   - If raw-vs-unique delta is large (`>=30%`), proceed with duplicate-aware Milestone 3 first and reassess against hard target.

2. Add TDD red-phase tests before optimization edits.

   Add tests in `src/waveform/mod.rs` (unit tests near existing sampling tests) that lock:

   - duplicate-preserving projection materializes exactly the same output vector as current behavior;
   - per-call load dedup does not alter returned values or missing-signal errors;
   - mixed duplicate/non-duplicate paths still preserve order and canonical `path` fields.

   Add/adjust tests in `src/engine/at.rs` for any new projection helper used by `run`.

   Run:

       cargo test waveform::tests::sample_signals_at_time_preserves_order_and_duplicates
       cargo test --lib at::tests
       cargo test --test at_cli

   Expected before implementation is complete: at least one newly added projection/dedup test fails.

3. Implement duplicate-aware optimization.

   In `src/waveform/mod.rs`:

   - Refactor `sample_signals_at_time` to build a unique signal-path list plus projection indices, then sample/decode only unique resolved signals once per query index.
   - Reconstruct `Vec<SampledSignal>` in original requested order using projection indices.
   - Update `ensure_signals_loaded` to avoid duplicate refs in one call (while preserving cross-call cache behavior).

   In `src/engine/at.rs`:

   - Keep CLI/time validation and output formatting unchanged.
   - If helper extraction is required for projection planning, keep it internal and unit-tested.

   Run:

       cargo test --test at_cli
       cargo test waveform::tests::sample_signals_at_time_preserves_order_and_duplicates
       cargo test waveform::tests::sample_signals_at_time_uses_latest_change_before_timestamp
       cargo test waveform::tests::sample_signals_at_time_returns_signal_error_for_missing_path

4. Run full correctness/quality gates.

       make check
       make ci

5. If hard target is missed after step 3, execute second optimization pass based on measured dominant residual cost.

   Examples of allowed second-pass directions (choose one backed by evidence):

   - reduce remaining decode/allocation overhead for wide vectors while keeping output literals identical;
   - reduce residual load overhead for high-activity unique signals without changing sampling semantics;
   - split internal `at` hot path into explicit "resolve once / sample once / format once" stages with per-stage caches local to one invocation.

   Re-run targeted tests after second pass:

       cargo test --test at_cli
       cargo test waveform::tests::sample_signals_at_time_preserves_order_and_duplicates

6. Capture final perf evidence and compare.

       cargo build --release
       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/at-anomaly-final --filter '^at_' --compare bench/e2e/runs/at-anomaly-golden
       python3 bench/e2e/perf.py compare --revised bench/e2e/runs/at-anomaly-final --golden bench/e2e/runs/at-anomaly-golden --max-negative-delta-pct 5
       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/at-anomaly-matrix --filter '^at_' --compare bench/e2e/runs/baseline
       python3 bench/e2e/perf.py compare --revised bench/e2e/runs/at-anomaly-matrix --golden bench/e2e/runs/baseline --max-negative-delta-pct 5
       python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/at-anomaly-final --compare bench/e2e/runs/at-anomaly-golden

7. Compute explicit speedup and cross-command comparison for closure summary.

       python3 - <<'PY'
       import json
       from pathlib import Path

       baseline = Path('bench/e2e/runs/baseline')
       revised = Path('bench/e2e/runs/at-anomaly-final')

       target = 'at_picorv32_signals_1000'
       slow_change = 'change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk'

       b_at = json.loads((baseline / f'{target}.hyperfine.json').read_text())['results'][0]['mean']
       r_at = json.loads((revised / f'{target}.hyperfine.json').read_text())['results'][0]['mean']
       b_change = json.loads((baseline / f'{slow_change}.hyperfine.json').read_text())['results'][0]['mean']

       speedup = float(b_at) / float(r_at)
       print(f'{target}: baseline={b_at:.6f}s revised={r_at:.6f}s speedup_x={speedup:.3f}')
       print(f'comparison_vs_change_tail: revised_at={r_at:.6f}s baseline_change_tail={b_change:.6f}s faster={r_at < b_change}')
       PY

8. Update `CHANGELOG.md` with delivered user-visible result and commit atomic units.

       git add src/waveform/mod.rs src/engine/at.rs tests/at_cli.rs CHANGELOG.md docs/exec-plans/active/2026-03-03-at-performance-anomaly/PLAN.md
       git commit -m "perf(at): remove duplicate-amplified sampling overhead"

9. Mandatory review pass #1.

   Use OpenCode review workflow (not shell) from a fresh reviewer context:

   - load `ask-review` skill;
   - request a `review` subagent pass with scope: `src/waveform/mod.rs`, `src/engine/at.rs`, tests, benchmark evidence, and this plan;
   - focus review on contract parity, duplicate handling, and perf-gate evidence quality.

   Apply valid fixes and commit.

10. Mandatory independent review pass #2 (fresh context).

   Start a new reviewer session (do not reuse pass #1 context) and request a second independent review with the same scope.

   If pass #2 finds issues, fix and rerun a fresh pass #2 until clean.

### Validation and Acceptance

Correctness acceptance:

- `at` integration contract remains unchanged (`tests/at_cli.rs` passes, including duplicate-order checks).
- Existing waveform sampling semantics remain unchanged (`sample_signals_at_time` tests and related point-sampling tests pass).
- `make check` and `make ci` pass.

Performance acceptance:

- Hard target: `at_picorv32_signals_1000` in `at-anomaly-final` is `>=3.5x` faster than baseline (`<=0.60s` from baseline `2.111695s`).
- Cross-command sanity target: revised `at_picorv32_signals_1000` is not slower than baseline `change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_posedge_clk` (`0.591289s`).
- Regression guard: `python3 bench/e2e/perf.py compare --revised bench/e2e/runs/at-anomaly-final --golden bench/e2e/runs/at-anomaly-golden --max-negative-delta-pct 5` passes for matched `at` tests.
- Matrix guard: no `at` row regresses by more than `5%` in the final matrix compare.

TDD acceptance:

- At least one new duplicate-aware test fails before optimization edits and passes after edits within the same branch history.

If hard target is not met, do not close this plan. Record the dominant residual cost in `Surprises & Discoveries` and continue with an additional optimization milestone in this same plan.

### Idempotence and Recovery

Benchmark commands are idempotent when reusing the same `--run-dir`; reruns safely overwrite artifacts.

If a run is interrupted, rerun with `--missing-only` to fill gaps without discarding completed artifacts.

If semantic drift appears in `at` outputs, revert only the optimization hunk in `src/waveform/mod.rs` and rerun targeted `at` tests before continuing.

If performance results are noisy, rerun the same benchmark command at least twice and compare hyperfine means before drawing conclusions.

### Artifacts and Notes

Expected modified files during implementation:

- `src/waveform/mod.rs`
- `src/engine/at.rs`
- `tests/at_cli.rs` (only if additional parity coverage is needed)
- `CHANGELOG.md`
- `docs/exec-plans/active/2026-03-03-at-performance-anomaly/PLAN.md`

Plan-local benchmark artifact directories:

- `bench/e2e/runs/at-anomaly-golden/`
- `bench/e2e/runs/at-anomaly-golden-hot/`
- `bench/e2e/runs/at-anomaly-final/`
- `bench/e2e/runs/at-anomaly-matrix/`

Before closure, include short excerpts here for:

- TDD red-phase failure line.
- TDD green-phase pass line.
- Perf closure output (`speedup_x=...` and `faster=True/False`).
- Review pass #1/#2 clean status (or fixed findings + rerun evidence).

Collected excerpts:

- TDD red-phase failure line:
  `test waveform::tests::duplicate_projection_deduplicates_paths_and_tracks_requested_order ... FAILED`

- TDD green-phase pass line:
  `test waveform::tests::duplicate_projection_deduplicates_paths_and_tracks_requested_order ... ok`

- Perf closure output:
  `at_picorv32_signals_1000: baseline=2.111695s revised=2.094000s speedup_x=1.008`
  `comparison_vs_change_tail: revised_at=2.094000s baseline_change_tail=0.591289s faster=False`

- Review pass #1/#2 status:
  original implementation pass #1/#2 clean after fixes (`8fc1f94`, `e1b086a`); simplification cycle pass #1 clean, pass #2 raised one shared-loader regression risk, fixed, then fresh re-check clean.

### Interfaces and Dependencies

No new runtime dependencies are allowed.

Required interface/state invariants after implementation:

- `Waveform::sample_signals_at_time` remains public and returns values in the exact requested token order, including duplicates.
- `Waveform::sample_signals_at_time` must preserve existing missing-signal error behavior.
- `Waveform::ensure_signals_loaded` must avoid duplicate refs in one call and keep cross-call cache semantics based on `loaded_signals`.
- `at` CLI/output contracts from `docs/DESIGN.md` section `3.2.4` remain unchanged.

Revision Note (2026-03-04): Initial plan authored to close backlog item `at` high-activity performance anomaly with explicit benchmark targets tied to the recent `change` performance baseline.

Revision Note (2026-03-04): Updated after review pass #1 to align with backlog root-cause wording, add mandatory attribution+pivot logic, fix acceptance command paths, and make review instructions executable without pseudo-shell commands.

Revision Note (2026-03-04): Updated after independent review pass #2 to add explicit matrix fail/pass compare command and require staging this plan file in implementation commits when changed.

Revision Note (2026-03-04): Updated after implementation Milestones 1-5 with TDD evidence, attribution/pivot results, benchmark outcomes, and explicit open follow-up because hard performance gate remains unmet.

Revision Note (2026-03-04): Updated after implementation review pass #1 fix cycle and independent pass #2 clean result.

Revision Note (2026-03-04): Updated after user-directed simplification rollback of `at` streaming/heuristic branches, including re-validation and a fresh two-pass review cycle.
