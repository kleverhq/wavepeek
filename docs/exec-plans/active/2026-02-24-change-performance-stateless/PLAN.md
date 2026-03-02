# Accelerate `change` in Stateless Mode

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek change` is currently too slow on large FST dumps for agent workflows. On the target benchmark scenario, users wait tens of seconds for a small output set. After this plan is implemented, the same query should complete in single-digit to low-double-digit seconds, while staying stateless and preserving command semantics.

The user-visible behavior remains the same: same rows, same ordering, same warnings/errors, same `--json` envelope. The improvement is purely execution speed from tighter iteration strategy, lower per-sample overhead, candidate-time reduction, and an FST-specific streaming fast path that still runs in one process invocation.

This revision aligns the plan with the performance harness that already exists in this repository. Perf measurement is now anchored on `bench/e2e/perf.py` plus `bench/e2e/tests.json` artifacts, so an implementer can pick up this plan and execute it without inventing new benchmark plumbing.

## Non-Goals

This plan does not introduce a daemon, persistent cache, background service, or cross-invocation memoization.

This plan does not change `change` contract semantics (baseline at `--from`, delta-only emission, warning text, scoped name resolution rules, `iff` deferred behavior).

This plan does not redesign unrelated commands (`at`, `scope`, `signal`, `when`) except shared utility extraction directly required for `change` performance.

This plan does not add a second benchmark framework (`cargo bench`/Criterion). Existing perf infrastructure (`bench/e2e/perf.py`) is the source of truth.

## Progress

- [x] (2026-02-24 14:37Z) Collected baseline benchmark evidence and mapped the end-to-end `change` hot path.
- [x] (2026-02-24 14:37Z) Drafted a stateless-first optimization strategy with phased risk and expected speedup per phase.
- [x] (2026-02-24 14:43Z) Review pass #1 completed; plan updated to include benchmark harness wiring, Milestone-3 baseline invariant, and explicit release-measurement protocol.
- [x] (2026-02-24 14:44Z) Fresh independent review pass #2 completed; plan updated with executable fallback dataset creation and fallback acceptance rules.
- [x] (2026-02-27 21:09Z) Reconciled this plan with shipped perf harness and baseline artifacts (`bench/e2e/perf.py`, `bench/e2e/tests.json`, `bench/e2e/runs/baseline/`); removed stale Criterion-first assumptions.
- [x] (2026-02-27 21:18Z) Addressed plan QA findings: added explicit `cargo build --release` before every perf capture, added `hyperfine` preflight, and clarified that fallback-fixture mode is retired for this revision.
- [x] (2026-02-27 21:20Z) Addressed independent QA finding: replaced filter-based parity invocation with a dedicated integration-test target (`tests/change_vcd_fst_parity.rs`) to prevent false-green Milestone 4 validation.
- [x] (2026-02-28 11:02Z) Added dedicated coremark `change` benchmark cases to `bench/e2e/tests.json` and captured dedicated golden run artifacts under `bench/e2e/runs/change-stateless-golden/`.
- [x] (2026-02-28 11:28Z) Implemented Milestone 2 random-access optimizations (resolved-handle cache + timestamp-slice iteration) and validated `change_cli` guardrails; later dedicated run artifacts were refreshed on final code and currently record `~56-57x` KPI speedup in all milestone run directories.
- [x] (2026-02-28 12:07Z) Implemented Milestone 3 candidate-timestamp reduction with strict previous-global-timestamp delta invariant and added `tests/change_opt_equivalence.rs`.
- [x] (2026-02-28 12:24Z) Implemented Milestone 4 FST streaming-capable fast-path integration (`wellen::stream` filter pushdown with random-access fallback heuristic) and added `tests/change_vcd_fst_parity.rs` (including forced-stream vs forced-random parity coverage).
- [x] (2026-02-28 13:11Z) Completed Milestone 5 hardening: `make ci` and `make check` passed, dedicated/final perf artifacts were archived, and broad `^change_` matrix compare against `bench/e2e/runs/baseline` passed (`--max-negative-delta-pct 5`) after rerunning a noisy outlier case.
- [x] (2026-03-02 07:23Z) Improved perf-report readability in `bench/e2e/perf.py`: Markdown report cells and compare failures now include speed factor in `x` form (for example `2.00x faster` / `1.50x slower`) alongside percentage deltas.
- [x] (2026-03-02 00:00Z) Recalibrated `change_picorv32_*` and `change_chipyard_*` benchmark cases from `runs=1, warmup=0` to `runs=5, warmup=1` now that perf anomalies are resolved; closed corresponding backlog entry.
- [ ] Implement Milestone 6 fused single-pass loop with incremental decoding and offset-based delta detection.
- [ ] Validate Milestone 6 correctness: `change_cli`, `change_opt_equivalence`, `change_vcd_fst_parity` tests green.
- [ ] Capture Milestone 6 perf evidence and run broad matrix regression.
- [ ] Full quality gate (`make ci`, `make check`) for Milestone 6.

## Surprises & Discoveries

- Observation: Runtime scales almost linearly with selected time range and with number of sampled signals, indicating per-(timestamp, signal) overhead dominates.
  Evidence: local warm runs: `--to=120ps` about `4.98s`, `--to=500ps` about `19.07s`, `--to=1000ps` about `37.55s`; one sampled signal at `1000ps` about `18.95s`.

- Observation: Rendering is not the bottleneck.
  Evidence: same `change` benchmark in human and `--json` modes both around `37.9s`.

- Observation: Startup/body-load cost exists but is not dominant for medium windows.
  Evidence: `change --max 1` and short-window run are both around `5s`, while full `--to=1000ps` is around `37.5s`.

- Observation: Current adapter resolves paths and samples with repeated allocation-heavy work per sample.
  Evidence: `src/engine/change.rs` uses cache keys like `HashMap<(String, u64), ...>` and probes via single-signal calls; `src/waveform/mod.rs` repeatedly resolves hierarchy on sampling calls.

- Observation: `change` currently clones the full time table into a `Vec<u64>` before filtering.
  Evidence: `src/waveform/mod.rs` `timestamps_raw()` uses `self.inner.time_table().to_vec()` and `src/engine/change.rs` filters that clone.

- Observation: Repository perf infrastructure already exists and is production-shaped.
  Evidence: executable harness in `bench/e2e/perf.py`, explicit matrix in `bench/e2e/tests.json`, committed baseline artifacts in `bench/e2e/runs/baseline/`.

- Observation: `perf/` currently provides navigation docs, while runnable harness code is in `bench/e2e/`.
  Evidence: `perf/AGENTS.md` and `perf/e2e/AGENTS.md` are breadcrumb docs; benchmark runner and test catalog live under `bench/e2e/`.

- Observation: Existing `change` perf matrix does not yet include the exact coremark query from this plan purpose (`scr1_max_axi_coremark.fst` with `i_imem_axi` scope and `araddr,arvalid`).
  Evidence: `bench/e2e/tests.json` currently references coremark only for `info`, while `change` cases target other fixtures.

- Observation: The largest practical runtime win came from avoiding repeated `load_signals` calls for the same refs, not from micro-tuning expression checks.
  Evidence: after introducing persistent loaded-signal tracking in `src/waveform/mod.rs`, the coremark KPI dropped from `~0.846s` to `~0.033s` mean on warm runs.

- Observation: Broad matrix variance includes occasional outlier regressions around the acceptance threshold; reruns can normalize to stable pass values.
  Evidence: in `change-stateless-final-matrix`, `change_scr1_signals_100_pos_50_window_2000_trigger_any` initially compared around `-10.51%`, then passed after rerun in the same run directory (`~6.147s` revised vs `~6.065s` golden).

- Observation: Percent-only deltas in perf reports are less intuitive for quick human interpretation than ratio-style speedup/slowdown.
  Evidence: `bench/e2e/perf.py` now renders both `%` and explicit `x` factor in report tables and compare failure strings.

- Observation: Baseline benchmark data shows that all `change` tests exceeding 3 seconds share one profile: `signal_count=100` with `trigger=*`. Window size has near-zero effect on runtime (scr1 100sig: 2ns=5.63s, 4ns=5.57s, 8ns=5.58s). The bottleneck is the per-candidate main loop doing O(C × S) sample operations, where C ≈ W because 100 diverse signals make almost every timestamp a candidate.
  Evidence: `bench/e2e/runs/baseline/README.md` — compare same-file same-signal-count rows across window sizes and trigger types. Subtracting `at`-equivalent file+load cost (scr1: ~0.06s, chipyard_dual: ~0.50s, picorv32: ~0.44s) leaves 3.5-5.5s of pure main-loop work.

- Observation: Switching from `trigger=*` to a specific signal trigger collapses runtime dramatically even with the same 100 requested signals (chipyard_dual 100sig: trigger=* 4.42s vs trigger=signal 0.24s), confirming that the cost is in candidate density times per-candidate sampling, not in signal loading or candidate collection.
  Evidence: `bench/e2e/runs/baseline/README.md` rows for chipyard_dualrocketconfig 100sig comparing trigger types.

## Decision Log

- Decision: Prioritize algorithmic wins before micro-optimizations.
  Rationale: Most observed cost is repeated sampling/scanning, not output formatting or tiny helper calls.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Keep correctness semantics frozen and optimize under equivalence tests.
  Rationale: `change` contract is detailed and fragile around baseline/edge/warning behavior.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Use phased delivery with a low-risk path first, then optional FST streaming fast path.
  Rationale: This de-risks regression while preserving large upside on target benchmarks.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Ship Milestone 2 first as default production path, then add Milestone 4 as an FST fast path with fallback to Milestone-2 logic.
  Rationale: This keeps rollout safe and enables immediate wins even if stream-path edge cases appear.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Keep CI perf checks report-only for one cycle, then consider blocking thresholds after baseline stabilization.
  Rationale: Perf noise and harness tuning risk are high in the first cycle; staged gating avoids false failures.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Replace stale `cargo bench`/Criterion instructions with repository-standard `bench/e2e/perf.py` workflows.
  Rationale: The harness is already implemented, documented, and used with committed baseline artifacts.
  Date/Author: 2026-02-27 / OpenCode

- Decision: Add dedicated coremark `change` perf entries to `bench/e2e/tests.json` before implementation milestones.
  Rationale: Acceptance targets must be measured on the exact user benchmark this plan promises to improve.
  Date/Author: 2026-02-27 / OpenCode

- Decision: Use dedicated run directories for this plan (`bench/e2e/runs/change-stateless-*`) and keep `bench/e2e/runs/baseline` as shared global reference.
  Rationale: Isolates this optimization campaign evidence while preserving global baseline continuity.
  Date/Author: 2026-02-27 / OpenCode

- Decision: Keep random-access candidate collection as default for small windows and enable FST streaming candidate collection only when estimated random work crosses a threshold.
  Rationale: Full-stream setup overhead can dominate small-window queries; thresholded dispatch preserves low-latency fast cases while retaining a stream-capable path for heavier scans.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Add Milestone 6 to fuse candidate collection, schedule building, and the main iteration loop into a single sequential pass through the time-table window, with incremental signal decoding and offset-based delta detection.
  Rationale: Baseline data shows the dominant remaining cost for high-signal-count queries (100 signals, trigger=*) is the per-candidate main loop performing O(C × S) decode and comparison operations. The current multi-pass architecture (collect candidates → build schedule → iterate candidates with full batch sampling) re-reads the same signal offsets multiple times and decodes all S signals at every candidate even when only 1-3 changed. Fusing these passes and decoding only changed signals eliminates redundant work and intermediate allocations, with estimated 10-20x improvement for the slow 100-signal cases.
  Date/Author: 2026-03-02 / OpenCode

- Decision: Do not pursue parallel candidate collection (rayon) for this milestone.
  Rationale: The bottleneck for slow tests is the main loop (O(C × S) per-candidate work), not candidate collection. With A typically 2-100 and candidate scan being cheap relative to the main loop, thread dispatch overhead would likely exceed the benefit.
  Date/Author: 2026-03-02 / OpenCode

## Outcomes & Retrospective

Plan-authoring outcome: bottlenecks were identified with reproducible evidence, implementation was split into independently verifiable milestones, and the measurement path stayed aligned with repository reality (`bench/e2e/perf.py` + committed run artifacts).

Implementation outcome: complete. The dedicated coremark KPI improved from `1.919s` mean (golden) to approximately `0.033-0.034s` in committed milestone run artifacts (`change-stateless-m2/m3/m4`), i.e. roughly `56-57x` speedup, with parity tests and fallback behavior preserved.

Hardening outcome: `make ci` and `make check` passed, dedicated plan run directories were captured (`change-stateless-golden/m2/m3/m4/final-matrix`), and broad `^change_` compare against shared baseline passed at `--max-negative-delta-pct 5` after rerunning one noisy benchmark outlier.

Milestone 6 reopens implementation. The outcomes above reflect Milestones 1-5. Milestone 6 targets the remaining bottleneck: high-signal-count queries (100 signals, trigger=*) where per-candidate O(C × S) sampling dominates. This is in progress.

## Context and Orientation

`change` command flow today is:

`src/cli/change.rs` (argument parsing) -> `src/engine/mod.rs` (dispatch) -> `src/engine/change.rs` (`run`) -> `src/waveform/mod.rs` (sampling and metadata) -> `src/output.rs` (human/JSON serialization).

The critical hot loop is in `src/engine/change.rs`: iterate timestamps in range, evaluate trigger terms per timestamp, and sample requested signals through `SampleCache::sample`. Sampling currently performs repeated path and allocation-heavy work when probing many times.

Waveform adapter behavior in `src/waveform/mod.rs` is currently random-access (`wellen::simple::read`) with full table materialization for timestamp access patterns. For FST, `wellen::stream` supports time-range and signal filtering while streaming value changes; this can provide an in-process stateless fast path.

Correctness guards already exist in `tests/change_cli.rs` and `src/waveform/mod.rs` unit tests. This plan adds optimization equivalence tests to preserve behavior while changing internals.

Performance measurement source of truth for this repository is:

- `bench/e2e/perf.py` (runner with `list`, `run`, `report`, `compare`)
- `bench/e2e/tests.json` (explicit benchmark catalog)
- `bench/e2e/runs/` (run artifacts and reports)

`perf/AGENTS.md` and `perf/e2e/AGENTS.md` are navigation maps. Executable harness code currently lives in `bench/e2e/`.

## Open Questions

No blocking open questions remain. This revision resolves benchmark workflow ambiguity by pinning `bench/e2e/perf.py` as canonical perf path.

## Plan of Work

Milestone 1 aligns benchmark contracts and guardrails with existing repository infrastructure. Add dedicated coremark `change` scenarios to `bench/e2e/tests.json`, capture a dedicated golden run for this plan, and lock correctness tests that must stay green throughout optimization work.

Milestone 2 removes avoidable overhead in the current random-access engine without changing architecture. Replace string-heavy sample caching with resolved handles, avoid per-sample path resolution, batch sample needed signals per timestamp, and iterate only timestamp slices overlapping `[from, to]`.

Milestone 3 introduces candidate-timestamp reduction so sparse queries stop evaluating every dump timestamp. Candidate sets should come from tracked/event signal change points plus mandatory baseline context. The invariant is strict: for candidate timestamp `t`, delta comparison must use sampled signal state strictly before `t`, not merely before the previous candidate.

Milestone 4 adds an FST-specific streaming fast path with `wellen::stream` filter pushdown (`start`, `end`, signals), while preserving fallback for unsupported formats/cases. Execution remains stateless per invocation.

Milestone 5 hardens and closes out: full quality gate, perf evidence archive, and collateral updates (`docs/*`, changelog, backlog notes as needed).

Milestone 6 fuses candidate collection, predecessor resolution, and the main iteration loop into a single sequential pass through the time-table window.

In the current architecture, three separate passes touch the same data: `collect_change_times` scans the window once per candidate signal comparing offsets to find change points, `build_candidate_schedule` binary-searches the time table per candidate to locate predecessors, and the main loop re-reads offsets and decodes values for every signal at every candidate timestamp.

Milestone 6 replaces all three with one time-major loop that walks the window once, tracks rolling offsets for candidate signals, detects candidates inline, carries forward the predecessor for free, and performs incremental signal decoding — only calling `get_value_at` for signals whose offset actually changed at that timestamp. Delta detection uses a two-stage gate: offset comparison as a fast-reject filter, followed by value-level confirmation for signals whose offset changed. Value strings are allocated only for rows that pass both the trigger check and the delta check and will be emitted (bounded by `--max`, default 50).

For edge-based triggers (`posedge`/`negedge`/`edge`), the trigger signal's bit string is decoded at candidate timestamps where its offset changed, but other signals are not decoded unless the row passes all filters.

The FST streaming candidate-collection fast path is preserved as an alternative when it is cheaper (large windows on FST files above the streaming threshold); in that case only the schedule-building and main-loop fusion applies, using the streaming candidate set as input.

Existing correctness tests (`change_cli`, `change_opt_equivalence`, `change_vcd_fst_parity`) must remain green. New equivalence tests should cover the fused path against the pre-fusion path using an environment variable to force-disable the fused loop.

### Concrete Steps

Run all commands from `/workspaces/wavepeek`.

0. Verify baseline environment and harness availability.

        make check-rtl-artifacts
        hyperfine --version
        python3 bench/e2e/perf.py list --filter '^change_'

   If `make check-rtl-artifacts` fails, run inside the repository devcontainer/CI image first. This plan assumes `/opt/rtl-artifacts` is available.
   The earlier fallback-fixture draft path is retired in this revision; run perf milestones only in environments with `/opt/rtl-artifacts`.

1. Add dedicated coremark benchmark entries to `bench/e2e/tests.json` before optimization work.

   Add these tests (same command semantics as purpose benchmark):

        {
          "name": "change_scr1_coremark_imem_axi_2sig_to_1000ps",
          "category": "change",
          "runs": 9,
          "warmup": 2,
          "command": [
            "{wavepeek_bin}",
            "change",
            "--waves", "/opt/rtl-artifacts/scr1_max_axi_coremark.fst",
            "--to", "1000ps",
            "--scope", "TOP.scr1_top_tb_axi.i_top.i_imem_axi",
            "--signals", "araddr,arvalid",
            "--max", "1000"
          ],
          "meta": {
            "waves": "/opt/rtl-artifacts/scr1_max_axi_coremark.fst",
            "scope": "TOP.scr1_top_tb_axi.i_top.i_imem_axi",
            "signal_count": 2,
            "window_to": "1000ps"
          }
        }

        {
          "name": "change_scr1_coremark_imem_axi_1sig_to_1000ps",
          "category": "change",
          "runs": 9,
          "warmup": 2,
          "command": [
            "{wavepeek_bin}",
            "change",
            "--waves", "/opt/rtl-artifacts/scr1_max_axi_coremark.fst",
            "--to", "1000ps",
            "--scope", "TOP.scr1_top_tb_axi.i_top.i_imem_axi",
            "--signals", "araddr",
            "--max", "1000"
          ],
          "meta": {
            "waves": "/opt/rtl-artifacts/scr1_max_axi_coremark.fst",
            "scope": "TOP.scr1_top_tb_axi.i_top.i_imem_axi",
            "signal_count": 1,
            "window_to": "1000ps"
          }
        }

   Validate catalog parse and names:

        python3 bench/e2e/perf.py list --filter '^change_scr1_coremark_imem_axi_'

2. Capture dedicated golden run artifacts for this plan.

        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-stateless-golden --filter '^change_scr1_coremark_imem_axi_'
        python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/change-stateless-golden

3. Lock correctness guardrails before optimization.

        cargo test --test change_cli
        cargo test waveform::tests::edge_classification_sv2023_matrix
        cargo test waveform::tests::delta_filter_mixed_prior_state_emits_on_comparable_change

4. Implement Milestone 2 optimizations in `src/engine/change.rs` and `src/waveform/mod.rs`, then validate.

        cargo test --test change_cli
        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-stateless-m2 --filter '^change_scr1_coremark_imem_axi_'
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-stateless-m2 --golden bench/e2e/runs/change-stateless-golden --max-negative-delta-pct 0

5. Implement Milestone 3 candidate-time reduction and equivalence tests (create `tests/change_opt_equivalence.rs`), then validate.

        cargo test --test change_cli
        cargo test --test change_opt_equivalence
        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-stateless-m3 --filter '^change_scr1_coremark_imem_axi_'
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-stateless-m3 --golden bench/e2e/runs/change-stateless-golden --max-negative-delta-pct 0

6. Implement Milestone 4 FST stream fast path and add dedicated parity test file (`tests/change_vcd_fst_parity.rs`), then validate.

        cargo test --test change_cli
        cargo test --test change_opt_equivalence
        cargo test --test change_vcd_fst_parity
        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-stateless-m4 --filter '^change_scr1_coremark_imem_axi_'
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-stateless-m4 --golden bench/e2e/runs/change-stateless-golden --max-negative-delta-pct 0

7. Run full quality gate and broader change-matrix regression pass.

        make ci
        make check
        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-stateless-final-matrix --filter '^change_' --compare bench/e2e/runs/baseline
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-stateless-final-matrix --golden bench/e2e/runs/baseline --max-negative-delta-pct 5

8. Implement Milestone 6 fused single-pass loop in `src/engine/change.rs` and `src/waveform/mod.rs`.

   In `src/waveform/mod.rs`, add low-level access methods that expose offset comparison and value decoding without going through the full `sample_resolved_optional` path:

        pub fn signal_offset_at_index(
            &self,
            signal_ref: SignalRef,
            time_table_idx: u32,
        ) -> Option<SignalOffsetData>

        pub fn decode_signal_at_index(
            &self,
            resolved: &ResolvedSignal,
            time_table_idx: u32,
        ) -> Result<SampledSignalState, WavepeekError>

   Both methods take `&self` because they only read from already-loaded signal data. Precondition: the caller must call `ensure_signals_loaded` for all relevant signal refs before invoking these methods. If the signal is not loaded, `signal_offset_at_index` returns `None` and `decode_signal_at_index` returns an error. The fused loop calls `ensure_signals_loaded` once for the union of all candidate, requested, and event signal refs before the loop begins. After that, only `&self` methods are called. Obtain `timestamps_raw_slice()` after the last `&mut self` call to avoid borrow conflicts.

   `signal_offset_at_index` wraps `self.inner.get_signal(signal_ref)?.get_offset(time_table_idx)` and extracts only the data-position fields (`start`, `elements`) into a `SignalOffsetData` newtype. It does NOT use the full wellen offset for comparison because metadata fields like `time_match` and `next_index` vary even when the data position is identical. `None` means no recorded value at or before this index. An offset change is necessary but not sufficient for a value change — see delta-detection invariant below. `decode_signal_at_index` performs the full `get_offset` + `get_value_at` + bit-string conversion for one signal at one time index.

   In `src/engine/change.rs`, replace the current `collect_change_times` → `build_candidate_schedule` → main loop sequence with a fused loop. All signal loading must happen before the loop: call `ensure_signals_loaded` for the union of all candidate, requested, and event signal refs. After that point, only `&self` methods are called on the waveform. Obtain `timestamps_raw_slice()` after the last `&mut self` call.

   The fused loop structure:

        // Pre-loop setup
        waveform.ensure_signals_loaded(union of all signal refs)
        let time_table = waveform.timestamps_raw_slice()
        let (start_idx, end_idx) = time_window_indices(time_table, from_raw, to_raw)

        // Rolling state — updated at EVERY time-table entry, not just candidates.
        // This is the key invariant: rolling state always reflects the latest
        // offset/value at the previous time-table entry when we evaluate a candidate.

        // Per candidate signal: rolling offset for candidate detection
        let mut cand_offset: Vec<Option<SignalOffsetData>>  // len = A (candidate signals)
        // Per requested signal: rolling offset + decoded value for delta detection
        let mut req_offset: Vec<Option<SignalOffsetData>>   // len = S (requested signals)
        let mut req_value: Vec<Option<String>>              // len = S
        // Per edge-trigger signal: rolling offset + decoded bits for edge classification
        let mut trigger_offset: Option<SignalOffsetData>
        let mut trigger_value: Option<String>

        // Initialize rolling state at start_idx - 1 (or None if start_idx == 0)
        // by querying offsets/values at that index.

        for idx in start_idx..end_idx:
            let timestamp = time_table[idx]

            // A. Update ALL rolling state unconditionally.
            //    This must happen at every time-table entry, not just candidates.
            //    Save previous req_value snapshot for delta comparison before updating.
            let prev_req_value = req_value.clone()  // snapshot before this step's updates

            let any_cand_changed = false
            for i in 0..A:
                let offset = waveform.signal_offset_at_index(cand_refs[i], idx as u32)
                if offset != cand_offset[i]:
                    any_cand_changed = true
                    cand_offset[i] = offset

            for i in 0..S:
                let offset = waveform.signal_offset_at_index(req_refs[i], idx as u32)
                if offset != req_offset[i]:
                    req_offset[i] = offset
                    req_value[i] = decode(req_refs[i], idx)  // incremental: decode only on change
                // else: req_value[i] carries forward unchanged

            // Update edge-trigger signal rolling state similarly
            let trig_offset_new = waveform.signal_offset_at_index(trigger_ref, idx)
            if trig_offset_new != trigger_offset:
                trigger_offset = trig_offset_new
                trigger_value = decode(trigger_ref, idx)

            // B. Candidate gate: skip if no candidate signal changed
            if !any_cand_changed: continue

            // C. Post-baseline filter
            if timestamp <= baseline_raw: continue

            // D. Trigger evaluation
            //    For AnyTracked: since candidate signals == requested signals when
            //       trigger is *, Phase B's any_cand_changed already answers this.
            //       The trigger fires (conservative: offset-change is a superset of
            //       value-change; Phase E catches false positives).
            //    For AnyChange(path): check if that signal's rolling offset changed
            //       at this idx (compare current vs previous offset for that signal).
            //    For Posedge/Negedge/Edge: use trigger_value (already decoded above
            //       when offset changed) and previous trigger_value for LSB edge
            //       classification.

            if !trigger_fired: continue

            // E. Delta detection — two-stage gate using rolling state
            //    Stage A (fast-reject): check if any req_offset[i] differs from
            //       what it was at the previous time-table entry. Since rolling state
            //       is updated at every entry, this is equivalent to comparing current
            //       vs prev_req_value via the offset proxy. If no req offset changed
            //       at this idx, skip.
            //    Stage B (value-level confirmation): compare req_value[i] (current,
            //       already decoded in phase A for changed signals) against
            //       prev_req_value[i] (snapshot from before phase A updates). Only if
            //       at least one decoded value actually differs should the row pass.
            //    This preserves the DESIGN.md contract: delta uses the state
            //       "strictly before the candidate timestamp in the underlying dump
            //       order" because prev_req_value is the rolling value from all
            //       prior time-table entries.

            if !delta_confirmed: continue

            // F. Emit snapshot
            //    Build output from req_value (already fully up to date).
            //    No additional decodes needed — values were decoded incrementally
            //    in phase A.
            //    Check --max limit; break if reached.

   The key invariants that must be preserved:

   - Rolling state (`req_offset`, `req_value`, `trigger_value`) is updated at every time-table entry in the window, not just at candidate timestamps. This ensures that when a candidate is evaluated, `prev_req_value` reflects the signal state at the time-table entry immediately before the candidate — matching the `docs/DESIGN.md` section 3.2.5 contract: "Delta comparison always uses sampled state strictly before the candidate timestamp in the underlying dump order (not merely before the previous emitted candidate)."
   - Delta detection must confirm at the value level, not just offset level. An offset change is necessary but not sufficient for value change — wellen may record a new offset for the same signal value (e.g., VCD re-stating the same state). The two-stage gate in phase E handles this: offset fast-reject followed by value-level confirmation.
   - Edge classification needs actual bit-string values. The trigger signal's bits are decoded incrementally (only when its offset changes) in phase A, and the previous bits are available from the pre-update snapshot.
   - When `AnyTracked` is the trigger, the candidate signal set equals the requested signal set (see `change.rs` lines 236-241). Phase B's `any_cand_changed` therefore already answers the trigger question for Phase D. Phase E still gates on actual value delta.
   - When the FST streaming path is used for candidate collection (large windows above the streaming threshold), the fused loop receives candidate timestamps from the stream result rather than scanning the full window. In this case, phase A still runs for each candidate to update rolling state, but Phase B is skipped (the candidate list is pre-computed). The implementer must also update rolling state for non-candidate timestamps between consecutive stream candidates to maintain the rolling-state invariant. This can be done by scanning the time-table slice between consecutive stream candidates and updating offsets (without triggering phases C-F).

   Testing strategy for the fused path:

   - Add an environment variable `WAVEPEEK_CHANGE_DISABLE_FUSED=1` that forces the engine to use the pre-fusion code path (Milestones 2-4). This lets `tests/change_opt_equivalence.rs` run both modes on the same input and assert byte-identical output.
   - Add unit tests in `src/waveform/mod.rs::tests` for `signal_offset_at_index` and `decode_signal_at_index`: verify offset equality/inequality semantics, verify decode produces the same result as `sample_resolved_optional`, verify behavior when the signal has no value at the queried index.
   - Add edge-case tests in `tests/change_opt_equivalence.rs`: empty window (from == to), window starting at first dump timestamp (no predecessor for baseline), all candidates at or before baseline, `--max 1` causing early termination, and a VCD fixture where wellen re-records the same value at a new offset (to exercise the two-stage delta gate). Note: craft the VCD with explicit redundant value dumps (e.g., `#5\nb0000 "` then `#10\nb0000 "`). Verify that wellen's VCD parser actually produces distinct offsets for these entries; if the parser coalesces redundant dumps, document this and treat the test as best-effort.
   - Add equivalence tests for all trigger types: `negedge`, `edge`, union triggers, and named non-edge triggers (`--when <signal>` where the trigger signal differs from requested signals).
   - Add a test that exercises the FST streaming + fused loop hybrid path (force streaming via `WAVEPEEK_CHANGE_STREAM_THRESHOLD=0` and verify output matches the non-streaming fused path).

   Validate:

        cargo test --test change_cli
        cargo test --test change_opt_equivalence
        cargo test --test change_vcd_fst_parity
        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-stateless-m6 --filter '^change_'
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-stateless-m6 --golden bench/e2e/runs/baseline --max-negative-delta-pct 5

9. Run full quality gate and capture final Milestone 6 evidence.

        make ci
        make check
        cargo build --release
        WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/change-stateless-m6-matrix --filter '^change_' --compare bench/e2e/runs/baseline
        python3 bench/e2e/perf.py compare --revised bench/e2e/runs/change-stateless-m6-matrix --golden bench/e2e/runs/baseline --max-negative-delta-pct 5

### Validation and Acceptance

Functional acceptance is unchanged from current `change` contract. After each milestone: CLI contract tests stay green, edge classification tests stay green, warning parity remains unchanged, and scoped resolution error categories remain unchanged.

Milestone 4 must include a dedicated integration test file `tests/change_vcd_fst_parity.rs` that asserts equivalent `change` behavior on matching VCD/FST fixtures; this prevents false-green filtered test invocations.

Performance acceptance for this plan uses hyperfine metrics exported by `bench/e2e/perf.py`. The primary KPI test is `change_scr1_coremark_imem_axi_2sig_to_1000ps`; the one-signal test is secondary guardrail and scaling evidence.

Speedup definition:

`speedup_x = golden_mean_seconds / revised_mean_seconds`

Use this helper after each milestone (`RUN_DIR` and `MIN_SPEEDUP` vary per milestone):

    RUN_DIR=bench/e2e/runs/change-stateless-m2 MIN_SPEEDUP=1.8 python3 - <<'PY'
    import json
    import os
    import pathlib
    import sys

    test_name = "change_scr1_coremark_imem_axi_2sig_to_1000ps"
    golden_dir = pathlib.Path("bench/e2e/runs/change-stateless-golden")
    revised_dir = pathlib.Path(os.environ["RUN_DIR"])
    min_speedup = float(os.environ["MIN_SPEEDUP"])

    def mean_seconds(run_dir: pathlib.Path) -> float:
        payload = json.loads((run_dir / f"{test_name}.json").read_text(encoding="utf-8"))
        return float(payload["results"][0]["mean"])

    golden = mean_seconds(golden_dir)
    revised = mean_seconds(revised_dir)
    speedup = golden / revised
    print(f"test={test_name}")
    print(f"golden_mean_s={golden:.6f}")
    print(f"revised_mean_s={revised:.6f}")
    print(f"speedup_x={speedup:.3f}")
    if speedup < min_speedup:
        raise SystemExit(f"speedup below target: required>={min_speedup:.3f} got={speedup:.3f}")
    PY

Minimum targets for the dedicated coremark benchmark:

- Milestone 2: `>=1.8x`
- Milestone 3: `>=3.0x`
- Milestone 4: `>=5.0x`

Milestone 6 acceptance is measured against the broad `^change_` baseline matrix, not the dedicated coremark benchmark. The coremark tests (2 signals, sparse changes) are already fast and are not expected to improve further. The target workload is the high-signal-count tests (`signal_count=100, trigger=*`) that currently take 4-6 seconds. Milestone 6 targets for these tests:

- `change_scr1_signals_100_*_trigger_any` tests (currently ~5.6s): target `<=1.0s` mean, i.e. `>=5x` speedup versus baseline.
- `change_chipyard_dualrocketconfig_dhrystone_signals_100_*_trigger_any` tests (currently ~4.4s): target `<=1.5s` mean, i.e. `>=3x` speedup versus baseline (file-open cost for 76M FST is ~0.5s floor).
- `change_picorv32_signals_100_*_trigger_any` tests (currently ~4.25s): target `<=1.0s` mean, i.e. `>=4x` speedup versus baseline.

These per-family targets are aspirational. If actual profiling reveals a lower ceiling (e.g., offset-change-but-same-value decodes are more frequent than expected), targets may be adjusted. The hard acceptance gate is: no regressions on any existing `change` test, i.e. `python3 bench/e2e/perf.py compare --max-negative-delta-pct 5` must pass against the shared baseline.

Secondary acceptance:

- `python3 bench/e2e/perf.py compare` must show no regressions for matching tests in the evaluated set (`--max-negative-delta-pct 0` for dedicated coremark run; `5` for broader matrix sweep against shared baseline).
- Final release-candidate expectation on target hardware/profile remains approximately `6-15s` for the 2-signal coremark query, with common outcomes near `8-12s`.

### Idempotence and Recovery

All commands are repeatable. `perf.py run --run-dir <dir>` rewrites matching JSON artifacts and refreshes `README.md`; rerunning does not corrupt unrelated run directories.

If a run is interrupted, rerun the same command with the same `--run-dir` to regenerate incomplete artifacts.

If correctness regresses in any milestone, revert only that milestone code path and keep equivalence tests as persistent guards.

If stream fast path diverges on edge cases, keep dispatch fallback to Milestone-2 random-access path and continue with correctness first.

If performance noise obscures conclusions, rerun the same run command and compare medians/means across repeated revised directories; do not accept/reject on one outlier sample.

If the fused loop introduces correctness divergence, the pre-fusion code path (Milestones 2-4) remains intact in git history and can be restored by reverting Milestone 6 commits. Equivalence tests in `tests/change_opt_equivalence.rs` serve as the regression gate.

### Artifacts and Notes

Artifacts produced by this plan should be tracked under dedicated directories:

- `bench/e2e/runs/change-stateless-golden/` (baseline for this plan)
- `bench/e2e/runs/change-stateless-m2/`
- `bench/e2e/runs/change-stateless-m3/`
- `bench/e2e/runs/change-stateless-m4/`
- `bench/e2e/runs/change-stateless-final-matrix/`
- `bench/e2e/runs/change-stateless-m6/` (Milestone 6 broad matrix)
- `bench/e2e/runs/change-stateless-m6-matrix/` (Milestone 6 final evidence)

Files expected to change during implementation:

    src/engine/change.rs
    src/waveform/mod.rs
    tests/change_cli.rs
    tests/change_opt_equivalence.rs
    tests/change_vcd_fst_parity.rs
    bench/e2e/tests.json
    bench/e2e/runs/change-stateless-golden/README.md
    bench/e2e/runs/change-stateless-golden/*.json
    docs/DESIGN.md
    docs/DEVELOPMENT.md
    CHANGELOG.md

### Interfaces and Dependencies

No new mandatory runtime dependencies are required.

Benchmark dependency policy for this plan:

- Use existing `hyperfine` + `bench/e2e/perf.py` stack.
- Do not introduce Criterion/Cargo bench harness for this workstream.

Required API additions in `src/waveform/mod.rs`:

    pub struct ResolvedSignal {
        pub path: String,
        pub signal_ref: wellen::SignalRef,
        pub width: u32,
    }

    pub fn resolve_signals(&self, canonical_paths: &[String]) -> Result<Vec<ResolvedSignal>, WavepeekError>
    pub fn timestamps_raw_slice(&self) -> &[u64]
    pub fn sample_resolved_optional(
        &mut self,
        resolved: &[ResolvedSignal],
        query_time_raw: u64,
    ) -> Result<Vec<SampledSignalState>, WavepeekError>
    pub fn collect_change_times(
        &mut self,
        resolved: &[ResolvedSignal],
        from_raw: u64,
        to_raw: u64,
    ) -> Result<Vec<u64>, WavepeekError>

Required API additions in `src/waveform/mod.rs` for Milestone 6:

    /// Opaque signal offset for change detection. Must be a newtype struct
    /// wrapping only the data-position fields from wellen's offset type
    /// (the `start` and `elements` fields of wellen::SignalChangeData),
    /// NOT the full struct. The full wellen offset includes metadata fields
    /// like `time_match` and `next_index` that vary across time indices even
    /// when the underlying signal data is identical — using raw PartialEq on
    /// the full struct would cause false "change" detections. The newtype
    /// must implement custom PartialEq comparing only data-position fields.
    /// An offset change is necessary but not sufficient for a value change —
    /// the caller must decode and compare values when offsets differ to
    /// confirm an actual value change. None means no recorded value at or
    /// before this index; None == None is considered "no change."
    #[derive(Debug, Clone, Copy)]
    pub struct SignalOffsetData { start: usize, elements: u16 }
    // impl PartialEq: compare start and elements only

    /// Loads signal data for the given refs if not already loaded.
    /// Currently private; must be made pub for Milestone 6 so the engine
    /// can pre-load all signals before entering the fused loop.
    pub fn ensure_signals_loaded(&mut self, signal_refs: &[SignalRef])

    /// Returns the signal offset at a given time-table index without decoding
    /// the value. Used for cheap change detection in the fused loop.
    /// Precondition: ensure_signals_loaded must have been called for this
    /// signal_ref before invoking this method. Returns None if the signal
    /// is not loaded or has no value at or before time_table_idx.
    pub fn signal_offset_at_index(
        &self,
        signal_ref: SignalRef,
        time_table_idx: u32,
    ) -> Option<SignalOffsetData>

    /// Decodes one signal's value at a given time-table index into a
    /// SampledSignalState. Used for value emission and edge classification.
    /// Precondition: ensure_signals_loaded must have been called for this
    /// signal's signal_ref before invoking this method.
    pub fn decode_signal_at_index(
        &self,
        resolved: &ResolvedSignal,
        time_table_idx: u32,
    ) -> Result<SampledSignalState, WavepeekError>

Required engine invariants in `src/engine/change.rs` for Milestone 3:

    - candidate iterator may skip non-candidate timestamps for trigger checks
    - baseline state for sampled signals at candidate t must reflect last known value strictly before t
    - implementation must prove equivalence with old path via tests/change_opt_equivalence.rs

Potential FST fast path integration uses `wellen::stream`:

    wellen::stream::read_from_file(...)
    StreamingWaveform::stream(&Filter { start, end, signals }, callback)

Engine contract to preserve in `src/engine/change.rs`:

    - candidate event semantics and union dedup behavior
    - baseline checkpoint at `--from` and strict post-baseline emission rule
    - delta-only emission and warning strings
    - JSON/human parity and ordering guarantees

Revision Note: 2026-02-24 / OpenCode - Initial performance-focused ExecPlan drafted for `change` command under strict stateless constraints, including measured baseline evidence, phased optimization milestones, and explicit speedup targets.
Revision Note: 2026-02-24 / OpenCode - Incorporated review-pass findings: pinned candidate-time delta invariant requirements, replaced open rollout questions with explicit decisions, and documented a fallback benchmark fixture workflow in that revision (historical; superseded by the 2026-02-27 no-fallback policy).
Revision Note: 2026-02-27 / OpenCode - Reconciled plan with current repository perf infrastructure: migrated execution and acceptance steps to `bench/e2e/perf.py` + `bench/e2e/tests.json`, removed stale Criterion/`cargo bench` assumptions, added dedicated coremark benchmark entries for this plan, and updated concrete steps to be directly executable from `/workspaces/wavepeek`.
Revision Note: 2026-02-27 / OpenCode - Incorporated plan QA fixes so perf measurements cannot use stale binaries: added `cargo build --release` before each perf capture, added `hyperfine --version` preflight, and made the no-fallback fixture assumption explicit.
Revision Note: 2026-02-27 / OpenCode - Incorporated independent review fix for Milestone 4 validation robustness by requiring a dedicated parity integration test target (`tests/change_vcd_fst_parity.rs`) instead of filter-based test selection.
Revision Note: 2026-02-28 / OpenCode - Completed end-to-end implementation: delivered Milestones 1-5, added dedicated equivalence/parity tests, archived golden+milestone perf runs, validated quality gates, and reconciled plan narrative with committed artifacts (current milestone run dirs all report approximately `56-57x` KPI speedup after final-code reruns).
Revision Note: 2026-03-02 / OpenCode - Improved perf-harness readability by augmenting percent deltas with explicit `x` speed factors in both Markdown reports (`run --compare` / `report --compare`) and `compare` regression diagnostics.
Revision Note: 2026-03-02 / OpenCode - Added Milestone 6: fused single-pass loop with incremental decoding and offset-based delta detection. Motivated by baseline analysis showing all >3s change tests share the profile (signal_count=100, trigger=*) where per-candidate O(C × S) sampling dominates. The fused approach replaces three separate passes with one time-major scan, decodes only changed signals per timestamp, and defers value materialization to emitted rows. Parallel candidate collection (rayon) was explicitly rejected as targeting the wrong bottleneck.
