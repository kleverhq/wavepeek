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
- [ ] Add dedicated coremark `change` benchmark cases to `bench/e2e/tests.json` and capture a dedicated golden run for this plan.
- [ ] Implement Milestone 2 random-access optimizations and verify `>=1.8x` speedup on the dedicated coremark benchmark.
- [ ] Implement Milestone 3 candidate-timestamp reduction with equivalence tests and verify `>=3.0x` speedup.
- [ ] Implement Milestone 4 FST streaming fast path with parity checks and verify `>=5.0x` speedup.
- [ ] Complete Milestone 5 hardening, collateral updates, and final quality/perf gates.

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

## Outcomes & Retrospective

Plan-authoring outcome: bottlenecks are identified with reproducible evidence, implementation is split into independently verifiable milestones, and the measurement path is now aligned with repository reality (existing perf harness and artifacts).

Implementation outcome is pending. Expected end state remains at least `3-6x` speedup from current warm baseline, with likely `5-12x` speedup on large FST windows when streaming fast path is enabled.

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

Secondary acceptance:

- `python3 bench/e2e/perf.py compare` must show no regressions for matching tests in the evaluated set (`--max-negative-delta-pct 0` for dedicated coremark run; `5` for broader matrix sweep against shared baseline).
- Final release-candidate expectation on target hardware/profile remains approximately `6-15s` for the 2-signal coremark query, with common outcomes near `8-12s`.

### Idempotence and Recovery

All commands are repeatable. `perf.py run --run-dir <dir>` rewrites matching JSON artifacts and refreshes `README.md`; rerunning does not corrupt unrelated run directories.

If a run is interrupted, rerun the same command with the same `--run-dir` to regenerate incomplete artifacts.

If correctness regresses in any milestone, revert only that milestone code path and keep equivalence tests as persistent guards.

If stream fast path diverges on edge cases, keep dispatch fallback to Milestone-2 random-access path and continue with correctness first.

If performance noise obscures conclusions, rerun the same run command and compare medians/means across repeated revised directories; do not accept/reject on one outlier sample.

### Artifacts and Notes

Artifacts produced by this plan should be tracked under dedicated directories:

- `bench/e2e/runs/change-stateless-golden/` (baseline for this plan)
- `bench/e2e/runs/change-stateless-m2/`
- `bench/e2e/runs/change-stateless-m3/`
- `bench/e2e/runs/change-stateless-m4/`
- `bench/e2e/runs/change-stateless-final-matrix/`

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
