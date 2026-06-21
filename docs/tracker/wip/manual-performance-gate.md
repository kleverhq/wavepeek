# Manual Performance Gate With Current Tooling

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, maintainers can compare two release binaries with one current benchmark apparatus instead of comparing two historical checkouts that each bring their own benchmark scripts, catalogs, fixtures, and Criterion microbenchmarks. A maintainer runs `just bench-gate <baseline-ref> <revised-ref>`, the helper builds `wavepeek` binaries from those refs, then measures both binaries through the benchmark tools and catalogs from the current working tree. This makes the gate answer the release question directly: whether user-visible CLI behavior regressed when the same current end-to-end tests run against two selected binaries.

The gate will only hard-fail on end-to-end CLI timing and functional parity. Same-format timing means FST baseline binary versus FST revised binary and, when FSDB is available, FSDB baseline binary versus FSDB revised binary. Cross-format FST-versus-FSDB checks remain functional-only within each binary ref because FST and FSDB use different readers and their wall-clock timings are not comparable. Timing uses median wall-clock time as the primary metric, allows a slowdown of `max(5%, 5ms)` so short commands are not failed by fixed millisecond-scale host jitter, and can run best-sample confirmation for same-format suites that fail only on median timing. Criterion expression microbenchmarks are removed from the repository because they are internal microbenchmarks, not release-gate evidence for user-visible CLI performance.

## Non-Goals

This work will not add a GitHub Actions performance workflow. This work will not store benchmark baseline runs in git. This work will not compare FST timing against FSDB timing. This work will not solve noisy-machine benchmarking with self-hosted runners or statistical modeling. This work will not keep Criterion as an advisory release-gate artifact; the expression microbenchmark harness is removed rather than demoted. The maintainer has now explicitly requested one full release-like `just bench-gate v1.0.0 v1.0.1` validation after the best-sample timing confirmation change is committed.

## Progress

- [x] (2026-06-18 00:00Z) Rejected the original CI benchmark workflow direction in GitHub issue 24 and recorded the manual gate direction in an issue comment.
- [x] (2026-06-18 03:30Z) Implemented and committed the first manual-gate version as `f3fd31c chore(bench): add manual performance gate`; it removed committed benchmark baselines and added public `just bench-gate`, `just bench-capture`, and `just bench-compare` entrypoints.
- [x] (2026-06-19 23:45Z) Split the overloaded helper into `tools/bench/common.py`, `tools/bench/capture.py`, `tools/bench/compare.py`, and a smaller `tools/bench/gate.py`; updated `justfile` to call the split scripts.
- [x] (2026-06-20 00:40Z) Committed the split-helper refactor locally as `54b28d8 refactor(bench): split manual gate helpers`.
- [x] (2026-06-20 12:22Z) Ran three full `just bench-gate v1.0.0 v1.0.1` checks in a quieter environment. All captures completed and all compares failed, proving orchestration worked but the policy produced false positives for a null runtime change.
- [x] (2026-06-20 18:00Z) Analyzed average, best-delta, best-median, warmup/runs, duration/noise, and version-wide slowdown hypotheses under `tmp/bench-gate/`. The data showed no broad `v1.0.1` runtime slowdown, exposed fixed millisecond-scale jitter in short tests, and showed Criterion can fail despite no runtime code changes.
- [x] (2026-06-20 19:05Z) Agreed on the new direction: remove Criterion completely, use current benchmark tooling against binaries built from selected refs, and gate same-format end-to-end timing on median with `max(5%, 5ms)` allowed slowdown.
- [x] (2026-06-20 19:15Z) Updated this ExecPlan and committed it as `46ee04a docs(tracker): plan current-tooling performance gate`.
- [x] (2026-06-20 19:30Z) Removed Criterion and expression microbenchmark files, removed expression benchmark capture/compare steps, and ran focused syntax, unit, and `cargo check` validation.
- [x] (2026-06-20 19:45Z) Dedicated cleanup review found stale environment-note help, pre-commit metadata, and root breadcrumb references; all were corrected.
- [x] (2026-06-20 19:55Z) Re-ran focused validation after cleanup fixes: syntax checks, tools/bench tests, bench/e2e tests, `cargo check`, `just test-aux`, and stale-reference search all passed.
- [x] (2026-06-20 20:00Z) Committed Criterion removal locally as `fcc849b chore(bench): remove expression microbenchmarks`.
- [x] (2026-06-20 20:25Z) Refactored manual gate capture to build selected refs as binaries but run current E2E tooling, and changed E2E compare to median-only timing with a `max(5%, 5ms)` slowdown allowance.
- [x] (2026-06-20 20:40Z) Ran focused validation for the harness/policy refactor: syntax checks, tools/bench tests, bench/e2e tests, `cargo check`, `just test-aux`, `just format-justfile-check`, and `just check-actions` passed.
- [x] (2026-06-20 21:10Z) Implementation review found a tooling-root mismatch in standalone compare and a misleading both-unsupported FSDB reason; both were fixed, and focused tests plus `just test-aux` passed again.
- [x] (2026-06-20 21:35Z) Control review found dirty-tooling provenance missing from standalone compare and a summary that omitted the 5ms floor; both were fixed, focused tests passed, and `just check` passed.
- [x] (2026-06-20 21:45Z) Final control re-check returned no substantive findings.
- [x] (2026-06-20 21:50Z) Committed the harness/policy refactor locally as `refactor(bench): use current tooling for performance gate`.
- [x] (2026-06-21 05:20Z) Re-ran `just bench-gate v1.0.0 v1.0.1`; timing failures dropped to 6 FST and 1 FSDB outlier in a quiet environment, but block-level revised timing clusters still caused false failures. Decided to move multi-binary scheduling into the E2E runner so tests can be interleaved by binary.
- [x] (2026-06-21 06:05Z) Implemented explicit labeled-binary E2E runner output and switched the gate to one round-robin runner invocation per suite. Focused syntax, E2E, tools/bench, auxiliary, justfile-format, and actionlint checks passed.
- [x] (2026-06-21 06:20Z) Review found reserved-label, movable-suite-path, report-root, gate-layout docs, and execution-order issues. Fixed them, added tests, and reran focused checks plus `just check`; all passed.
- [x] (2026-06-21 06:35Z) Final control review returned no substantive findings.
- [x] (2026-06-21 10:40Z) Committed the labeled-runner change as `52e6858 refactor(bench): interleave labeled e2e binaries` and ran one full `just bench-gate v1.0.0 v1.0.1` validation. Capture passed; compare failed with 5 FST timing failures, 6 FSDB timing failures, and the known cross-format functional mismatches.
- [x] (2026-06-21 11:05Z) Modeled best-sample and worst-sample confirmation on the failed timing tests. Best-sample confirmation cleared all FST timing failures and 4 of 6 FSDB timing failures; worst-sample cleared all timing failures. The maintainer selected best-sample confirmation plus larger sample sizes rather than worst-sample confirmation.
- [x] (2026-06-21 11:45Z) Added `bench/e2e/perf.py compare --result-json`, `bench/e2e/perf.py confirm`, and `tools/bench/compare.py` orchestration that runs best-sample confirmation only for same-format median timing failures with no functional hard failures.
- [x] (2026-06-21 11:45Z) Raised every committed E2E catalog entry in `tests.json`, `tests_fsdb.json`, and `tests_commit.json` to at least 10 measured runs and 5 warmups, and regenerated the FSDB catalog.
- [x] (2026-06-21 11:50Z) Focused Python tests, `just test-aux`, `just format-justfile-check`, `just check-actions`, and `just check` passed.
- [x] (2026-06-21 12:00Z) Review found that functional timeout warnings could be incorrectly overridden by best-sample timing confirmation; fixed the blocker check and added a regression test. Docs review found stale median-only and requested-gate wording in this plan; corrected the plan.
- [x] (2026-06-21 12:15Z) Final control review found only unclear documentation about timeout warnings blocking confirmation; clarified maintainer docs, helper README, and this plan.
- [ ] Commit and run one full gate validation from a clean worktree, then record the result.

## Surprises & Discoveries

- Observation: The repository originally tracked 591 files under benchmark run directories, including FST, FSDB, and expression baseline artifacts.
  Evidence: `git ls-files 'bench/e2e/runs/*' 'bench/expr/runs/*' | wc -l` printed `591` before baseline removal.
- Observation: The generated FSDB catalog can contain VCD-style scalar element signal paths that converted RTL FSDB fixtures do not expose under the same canonical names.
  Evidence: A `v1.0.0` FSDB command for `change_scr1_signals_100_pos_50_window_2ns_trigger_any` failed with `fatal: signal: signal 'TOP.scr1_top_tb_axi.i_memory_tb.araddr.[0]' not found in dump`, while `wavepeek signal` listed `TOP.scr1_top_tb_axi.i_memory_tb.araddr[0] [31:0]`. Gate-local runnable catalog filtering skipped 12 of 142 FSDB tests and left 130 runnable FSDB tests.
- Observation: Three full gate runs of `v1.0.0` versus `v1.0.1` are a useful null experiment because `git diff v1.0.0..v1.0.1` changes release metadata, CI/docs/tests, and the package version but no runtime source or dependencies.
  Evidence: The three full runs captured successfully but failed compare. Pooled timing deltas over all suites were approximately centered around zero, and there was no broad `v1.0.1` slowdown.
- Observation: Short end-to-end tests suffer from fixed millisecond-scale host jitter.
  Evidence: The duration/noise analysis under `tmp/bench-gate/duration-vs-noise-20260620/` found that tests under 75ms had about 1.5ms to 2ms typical absolute run-to-run range, enough to exceed a strict 5% relative threshold.
- Observation: Criterion expression microbenchmarks do not belong in the release gate.
  Evidence: Criterion failed in one of three null-experiment runs despite no runtime code changes. Its measurements target internal parser/binder/evaluator operations rather than user-visible CLI latency under real waveform workloads.
- Observation: Round-robin scheduling removed broad suite-level bias but not per-test bimodal timing.
  Evidence: The full gate under `tmp/bench-gate/gates/20260621T104432Z-a93aad1db823..81777d4e939e` had suite median deltas around zero, while individual failed tests alternated between fast and slow modes about 50ms apart. On that data, best-sample confirmation passed all 5 FST median failures and 4 of 6 FSDB median failures.

## Decision Log

- Decision: Replace committed benchmark baselines with explicit baseline source revisions selected at gate time.
  Rationale: A baseline ref is small, reviewable, and naturally tied to releases. It avoids storing generated JSON/CSV snapshots while still allowing maintainers to compare two revisions in one environment.
  Date/Author: 2026-06-18 / Grin
- Decision: Do not add a GitHub Actions benchmark workflow for this issue.
  Rationale: GitHub-hosted runners are noisy, and a heavy manual benchmark workflow would add friction for forks and contributors without producing authoritative performance decisions.
  Date/Author: 2026-06-18 / Grin
- Decision: Remove Criterion expression microbenchmarks entirely.
  Rationale: The release gate should measure user-visible CLI behavior. Internal microbenchmarks can detect function-level changes that users do not observe and add noise, maintenance, dependencies, and an incompatible source-ref-based benchmark model. If expression performance matters for release decisions, it should be represented by current end-to-end expression-heavy CLI scenarios.
  Date/Author: 2026-06-20 / Grin
- Decision: End-to-end benchmark tooling, catalogs, fixtures, FSDB preparation scripts, and compare scripts come from the current working tree; selected refs provide only the subject binaries.
  Rationale: The benchmark apparatus should be one ruler. Historical tags cannot be updated when the benchmark harness is fixed. Measuring old and new binaries with current tooling applies catalog and normalization fixes consistently and avoids comparing different benchmark scripts against each other.
  Date/Author: 2026-06-20 / Grin
- Decision: The current tooling worktree must be clean before `bench-gate` or `bench-capture` runs.
  Rationale: If current scripts and catalogs define the measurement apparatus, uncommitted local edits would make the gate irreproducible. Requiring a clean worktree records the tooling SHA as durable provenance.
  Date/Author: 2026-06-20 / Grin
- Decision: Same-format end-to-end timing gates on median as the primary metric, with an allowed slowdown of `max(golden_median * 5%, 0.005 seconds)` before any timing-only confirmation is considered.
  Rationale: Mean wall-clock time is sensitive to rare scheduler or I/O stalls. Median is more stable for hyperfine timing. A 5ms absolute floor prevents short commands from failing on fixed host jitter while preserving a 5% relative threshold for longer commands. Later best-sample confirmation does not replace median as the primary evidence; it only rechecks median-only failures for the same low-latency envelope.
  Date/Author: 2026-06-20 / Grin
- Decision: `bench/e2e/perf.py run` requires one or more explicit `--binary label=path` arguments, writes one labeled artifact directory per binary, and no longer reads `WAVEPEEK_BIN` or writes flat run directories.
  Rationale: The runner owns test order and output layout. Explicit labels make provenance visible and let the gate run the same test on all compared binaries before moving to the next test, reducing block-level noise while still using hyperfine for timing.
  Date/Author: 2026-06-21 / Grin
- Decision: The gate uses the E2E runner's default round-robin schedule for FST and FSDB captures.
  Rationale: Prior captures measured all baseline tests before all revised tests. A quiet rerun still produced a cluster of revised-only slowdowns around 50ms. Test-level interleaving reduces exposure to block-wide thermal, frequency, scheduler, or I/O shifts without adding a custom timing runner.
  Date/Author: 2026-06-21 / Grin
- Decision: Cross-format FST-versus-FSDB checks remain functional-only hard gates.
  Rationale: FST and FSDB timing is not comparable, but payload parity matters. Current tooling should own any needed catalog filtering or functional normalization.
  Date/Author: 2026-06-20 / Grin
- Decision: Add a separate best-sample confirmation step after same-format median timing failures.
  Rationale: Median remains the primary gate metric, but the current host shows discrete high-latency samples that can land unevenly between binaries. Confirming only failed tests with the best observed sample from each binary checks whether both binaries can still reach the same low-latency envelope. This is less permissive than worst-sample confirmation and matches the maintainer's request.
  Date/Author: 2026-06-21 / Grin
- Decision: Every committed E2E catalog entry should use at least 10 measured hyperfine runs and at least 5 warmup runs.
  Rationale: The previous 5 measured runs and 3 warmups left too little chance for both binaries to show their clean low-latency mode on noisy hosts. Larger samples make median and best-sample confirmation less dependent on a single unlucky scheduler or I/O event.
  Date/Author: 2026-06-21 / Grin

## Outcomes & Retrospective

Criterion removal, the current-tooling gate model, and the explicit labeled-binary round-robin runner are committed locally. The latest full gate validation proved round-robin reduced broad bias but still left bimodal per-test timing outliers and known cross-format functional mismatches. The active change adds best-sample confirmation for same-format median timing failures and raises committed E2E catalog sample counts before a new clean full gate run.

## Context and Orientation

`wavepeek` is a Rust command-line tool for deterministic waveform inspection. The end-to-end benchmark harness is `bench/e2e/perf.py`; it reads benchmark catalogs such as `bench/e2e/tests.json`, runs one or more explicit labeled `wavepeek` binaries through `hyperfine`, captures functional `wavepeek --json` payloads, writes one `*.hyperfine.json` and one `*.wavepeek.json` per test under each binary label directory, and can compare any two label directories. Hyperfine is an external command-line timing tool. A runner root contains a `manifest.json`, a `README.md` index, and one artifact directory per binary label.

FST and FSDB are waveform file formats. FST is open and handled through the normal release binary. FSDB is proprietary and requires the optional Synopsys Verdi FSDB Reader SDK; an FSDB-enabled binary is built with `cargo build --release --features fsdb` and stored under `target/fsdb/release/wavepeek`. FSDB capture is automatic in `auto` mode when Verdi and both binary refs support FSDB, skipped when Verdi is unavailable, and failed when support is asymmetric while Verdi is available.

The root `justfile` is the stable local automation interface. Public benchmark recipes are `bench-gate`, `bench-capture`, and `bench-compare`. Generated benchmark artifacts live under ignored directories such as `tmp/bench-gate/` and `bench/e2e/runs/`.

The helper group under `tools/bench/` has four scripts. `common.py` owns shared subprocess, git, JSON, timestamp, and output-directory helpers. `capture.py` owns binary capture orchestration and FSDB runnable-catalog generation. `compare.py` owns same-format timing comparisons plus cross-format functional comparisons. `gate.py` owns the two-ref release orchestration.

## Open Questions

There are no blocking open questions. The implementation should keep the public `just` entrypoints stable. `bench-capture <ref>` must use current tooling against the binary built from `<ref>`, matching `bench-gate` semantics. `bench-compare` remains a standalone comparison over existing capture directories using current compare scripts.

## Plan of Work

The current milestone has two implementation commits after this plan commit.

First, remove Criterion completely. Delete `bench/expr/`, remove the Criterion dependency and `[[bench]]` target entries from `Cargo.toml`, update `Cargo.lock`, remove the private `bench-expr-run` recipe, remove expression benchmark runs from `tools/bench/capture.py`, `tools/bench/compare.py`, and `tools/bench/gate.py`, and update docs/tests that mention expression microbenchmarks. Run focused tests that prove no Python unit test suite still references `bench/expr`, and request a review focused only on cleanup completeness.

Second, refactor the gate so selected refs provide only compiled binaries. `tools/bench/gate.py` should still clone both refs under the gate output directory and build both release binaries before measurement. `tools/bench/capture.py` should treat `REPO_ROOT` as the tooling root and the cloned checkout as the binary source root. End-to-end capture must run `python3 -B bench/e2e/perf.py run` with `cwd=REPO_ROOT`, current benchmark catalogs, current fixtures, and explicit `--binary label=path` arguments pointing at the binaries in the cloned checkouts. FSDB preparation and catalog checks should run once through current tools and current catalogs; the generated runnable catalog should be written into each capture directory so manifests preserve the skip list used for that capture. Capture manifests should record both `tooling_sha` and `binary_sha`.

The E2E comparator should first fail timing only when median slowdown exceeds both the relative and absolute allowances, expressed as `revised_median - golden_median > max(golden_median * threshold_pct / 100, threshold_seconds)`. The default threshold remains 5%, and the default absolute floor is 0.005 seconds. Mean remains available in reports and hyperfine artifacts but is not a gate timing metric. If a same-format compare fails only because of median timing failures, with no functional mismatches, missing or invalid artifacts, or timeout warnings, the manual gate should run a separate confirmation step over those failed tests. Confirmation uses the best observed hyperfine sample for each binary, where best means the minimum value in the `times` array in the hyperfine JSON artifact. A confirmed test passes when `revised_best - golden_best <= max(golden_best * threshold_pct / 100, threshold_seconds)`. The compare manifest should record both timing threshold values, identify median as the primary gating metric, and record best-sample confirmation details when confirmation is attempted.

### Concrete Steps

From repository root `/workspaces/wavepeek`, keep these stable user commands:

    just bench-gate <baseline-ref> [revised-ref] [fsdb-mode]
    just bench-capture [ref] [fsdb-mode]
    just bench-compare <golden-capture-dir> <revised-capture-dir>

After the refactor, `bench-gate` and `bench-capture` must refuse to run if the current tooling worktree is dirty, regardless of whether the selected ref is a tag, SHA, or `HEAD`. This is separate from the old `HEAD`-only dirty check and exists because current scripts and catalogs are now part of the measurement.

Remove these source artifacts:

    bench/expr/
    criterion from Cargo.toml dev-dependencies
    expr_* [[bench]] targets from Cargo.toml
    private bench-expr-run just recipe

Update these helper paths:

    tools/bench/capture.py
    tools/bench/compare.py
    tools/bench/gate.py
    tools/bench/test_gate.py
    bench/e2e/perf.py
    bench/e2e/test_perf.py
    justfile

For the best-sample confirmation change, add machine-readable timing failure output to `bench/e2e/perf.py compare` with a `--result-json` option, add a `confirm` subcommand in `bench/e2e/perf.py` that accepts `--golden`, `--revised`, repeated `--test`, and the same threshold flags, then call that subcommand from `tools/bench/compare.py` only after a same-format median compare fails with timing failures and no functional hard failures. The confirmation log should be a separate file such as `compare/e2e-fst.best-confirm.log`.

Update these docs as needed:

    docs/dev/benchmarking.md
    docs/dev/quality.md
    docs/dev/automation.md
    docs/dev/fsdb.md
    docs/dev/release.md
    tools/bench/README.md
    bench/AGENTS.md

Run focused validation after Criterion removal:

    rg -n "criterion|bench/expr|bench-expr|expr_syntax|expr_logical|expr_event|expr_waveform_host" . --glob '!target/**' --glob '!tmp/**'
    cargo check
    python3 -B -m unittest discover -s bench/e2e -p 'test_*.py'
    python3 -B -m unittest discover -s tools/bench -p 'test_*.py'

The search should return no source references except this ExecPlan while it remains in `docs/tracker/wip/`.

Run focused validation after the harness/policy refactor and after the best-sample confirmation update:

    python3 -B -m py_compile tools/bench/common.py tools/bench/capture.py tools/bench/compare.py tools/bench/gate.py bench/e2e/perf.py tools/fsdb/generate_bench_catalog.py
    python3 -B -m unittest discover -s bench/e2e -p 'test_*.py'
    python3 -B -m unittest discover -s tools/bench -p 'test_*.py'
    python3 -B -m unittest discover -s tools/fsdb -p 'test_*.py'
    just test-aux
    just format-justfile-check
    just check-actions

If the environment supports the full project gates, run:

    just check

The maintainer has explicitly requested one full release-like gate validation after this confirmation change is committed:

    just bench-gate v1.0.0 v1.0.1

### Validation and Acceptance

Unit tests should prove that Criterion is absent from source, the public benchmark recipes still point at the manual gate helpers, dirty current tooling is rejected, selected refs are built as binaries, current `bench/e2e/perf.py` is used for capture, current FSDB tools are used for preparation, compare no longer requires expression artifacts, same-format timing invokes median-only compare with a 5ms absolute floor, and cross-format checks remain functional-only.

The requested full release-like validation run is:

    just bench-gate v1.0.0 v1.0.1

A successful gate directory should contain root-level `e2e-fst/baseline/` and `e2e-fst/revised/` artifact directories, optional root-level `e2e-fsdb/baseline/` and `e2e-fsdb/revised/` artifact directories, `baseline/` and `revised/` capture manifest/log directories, `compare/`, `checkouts/`, `manifest.json`, and `summary.md`. There should be no `expr/` capture directory. The compare manifest should show `e2e-fst`, optional `e2e-fsdb`, and cross-format functional checks. Top-level compare metadata records `timing_metric`, `timing_threshold_pct`, and `timing_threshold_seconds`; same-format suite entries record `timing_metric`, `threshold_pct`, and `threshold_seconds`. When median timing fails but best-sample confirmation passes, the same-format suite should be reported as passed with an attached `timing_confirm` object and a separate confirmation log path, so maintainers can see that the primary median compare needed confirmation.

### Idempotence and Recovery

The helper must be safe to run repeatedly because each default run creates a unique timestamped output directory under `tmp/bench-gate/`. If a user passes an explicit output directory and it already contains files, the helper should fail before cloning or running benchmarks. The safe recovery is to choose a new output directory or manually remove the old one after confirming it is disposable. The implementation must not delete arbitrary paths automatically.

If benchmark capture fails halfway, the partial output directory and logs should remain for diagnosis. A maintainer can rerun the command with a fresh output directory after fixing the problem.

### Artifacts and Notes

Issue 24 has been updated with the rejected CI direction and selected manual gate direction. The comment URL is `https://github.com/kleverhq/wavepeek/issues/24#issuecomment-4748912184`.

Recent analysis artifacts are local ignored evidence under `tmp/bench-gate/`, especially:

    tmp/bench-gate/manual-three-runs-20260620T122235Z/
    tmp/bench-gate/virtual-best-median-20260620T122235Z-3runs/
    tmp/bench-gate/duration-vs-noise-20260620/

These artifacts explain the policy change but should not be committed.

### Interfaces and Dependencies

The helper scripts must use only Python standard-library modules. Each script should expose `main(argv: Sequence[str] | None = None) -> int` and return process exit codes rather than calling `sys.exit` deep inside helper functions. The stable public interface is the root `justfile`, not direct helper paths.

`tools/bench/capture.py` must invoke current benchmark harnesses from `REPO_ROOT` and must pass selected binaries with explicit `--binary label=path` arguments. `tools/bench/compare.py` invokes current comparison harnesses from `REPO_ROOT`. Capture manifests must distinguish current tooling provenance from binary source provenance.

### Revision Notes

- 2026-06-18 / Grin: Initial plan created after discussion switched issue 24 away from CI-collected benchmark artifacts and toward a manual source-ref comparison gate.
- 2026-06-19 / Grin: Revised plan after real local runs exposed FSDB runnable-catalog issues and after discussion corrected timing policy.
- 2026-06-20 / Grin: Replaced the previous source-ref-tooling plan with the current-tooling plan after three full null-experiment gate runs showed false positives. Added complete Criterion removal, current-tooling capture semantics, median-only E2E timing, and a 5ms absolute slowdown floor.
