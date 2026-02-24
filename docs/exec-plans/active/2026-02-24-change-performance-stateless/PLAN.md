# Accelerate `change` in Stateless Mode

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek change` is currently too slow on large FST dumps for agent workflows. On the benchmark scenario below, users wait tens of seconds for a small output set. After this plan is implemented, the same query should complete in single-digit to low-double-digit seconds (depending on phase), without breaking stateless execution and without changing command semantics.

The user-visible behavior remains the same: same rows, same ordering, same warnings/errors, same `--json` envelope. The improvement is purely execution speed from tighter iteration strategy, lower per-sample overhead, and an FST-specific streaming fast path that still runs inside one stateless process invocation.

## Non-Goals

This plan does not introduce a daemon, persistent cache, background service, or cross-invocation memoization.

This plan does not change `change` contract semantics (baseline at `--from`, delta-only emission, warning text, scoped name resolution rules, `iff` deferred behavior).

This plan does not redesign unrelated commands (`at`, `scope`, `signal`, `when`) except shared utility extraction that is directly required for `change` performance.

## Progress

- [x] (2026-02-24 14:37Z) Collected baseline benchmark evidence and mapped the end-to-end `change` hot path.
- [x] (2026-02-24 14:37Z) Drafted a stateless-first optimization strategy with phased risk and expected speedup per phase.
- [x] (2026-02-24 14:43Z) Review pass #1 completed; plan updated to include benchmark harness wiring, Milestone-3 baseline invariant, and explicit release-measurement protocol.
- [x] (2026-02-24 14:44Z) Fresh independent review pass #2 completed; plan updated with executable fallback dataset creation and fallback acceptance rules.

## Surprises & Discoveries

- Observation: Runtime scales almost linearly with selected time range and with number of sampled signals, which indicates per-(timestamp, signal) overhead dominates.
  Evidence: local warm runs: `--to=120ps` ~`4.98s`, `--to=500ps` ~`19.07s`, `--to=1000ps` ~`37.55s`; one sampled signal at `1000ps` ~`18.95s`.

- Observation: Rendering is not the bottleneck.
  Evidence: same `change` benchmark in human and `--json` modes both ~`37.9s`.

- Observation: Startup/body-load cost exists but is not dominant for medium windows.
  Evidence: `change --max 1` and short-window run are both ~`5s`, while full `--to=1000ps` is ~`37.5s`.

- Observation: Current adapter resolves paths and samples with repeated allocation-heavy work per sample.
  Evidence: `src/engine/change.rs` uses `HashMap<(String, u64), ...>` cache keys and `sample_signals_at_time_optional(&[path.to_string()], raw_time)` for each probe; `src/waveform/mod.rs` repeatedly splits paths and resolves hierarchy on each call.

- Observation: `change` currently clones the full time table into a `Vec<u64>` before filtering.
  Evidence: `src/waveform/mod.rs` `timestamps_raw()` calls `self.inner.time_table().to_vec()` and `src/engine/change.rs` then filters in loop.

## Decision Log

- Decision: Prioritize algorithmic wins before micro-optimizations.
  Rationale: Most observed cost is repeated sampling/scanning, not output formatting or tiny helper calls.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Keep correctness semantics frozen and optimize under equivalence tests.
  Rationale: `change` contract is already detailed and fragile around baseline/edge/warning behavior.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Use a phased delivery with a low-risk path first, then optional FST streaming fast path.
  Rationale: This de-risks regression while still enabling large speedups in the target benchmark.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Ship Milestone 2 first as the default production path, then add Milestone 4 as an FST fast path with fallback to Milestone-2 logic.
  Rationale: This keeps rollout safe and allows immediate wins even if stream-path edge cases appear.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Keep CI performance checks report-only for one cycle, then turn on blocking threshold checks after baseline stabilization.
  Rationale: Perf noise and harness bring-up risk are high in first cycle; staged gating avoids false failures.
  Date/Author: 2026-02-24 / OpenCode

## Outcomes & Retrospective

Plan-authoring outcome: bottlenecks are identified with reproducible evidence, and implementation is split into independently verifiable milestones that preserve stateless architecture.

Implementation outcome is pending. Expected end state is at least 3-6x speedup from current warm baseline, with a likely 6-12x speedup on large FST queries when the streaming fast path is enabled.

## Context and Orientation

`change` command flow today is:

`src/cli/change.rs` (argument shape) -> `src/engine/mod.rs` (dispatch) -> `src/engine/change.rs` (`run`) -> `src/waveform/mod.rs` (sampling and metadata) -> `src/output.rs` (human/JSON serialization).

The critical hot loop is in `src/engine/change.rs` where code iterates all timestamps in range, evaluates trigger terms per timestamp, and samples each requested signal through `SampleCache::sample`. Sampling currently calls `Waveform::sample_signals_at_time_optional` one signal at a time, which repeatedly resolves signal paths and performs allocation-heavy map keys.

The waveform adapter in `src/waveform/mod.rs` currently uses `wellen::simple::read`, which loads header+body and offers random-access sampling. For FST there is also a `wellen::stream` API (external dependency API) that can filter by time range and signal list while streaming value changes; this can be used as an in-process, stateless fast path.

Key correctness guards already exist in `tests/change_cli.rs` and in `src/waveform/mod.rs` unit tests. These must remain green at every milestone.

## Open Questions

No blocking open questions remain for this revision. Rollout and CI-gating decisions are fixed in the Decision Log above.

## Plan of Work

Milestone 1 establishes measurement and non-regression guardrails before optimization code lands. Add a reproducible benchmark harness and lock correctness invariants that are historically easy to break (`--from` baseline-only semantics, zero-delta suppression, warning parity, scoped lookup behavior, union dedup). This milestone is complete when baseline numbers are reproducible and correctness tests are green.

Milestone 2 removes avoidable overhead in the current random-access engine without changing the architecture. Replace string-heavy sample caching with resolved signal handles, avoid per-sample path resolution, batch sample all needed signals per timestamp, and iterate only timestamp slices that overlap `[from, to]` instead of scanning/cloning all dump timestamps. This milestone is complete when all correctness tests pass and benchmark runtime improves by at least 1.8x versus milestone-1 baseline on the target query.

Milestone 3 introduces candidate-timestamp reduction so `change` no longer evaluates every dump timestamp for sparse queries. Build candidate sets from tracked/event signal change points and evaluate delta/event semantics only on candidate times plus mandatory baseline context. The invariant that must remain true is: for any candidate timestamp `t`, delta comparison uses the sampled signal state strictly before `t` (not merely the previous candidate timestamp). This milestone is complete when functional equivalence tests pass (including interleaved non-candidate-change cases) and benchmark runtime improves by at least 3x versus milestone-1 baseline.

Milestone 4 adds an FST-specific streaming fast path using `wellen::stream::Filter` with `start/end/signals` pushdown, while preserving the existing path for VCD and fallback. This path still opens, computes, and exits per invocation (stateless). This milestone is complete when FST correctness parity holds and benchmark runtime improves by at least 5x versus milestone-1 baseline on the target query.

Milestone 5 hardens, documents, and rolls out. It finalizes perf acceptance thresholds, updates design/development docs with performance methodology, and records benchmark deltas in changelog/release notes. This milestone is complete when CI gates pass and the speedup evidence is archived.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-change`.

Prerequisite benchmark dataset check (required before Milestone 1 metrics):

    test -f /opt/rtl-artifacts/scr1_max_axi_coremark.fst && echo "external benchmark fixture found"

If the external fixture is missing, create repository-local fallback fixture before Step 1:

    cp tests/fixtures/hand/change_many_events.vcd tests/fixtures/hand/change_perf_dense.vcd

Then use `tests/fixtures/hand/change_perf_dense.vcd` for all benchmark commands. In fallback mode, use only relative speedup thresholds (x-times faster) and do not compare absolute wall-time against `/opt`-based expectations.

1. Capture baseline (warm run, no rebuild noise) and keep transcript artifacts.

        TIMEFORMAT=$'real %R\nuser %U\nsys %S'; time cargo run --quiet -- change --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst --to=1000ps --scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi --signals araddr,arvalid --max 1000 >/tmp/change_baseline.out
        TIMEFORMAT=$'real %R\nuser %U\nsys %S'; time cargo run --quiet -- change --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst --to=1000ps --scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi --signals araddr --max 1000 >/tmp/change_baseline_1sig.out

   Fallback baseline form (if `/opt` fixture is unavailable):

        TIMEFORMAT=$'real %R\nuser %U\nsys %S'; time cargo run --quiet -- change --waves tests/fixtures/hand/change_perf_dense.vcd --to=1000ns --scope top --signals data,valid --max 1000 >/tmp/change_baseline.out

2. Lock correctness guardrails before optimization.

        cargo test --test change_cli
        cargo test waveform::tests::edge_classification_sv2023_matrix
        cargo test waveform::tests::delta_filter_mixed_prior_state_emits_on_comparable_change

3. Add perf harness and report command.

   First wire benchmark support in `Cargo.toml`:

        [dev-dependencies]
        criterion = "~0.5"

        [[bench]]
        name = "change_perf"
        harness = false

   Then add new files `benches/change_perf.rs` and `tests/perf/change_perf_budget.json` (optional helper in `tests/common/mod.rs`).

        cargo bench --bench change_perf

4. Implement Milestone-2 optimizations in `src/engine/change.rs` and `src/waveform/mod.rs`, then re-run correctness and benchmark.

        cargo test --test change_cli
        cargo bench --bench change_perf

5. Implement Milestone-3 candidate timestamp reduction and re-run equivalence checks.

        cargo test --test change_cli
        cargo test --test change_opt_equivalence
        cargo bench --bench change_perf

6. Implement Milestone-4 FST stream fast path and verify parity plus performance.

        cargo test --test change_cli
        cargo test --test change_opt_equivalence
        cargo bench --bench change_perf

   Run explicit cross-format parity checks after stream-path split:

        cargo test --test change_cli change_vcd_and_fst_parity

7. Run full gate and update collateral.

        make ci
        make check

### Validation and Acceptance

Functional acceptance is unchanged from current `change` contract. At minimum, all of the following must hold after each milestone: CLI contract tests remain green, edge classification tests remain green, warning parity behavior remains unchanged, and scoped resolution error categories remain unchanged.

Performance acceptance is measured on release binaries for final sign-off, with warm-cache medians (2 warmups + 9 measured runs). Use relative thresholds, not absolute single-run numbers.

Canonical measurement procedure (single source of truth):

    cargo build --release
    python - <<'PY'
    import statistics, subprocess, time
    cmd = [
        "./target/release/wavepeek", "change",
        "--waves", "/opt/rtl-artifacts/scr1_max_axi_coremark.fst",
        "--to=1000ps",
        "--scope=TOP.scr1_top_tb_axi.i_top.i_imem_axi",
        "--signals", "araddr,arvalid",
        "--max", "1000",
    ]
    for _ in range(2):
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    samples = []
    for _ in range(9):
        t0 = time.perf_counter()
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        samples.append(time.perf_counter() - t0)
    print("median_sec", statistics.median(samples))
    print("samples_sec", ",".join(f"{v:.6f}" for v in samples))
    PY

Use this exact script for baseline and post-change runs; store outputs in `/tmp/change_perf_*` artifacts and compare medians.

If `/opt/rtl-artifacts/scr1_max_axi_coremark.fst` is not available, run the same script with the fallback command tuple (`tests/fixtures/hand/change_perf_dense.vcd`, matching scope/signals/time-unit) and evaluate only relative speedup. Absolute wall-time expectation bands below apply only to the `/opt` benchmark dataset.

Minimum target envelopes for the user benchmark (`scr1_max_axi_coremark.fst`, `--to=1000ps`, scope `TOP.scr1_top_tb_axi.i_top.i_imem_axi`, signals `araddr,arvalid`, `--max 1000`):

- Milestone 2 target: 1.8-2.8x faster than baseline.
- Milestone 3 target: 3.0-5.0x faster than baseline.
- Milestone 4 target (FST fast path): 5.0-12.0x faster than baseline.

Given reported baseline variability (user: ~73.9s, local warm: ~37.6s), expected wall-time after full plan is approximately 6-15s on the same hardware/profile, with most realistic outcomes in the 8-12s range on large FST windows.

### Idempotence and Recovery

All steps are repeatable. Benchmark runs and tests do not mutate repository state except optional artifacts intentionally written by the benchmark harness.

If a milestone regresses correctness, rollback only that milestone’s code path and keep equivalence tests as the guard. If streaming fast path introduces edge-case drift, keep it behind internal dispatch fallback and ship only random-access improvements first.

If performance noise obscures conclusions, re-run in release mode on pinned CPU conditions and compare medians only; do not gate on one-off outliers.

### Artifacts and Notes

Baseline transcripts captured during plan authoring:

    change benchmark (2 signals, to=1000ps):
    real 37.552
    user 37.377
    sys  0.119

    change benchmark (1 signal, to=1000ps):
    real 18.950
    user 18.864
    sys  0.084

    change benchmark (2 signals, to=120ps):
    real 4.981
    user 4.909
    sys  0.072

    at benchmark (same scope/signals, one time point):
    real 0.400
    user 0.355
    sys  0.047

Files expected to change during implementation:

    src/engine/change.rs
    src/waveform/mod.rs
    tests/change_cli.rs
    tests/change_opt_equivalence.rs
    benches/change_perf.rs
    tests/perf/change_perf_budget.json
    tests/fixtures/hand/change_perf_dense.vcd
    Cargo.toml
    docs/DESIGN.md
    docs/DEVELOPMENT.md
    CHANGELOG.md

### Interfaces and Dependencies

No new mandatory runtime dependencies are required.

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

Required engine invariants in `src/engine/change.rs` for Milestone 3:

    - candidate iterator may skip non-candidate timestamps for trigger checks
    - baseline state for sampled signals at candidate t must reflect last known value strictly before t
    - implementation must prove equivalence with old path via `tests/change_opt_equivalence.rs`

Potential FST fast path integration uses `wellen::stream`:

    wellen::stream::read_from_file(...)
    StreamingWaveform::stream(&Filter { start, end, signals }, callback)

Engine contract to preserve in `src/engine/change.rs`:

    - candidate event semantics and union dedup behavior
    - baseline checkpoint at `--from` and strict post-baseline emission rule
    - delta-only emission and warning strings
    - JSON/human parity and ordering guarantees

Revision Note: 2026-02-24 / OpenCode - Initial performance-focused ExecPlan drafted for `change` command under strict stateless constraints, including measured baseline evidence, phased optimization milestones, and explicit speedup targets.
Revision Note: 2026-02-24 / OpenCode - Incorporated review-pass findings: added concrete benchmark harness setup in `Cargo.toml`, pinned candidate-time delta invariant requirements, replaced open rollout questions with explicit decisions, and added a self-contained fallback benchmark fixture workflow for environments without `/opt/rtl-artifacts`.
