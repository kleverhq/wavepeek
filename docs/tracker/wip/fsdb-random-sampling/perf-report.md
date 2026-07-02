# FSDB random sampling performance report

This report tracks before/after evidence for GitHub issue #47 on branch `fix/fsdb-random-sampling` in worktree `/workspaces/wavepeek/.worktrees/fix-fsdb-random-sampling`.

## Environment

- Date: 2026-07-02.
- Baseline commit: `4b35f03d9f47`.
- After-fix commit: `de5fc613b095`.
- Container: `WAVEPEEK_IN_CONTAINER=1`.
- Verdi FSDB Reader SDK: `python3 -B tools/fsdb/check_fsdb_env.py` reported `ok: fsdb: Verdi FSDB Reader SDK found`.
- RTL artifacts root: `/opt/rtl-artifacts`.
- Binary: `target/fsdb/debug/wavepeek`, built with `CARGO_TARGET_DIR=target/fsdb cargo build --features fsdb`.

## Commands

FSDB fixtures were prepared and verified with:

    just prepare-and-check-fsdb-rtl-artifacts

A reduced debug benchmark catalog was generated from `bench/e2e/tests_fsdb.json` by selecting these tests and setting `runs=1`, `warmup=0`:

- `extract_scr1_coremark_dmem_axi_1ch_cli`
- `extract_scr1_coremark_dmem_axi_2ch_source`
- `extract_scr1_coremark_dmem_axi_5ch_source`

Baseline benchmark:

    python3 -B bench/e2e/perf.py run \
      --binary baseline=target/fsdb/debug/wavepeek \
      --tests tmp/fsdb-random-sampling/baseline/extract_fsdb_debug_tests.json \
      --run-dir tmp/fsdb-random-sampling/baseline/bench \
      --wavepeek-timeout-seconds 900 \
      --verbose

After-fix benchmark:

    python3 -B bench/e2e/perf.py run \
      --binary after=target/fsdb/debug/wavepeek \
      --tests tmp/fsdb-random-sampling/after/extract_fsdb_debug_tests.json \
      --run-dir tmp/fsdb-random-sampling/after/bench \
      --wavepeek-timeout-seconds 900 \
      --verbose

Focused DEBUG diagnostics used shell `time` because `/usr/bin/time` is not available in the container. The 2ch and 5ch commands used the same form with baseline and after output directories:

    { time env DEBUG=1 target/fsdb/debug/wavepeek extract generic \
      --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fsdb \
      --scope TOP.scr1_top_tb_axi.i_top \
      --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_2ch.json \
      --max unlimited \
      --jsonl \
      > tmp/fsdb-random-sampling/<phase>/extract_2ch.jsonl; } \
      2> tmp/fsdb-random-sampling/<phase>/extract_2ch.debug.log

    { time env DEBUG=1 target/fsdb/debug/wavepeek extract generic \
      --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fsdb \
      --scope TOP.scr1_top_tb_axi.i_top \
      --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_5ch.json \
      --max unlimited \
      --jsonl \
      > tmp/fsdb-random-sampling/<phase>/extract_5ch.jsonl; } \
      2> tmp/fsdb-random-sampling/<phase>/extract_5ch.debug.log

## Benchmark comparison

| Workload | Baseline mean | After mean | Speedup | Reduction |
| --- | ---: | ---: | ---: | ---: |
| 1-channel CLI | 51.409s | 11.368s | 4.52x | 77.9% |
| 2-channel JSON source | 60.227s | 12.821s | 4.70x | 78.7% |
| 5-channel JSON source | 85.989s | 18.335s | 4.69x | 78.7% |

## Release reference

The release reference was measured on commit `f3215b4` with the optimized FSDB binary:

    CARGO_TARGET_DIR=target/fsdb cargo build --release --features fsdb
    python3 -B bench/e2e/perf.py run \
      --binary release=target/fsdb/release/wavepeek \
      --tests tmp/fsdb-random-sampling/release/extract_fsdb_release_tests.json \
      --run-dir tmp/fsdb-random-sampling/release/bench \
      --wavepeek-timeout-seconds 900 \
      --verbose

The catalog used the standard `bench/e2e/tests_fsdb.json` run counts for these three workloads: `runs=10`, `warmup=5`.

| Workload | Mean ± σ | Median | Min | Max | User | System |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1-channel CLI | 1.535s ± 0.016s | 1.540s | 1.488s | 1.542s | 1.346s | 0.162s |
| 2-channel JSON source | 1.730s ± 0.022s | 1.740s | 1.689s | 1.741s | 1.543s | 0.172s |
| 5-channel JSON source | 2.418s ± 0.039s | 2.393s | 2.389s | 2.508s | 2.229s | 0.174s |

Hyperfine reported statistical outliers for all three release runs. The variation is small relative to the means, so these values are suitable as reference numbers rather than formal baselines.

## Full FSDB catalog against main

A full release FSDB catalog comparison was measured after the fix using the current branch binary against `main`.

- Main commit: `d915a0f5d4fa`.
- Current commit used for the run: `dc10e9944a36`.
- Test catalog: `bench/e2e/tests_fsdb.json`.
- Catalog size: 153 tests per binary.
- Successful benchmark pairs: 141.
- Symmetric preflight failures: 12 tests failed before timing on both binaries.
- Raw artifacts: `tmp/fsdb-random-sampling/full-fsdb/bench/`.

Commands:

    git worktree add --detach tmp/fsdb-random-sampling/main-worktree main
    CARGO_TARGET_DIR=target/fsdb cargo build --release --features fsdb
    cd tmp/fsdb-random-sampling/main-worktree
    CARGO_TARGET_DIR=/workspaces/wavepeek/.worktrees/fix-fsdb-random-sampling/target/fsdb-main cargo build --release --features fsdb
    cd /workspaces/wavepeek/.worktrees/fix-fsdb-random-sampling
    python3 -B bench/e2e/perf.py run \
      --binary main=target/fsdb-main/release/wavepeek \
      --binary current=target/fsdb/release/wavepeek \
      --tests bench/e2e/tests_fsdb.json \
      --run-dir tmp/fsdb-random-sampling/full-fsdb/bench \
      --wavepeek-timeout-seconds 900 \
      --verbose

Category totals use the sum of per-test hyperfine means for successful pairs. Geometric speedup is the geometric mean of per-test speedups within the category.

| Category | Successful pairs | Main total | Current total | Total speedup | Geomean speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `change` | 105 | 32.433s | 32.907s | 0.99x | 1.00x |
| `extract` | 3 | 98.017s | 6.339s | 15.46x | 15.39x |
| `info` | 8 | 0.384s | 0.381s | 1.01x | 1.01x |
| `property` | 6 | 1.817s | 1.831s | 0.99x | 0.97x |
| `scope` | 3 | 0.534s | 0.534s | 1.00x | 1.00x |
| `signal` | 3 | 0.141s | 0.147s | 0.96x | 0.97x |
| `value` | 13 | 2.722s | 2.741s | 0.99x | 0.99x |

Extract workloads dominate the branch-level improvement:

| Workload | Main mean | Current mean | Speedup |
| --- | ---: | ---: | ---: |
| `extract_scr1_coremark_dmem_axi_1ch_cli` | 25.565s | 1.716s | 14.89x |
| `extract_scr1_coremark_dmem_axi_2ch_source` | 29.841s | 1.928s | 15.47x |
| `extract_scr1_coremark_dmem_axi_5ch_source` | 42.611s | 2.694s | 15.82x |

The non-extract categories are essentially flat in aggregate. Individual cases with more than 5% apparent slowdown were small or noisy enough to need a focused rerun before treating them as regressions. The largest were:

| Workload | Category | Main mean | Current mean | Change |
| --- | --- | ---: | ---: | ---: |
| `property_scr1_coremark_imem_axi_araddr_to_200ps_match_posedge_clk_sample_pre_edge` | `property` | 0.071s | 0.079s | +11.2% |
| `signal_scr1_top_recursive_filter_valid_json` | `signal` | 0.046s | 0.051s | +10.0% |
| `change_chipyard_dualrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk` | `change` | 1.251s | 1.346s | +7.7% |
| `change_chipyard_dualrocketconfig_dhrystone_signals_1_window_32us_trigger_any` | `change` | 0.168s | 0.179s | +6.9% |
| `change_chipyard_dualrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk` | `change` | 4.478s | 4.769s | +6.5% |

The symmetric preflight failures were:

- `change_scr1_signals_100_pos_50_window_2ns_trigger_any`
- `change_scr1_signals_100_window_2ns_trigger_posedge_clk`
- `change_scr1_signals_100_window_2ns_trigger_signal`
- `change_scr1_signals_100_window_4ns_trigger_any`
- `change_scr1_signals_100_window_4ns_trigger_posedge_clk`
- `change_scr1_signals_100_window_4ns_trigger_signal`
- `change_scr1_signals_100_window_8ns_trigger_any`
- `change_scr1_signals_100_window_8ns_trigger_posedge_clk`
- `change_scr1_signals_100_window_8ns_trigger_signal`
- `value_chipyard_clusteredrocketconfig_dhrystone_signals_1000`
- `value_chipyard_dualrocketconfig_dhrystone_signals_1000`
- `value_scr1_signals_1000`

## DEBUG comparison

| Metric | 2ch baseline | 2ch after | 5ch baseline | 5ch after |
| --- | ---: | ---: | ---: | ---: |
| Shell `real` | 61.107s | 13.666s | 86.282s | 19.328s |
| Shell `user` | 41.533s | 13.345s | 58.160s | 18.939s |
| Shell `sys` | 19.576s | 0.325s | 28.159s | 0.396s |
| DEBUG elapsed (`extract.emit.done`) | 60.290s | 13.622s | 85.161s | 19.275s |
| Candidate timestamps | 1,248,862 | 1,248,862 | 1,248,862 | 1,248,862 |
| Event matches | 624,423 | 624,423 | 624,423 | 624,423 |
| Rows emitted | 9,878 | 9,878 | 20,242 | 20,242 |
| JSONL records | 9,881 | 9,881 | 20,245 | 20,245 |
| `expr_sample_cache_misses` | 4,380,878 | 1 | 6,264,511 | 1 |
| `expr_sample_cache_len` | 4,380,878 | 1 | 6,264,511 | 1 |
| `timeline_preload_calls` | n/a | 2 | n/a | 2 |
| `timeline_preload_idcodes` | n/a | 12 | n/a | 30 |
| `timeline_preload_changes` | n/a | 1,296,820 | n/a | 1,327,654 |
| `timeline_preload_native_ns` | n/a | 0.519s | n/a | 0.530s |
| `timeline_sample_hits` | n/a | 4,420,390 | n/a | 6,364,263 |
| `timeline_sample_misses` | n/a | 1 | n/a | 1 |
| `sample_resolved_calls` | 4,390,756 | 1 | 6,284,753 | 1 |
| `sample_resolved_idcodes` | 4,420,390 | 1 | 6,364,263 | 1 |
| `sample_resolved_native_ns` | 34.632s | 0.000070s | 49.066s | 0.000078s |
| `expr_sample_uncached_ns` | 41.097s | 0.000089s | 58.074s | 0.000096s |
| `collect_change_times_ns` | 0.543s | 0.588s | 0.553s | 0.545s |
| `signal_session_opens` | 8 | 3 | 17 | 3 |
| `signal_session_reuses` | 4,390,749 | 1 | 6,284,737 | 1 |
| `loaded_session_idcodes` | 12 | 12 | 30 | 30 |

## Functional parity

The after-fix JSONL outputs match the baseline outputs byte-for-byte:

    cmp -s tmp/fsdb-random-sampling/baseline/extract_2ch.jsonl tmp/fsdb-random-sampling/after/extract_2ch.jsonl
    cmp -s tmp/fsdb-random-sampling/baseline/extract_5ch.jsonl tmp/fsdb-random-sampling/after/extract_5ch.jsonl

Both commands returned exit status 0. Row counts remained unchanged.

## Conclusion

The original hot path has been removed for the targeted unlimited extract workload. Native random sampling dropped from millions of calls to one fallback call, and timeline hits account for the sampled values. The remaining time is dominated by Rust expression evaluation and row construction rather than FSDB random access. Debug-build elapsed time improved by about 4.7x on both 2ch and 5ch source workloads.
