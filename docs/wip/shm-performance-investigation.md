# SHM performance investigation

## Result

The per-scenario 2x FSDB target is not met. Common selected-declaration lookup
reduces wavepeek's indexing costs and replaces separate format lookup
algorithms. Cross-format controls, regression gates, and full functional
recapture pass. In the complete direct matrix, 51 of 134 identical-payload
pairs exceed 2x; one is borderline and measures 1.984x in a 20-run confirmation.
The largest remaining
delays are reproducible inside a single Ondas 1.1.0 call without wavepeek. New
format-specific query strategies are excluded by the user's architecture
requirement.

The initial cost was not uniformly attributable to Ondas: full canonical
indexing cost about 170–200 ms in large Chipyard cases. Some commands also make
separate candidate and payload queries. These mixed cases motivated the common
lookup change and remain distinct from demonstrated library bottlenecks.

## Scope and method

The original complete matrix and environment are recorded in
[the integration report](shm-benchmark-report.md). It contains 156 successful SHM
cases and 144 paired FSDB successes, including 55 ratios above 2. All 55 slow
pairs have exactly equal SHM and FSDB payloads in that baseline.

Diagnostic builds use the published Ondas 1.1.0 dependency and the same dependency
versions as wavepeek's lockfile. A temporary standalone Rust consumer opens the
same inputs, resolves the requested signals, then makes one `samples()` call for
`value` or one `traces()` call for a `change` window. Change selections also
include their explicit trigger. Each of the 55 SHM cases has three repetitions.
These are diagnostic operations, not replacements for the CLI workload: they
do not evaluate expressions, render rows, or implement output limits.

The consumer's simplified name resolver is suitable only for the selected
ordinary benchmark paths. It is not a proposed canonical-path implementation.
Its native query timer excludes name resolution. All 13 point cases have been
verified against wavepeek's rendered values and against raw FSDB bits. The
95-path PicoRV32 traces have matching entering values and all 16,318 changes in
SHM and FSDB, not only matching counts. Mixed-X hex digits cannot be expanded
back into exact bits; equivalence checks render native bits forward instead.

An initial direct Hyperfine capture of all 55 baseline pairs uses five measured runs
and two warmups, alternating the format order between scenarios. It launches
the preserved release binaries directly, without the timed Python wrapper.
Every functional preflight is checked against its original same-format capture.
`DEBUG=1` is enabled only for those untimed preflights. Builds, conversion,
tests, and other benchmarks do not run concurrently with timing.

All 55 direct pairs completed without failure, matched their original
same-format captures, and also matched across formats. All remain above 2x in
this control. In 42 cases the median native SHM `open` plus query alone exceeds
twice the direct FSDB CLI median, even excluding consumer resolution and waveform
destruction. The other 13 are mixed cases; this comparison does not establish a
library-only limit for them. These counts apply to the selected slow subset,
not to the full 144-pair matrix.

The existing timed wrapper uses `subprocess.run(timeout=...)` with inherited
output. Its process waiting can add polling delays up to approximately 50 ms,
besides Python startup. The old results show corresponding time steps. Direct
control measurements therefore have a different measurement boundary and must
not be presented as speedups from a code change.

## Final measurements

The revised full direct capture includes all 144 successful FSDB pairs, with
five measurements and two warmups per format. All preflights preserve the
original same-format payloads. The median ratio is 1.460 over all 144 pairs and
1.593 over the 134 identical-payload pairs. Both sets have 51 ratios above 2;
an aggregate median below 2 does not meet the per-scenario target.

Among the 55 initially slow pairs, 11 now meet 2x in this capture. Their median
ratio moves from 4.539 to 4.123; the median SHM speedup is 1.147x and the largest
is 1.886x. These before/after comparisons use the same direct measurement method.

Representative process medians, in seconds:

| Scenario | SHM before | SHM after | FSDB after | After ratio |
| --- | ---: | ---: | ---: | ---: |
| ClusteredRocket `change`, 1 signal, 8 us, any | 0.523 | 0.277 | 0.210 | 1.32 |
| ClusteredRocket `value`, 1 signal | 0.756 | 0.527 | 0.218 | 2.41 |
| DualRocket `value`, 100 signals | 3.639 | 3.648 | 0.150 | 24.25 |
| PicoRV32 `value`, 1 signal | 0.101 | 0.102 | 0.0129 | 7.91 |
| PicoRV32 `value`, 100 signals | 1.962 | 1.980 | 0.0539 | 36.73 |
| PicoRV32 `value`, 1000 signals | 6.964 | 7.052 | 0.159 | 44.22 |
| PicoRV32 `change`, 95 paths, 8 us, any | 1.962 | 1.975 | 0.0664 | 29.73 |

The complete direct capture has seven additional ratios above 2 that were not
visible in the original wrapped matrix: six short SCR1 single-clock windows and
DualRocket's 100-signal 32-us signal trigger. Direct before/after controls show
SHM times improve by 4–8% and 12%, respectively. The changed ratios do not
indicate an SHM slowdown; the measurement boundary and FSDB latency also affect
them. The six SCR1 native `open + traces + drop` medians are
44–46 ms against complete FSDB CLI budgets of 29–31 ms. Entering values and all
changes match FSDB for each input.

DualRocket's 10-signal 32-us signal-trigger case is borderline: the full capture
has SHM 593.933 ms / FSDB 286.629 ms = 2.072x; a separate 20-run, five-warmup
confirmation gives 586.864 / 295.760 ms = 1.984x. The full-matrix count retains
its original observation; this pair is not a robust failure of the 2x threshold.

Twelve further direct controls cover global extraction, large FST/FSDB point
batches, and the seven additional pairs. Global-extraction SHM improves from
176.403 to 128.941 ms, FST from 31.206 to 26.352 ms, and FSDB remains near 99 ms.
PicoRV32's 1000-path FST/FSDB controls remain within 1% of their baselines.

## Measured API costs

Initial independent diagnostic medians, in seconds:

| Operation | SHM query | FSDB query | SHM open + resolve |
| --- | ---: | ---: | ---: |
| PicoRV32 `samples`, 1 signal | 0.100 | 0.004 | 0.001 |
| PicoRV32 `samples`, 100 signals | 1.989 | 0.034 | 0.001 |
| PicoRV32 `samples`, 1000 signals | 7.153 | 0.111 | 0.001 |
| DualRocket `samples`, 100 signals | 3.464 | 0.062 | 0.070 |
| PicoRV32 `traces`, 95 paths, 8 us | 1.919 | 0.046 | 0.001 |

Both trace queries return 16,318 changes. In PicoRV32's 1000-path query Ondas
deduplicates aliases to 419 native histories. The adapter is not issuing 1000
native calls. One SHM API call in these examples exceeds twice the complete
original FSDB CLI time, even if all wavepeek overhead were removed.

The query timer and initial `total` field precede destruction of the waveform.
Neither that total nor `change.run.done` is a complete process lifetime. Treat
process completion, hierarchy destruction, and rendering separately when
interpreting a residual difference.

A later consumer explicitly times waveform destruction: about 19 ms for the
single-point clustered Chipyard example and 10 ms for its narrow two-signal
trace example. On the PicoRV32 window, `traces()`, `scan()`, and `query()` with
`Break` after 50 candidate callbacks have medians 1.887, 1.918, and 1.900 seconds.
Changing the common API traversal form does not remove this prefix cost.

Three mixed cases also have a staged diagnostic on one `Waveform`: read the
trigger, retain its trace, then read the remaining requested paths. Only the
already-covered requested path is removed; physical aliases are not globally
interned. The initial values and all changes match both a single union read and
FSDB: 96,602 changes for ClusteredRocket and 64,101 / 64,801 for DualRocket.

SHM `open + query + drop` medians over three repetitions, in seconds:

| 32-us signal-trigger selection | Union call | Staged calls | 2x final FSDB CLI |
| --- | ---: | ---: | ---: |
| ClusteredRocket, 100 requested paths | 1.040 | 1.150 | 1.063 |
| DualRocket, 100 requested paths | 0.646 | 0.697 | 0.662 |
| DualRocket, 10 requested paths plus trigger | 0.526 | 0.577 | 0.573 |

This measures the current correct call sequence, not a semantic lower bound on
all possible algorithms. The last row's confirmed FSDB budget is 0.592 seconds,
so it remains mixed and near the threshold. The first two show why comparing
only one union call with CLI time understates repeated-session costs. The
remaining 48 over-budget cases already have single-query diagnostic evidence:
42 from the original slow set and the six additional SCR1 windows.

## Rejected hypotheses and changes

`value` already passes all requested signals to one `Waveform::samples()` call
per requested timestamp. All 16 original `value` cases request one timestamp;
retaining a selection across multiple timestamps cannot improve these cases.

`change` caches owned traces for its command window. Its preload filters covered
histories and deduplicates missing handles. The slow 95-path wildcard case
loads the candidate/payload union once. Property and extraction paths also
retain covered traces. No repeated read of an already-covered history was found
in the measured slow paths.

Adding SHM to existing VCD/FST direct-lookup branches would introduce another
format-specific optimization. It also needs careful handling of escaped names,
canonical collisions, and split vectors. It cannot eliminate the measured
multi-second native calls and is not retained.

Upfront loading of every candidate, condition, and payload can reduce separate
native sessions. It can also read payloads that a trigger never requests and
change error behavior and output-limit costs. A reusable library execution path
should address that tradeoff; an unconditional eager-load change is not a
validated substitute.

The common prototype builds a temporary index for selected paths, then caches
only unique bit declarations backed by a physical signal. Rich values, events,
split vectors, and ambiguous paths retain full resolution. Conservative leaf
filtering skips only declarations whose ordinary spelling is unchanged by
canonicalization. This removes separate direct-lookup implementations while
preserving one canonical builder. It does not remove the native bottlenecks.

Both bulk resolvers prepare this cache once for their complete input. Later
preparation excludes cached paths. This avoids per-path hierarchy scans for
protocol extraction callers and keeps batches above the 128-path single-lookup
limit lazy. The existing 1024-path batch limit is independent; larger batches
use the full index. No backend query strategy is added.

Its first version overprocessed FSDB declarations whose reader spelling adds a
packed range. The final common filter recognizes the exact range suffix and
avoids that work. Five-run controls retain SHM improvements and keep FST/FSDB
within the repository's 5% or 5-ms timing tolerance, with a 20-run confirmation
for the initially noisy FSDB case: revised median 205.896 ms, baseline 209.052 ms;
best samples 200.364 and 200.502 ms.

## Future Ondas tasks

### Support efficient points on converted SHM bit declarations

An ignored diagnostic copy of the published library instruments
`native/shm.cpp::ondas_shm_sample_bits`. PicoRV32 selections of 1, 100, and 1000
paths and DualRocket's 100-path selection all report `certified=0`, with
qualification under 2 microseconds and no `point_prefix` call. Rejection occurs
in `point_type`; the subsequent ordinary fallback accounts for the latency.
The earlier hypothesis that equality-expression certification caused these
particular slow points is disproved.

Investigate the actual Cadence types emitted by VCD conversion and extend safe
qualification or another correct native point path inside Ondas. Preserve
missing states, unavailable intervals, nine-state values, same-tick final
states, alias/projection behavior, and `changed_at`. One unsuitable base must
not unnecessarily force otherwise eligible histories through a large cold
prefix. Keep this decision inside the common API implementation.

Reproduce with `value_picorv32_signals_1`, `_100`, `_1000`, and
`value_chipyard_dualrocketconfig_dhrystone_signals_100` in the SHM catalog.
The single-clock case alone isolates the issue without hierarchy-index cost.

### Avoid full cold prefixes for late bounded windows

The ordinary native SHM reader collects selected histories from the beginning
through the requested inclusive end, then merges them. A late short window
therefore pays for its old prefix. `Break` stops visitor delivery after native
collection; it does not stop that earlier work.

Provide an efficient cold-window path with a correct entering state and the
existing validation/error contract. Do not simply seek past unavailable
intervals or malformed prefix records. The public point, trace, scan, and query
methods should choose the appropriate implementation without client format
checks. Reproduce with PicoRV32's 100-signal wildcard windows of 2, 8, and 32 us;
the 8-us case costs about 1.919 seconds in a single `traces()` call.

### Reduce repeated session and hierarchy import costs

Each native operation opens a session and reimports the full hierarchy before
reading selected histories. Candidate-then-payload queries can repeat that
fixed work even when the client correctly caches overlapping histories.
Profile these phases on narrow Chipyard cases and make common selection/query
reuse useful without retaining unsafe vendor handles or requiring client
format strategies. The staged measurements above quantify the additional-call
cost, including waveform destruction. Internal phase profiling is still needed
to distinguish hierarchy import, history setup, and prefix work. Keep the
borderline DualRocket case separate from demonstrated single-call limits.

## Review and validation

A `gpt-6.1-sol` xhigh subagent independently reviewed hypotheses, native
measurements, cache usage, mixed cases, and candidate changes. It found no small
common correction that removes the demonstrated bottlenecks and requested
explicit treatment of mixed costs and destruction.

OCR reviewed the integration adapter. Its claim that full hierarchy indexing
dominates the 6.85-second PicoRV32 case is contradicted by the standalone query
timers. Its proposed format-specific direct path is excluded. A separate
escaped-simple-name concern is unconfirmed and requires a fixture reproduction;
the low-severity message-constant suggestion is unrelated to this task.

OCR also reviewed the common prototype. Scope-enumeration, obsolete-helper, and
test-diagnostic observations were addressed. The core
file reviews completed, with one reported planning request failure. The
independent subagent checked the range-suffix guard and selected/full reference
tests after subsequent edits.

A final OCR pass found repeated preparation in unseeded bulk resolver calls;
the common preparation and cached-path filtering address it. Its nonempty
reference-index assertion was added. Reordering the leaf guard does not remove
the required spelling check for skipped declarations; an additional packed-range
parser has no demonstrated material benefit. The different single-lookup and
batch limits are intentional. The subagent reviewed the resulting source and
the 200-signal order, width, sample, and cache tests without correctness findings.

The independent reviewer recomputed the final aggregates from all 144 raw
Hyperfine artifacts and verified five timings and successful exits per format.
It accepted the distinction between single-query costs, staged workflows, and
the separate borderline confirmation.

`just ci` and `just check` pass. The combined `fsdb,shm` regression run passes
776 tests: 693 library, 47 CLI
contract, 24 FSDB CLI, and 12 SHM CLI tests. Source coverage is 93.60% lines,
92.67% regions, and 92.13% functions. Complete functional recapture preserves
all 156 successful SHM and FST results and FSDB's 144 successes plus its 12
existing failures, including exit status and fatal JSON. Final timing controls
and native trace parity checks pass. No further production change is justified
by these measurements: common lookup and bulk preparation address the material
adapter costs; larger query-strategy changes require library work or unvalidated
eager-loading tradeoffs.

## Evidence and reproduction

Run the repository's original focused benchmark through the host entrypoint:

    ./dev just bench-e2e-shm-run 'value_picorv32|change_picorv32_signals_100'

All task-owned local evidence is under `tmp/shm-opt/`: `probe-stages.json` for
initial paired diagnostics, `slow-native.json` and `native-summary.json` for
all 55 SHM diagnostic cases, `diagnostic-native.log` for point qualification,
and `direct/` plus `direct-summary.json` for direct paired CLI captures.
The standalone consumer lives in `probe/`; `run_direct.py` reproduces the direct
control capture. The instrumented library copy lives only in ignored scratch;
the Ondas checkout, Cargo dependency, and production builds are unchanged.

`binaries-v3.json` records the final binary and source-diff hashes.
`functional-v3-summary.json`, `direct-final-v3-summary.json`, and
`final-v3-summary.json` contain the complete final captures and aggregates.
`run_final_v3.py` resumes completed scenarios without overwriting them.
`controls-v3-summary.json`, `extra-native-summary.json`, the three
`staged*-summary.json` files, and `border-confirm-summary.json` contain the
additional controls. Reproduce them through `./dev bash
tools/shm/with_shm_env.sh python3 -B tmp/shm-opt/<script>.py` using
`controls_v3.py`, `extra_probe.py`, `staged_measure.py` (optional `-dual10` or
`-dual100` argument), and `confirm_border.py`. Use fresh output directories for
repeated captures. The scripts and SDK-dependent inputs remain local scratch.

For a fresh standalone reproduction, create an ignored Cargo application under
`tmp/` with `ondas = { version = "=1.1.0", features = ["shm-lib", "fsdb-lib"] }`
and a separate empty `[workspace]` table. This minimal program isolates the
single-clock point on either converted input without wavepeek:

```rust
use std::time::Instant;
use ondas::Time;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("waveform path");
    let start = Instant::now();
    let mut wave = ondas::open(path)?;
    let opened = start.elapsed();
    let signal = wave.hierarchy().signal("testbench.clk")?;
    let query = Instant::now();
    let samples = wave.samples(&[signal], Time::from_ticks(2_455_330_000))?;
    eprintln!("open={opened:?} query={:?} {samples:?}", query.elapsed());
    Ok(())
}
```

Build through `./dev bash tools/shm/with_shm_env.sh cargo build --release
--manifest-path tmp/<application>/Cargo.toml`. Launch through the same wrapper
with the database at
`/opt/ondas-fixtures/fst/fst0013-picorv32-test-vcd/waveform.shm`, then its
`waveform.fsdb` neighbor. Both inputs use 1-ps ticks. Conversion and compilation
must complete before timing.
