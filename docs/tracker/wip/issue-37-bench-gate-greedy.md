# Make the benchmark gate greedy on per-test CLI failures

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

The manual benchmark gate currently stops when one selected benchmark command fails for one binary. That is a bad default for release comparisons because the current benchmark catalog can contain command-line flags that an older baseline binary does not support. After this change, `bench/e2e/perf.py run` will keep running the rest of the selected benchmark matrix, write an explicit per-test failure artifact for each unsupported command, and let `bench/e2e/perf.py compare` compare only tests that have normal timing and functional artifacts on both sides. A user can see the behavior by benchmarking the current branch against `v1.0.1`: tests using newer `--sample-mode` CLI flags should be recorded as baseline failures instead of aborting the whole gate, while other comparable tests should still be measured and compared.

A "failure artifact" in this plan means a small JSON file next to the usual benchmark artifacts that records that a specific binary/test pair could not produce a valid benchmark result. It is not a timing artifact and must never be measured as a fast success path. A "comparable test" means a test that produced both normal artifacts, `*.hyperfine.json` and `*.wavepeek.json`, in both the golden/baseline run directory and the revised run directory.

## Non-Goals

This work does not change the Rust `wavepeek` command-line interface. It does not make old binaries understand new flags. It does not use `hyperfine --ignore-failure` as the main solution because that would risk timing argument-error exits. It does not make missing files acceptable: a test without either normal artifacts or an explicit failure artifact remains a harness integrity error.

## Progress

- [x] (2026-06-23 06:29Z) Created branch `issue-37-bench-gate-greedy` from `main`.
- [x] (2026-06-23 06:29Z) Read issue #37, `bench/e2e/perf.py`, `tools/bench/compare.py`, `tools/bench/capture.py`, `tools/bench/gate.py`, `bench/e2e/test_perf.py`, `tools/bench/test_gate.py`, and relevant benchmark guidance.
- [x] (2026-06-23 06:29Z) Wrote this initial implementation plan.
- [x] (2026-06-23 06:30Z) Committed the plan as `docs: plan greedy benchmark gate failures`.
- [x] (2026-06-23 06:35Z) Ran a read-only plan review and updated this plan for the findings.
- [x] (2026-06-23 06:56Z) Implemented explicit per-test failure artifacts and greedy run behavior in `bench/e2e/perf.py`.
- [x] (2026-06-23 06:56Z) Updated compare policy in `bench/e2e/perf.py` so success/success tests are compared, baseline-only failures are skipped as unsupported, revised-only failures fail the gate, both-side failures are skipped but reported, and missing outcomes fail as integrity errors in timing and functional-only modes.
- [x] (2026-06-23 06:56Z) Updated manual gate helper checks in `tools/bench/compare.py` so explicit failure artifacts are valid benchmark outcomes rather than incomplete artifact sets.
- [x] (2026-06-23 06:56Z) Updated reports, compare manifests, the top-level gate manifest, and docs to expose comparable counts, skipped/uncomparable counts, and grouped failure records.
- [x] (2026-06-23 06:56Z) Added and updated unit tests covering parser flags, failure artifact writing/loading, greedy run behavior, compare classification, invalid uncomparable artifacts, functional-only subset validation, and gate summary aggregation.
- [x] (2026-06-23 06:59Z) Ran focused checks and `just test-aux`, then ran read-only code/tooling/docs reviews.
- [x] (2026-06-23 07:04Z) Ran final control review with no substantive findings, ran `just check` successfully, and prepared implementation for commit.
- [ ] Run the proof benchmark/gate against `v1.0.1` and record evidence showing `--sample-mode` baseline failures are recorded while the gate continues.
- [ ] Push the branch and open a pull request linked to issue #37.

## Surprises & Discoveries

- Observation: The existing run order in `cmd_run` executes `hyperfine` before the functional JSON capture.
  Evidence: `bench/e2e/perf.py` function `run_one` calls `run_test(...)`, then `run_functional_capture(...)`. Issue #37 explicitly requires a functional/preflight command first so incompatible CLI surfaces are excluded from timing.
- Observation: The gate helper currently rejects any same-format comparison whose normal artifact stems differ before `perf.py compare` can classify failures.
  Evidence: `tools/bench/compare.py` has `assert_matching_e2e_artifacts`, which requires equal sets of `*.hyperfine.json` and `*.wavepeek.json` stems and raises `BenchGateError` on differences.
- Observation: A benchmark-phase failure must not leave a `*.wavepeek.json` file without a matching `*.hyperfine.json` file unless every validator explicitly treats that combination as a valid failure outcome.
  Evidence: The plan review noted that writing the wavepeek artifact before `hyperfine` would conflict with the intended integrity check that dangling normal artifacts are invalid.
- Observation: Parsing all hyperfine files during compare would let stale or partial uncomparable files abort before the structured compare result could report an integrity error.
  Evidence: The implementation review found that `cmd_compare` loaded every `*.hyperfine.json`; this was changed so timing mode parses only `classification["comparable"]` tests.
- Observation: Cross-format functional-only subset validation must not require timing artifacts.
  Evidence: The implementation review found that `tools/bench/compare.py` was still using same-format hyperfine+wavepeek success validation for FSDB-vs-FST functional subset checks; this was changed to wavepeek-or-failure functional outcomes for subset validation.

## Decision Log

- Decision: Use a separate `*.failure.json` artifact for per-test failures rather than overloading `*.wavepeek.json` or `*.hyperfine.json`.
  Rationale: Normal artifacts already mean a command completed far enough to be functionally compared and timed. Separate failure artifacts make it impossible to accidentally treat a fast argument-error path as a benchmark timing.
  Date/Author: 2026-06-23 / Grin
- Decision: Treat `perf.py run` as greedy by default and add `--fail-fast` to restore the previous aborting behavior.
  Rationale: The issue asks for greedy default behavior and an opt-in old behavior. Existing harness-integrity failures such as invalid test catalogs, missing binary paths, missing `hyperfine`, and invalid output directories should still abort.
  Date/Author: 2026-06-23 / Grin
- Decision: Run the functional/preflight command before `hyperfine`, keep the successful payload in memory, and write the `*.wavepeek.json` artifact only after `hyperfine` succeeds.
  Rationale: This satisfies the requirement to preflight first, avoids running each successful command an extra time beyond current behavior, and prevents benchmark-phase failures from producing dangling normal artifacts that later validators must treat as corrupt.
  Date/Author: 2026-06-23 / Grin
- Decision: Keep a timeout `{}` wavepeek artifact as a normal functional artifact timeout warning for existing completed success artifacts, but record preflight timeouts, command failures, and invalid functional output as failure artifacts in the new greedy run path.
  Rationale: The existing compare logic already understands `{}` as a timeout marker for old artifacts. For new preflight-first capture, a timeout means the command did not produce a comparable benchmark artifact and should be explicit failure evidence.
  Date/Author: 2026-06-23 / Grin
- Decision: Remove any stale normal or failure artifact for a `(binary, test)` before rerunning that cell unless `--missing-only` skipped it.
  Rationale: A reused `--run-dir` must not accidentally combine an old success with a new failure, or an old failure with a new success. One outcome per binary/test keeps comparison deterministic.
  Date/Author: 2026-06-23 / Grin
- Decision: Propagate per-suite compare summary fields into the top-level gate manifest rather than relying only on paths to suite result JSON files.
  Rationale: Issue #37 explicitly requires the gate manifest to include comparable and skipped counts plus grouped failure records. Links alone would make the top-level manifest incomplete.
  Date/Author: 2026-06-23 / Grin

## Outcomes & Retrospective

Implementation is complete in the working tree and validation passes. `python3 -B -m unittest bench.e2e.test_perf tools.bench.test_gate` passed with 115 tests, `just test-aux` passed after review fixes, and `just check` passed. The remaining outcome work is the proof benchmark against `v1.0.1`, push, and PR.

## Context and Orientation

The repository root is `/workspaces/wavepeek`. The project is a Rust CLI, but this task is about Python benchmark harnesses under `bench/e2e/` and helper automation under `tools/bench/`.

`bench/e2e/perf.py` is the end-to-end benchmark harness. It has subcommands: `list`, `run`, `report`, `compare`, and `confirm`. The `run` subcommand accepts one or more `--binary label=path` arguments and writes one labeled artifact directory per binary. Each successful test currently writes `TEST.hyperfine.json` from `hyperfine` and `TEST.wavepeek.json` from a JSON functional capture. The `compare` subcommand compares a revised run directory against a golden run directory. It currently loads only hyperfine and wavepeek artifacts, compares matched timing by median, compares functional JSON payloads, and warns about tests present only on one side.

`tools/bench/capture.py` builds release binaries and invokes `perf.py run` for FST and optional FSDB suites. In a two-binary manual gate, `run_e2e_many` calls `perf.py run` once with `--binary baseline=...` and `--binary revised=...`, so greedy behavior must work across a whole matrix of binaries and tests. `tools/bench/compare.py` compares capture directories and currently performs strict artifact identity checks before calling `perf.py compare`. Those checks must recognize explicit failure artifacts as valid outcomes. `tools/bench/gate.py` orchestrates the full manual gate and writes the top-level manifest and summary.

`bench/e2e/test_perf.py` contains Python unit tests for `perf.py`. `tools/bench/test_gate.py` contains Python unit tests for `tools/bench` helper behavior. The repository’s root `justfile` defines the standard gates. Project guidance says `just check` is the local pre-handoff gate and `just ci` is the full gate.

## Open Questions

There are no unresolved product questions at this point. The top-level manual gate manifest will include structured per-suite comparable and skipped counts plus grouped failure records. The Markdown summary may stay concise and point to detailed suite artifacts, but the machine-readable manifest must be self-contained enough to satisfy issue #37.

## Plan of Work

First, add a failure artifact model to `bench/e2e/perf.py`. Define a suffix such as `.failure.json`, path helpers, a writer for failure artifacts, and a loader that validates basic shape. A failure artifact should include `kind`, `schema_version`, `test_name`, `phase`, `exit_code` when available, `timed_out`, the executed command as a list of strings, summarized stdout and stderr, and generation time. Include binary label when known. The summary should be short and deterministic, for example the first few kilobytes or first few lines, so reports stay readable.

Second, change `cmd_run` so each `(binary, test)` starts by deleting stale artifacts for that binary/test cell, then runs the functional command. If the functional subprocess exits non-zero, times out, emits invalid JSON, or emits JSON that fails the machine-output validator, write a failure artifact with phase `preflight` and continue unless `--fail-fast` is set. If it succeeds, keep the functional payload in memory and run `hyperfine`. Only after `hyperfine` succeeds should the runner write the normal wavepeek artifact, so a success outcome always has both normal files. If `hyperfine` fails, write a failure artifact with phase `benchmark` and continue unless `--fail-fast` is set. Keep harness-integrity failures as aborts: invalid catalog JSON, missing binary paths, invalid binary labels, missing `hyperfine` when there is work to time, invalid regex filters, missing run directories, and unsupported schedules should still exit non-zero.

Third, update `perf.py report` and generated `README.md` files. The report should show normal hyperfine and wavepeek artifact counts, explicit failure count, comparable count when comparing to a baseline, skipped/uncomparable count, and a concise section listing failure artifacts grouped by phase. This is user-facing evidence that skipped tests are visible rather than silently ignored.

Fourth, update `perf.py compare`. It must classify outcomes by test name for the golden and revised directories in both normal timing mode and `--functional-only` mode. The outcome for a test is `success` when both normal artifacts exist and no failure artifact exists, `failure` when a failure artifact exists and normal artifacts for that test do not form a complete success, and `missing` when neither success nor failure exists for a selected side. In timing mode, only success/success tests enter timing and functional comparison. In functional-only mode, only success/success tests enter functional comparison, and timing artifacts are ignored except for integrity checks on same-directory normal artifact consistency when timing artifacts are present. Golden failure with revised success is skipped as baseline unsupported. Golden failure with revised failure is skipped as both-side failure. Revised failure with golden success fails. Missing outcomes without explicit failure artifacts fail as harness integrity errors. The result JSON should include `comparable_count`, `skipped_uncomparable_count`, `baseline_unsupported`, `revised_failures`, `both_side_failures`, `integrity_errors`, and the existing timing and functional fields.

Fifth, update `tools/bench/compare.py`. Replace `e2e_artifact_stems` and `assert_matching_e2e_artifacts` semantics with outcome-aware validation. Same-format comparisons must allow tests that have explicit failure artifacts on one or both sides, but must still fail when a test has a dangling hyperfine without wavepeek, a dangling wavepeek without hyperfine, both normal success and failure artifacts, or appears on only one side with no explicit failure artifact. Cross-format functional-only subset checks should use successful functional artifact stems plus explicit failures so that an FSDB subset remains valid while incomplete artifacts still fail. When `perf.py compare` writes a result JSON, copy its comparable counts, skipped counts, baseline unsupported records, both-side failures, revised failure records, and integrity errors into the suite entry in `tools/bench/compare.py`'s manifest. Then update `tools/bench/gate.py` to copy those per-suite fields into the top-level gate manifest under its `compare` section.

Sixth, update tests. In `bench/e2e/test_perf.py`, add parser coverage for `--fail-fast`, writer/loader tests for `*.failure.json`, greedy `cmd_run` behavior using mocked subprocess calls, and compare classification tests for success/success, golden failure/revised success, both failures, revised failure/golden success, and missing artifact integrity failures. In `tools/bench/test_gate.py`, update artifact identity tests so explicit failure artifacts are accepted and missing outcomes are still rejected. Keep tests deterministic and use temporary directories.

Seventh, update `docs/dev/benchmarking.md` to document that the run is greedy by default, `--fail-fast` restores abort-on-first-test-failure behavior, explicit failure artifacts are part of the benchmark evidence, and compare/gate status distinguishes unsupported baseline tests from revised regressions.

Finally, run checks and a proof benchmark. The focused tests are `python3 -B -m unittest bench.e2e.test_perf tools.bench.test_gate` if module discovery works from the repository root; otherwise run the two files directly with `python3 -B bench/e2e/test_perf.py` and `python3 -B tools/bench/test_gate.py`. The repository pre-handoff check is `just check`. For proof, run `just bench-gate v1.0.1 HEAD never` after committing implementation, or use `python3 -B tools/bench/gate.py --baseline-ref v1.0.1 --revised-ref HEAD --fsdb never --out-dir tmp/bench-gate/gates/issue-37-proof-v1.0.1..HEAD` if an explicit output directory is needed. Expect the run to finish even though baseline `v1.0.1` does not support tests using `--sample-mode`, and expect compare/gate artifacts to report those as baseline unsupported or uncomparable rather than aborting during capture.

### Concrete Steps

Work from `/workspaces/wavepeek`.

1. Commit this plan:

    git add docs/tracker/wip/issue-37-bench-gate-greedy.md
    git commit -m "docs: plan greedy benchmark gate failures"

2. Run a read-only plan review with a focused review worker. The reviewer should inspect this plan, issue #37, and relevant benchmark files without editing anything. Address any concrete findings by editing this plan and amending or adding a follow-up commit.

3. Edit `bench/e2e/perf.py` to add failure artifacts, greedy run behavior, outcome-aware compare, and report fields.

4. Edit `tools/bench/compare.py` to use outcome-aware artifact validation before calling `perf.py compare`.

5. Edit `bench/e2e/test_perf.py` and `tools/bench/test_gate.py` to cover the new behavior.

6. Edit `docs/dev/benchmarking.md` to document the new benchmark contract.

7. Run focused tests:

    python3 -B -m unittest bench.e2e.test_perf tools.bench.test_gate

   If import layout prevents that, run:

    python3 -B bench/e2e/test_perf.py
    python3 -B tools/bench/test_gate.py

8. Run the repository pre-handoff check:

    just check

9. Run a second read-only review on the full diff. Apply fixes and rerun affected tests.

10. Commit the implementation with a conventional commit message, push the branch, and open a PR whose body includes `Closes #37`.

11. Run or record the proof benchmark output directory and summarize the important artifact paths in the PR.

### Validation and Acceptance

The change is accepted when a benchmark run with two binaries continues after a per-test CLI failure and writes a `*.failure.json` artifact for the failing binary/test pair. The `--fail-fast` flag is accepted when the same failing scenario exits non-zero at the first per-test failure.

`perf.py compare` is accepted when success/success tests are counted in `comparable_count` and compared normally, baseline failure/revised success tests appear in baseline unsupported records and do not fail the gate, both-side failures appear in skipped records and do not fail the gate, revised failure/baseline success tests fail the gate, and missing outcomes without failure artifacts appear in integrity errors and fail the gate.

The manual gate is accepted when `just bench-gate v1.0.1 HEAD never` or the equivalent `tools/bench/gate.py` command completes capture instead of aborting during baseline sample-mode failures. The compare result should include a positive comparable count and explicit skipped/uncomparable records. If revised has no real regression, the final gate should not fail solely because `v1.0.1` lacks `--sample-mode`.

### Idempotence and Recovery

All generated benchmark artifacts belong under ignored paths such as `tmp/bench-gate/` or `bench/e2e/runs/`. If a proof run fails halfway, create a new output directory rather than deleting arbitrary existing `tmp/` contents. If a unit test creates temporary directories, it must use `tempfile.TemporaryDirectory()` so reruns are safe. If the branch needs to be reset during development, committed plan and implementation commits are recoverable through Git.

### Artifacts and Notes

Relevant issue text from #37, summarized: `perf.py run` should keep going on per-test failures by default; add `--fail-fast`; run a functional/preflight command first; write explicit failure artifacts; do not run hyperfine after failed preflight; compare only tests that produced comparable artifacts on both sides; baseline failed/revised passed and both failed are skipped as uncomparable; revised failed/baseline passed fails; missing artifacts without explicit failure artifacts fail as harness integrity problems; reports and manifests must include comparable/skipped counts and grouped per-test failure records.

Expected final proof evidence should look like this in concise form:

    gate written to tmp/bench-gate/gates/issue-37-proof-v1.0.1..HEAD
    compare manifest: tmp/bench-gate/gates/issue-37-proof-v1.0.1..HEAD/compare/manifest.json
    e2e-fst comparable_count: <positive number>
    e2e-fst skipped_uncomparable_count: <positive number>
    baseline unsupported includes sample-mode tests such as change_*_sample_native or property_*_sample_pre_edge

### Interfaces and Dependencies

`bench/e2e/perf.py` should expose these helper functions or equivalent internal interfaces:

    FAILURE_SUFFIX = ".failure.json"
    def failure_result_path(run_dir: pathlib.Path, test_name: str) -> pathlib.Path: ...
    def write_failure_artifact(...): ...
    def load_failure_results(run_dir: pathlib.Path) -> dict[str, dict[str, Any]]: ...
    def classify_compare_outcomes(...): ...

The exact signatures may change if a simpler design emerges, but the behavior must remain: failure artifacts are separately written, separately loaded, and used by compare and report code. The implementation must stay Python-stdlib-only because `bench/e2e/perf.py` is documented as stdlib-only.

Revision note, 2026-06-23: Initial plan created from issue #37 and source inspection so implementation can proceed from a self-contained document.

Revision note, 2026-06-23: Updated after read-only plan review. The plan now avoids dangling wavepeek artifacts on benchmark failure, covers functional-only compare classification, and requires top-level gate manifest propagation of uncomparable/failure summaries.

Revision note, 2026-06-23: Updated after implementation and focused review fixes. The implemented design now parses hyperfine artifacts only for comparable tests and uses functional-only outcome validation for cross-format subset checks.

Revision note, 2026-06-23: Updated after final control review and `just check`; no additional control-review fixes were needed.
