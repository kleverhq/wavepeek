# Reduce `change` Dense Edge-Trigger Hotspots

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek change` is now very fast for dense `--when *` workloads after Milestone 6, but dense clock-edge workloads (`--when posedge <clk>`) are still the slowest path in large chipyard windows. Users still wait multiple seconds for queries that conceptually ask for "rows only on clock edges". After this plan is implemented, those dense edge-trigger queries should run substantially faster while preserving the exact `change` contract: same rows, same ordering, same warning text, and same JSON envelope.

The user-visible win is simple to verify: the same chipyard `trigger_posedge_clk` benchmarks that are currently among the slowest `change` cases should show a clear speedup with no semantic drift in integration parity tests.

## Non-Goals

This plan does not change command-line behavior, default flags, output formatting, or warning/error strings.

This plan does not add a daemon, persistent cache, or cross-invocation state.

This plan does not revisit unrelated commands (`at`, `scope`, `signal`, `schema`, `info`).

This plan does not replace the benchmark harness. `bench/e2e/perf.py` remains the only performance source of truth.

## Progress

- [x] (2026-03-03 16:08Z) Archived previous stateless-performance plan under `docs/exec-plans/completed/2026-02-24-change-performance-stateless/PLAN.md` and split remaining dense edge-trigger latency into a dedicated active plan.
- [x] (2026-03-03 16:14Z) Collected current-state hotspot evidence: `trigger=*` workloads are much faster, while dense `trigger_posedge_clk` windows remain the long-tail bottleneck.
- [x] (2026-03-03 16:21Z) Drafted implementation strategy centered on event-first gating, offset-first delta filtering, and decode deferral for dense edge-trigger workloads.
- [ ] Capture a dedicated edge-hotspot golden run (`change_.*trigger_posedge_clk`) from current HEAD for this plan.
- [ ] Add failing/guard tests for dense edge-trigger equivalence and corner-case semantics before implementation (TDD stage).
- [ ] Implement edge-hotspot fast path in `src/engine/change.rs` with strict predecessor semantics and lazy decode.
- [ ] Add/update waveform helpers only as needed for index/offset-based evaluation without semantic drift.
- [ ] Validate correctness (`change_cli`, `change_opt_equivalence`, `change_vcd_fst_parity`, new tests) and run `make ci` + `make check`.
- [ ] Capture final perf evidence and satisfy acceptance gates (target edge speedups + broad regression guard).

## Surprises & Discoveries

- Observation: Current auto engine selection intentionally keeps edge-trigger expressions on pre-fusion, so dense `posedge clk` workloads do not benefit from the fused speedups seen for `trigger=*`.
  Evidence: `src/engine/change.rs` `select_engine_mode` chooses fused only for AnyTracked-only expressions with sufficient requested signal count.

- Observation: In pre-fusion, requested signals are batch-sampled before event firing is known, so rejected edge candidates still pay heavy decode and allocation cost.
  Evidence: `run_prefusion` samples current/previous requested vectors before `evaluate_event_term` short-circuits.

- Observation: Bench evidence from campaign runs and user report (`bench/e2e/runs/eval-m6`) agrees on shape: `change_.*trigger_any` improved dramatically, but dense `change_.*trigger_posedge_clk` remains the latency tail for large chipyard windows.
  Evidence: repository run artifacts under `bench/e2e/runs/change-stateless-m6*` plus user summary in this task thread.

## Decision Log

- Decision: Keep optimization work contract-preserving and focus on execution order changes (when to sample/decode), not behavior changes.
  Rationale: `change` semantics are already locked by integration tests and `docs/DESIGN.md`; we need runtime wins without output drift.
  Date/Author: 2026-03-03 / OpenCode

- Decision: Target dense edge-trigger hotspots with a dedicated path instead of broadening fused auto-dispatch blindly.
  Rationale: Forcing fused on selective triggers previously risked regressions; edge workloads need a specific low-risk optimization strategy.
  Date/Author: 2026-03-03 / OpenCode

- Decision: Use TDD for new edge-hotspot invariants by adding forced-mode parity tests before implementation.
  Rationale: The optimization changes execution order in hot loops; failing parity tests are the safest guard against subtle semantic drift.
  Date/Author: 2026-03-03 / OpenCode

## Outcomes & Retrospective

Initial state: this plan starts after Milestone 6 foundation is merged. Current behavior is excellent for dense `trigger=*`, acceptable for many sparse workloads, and still slow for dense edge-trigger windows in chipyard datasets.

Completion criteria for this retrospective: dense edge-trigger benchmarks show material speedup, broad regression gate remains green, and all contract tests remain unchanged.

## Context and Orientation

The `change` command lives across these files:

- `src/cli/change.rs`: argument parsing and hidden internal engine/candidate controls.
- `src/engine/change.rs`: main execution strategy (`run_prefusion`, `run_fused`, event and delta logic).
- `src/waveform/mod.rs`: waveform abstraction for timestamp access, offset detection, and signal decoding.

Important terms used in this plan:

- Candidate timestamp: a dump timestamp where at least one trigger-relevant signal changed and is therefore eligible for event/delta evaluation.
- Dense edge-trigger workload: a query where `--when` is edge-based (`posedge`, `negedge`, or `edge`) on a frequently toggling clock over a large time window, producing many candidates.
- Strict predecessor semantics: delta comparison at candidate timestamp `t` must use signal state from the dump timestamp immediately before `t` in dump order, not from the previous emitted row.

Current architecture has two engines. Fused is already optimized for dense AnyTracked (`*`) patterns. Dense edge-trigger queries typically run through pre-fusion and pay repeated per-candidate sampling/decode of requested vectors even when edge checks fail or no true value delta exists.

## Open Questions

No blocking product questions remain. Implementation choices are technical and can be resolved in code review:

- Whether edge-hotspot optimization should be a dedicated internal engine mode or a specialized pre-fusion subpath selected automatically.
- Exact auto-dispatch heuristics for enabling edge-hotspot path (candidate density threshold, requested signal count, trigger shape).

## Plan of Work

Milestone 1 establishes a reproducible golden baseline for the exact hotspot class. This run is the plan-local comparison target so improvements are measurable even if global baseline evolves.

Milestone 2 adds failing/guard tests first (TDD) for edge-dense parity and corner cases. The tests must force engine modes to compare old and new behavior on the same fixtures.

Milestone 3 implements the core optimization in `src/engine/change.rs`: reorder execution so edge event checks happen before expensive requested-vector sampling where safe, apply offset-first delta prefilter, and decode values lazily only when needed to confirm emission.

Milestone 4 hardens auto-dispatch and mode forcing. The engine should choose the edge-hotspot path only for workloads where it is expected to help and keep existing paths for non-target shapes.

Milestone 5 closes with full validation and perf evidence. We keep the broad `^change_` regression gate while proving material speedups on dense `trigger_posedge_clk` families.

### Concrete Steps

Run all commands from `/workspaces/perf-change`.

0. Create plan-local golden evidence for dense edge-trigger tests.

        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-edge-hotspot-golden --filter '^change_.*trigger_posedge_clk$' --compare bench/e2e/runs/baseline
        python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/change-edge-hotspot-golden --compare bench/e2e/runs/baseline

1. Add TDD guard tests before code changes.

   Extend `tests/change_opt_equivalence.rs` with forced-mode parity cases for dense edge windows and edge-specific corner cases:

   - dense periodic clock, many candidate edges, sparse actual requested-value changes;
   - edge fires but requested signals do not change value (must not emit row);
   - union trigger dedup (`posedge clk, negedge gate`) on same timestamp;
   - truncation behavior (`--max 1`) in dense edge window remains identical.

   If introducing a dedicated internal mode (for example `edge-fast`), add hidden mode wiring in `src/cli/change.rs` and force it in tests exactly as current `pre-fusion`/`fused` forcing is done.

   Run:

        cargo test --test change_opt_equivalence

2. Implement Milestone 3 engine optimization in `src/engine/change.rs`.

   Required behaviors:

   - Preserve strict predecessor semantics for delta checks.
   - For dense edge-trigger path, evaluate edge event from trigger signal state first when possible.
   - Add offset-first delta precheck on requested signals; if no requested offset changed, skip decode/emission work.
   - Decode requested values lazily (only changed/needed values) and materialize full snapshot only when row passes all gates.
   - Preserve existing behavior for AnyTracked and non-edge trigger combinations not selected for edge-hotspot path.

   Run:

        cargo test --test change_cli
        cargo test --test change_opt_equivalence
        cargo test --test change_vcd_fst_parity

3. Add or adjust waveform helpers only if required by the new path.

   Any new helper in `src/waveform/mod.rs` must remain stateless per invocation and avoid hidden env toggles. Keep `SignalOffsetData`-based comparison semantics (data-position only).

   Run targeted unit tests:

        cargo test waveform::tests::signal_offset_at_index_compares_data_position_only
        cargo test waveform::tests::decode_signal_at_index_matches_sample_resolved_optional

4. Harden dispatch and run full quality gates.

        make ci
        make check

5. Capture final perf evidence and compare.

        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-edge-hotspot-final --filter '^change_.*trigger_posedge_clk$' --compare bench/e2e/runs/change-edge-hotspot-golden
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-edge-hotspot-final --golden bench/e2e/runs/change-edge-hotspot-golden --max-negative-delta-pct 5
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-edge-hotspot-matrix --filter '^change_' --compare bench/e2e/runs/baseline
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-edge-hotspot-matrix --golden bench/e2e/runs/baseline --max-negative-delta-pct 5

6. Compute explicit edge-speedup factors versus global baseline for acceptance summary.

        python3 - <<'PY'
        import json
        from pathlib import Path

        baseline = Path('bench/e2e/runs/baseline')
        revised = Path('bench/e2e/runs/change-edge-hotspot-final')
        keys = [
            'change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk',
            'change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk',
            'change_picorv32_signals_100_window_32us_trigger_posedge_clk',
        ]
        for key in keys:
            b = json.loads((baseline / f'{key}.json').read_text())['results'][0]['mean']
            r = json.loads((revised / f'{key}.json').read_text())['results'][0]['mean']
            print(f"{key}: speedup_x={float(b)/float(r):.3f}")
        PY

### Validation and Acceptance

Correctness acceptance:

- Existing suites remain green: `change_cli`, `change_opt_equivalence`, `change_vcd_fst_parity`.
- New dense edge-trigger parity tests pass for forced old/new paths.
- `make ci` and `make check` pass with no contract-output changes.

Performance acceptance:

- Primary hotspot targets (versus `bench/e2e/runs/baseline`):
  - `change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk` >= `1.5x` faster.
  - `change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk` >= `1.5x` faster.
  - `change_picorv32_signals_100_window_32us_trigger_posedge_clk` >= `1.3x` faster.
- Required closure gate for this plan:
  - At least 2 of the 3 primary hotspot targets above must be met.
  - The remaining 1 target must still be >= `1.15x` faster (not flat/noise-level).
- Plan-local guard: `change-edge-hotspot-final` must not regress more than 5% against `change-edge-hotspot-golden` for the filtered `trigger_posedge_clk` set.
- Broad guard: full `^change_` compare against `bench/e2e/runs/baseline` must pass `--max-negative-delta-pct 5`.

If the required closure gate is missed, do not close this plan. Document profiler evidence in `Surprises & Discoveries`, keep safe improvements, and continue with another milestone iteration.

### Idempotence and Recovery

All commands are idempotent with fixed `--run-dir` paths. Re-running regenerates JSON artifacts and README reports for that run directory.

If a milestone introduces semantic divergence, keep tests as guards, revert only the offending engine changes, and preserve benchmark artifacts for diagnosis.

If perf results are noisy, rerun the same command in the same run directory and inspect mean plus outlier warnings before concluding.

### Artifacts and Notes

Expected changed files for implementation:

    src/cli/change.rs
    src/engine/change.rs
    src/waveform/mod.rs
    tests/change_opt_equivalence.rs
    tests/change_vcd_fst_parity.rs
    bench/e2e/runs/change-edge-hotspot-golden/README.md
    bench/e2e/runs/change-edge-hotspot-final/README.md
    bench/e2e/runs/change-edge-hotspot-matrix/README.md
    CHANGELOG.md

Plan-local benchmark evidence directories:

    bench/e2e/runs/change-edge-hotspot-golden/
    bench/e2e/runs/change-edge-hotspot-final/
    bench/e2e/runs/change-edge-hotspot-matrix/

### Interfaces and Dependencies

No new runtime dependency is allowed.

Keep using existing internal controls and waveform interfaces. If new interfaces are required, they must stay internal and explicit (no environment-variable behavior toggles in `src/**`).

Required invariants to keep in code:

- `change` output contract from `docs/DESIGN.md` is unchanged.
- Delta check uses strict predecessor in dump order.
- Edge classification semantics remain based on current `classify_edge` behavior.
- Candidate union dedup and warning ordering remain unchanged.

Revision Note: 2026-03-03 / OpenCode - Initial focused ExecPlan created to address the remaining dense edge-trigger hotspot after completing and archiving the broader stateless `change` performance plan.
Revision Note: 2026-03-03 / OpenCode - Tightened acceptance policy to deterministic closure gates (2/3 primary targets plus minimum floor on the remaining target) and explicit "do not close plan" behavior when the gate is missed.
