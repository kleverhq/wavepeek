# FSDB random sampling performance report

This report tracks before/after evidence for GitHub issue #47 on branch `fix/fsdb-random-sampling` in worktree `/workspaces/wavepeek/.worktrees/fix-fsdb-random-sampling`.

## Environment

- Date: 2026-07-02.
- Baseline commit: `4b35f03d9f47`.
- Container: `WAVEPEEK_IN_CONTAINER=1`.
- Verdi FSDB Reader SDK: `python3 -B tools/fsdb/check_fsdb_env.py` reported `ok: fsdb: Verdi FSDB Reader SDK found`.
- RTL artifacts root: `/opt/rtl-artifacts`.
- Binary: `target/fsdb/debug/wavepeek`, built with `CARGO_TARGET_DIR=target/fsdb cargo build --features fsdb`.

## Baseline commands

FSDB fixtures were prepared and verified with:

    just prepare-and-check-fsdb-rtl-artifacts

A reduced debug benchmark catalog was generated from `bench/e2e/tests_fsdb.json` by selecting these tests and setting `runs=1`, `warmup=0`:

- `extract_scr1_coremark_dmem_axi_1ch_cli`
- `extract_scr1_coremark_dmem_axi_2ch_source`
- `extract_scr1_coremark_dmem_axi_5ch_source`

The benchmark command was:

    python3 -B bench/e2e/perf.py run \
      --binary baseline=target/fsdb/debug/wavepeek \
      --tests tmp/fsdb-random-sampling/baseline/extract_fsdb_debug_tests.json \
      --run-dir tmp/fsdb-random-sampling/baseline/bench \
      --wavepeek-timeout-seconds 900 \
      --verbose

Focused DEBUG diagnostics were captured with shell `time` because `/usr/bin/time` is not available in the container:

    { time env DEBUG=1 target/fsdb/debug/wavepeek extract generic \
      --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fsdb \
      --scope TOP.scr1_top_tb_axi.i_top \
      --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_2ch.json \
      --max unlimited \
      --jsonl \
      > tmp/fsdb-random-sampling/baseline/extract_2ch.jsonl; } \
      2> tmp/fsdb-random-sampling/baseline/extract_2ch.debug.log

    { time env DEBUG=1 target/fsdb/debug/wavepeek extract generic \
      --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fsdb \
      --scope TOP.scr1_top_tb_axi.i_top \
      --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_5ch.json \
      --max unlimited \
      --jsonl \
      > tmp/fsdb-random-sampling/baseline/extract_5ch.jsonl; } \
      2> tmp/fsdb-random-sampling/baseline/extract_5ch.debug.log

## Baseline benchmark summary

| Workload | Mean | User | System |
| --- | ---: | ---: | ---: |
| 1-channel CLI | 51.409s | 34.398s | 17.049s |
| 2-channel JSON source | 60.227s | 41.074s | 19.198s |
| 5-channel JSON source | 85.989s | 58.080s | 28.022s |

## Baseline DEBUG summary

| Metric | 2-channel | 5-channel |
| --- | ---: | ---: |
| Shell `real` | 61.107s | 86.282s |
| Shell `user` | 41.533s | 58.160s |
| Shell `sys` | 19.576s | 28.159s |
| DEBUG elapsed (`extract.emit.done`) | 60.290s | 85.161s |
| Candidate timestamps | 1,248,862 | 1,248,862 |
| Event matches | 624,423 | 624,423 |
| Rows emitted | 9,878 | 20,242 |
| JSONL records | 9,881 | 20,245 |
| `expr_sample_cache_misses` | 4,380,878 | 6,264,511 |
| `expr_sample_cache_hits` | 1 | 1 |
| `expr_sample_cache_len` | 4,380,878 | 6,264,511 |
| `sample_resolved_calls` | 4,390,756 | 6,284,753 |
| `sample_resolved_idcodes` | 4,420,390 | 6,364,263 |
| `sample_resolved_native_ns` | 34.632s | 49.066s |
| `expr_sample_uncached_ns` | 41.097s | 58.074s |
| `collect_change_times_calls` | 1 | 1 |
| `collect_change_times_ns` | 0.543s | 0.553s |
| `signal_session_opens` | 8 | 17 |
| `signal_session_reuses` | 4,390,749 | 6,284,737 |
| `loaded_session_idcodes` | 12 | 30 |

## Baseline conclusion

The baseline reproduces issue #47. Candidate collection is below one second, while expression sampling causes millions of cache misses and native sampling calls. `signal_session_reuses` is high, so repeated session opening is not the dominant cost. The hot path to remove is per-value random native sampling during expression evaluation.

## After-fix results

Pending. Re-run the same benchmark and DEBUG diagnostics into `tmp/fsdb-random-sampling/after/` after implementation, then update this section with before/after elapsed time, DEBUG counters, and functional parity notes.
