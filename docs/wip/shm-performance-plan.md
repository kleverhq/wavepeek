# Reduce SHM query latency

## Purpose / Big Picture

Reduce SHM query times to at most twice FSDB for each common correct E2E scenario,
while preserving every SHM result. If Ondas prevents that target, exhaust
reasonable changes in wavepeek and record reproducible library tasks. Keep the
implementation small; do not add a reader, persistent service, or dependency fork.
The completed integration is described in `shm-integration-plan.md` and its
baseline results in `shm-benchmark-report.md` in this directory.

## Progress

- [x] (2026-10-16) Preserve baseline binaries, functional captures, and five-run
  timings under `tmp/shm/comparison/`; create the optimization goal.
- [x] (2026-10-16) Separate hierarchy and query costs in five representative
  scenarios; start native measurements for all 55 baseline pairs above 2x.
- [x] (2026-10-16) Review hypotheses and common lookup changes with a
  `gpt-6.1-sol` xhigh subagent and OCR; address scope/filter/test observations.
- [x] (2026-10-16) Retain common selected-declaration resolution after focused
  SHM gains up to 1.86x, cross-format controls, regression confirmation, and
  selected/full canonical-path tests.
- [x] (2026-10-16) Measure all 55 initially slow SHM cases through Ondas;
  verify all 13 point selections and the 95-path trace against FSDB. Compare
  traces, scans, and early-stopped queries; record point-type rejection and
  cold-prefix costs in `shm-performance-investigation.md`.
- [x] (2026-10-16) Prepare selected declarations inside both bulk resolvers,
  skip cached paths, and verify 200-signal resolution without a full index.
  The independent reviewer accepts the source diff; 776 combined-feature tests pass.
- [x] (2026-10-16) Pass fresh `just ci`, `just check`, and 776 combined-feature
  tests. Recapture all 156 scenarios per format: SHM/FST preserve all successes;
  FSDB preserves 144 successes and the same 12 failures.
- [x] (2026-10-16) Finish all 144 successful-pair timings, 12 direct controls,
  additional native diagnostics, three staged-query comparisons, and a 20-run
  boundary confirmation. Independent review verifies the raw timings and counts.
- [x] (2026-10-16) Record the unmet target, retained common optimizations,
  measured Ondas constraints, and three future library tasks for handoff.

## Context and Orientation

Work in `/home/esynr3z/projects/wavepeek-shm`, branch `feat/shm`. Host Git and
root `./dev` are the required execution boundary: Cargo, waveform tools, and
quality checks run inside the worktree's private devcontainer. Its ignored
profile already selects Xcelium 25.03.001 and Verdi X-2025.06. Never prepare
fixtures concurrently or time queries during builds and conversions.

`src/waveform/ondas_backend.rs` adapts Ondas 1.1.0 into engine signal handles,
canonical names, sampling, and cached traces. Selected bit declarations use one
canonical builder and lazy cache across all formats. Range commands
load traces for their sampling window and reuse those owned results. Ondas is
pinned in `Cargo.toml`; its clean source is available for read-only inspection at
`/home/esynr3z/projects/ondas/ondas-lib`. The published Cargo dependency is the
code used by builds.

All disposable experiments belong in `tmp/shm-opt/`. Preserve the previous
`tmp/shm/comparison/bin/wavepeek-shm` and `wavepeek-fsdb` as baseline binaries.
`tmp/shm/comparison/tests_shm.json` and `tests_fsdb.json` contain 156 equivalent
scenarios with five measurements and two warmups. FSDB has 12 existing scalar
array-path failures. SHM has ten known payload differences against FSDB, fully
described in the baseline report. Optimization must match the frozen SHM output
in all 156 scenarios, including those format-specific observations.

## Milestones

### Establish costs and review hypotheses

Rank individual scenarios from `tmp/shm/comparison/timing-summary.json`. Inspect
the adapter and native Ondas implementation. Use a temporary consumer under
`tmp/shm-opt/` to time opening, resolution, and equivalent native queries without
the wavepeek engine. Time the same signals and bounds; distinguish experiments
from validated optimizations. Have the requested subagent review opportunities
and correctness risks. Provide OCR with the hypothesis and resulting diff as
business context, and assess its findings before accepting changes.

### Iterate on the adapter

Implement only format-independent improvements supported by measurements. The
user requires Ondas to own differences in backend query strategies. Reject new
format tests, batch partitioning, or native reader workarounds in wavepeek.
Candidate areas are redundant work and reusable calls through the common API. Preserve
ambiguity errors, split-vector joining, escaped/indexed names, aliases, event
occurrences, and real/string expressions. Add focused regression tests where a
new path changes resolution or cache behavior. Build with `./dev just
build-release-shm`, then measure representative slow and fast cases against the
frozen SHM binary using the same catalog and round-robin scheduling. Retain only
changes with useful improvement and exact functional parity.

### Validate the target or establish the library boundary

Run all SHM cases and compare their captures with the frozen SHM baseline. Refresh
FSDB measurements and evaluate each supported, identical-payload pair against
the 2x bound; also report the other supported pairs without hiding differences.
If the target remains unmet, use the minimal consumer to demonstrate the native
cost of the remaining operations. Inspect every plausible adapter alternative
and explain rejected approaches. Record focused future Ondas tasks with current
behavior, measured impact, expected behavior, and reproduction commands. Do not
create issues or change Ondas itself.

Run `./dev just ci` and `./dev just check`, including the configured native
features. Commit retained implementation and evidence through normal hooks.
Report improvements, remaining ratios, validation, and the measured boundary.

## Concrete Steps

From the worktree root, build and capture a focused revised run:

    ./dev just build-release-shm
    ./dev bash tools/shm/with_shm_env.sh python3 -B bench/e2e/perf.py run --binary baseline=./tmp/shm/comparison/bin/wavepeek-shm --binary revised=./target/shm/release/wavepeek --tests tmp/shm/comparison/tests_shm.json --run-dir tmp/shm-opt/iteration-1 --schedule round-robin --filter 'value_picorv32|change_picorv32_signals_100' --wavepeek-timeout-seconds 120

Each label directory contains functional `.wavepeek.json` captures and timing
`.hyperfine.json` files. Check the run manifest and failure artifacts: process
exit success alone does not establish that every scenario succeeded. Compare
payloads with `bench/e2e/perf.py compare --golden <baseline-label-directory>
--revised <revised-label-directory> --functional-only --result-json <scratch-path>`.
For final captures use unique run directories and the complete catalog. Reuse
valid generated waveforms; conversion is excluded from all timing.

## Validation and Acceptance

All retained changes must pass tests and preserve the complete SHM baseline
payloads. Timing acceptance is a median SHM/FSDB ratio no greater than 2 for every
common identical-payload scenario, not merely an aggregate median. Otherwise
the handoff must identify measured Ondas bottlenecks and explain why remaining
wavepeek alternatives cannot safely remove them. SDKs, licenses, databases,
profiling binaries, and raw captures stay ignored.

## Surprises & Discoveries

The initial aggregate ratio is already below 2, but individual cases reach
36.389. Therefore aggregate timing cannot satisfy this task. Direct process
measurements avoid the timed wrapper's polling delay and require their own
before/after controls. Native diagnostics locate point rejection in `point_type`
and the subsequent cost in ordinary history-prefix reads. The trace, scan, and
early-stopped query APIs have similar cold-window costs.

Complete direct timings also expose short scenarios whose SHM/FSDB ratio was
masked by the timed wrapper's fixed overhead. Additional native diagnostics and
direct before/after controls distinguish this effect from adapter regressions.
One near-threshold DualRocket pair measures 2.072x in the full capture and
1.984x in a separate 20-run confirmation; preserve both observations.

## Decision Log

Use the frozen SHM binary as the exact functional reference and preserve all
known format differences. Evaluate the target per scenario. Date: 2026-10-16.

Use the requested `gpt-6.1-sol` xhigh subagent for independent hypotheses/results
review and the `open-code-review` skill for OCR. Review is advisory; verify each
finding against code and measurements. Date: 2026-10-16.

Do not add backend-specific optimization strategies in wavepeek. Its common
Ondas calls must remain sufficient; record backend query limitations for Ondas
instead. This follows the user's explicit architecture constraint. Date:
2026-10-16.

Retain the common selected-declaration approach after focused SHM improvements
up to 1.86x and cross-format controls. The first FSDB regression came from
unnecessarily canonicalizing packed reader spellings; recognize exact packed
suffixes in the common filter. A 20-run reverse-order confirmation resolves the
remaining opening-time noise. Date: 2026-10-16.

Prepare the selected cache inside both bulk resolver methods and exclude already
cached paths from later preparation. This addresses OCR's repeated-scan finding
without caller assumptions or backend conditions. Preserve engine preparation
for projections resolved one token at a time. Date: 2026-10-16.

Stop production optimization at the measured Ondas boundary. Do not replace
the conditional candidate/payload workflow with unconditional eager loading or
a new client traversal to improve isolated mixed cases. The single-call costs,
staged-query diagnostics, and independent review support library work as the
next step. Record hypotheses as future tasks without creating issues. Date:
2026-10-16.

## Outcomes & Retrospective

The target remains unmet: the full 144-pair direct capture has 51 ratios above
2x, all in the 134 identical-payload pairs, with a maximum of 44.218x. One of
these pairs is borderline and measures 1.984x in confirmation. Eleven of the
initial 55 slow pairs now meet 2x in the full capture. Their median SHM speedup
is 1.147x and their largest is 1.886x. Common canonical selection and bulk
preparation address material wavepeek costs without backend query strategies.

Forty-eight remaining cases have measured single-query native costs above the
FSDB budget. Two more show over-budget native costs for the current staged
workflow; the remaining boundary case stays explicitly mixed. These are API
measurements, not lower bounds on every possible algorithm. The investigation
records three Ondas tasks: efficient converted-bit points, cold bounded windows,
and repeated-session costs. No Ondas checkout or dependency was changed.

Fresh `just ci`, `just check`, and 776 combined-feature tests pass. All functional
captures match the baseline. OCR and the requested subagent reviewed the
hypotheses and implementation; independent review also reproduces the final
raw timing aggregates. The implementation and library-task notes are ready for
handoff.

Revision 2026-10-16: Created the optimization plan after completion of native
integration; the new acceptance criterion applies to individual common cases.

Revision 2026-10-16: Limited accepted changes to common API usage after the user
clarified that backend-specific strategies belong inside Ondas.

Revision 2026-10-16: Added a common selected-declaration prototype after direct
controls exposed 13 mixed cases with material wavepeek index costs. It replaces
format-specific lookup strategies rather than extending them to SHM.

Revision 2026-10-16: Added bulk preparation after OCR review and expanded final
direct timings to all 144 successful FSDB pairs, including formerly fast cases.

Revision 2026-10-16: Completed final validation and direct controls, retained
the borderline confirmation separately, and documented the measured library
boundary and future tasks.
