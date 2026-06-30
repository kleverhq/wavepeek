# Extract Generic SCR1 AXI Performance Baseline

## Context

- Date: 2026-06-30 UTC.
- Branch: `feat/cmd-extract`.
- Baseline SHA: `bc0643a903c9c349765f8af988c98c52adf1a665` (`perf(extract): trace generic extraction phases`).
- Binary: `./target/debug/wavepeek`.
- Waveform: `/opt/rtl-artifacts/scr1_max_axi_coremark.fst`.
- Scope: `TOP.scr1_top_tb_axi.i_top`.
- Debug trace mode: `DEBUG=1`, which emits JSON debug events to stderr.
- Output mode: `--jsonl`.
- Limit mode: `--max unlimited`.

The first 2-channel timing probe was run before the extra DEBUG timing events were committed. It used the same debug binary line on the preceding branch state and measured `23.916s` wall time with only `backend.open`, `extract.bind`, and `candidate.collect` events. The detailed phase measurements below were taken from the instrumented code now recorded at the baseline SHA.

## Waveform metadata

Command:

```bash
./target/debug/wavepeek info \
  --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst \
  --json > tmp/axi_prof_info.json
```

Observed metadata:

- `time_unit`: `1ps`
- `time_start`: `1ps`
- `time_end`: `6244302ps`
- Simulation span: `6244301ps`, approximately `6.244301us`.
- Raw inclusive 1ps positions in the dump window: `6,244,302`.
- Indexed FST timestamps reported by extract debug trace: `1,249,106`.
- Window timestamps for the measured runs: `1,249,106`.

## Source manifests

### 2-channel manifest

Path: `tmp/axi_dmem_extract.json`.

Sources:

| Source | Event | Predicate | Payload entries |
| --- | --- | --- | ---: |
| `axi_dmem_aw` | `posedge clk iff axi_rst_n` | `io_axi_dmem_awvalid && io_axi_dmem_awready` | 4 |
| `axi_dmem_ar` | `posedge clk iff axi_rst_n` | `io_axi_dmem_arvalid && io_axi_dmem_arready` | 4 |

### 5-channel manifest

Path: `tmp/axi_dmem_extract_5ch.json`.

Sources:

| Source | Event | Predicate | Payload entries |
| --- | --- | --- | ---: |
| `axi_dmem_aw` | `posedge clk iff axi_rst_n` | `io_axi_dmem_awvalid && io_axi_dmem_awready` | 5 |
| `axi_dmem_w` | `posedge clk iff axi_rst_n` | `io_axi_dmem_wvalid && io_axi_dmem_wready` | 4 |
| `axi_dmem_b` | `posedge clk iff axi_rst_n` | `io_axi_dmem_bvalid && io_axi_dmem_bready` | 3 |
| `axi_dmem_ar` | `posedge clk iff axi_rst_n` | `io_axi_dmem_arvalid && io_axi_dmem_arready` | 5 |
| `axi_dmem_r` | `posedge clk iff axi_rst_n` | `io_axi_dmem_rvalid && io_axi_dmem_rready` | 5 |

Signal discovery command used before creating the 5-channel manifest:

```bash
./target/debug/wavepeek signal \
  --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst \
  --scope TOP.scr1_top_tb_axi.i_top \
  --filter '^io_axi_dmem_' \
  --max unlimited \
  --json > tmp/axi_dmem_signals.json
```

Observed `io_axi_dmem_*` signal count: `44`.

## 2-channel extraction run

Command with DEBUG timing:

```bash
DEBUG=1 ./target/debug/wavepeek extract generic \
  --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst \
  --scope TOP.scr1_top_tb_axi.i_top \
  --source tmp/axi_dmem_extract.json \
  --max unlimited \
  --jsonl > tmp/axi_extract_profile2.jsonl \
  2> tmp/axi_extract_profile2.debug.jsonl
```

The wall/user/system/RSS values were measured with a Python `subprocess` wrapper using `time.perf_counter()` and `resource.getrusage(RUSAGE_CHILDREN)`.

Overall timing:

| Metric | Value |
| --- | ---: |
| Wall time | `23.418s` |
| User CPU | `23.304s` |
| System CPU | `0.111s` |
| Max RSS | `77,020 KiB` |

Phase timing from DEBUG trace:

| Phase | Time | Wall share |
| --- | ---: | ---: |
| Backend open | `0.090s` | `0.4%` |
| Metadata and time parsing | `0.0001s` | `<0.1%` |
| Source binding | `0.0006s` | `<0.1%` |
| Candidate collection | `2.465s` | `10.5%` |
| Emit/evaluate/sample/write | `20.853s` | `89.0%` |
| Finish after emit | `0.009s` | `<0.1%` |

Emit-loop counters:

| Counter | Value |
| --- | ---: |
| Candidate timestamps | `1,248,862` |
| Event checks | `2,497,724` |
| Event matches | `1,248,846` |
| Predicate-true rows | `9,878` |
| Emitted rows | `9,878` |
| Rows skipped for missing sample time | `0` |
| Truncated | `false` |

Emit-loop timing:

| Work | Time | Wall share |
| --- | ---: | ---: |
| Event matching | `16.039s` | `68.5%` |
| Predicate evaluation | `3.877s` | `16.6%` |
| Row build and sink emit | `0.495s` | `2.1%` |
| Emit-loop unattributed overhead | `0.443s` | `1.9%` |

JSONL output summary:

| Record type | Count |
| --- | ---: |
| `begin` | `1` |
| `item` | `9,878` |
| `diagnostic` | `1` |
| `end` | `1` |

Rows by source:

| Source | Rows |
| --- | ---: |
| `axi_dmem_ar` | `9,392` |
| `axi_dmem_aw` | `486` |

Additional observations:

- First emitted row time: `602ps`.
- Last emitted row time: `6243572ps`.
- JSONL file size: `4,294,747 bytes` (`4.1 MiB`).
- Diagnostic: `WPK-W0001`, `limit disabled: --max=unlimited`.

Non-DEBUG control timings:

| Output target | Wall | User | System | Max RSS | Output size |
| --- | ---: | ---: | ---: | ---: | ---: |
| File `tmp/axi_extract_nodebug_file.jsonl` | `23.425s` | `23.310s` | `0.112s` | `76,796 KiB` | `4.1 MiB` |
| `/dev/null` | `23.078s` | `23.028s` | `0.047s` | `77,108 KiB` | `0` |

## 5-channel extraction run

Command with DEBUG timing:

```bash
DEBUG=1 ./target/debug/wavepeek extract generic \
  --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst \
  --scope TOP.scr1_top_tb_axi.i_top \
  --source tmp/axi_dmem_extract_5ch.json \
  --max unlimited \
  --jsonl > tmp/axi_extract_5ch_profile.jsonl \
  2> tmp/axi_extract_5ch_profile.debug.jsonl
```

A binding smoke test was run first with the same command and `--max 1`.

Overall timing:

| Metric | Value |
| --- | ---: |
| Wall time | `57.683s` |
| User CPU | `57.411s` |
| System CPU | `0.266s` |
| Max RSS | `76,944 KiB` |

Phase timing from DEBUG trace:

| Phase | Time | Wall share |
| --- | ---: | ---: |
| Backend open | `0.090s` | `0.2%` |
| Metadata and time parsing | `0.0001s` | `<0.1%` |
| Source binding | `0.001s` | `<0.1%` |
| Candidate collection | `2.644s` | `4.6%` |
| Emit/evaluate/sample/write | `54.938s` | `95.2%` |
| Finish after emit | `0.010s` | `<0.1%` |

Emit-loop counters:

| Counter | Value |
| --- | ---: |
| Candidate timestamps | `1,248,862` |
| Event checks | `6,244,310` |
| Event matches | `3,122,115` |
| Predicate-true rows | `20,242` |
| Emitted rows | `20,242` |
| Rows skipped for missing sample time | `0` |
| Truncated | `false` |

Emit-loop timing:

| Work | Time | Wall share |
| --- | ---: | ---: |
| Event matching | `42.193s` | `73.1%` |
| Predicate evaluation | `10.584s` | `18.3%` |
| Row build and sink emit | `1.378s` | `2.4%` |
| Emit-loop unattributed overhead | `0.782s` | `1.4%` |

JSONL output summary:

| Record type | Count |
| --- | ---: |
| `begin` | `1` |
| `item` | `20,242` |
| `diagnostic` | `1` |
| `end` | `1` |

Rows by source:

| Source | Rows |
| --- | ---: |
| `axi_dmem_ar` | `9,392` |
| `axi_dmem_r` | `9,392` |
| `axi_dmem_aw` | `486` |
| `axi_dmem_w` | `486` |
| `axi_dmem_b` | `486` |

Additional observations:

- First emitted row time: `602ps`.
- Last emitted row time: `6243582ps`.
- JSONL file size: `10,031,969 bytes` (`9.6 MiB`).
- Diagnostic: `WPK-W0001`, `limit disabled: --max=unlimited`.

Non-DEBUG control timings:

| Output target | Wall | User | System | Max RSS | Output size |
| --- | ---: | ---: | ---: | ---: | ---: |
| File `tmp/axi_extract_5ch_nodebug_file.jsonl` | `56.535s` | `56.246s` | `0.282s` | `80,684 KiB` | `9.6 MiB` |
| `/dev/null` | `56.607s` | `56.519s` | `0.082s` | `80,616 KiB` | `0` |

## Comparison

| Metric | 2 channels | 5 channels | Ratio |
| --- | ---: | ---: | ---: |
| Non-DEBUG wall to `/dev/null` | `23.078s` | `56.607s` | `2.45x` |
| DEBUG wall | `23.418s` | `57.683s` | `2.46x` |
| Candidate timestamps | `1,248,862` | `1,248,862` | `1.00x` |
| Event checks | `2,497,724` | `6,244,310` | `2.50x` |
| Event matches | `1,248,846` | `3,122,115` | `2.50x` |
| Emitted rows | `9,878` | `20,242` | `2.05x` |
| Event matching time | `16.039s` | `42.193s` | `2.63x` |
| Predicate evaluation time | `3.877s` | `10.584s` | `2.73x` |
| Row build and emit time | `0.495s` | `1.378s` | `2.79x` |

## Analysis conclusions

- The measured runs are CPU-bound. User CPU time is close to wall time, and system CPU time is small.
- JSONL output is not the dominant cost. Redirecting 2-channel output to `/dev/null` saved about `0.35s`; redirecting 5-channel output to `/dev/null` did not produce a meaningful improvement over file output in this run.
- Backend open and binding are negligible for these runs. Candidate collection is visible but not dominant: `2.465s` for 2 channels and `2.644s` for 5 channels.
- The dominant phase is emit/evaluate/sample/write: `89.0%` of 2-channel wall time and `95.2%` of 5-channel wall time in DEBUG runs.
- Inside the emit loop, event matching is the largest measured component: `16.039s` for 2 channels and `42.193s` for 5 channels.
- Event checks scale with the number of sources in these manifests because all sources share the same event expression and each source is evaluated independently at each candidate timestamp. The observed event-check ratio from 2 to 5 channels is `2.50x`, matching the source-count ratio.
- Predicate evaluation also grows with channel count and is the second largest measured component. The 5-channel predicate-evaluation time was `10.584s`.
- The 5-channel run emitted about `2.05x` as many rows as the 2-channel run, but total wall time grew about `2.45x`. The additional runtime is therefore not explained only by output row count.
