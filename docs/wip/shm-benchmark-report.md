# SHM integration and benchmark results

Measured on 2026-10-16 in the worktree's private devcontainer, on an AMD Ryzen 7
PRO 8840HS. Builds use Rust 1.93.0, Ondas 1.1.0, Xcelium 25.03.001, Verdi
X-2025.06, and Hyperfine 1.18.0. SHM and FSDB have separate release binaries.

## Delivered behavior

Cargo feature `shm` enables native SHM directory input through Ondas. Metadata,
hierarchy, sampling, changes, properties, and all extraction families use the
existing engine. Disabled-feature diagnostics, escaped/indexed paths, native
string/real/event values, loader setup, conditional gates, fixture preparation,
and the SHM E2E catalog are covered. No dependency update or new parser is needed.

## Validation

`just ci` passed with both SDKs configured and again with `XCELIUM_HOME` and
`VERDI_HOME` unset in the same container. Source coverage is 93.01% lines, 91.85%
regions, and 91.66% functions, above the 90% thresholds.

The combined `fsdb,shm` check passed 773 tests: 690 library, 47 CLI contract,
24 FSDB CLI, and 12 SHM CLI tests. SHM-specific tests cover VCD parity, native
rich values, protocol extraction, JSON/JSONL, directory recognition, and errors.
SDK-free tests cover feature guidance and fixture tooling.

The normal host hooks passed for implementation commit `e317ec1`, including
Rust tests, auxiliary tests, both native benchmark smokes, and commit style.
`just check` also passed, including browser/WASM, docs, and direct SDK-linked
binary launches.

## Method

Use all 156 named scenarios from `bench/e2e/tests.json`, with equivalent converted
inputs. Each scenario has two warmups and five measured runs; the invocation cap
is 120 seconds. Functional capture precedes timing. Run SHM, then FSDB, without
concurrent task builds or waveform conversion. Conversion time is excluded.

SHM completed all 156 scenarios without failures or timeouts. FSDB completed 144;
its 12 preflight failures exactly match the existing FSDB runnable-catalog
exclusions for VCD-style scalar array-element paths such as `araddr.[0]`.
There were no additional FSDB failures and no timeouts.

## Timing comparison

The table gives medians across paired successful scenarios. The ratio column is
the median of each scenario's SHM time divided by its FSDB time, rather than the
ratio of the two group medians. A ratio above 1 means SHM is slower.

| Command | Pairs | SHM median, ms | FSDB median, ms | SHM / FSDB |
| --- | ---: | ---: | ---: | ---: |
| `change` | 105 | 692.394 | 288.126 | 1.778 |
| `extract` | 6 | 1617.806 | 1666.784 | 0.987 |
| `info` | 8 | 30.838 | 41.770 | 0.734 |
| `property` | 6 | 839.531 | 939.036 | 0.707 |
| `scope` | 3 | 87.547 | 139.905 | 0.652 |
| `signal` | 3 | 39.487 | 55.488 | 0.711 |
| `value` | 13 | 588.138 | 137.465 | 4.265 |

Across all 144 paired successes, the median ratio is 1.712 and the geometric mean
ratio is 1.671. Restricting to the 134 scenarios with exactly identical payloads
gives a median ratio of 1.776.

The largest paired slowdown is `value_picorv32_signals_1000`: SHM 6.854 seconds
versus FSDB 0.188 seconds, or 36.389 times slower. The two Chipyard 1000-signal
queries complete in SHM, including 20.787 seconds for clustered Dhrystone, but
cannot be paired because FSDB rejects their scalar element paths.

The strongest paired speedup is `change_picorv32_signals_1_window_8us_trigger_signal`:
SHM 55.943 ms versus FSDB 287.729 ms, or 5.143 times faster. These are results for
the selected fixtures and workloads, not a format-wide performance guarantee.

## Functional comparison

SHM and FSDB payloads match exactly in 134 of the 144 paired successes. The ten
differences are three signal listings, two scope listings, three start-bound
metadata cases, and two initial change-row cases.

All 156 original FST scenarios also completed a separate functional capture.
SHM matches FST exactly in 148 cases. The eight differences are fully accounted
for as follows.

The three SCR1 signal listings have identical paths, widths, ordering, and row
counts after excluding `kind`; converted SHM declarations expose `bit_vector`.
The three SCR1 `info` cases report SHM start `0ps` versus FST/FSDB `1ps`, with
matching units and end bounds. Consequently, the one- and two-signal SCR1 IMEM
`change` scenarios include one additional SHM row at `1ps`. Removing that row
makes the remaining 79 rows exactly equal to FST/FSDB. The earliest FST/FSDB row
is `112ps`. The SHM adapter reports its reader's bounds without adjusting them
to another format's metadata.

Both differing FSDB scope listings match FST in SHM; the difference is the
existing FSDB representation of array hierarchy. All sampled `value` results,
protocol extraction results, and property results match the FST reference.

## Reproduction and evidence

From the host, run the focused entrypoint with a private profile:

    ./dev just bench-e2e-shm-run

The recorded comparison uses generated local catalog copies with `runs=5` and
`warmup=2`. Existing E2E commands reproduce capture and comparison:

    ./dev bash tools/shm/with_shm_env.sh python3 -B bench/e2e/perf.py run --binary subject=./tmp/shm/comparison/bin/wavepeek-shm --tests tmp/shm/comparison/tests_shm.json --run-dir tmp/shm/comparison/shm --wavepeek-timeout-seconds 120
    ./dev python3 -B bench/e2e/perf.py run --binary subject=./tmp/shm/comparison/bin/wavepeek-fsdb --tests tmp/shm/comparison/tests_fsdb.json --run-dir tmp/shm/comparison/fsdb --wavepeek-timeout-seconds 120

Raw [SHM results](../../tmp/shm/comparison/shm/subject/README.md),
[FSDB results](../../tmp/shm/comparison/fsdb/subject/README.md),
[timing rows](../../tmp/shm/comparison/timing-summary.json),
[FST comparison](../../tmp/shm/comparison/fst-reference.json), and
[measured binary snapshots and hashes](../../tmp/shm/comparison/binaries.json)
remain ignored local evidence. All SDKs, licenses, generated databases, and raw
benchmark artifacts remain outside tracked repository content.
