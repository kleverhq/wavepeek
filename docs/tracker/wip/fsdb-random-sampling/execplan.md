# ExecPlan: Reduce FSDB random sampling in expression evaluation

This plan fixes GitHub issue #47, "FSDB performance bottleneck: per-value random sampling during expression evaluation." Users should be able to run expression-heavy FSDB commands such as `extract generic` and `property` without paying millions of native random `ffrGotoXTag` lookups. The visible outcome is lower elapsed time and DEBUG counters showing that expression sampling is served from an FSDB value-change timeline cache instead of one native sample call per `(signal, timestamp)` pair.

The work happens in git worktree `/workspaces/wavepeek/.worktrees/fix-fsdb-random-sampling` on branch `fix/fsdb-random-sampling`. Keep all source, docs, reports, and commits in this worktree. Disposable logs and generated benchmark artifacts may use repository-root `tmp/`, but the final human-readable report must be written under `docs/tracker/wip/fsdb-random-sampling/`.

## Repository orientation

`wavepeek` is a Rust CLI for deterministic waveform inspection. The regular VCD/FST path is implemented with the Rust `wellen` library in `src/waveform/wellen_backend.rs`. Optional FSDB support is implemented by Rust wrappers in `src/waveform/fsdb_backend.rs` and `src/waveform/fsdb_native.rs`, plus a C++ native shim in `native/fsdb/wavepeek_fsdb_shim.cpp` and its C ABI header `native/fsdb/wavepeek_fsdb_shim.h`. Commands such as `extract generic`, `property`, and expression-heavy `change` bind expressions through `src/engine/expr_runtime.rs` and evaluate them through `src/waveform/expr_host.rs`, which calls `Waveform::sample_expr_value` and `Waveform::expr_event_occurred`.

A value-change is one recorded change of one signal at one waveform timestamp. A random sample means asking the FSDB Reader for the last value at or before one timestamp, currently via a native traversal that calls `ffrGotoXTag`. A timeline cache means reading each selected signal sequentially over a time window once, storing `time_raw` plus decoded bit strings in Rust, and answering later expression samples with binary search. Binary search means locating the greatest cached timestamp not greater than the query timestamp.

## Problem statement

Issue #47 reports that SCR1 AXI `extract generic` workloads are about 17x slower on FSDB than equivalent FST workloads. A DEBUG run on the two-channel source workload recorded about 4.38 million expression sample cache misses and 35.454 seconds inside native FSDB sampling. Candidate time collection took only 0.558 seconds, and signal sessions were reused, so the bottleneck is the per-value native random sample path rather than command scheduling or session setup.

The current hot path is:

    extract/property/change command loop
      expression evaluation
        WaveformExprHost::sample_value(signal, timestamp)
          Waveform::sample_expr_value
            FsdbBackend::sample_expr_value
              FsdbBackend::sample_resolved_optional([one signal], timestamp)
                FsdbSignalSession::sample_signal_values([one idcode], timestamp)
                  native wp_fsdb_signal_session_sample
                    ffrGotoXTag(timestamp)

The existing `expr_sample_cache` in `FsdbBackend` is keyed by `(SignalId, timestamp)`. It does not help when the workload evaluates mostly unique signal-time pairs.

## Constraints and safety

FSDB support requires the local Verdi FSDB Reader SDK. This work assumes the devcontainer environment reports `ok: fsdb: Verdi FSDB Reader SDK found` from `python3 -B tools/fsdb/check_fsdb_env.py`. Do not commit generated `.fsdb` files, proprietary Verdi headers, copied SDK declarations, or debug dumps. Treat waveform dumps as binary data and inspect them only through `wavepeek`, Verdi tools, or binary-safe metadata commands.

The native C++ shim must expose only project-owned ABI declarations in `native/fsdb/wavepeek_fsdb_shim.h`; do not copy proprietary header text or manual excerpts. Keep repository artifacts concise and neutral. Use `just` recipes for repository gates. Commit each independently useful milestone with a conventional commit message.

## Baseline and evidence protocol

Before implementing the fix, collect baseline evidence on the current commit. Use debug builds because the issue asks for DEBUG counters from the debug FSDB binary. Prepare FSDB RTL artifacts first, then build the debug binary:

    cd /workspaces/wavepeek/.worktrees/fix-fsdb-random-sampling
    mkdir -p tmp/fsdb-random-sampling/baseline
    just prepare-and-check-fsdb-rtl-artifacts
    CARGO_TARGET_DIR=target/fsdb cargo build --features fsdb

Create a small temporary benchmark catalog by selecting the SCR1 AXI extract workloads from `bench/e2e/tests_fsdb.json` and overriding them to one measured run and no warmup. The full catalog run counts are intentionally not used for debug builds because they would spend many minutes repeating already known slow commands.

    python3 -B - <<'PY'
    import json, pathlib
    source = pathlib.Path('bench/e2e/tests_fsdb.json')
    out = pathlib.Path('tmp/fsdb-random-sampling/baseline/extract_fsdb_debug_tests.json')
    keep = {
        'extract_scr1_coremark_dmem_axi_1ch_cli',
        'extract_scr1_coremark_dmem_axi_2ch_source',
        'extract_scr1_coremark_dmem_axi_5ch_source',
    }
    data = json.loads(source.read_text(encoding='utf-8'))
    tests = []
    for test in data['tests']:
        if test['name'] in keep:
            test = dict(test)
            test['runs'] = 1
            test['warmup'] = 0
            tests.append(test)
    out.write_text(json.dumps({'tests': tests}, indent=2) + '\n', encoding='utf-8')
    print(out)
    PY

Run the debug benchmark gate against the current debug FSDB binary:

    python3 -B bench/e2e/perf.py run \
      --binary baseline=target/fsdb/debug/wavepeek \
      --tests tmp/fsdb-random-sampling/baseline/extract_fsdb_debug_tests.json \
      --run-dir tmp/fsdb-random-sampling/baseline/bench \
      --wavepeek-timeout-seconds 900 \
      --verbose

Run focused DEBUG diagnostics for the two-channel and five-channel source workloads. Redirect stdout to files because JSONL output can be large; keep stderr because DEBUG events and backend counters are emitted there.

    /usr/bin/time -v env DEBUG=1 target/fsdb/debug/wavepeek extract generic \
      --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fsdb \
      --scope TOP.scr1_top_tb_axi.i_top \
      --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_2ch.json \
      --max unlimited \
      --jsonl \
      > tmp/fsdb-random-sampling/baseline/extract_2ch.jsonl \
      2> tmp/fsdb-random-sampling/baseline/extract_2ch.debug.log

    /usr/bin/time -v env DEBUG=1 target/fsdb/debug/wavepeek extract generic \
      --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fsdb \
      --scope TOP.scr1_top_tb_axi.i_top \
      --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_5ch.json \
      --max unlimited \
      --jsonl \
      > tmp/fsdb-random-sampling/baseline/extract_5ch.jsonl \
      2> tmp/fsdb-random-sampling/baseline/extract_5ch.debug.log

Extract the final `extract.emit.done` DEBUG event from each log and write a short baseline summary into `docs/tracker/wip/fsdb-random-sampling/perf-report.md`. At this stage the report must be clearly marked as baseline-only and updated later with after-fix results.

## Implementation strategy

The preferred fix is an FSDB value-change timeline cache. Add a native session API that reads selected signal value-change streams over a time range with sequential traversal. For each requested signal, the native function should return a sorted list of final bit-vector values by timestamp. It should include one initial sample at or before the requested `from_raw` timestamp when such a sample exists, because predicates at the start of a window need the current state even if the last transition occurred before the window.

The Rust FSDB backend should store these timelines per `SignalId`. A cache entry is valid for a loaded query window, for example `from_raw..=to_raw`. `FsdbBackend::sample_expr_value` should first try the timeline cache and return an integral `SampledValue` when the query timestamp is inside the cached window. `FsdbBackend::sample_resolved_optional` should also use cached timelines when all requested signals are covered, so payload sampling can benefit when payload signals were preloaded. If a signal or timestamp is not covered, the existing native random sampling path remains the fallback.

Expression-heavy commands must explicitly preload the signals they will evaluate. For `extract generic`, preload event operands, `iff` dependencies, predicate dependencies, and payload signals after binding and before the emit loop. For `property`, preload trigger and evaluation dependencies before the candidate loop. If change-command integration is small and safe after the generic backend API exists, preload its event and requested signals as well; otherwise leave change on the fallback path and document the remaining work in the report.

Add DEBUG counters to prove the hot path moved. Useful counters are timeline preload calls, requested idcodes, loaded value changes, native preload time, timeline sample hits, timeline sample misses, and native fallback calls. The successful signature is a large drop in `sample_resolved_calls` and `sample_resolved_native_ns` during the SCR1 extract DEBUG runs, with new timeline hit counters accounting for expression sampling.

## Milestones

Milestone 1 produces a committed, reviewed execution plan. Create `docs/tracker/wip/fsdb-random-sampling/execplan.md`, commit it, and run a read-only docs/design review against that commit. Acceptance is that the plan is self-contained, names the exact files and commands, and the review has no blocking findings or those findings are incorporated in a follow-up commit.

Milestone 2 captures baseline performance and diagnostics. Run the baseline protocol above, then create `docs/tracker/wip/fsdb-random-sampling/perf-report.md` with the current commit hash, environment notes, benchmark command lines, elapsed times, and DEBUG backend counters for 2ch and 5ch. Commit the report. Acceptance is that the report makes the existing bottleneck visible: millions of expression sample misses and native sample calls.

Milestone 3 adds the native sequential timeline API and Rust FFI plumbing. Edit `native/fsdb/wavepeek_fsdb_shim.h`, `native/fsdb/wavepeek_fsdb_shim.cpp`, and `src/waveform/fsdb_native.rs`. The native API must allocate and free timeline records safely, reject unsupported non-bit-vector encodings consistently with existing sampling, deduplicate equal-time changes by retaining the final value, and include an initial sample at or before the requested start time. Add or update FSDB tests that can run on generated VCD-derived FSDB fixtures. Commit this milestone after `CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb` and focused tests pass.

Milestone 4 adds the `FsdbBackend` timeline cache and preload hooks. Edit `src/waveform/fsdb_backend.rs`, `src/waveform/mod.rs`, and command engines such as `src/engine/extract.rs` and `src/engine/property.rs`. Add no-op behavior for non-FSDB backends so command code can call the preload API unconditionally. Update DEBUG stats. Commit this milestone after focused tests and a small manual FSDB command confirm functional output still matches existing behavior.

Milestone 5 runs code, architecture, and performance review on the implementation commits. Use read-only review subagents with separate lanes for native/FFI correctness, Rust command/backend correctness, and performance behavior. Fix substantive findings in follow-up commits and re-run relevant checks.

Milestone 6 captures after-fix performance evidence. Re-run the same debug benchmark gate and DEBUG 2ch/5ch diagnostics into `tmp/fsdb-random-sampling/after/`. Compare outputs against baseline where practical, at minimum by verifying command success and stable row counts or byte counts. Update `docs/tracker/wip/fsdb-random-sampling/perf-report.md` with before/after tables. Acceptance is a substantial reduction in native random sample calls and elapsed time; ideally FSDB approaches FST scale, but it is acceptable if the remaining hot path is FSDB open/load time rather than random sampling.

Milestone 7 runs final gates and final review. Run `just check` and FSDB-focused checks appropriate to the changed files. Run a final independent control review over the complete branch diff. Commit any final report or fix updates. Acceptance is a clean working tree, documented benchmark evidence, and no unresolved review findings except explicitly documented follow-up work.

## Validation commands

Use these commands as the minimum implementation gates, adapting only when a prior command proves the environment lacks optional FSDB resources. In this worktree Verdi is expected to be present.

    python3 -B tools/fsdb/check_fsdb_env.py
    CARGO_TARGET_DIR=target/fsdb cargo check --features fsdb
    CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --lib fsdb_expr_event_occurred_rejects_non_event_signal -- --nocapture
    CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --test fsdb_cli
    just check

For performance evidence, use the baseline protocol before implementation and the same commands under `tmp/fsdb-random-sampling/after/` after implementation. The final report must compare the same workloads and binary type before and after the fix.

## Progress

- [x] Read issue #47 and repository context for FSDB sampling, expression hosts, extract workloads, and benchmark fixtures.
- [x] Draft the self-contained ExecPlan under `docs/tracker/wip/fsdb-random-sampling/execplan.md`.
- [x] Commit the ExecPlan.
- [ ] Run read-only review of the ExecPlan and incorporate any blocking findings.
- [ ] Capture baseline debug benchmark and DEBUG diagnostic evidence.
- [ ] Commit the baseline report.
- [ ] Implement native sequential timeline API and Rust FFI wrappers.
- [ ] Implement FSDB timeline cache, preload API, command hooks, and DEBUG counters.
- [ ] Run focused tests and implementation review.
- [ ] Capture after-fix benchmark and DEBUG diagnostic evidence.
- [ ] Update and commit the final performance report.
- [ ] Run final gates and control review.

## Surprises & Discoveries

The existing DEBUG evidence in issue #47 already isolates the bottleneck: `collect_change_times` and signal-session opening are not dominant, while `sample_resolved_calls` and `expr_sample_cache_misses` are in the millions. The local repository confirms this path in `src/waveform/fsdb_backend.rs`: `sample_expr_value_uncached` samples a one-element `ResolvedSignal` slice and reaches `FsdbSignalSession::sample_signal_values` for each unique expression sample.

The FST path in `src/waveform/wellen_backend.rs` has indexed timestamps and loaded signal offsets, while the FSDB backend currently returns `None` for `indexed_timestamps`. This explains why FST expression sampling can be served from loaded signal data while FSDB expression sampling falls back to native random access.

## Decision Log

Decision: implement a reusable FSDB timeline cache instead of a command-specific bulk query API. Rationale: issue #47 affects `extract generic`, `property`, and likely future expression-driven commands. A backend cache makes `WaveformExprHost::sample_value` faster without teaching every expression operator about FSDB.

Decision: keep the existing native random sampling path as fallback. Rationale: small one-off `value` commands and unpreloaded windows should retain existing behavior, and the fallback limits correctness risk while the cache is introduced incrementally.

Decision: store final value per timestamp in the timeline cache. Rationale: the existing sampling path drains equal-time records and returns the final value at that timestamp, so retaining final values preserves current user-visible semantics.

Decision: collect baseline and after-fix evidence with debug builds and reduced benchmark repetitions. Rationale: the requested DEBUG counters require debug binaries, and full release benchmark repetition counts would be too slow for iterative development while adding little diagnostic value.

## Outcomes & Retrospective

No implementation outcome yet. This section will be updated after each major milestone with what changed, which commands passed, and what remains.

## Revision notes

2026-07-02: Initial ExecPlan created from issue #47 and repository context. The plan includes baseline collection, implementation strategy, review gates, after-fix evidence, and final report requirements.

2026-07-02: Marked the plan commit milestone complete immediately before committing the plan, so the committed plan records its own milestone state.
