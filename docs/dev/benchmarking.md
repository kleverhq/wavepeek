# Benchmarking Guide

Benchmark work must run in the devcontainer or CI image so fixture availability, Rust toolchain versions, and helper behavior match project gates. Preserve the public behavior contracts in `docs/public/reference/command-model.md` and `docs/public/reference/machine-output.md` while optimizing.

## Manual performance gate

`wavepeek` does not commit benchmark baseline runs. Release performance review compares explicit source refs in one controlled environment and stores generated artifacts under ignored paths such as `tmp/bench-gate/`.

Public benchmark entrypoints are:

    just bench-gate <baseline-ref> [revised-ref] [fsdb-mode]
    just bench-capture [ref] [fsdb-mode]
    just bench-compare <golden-capture-dir> <revised-capture-dir>

`just bench-gate vX.Y.Z HEAD` clones both refs under `tmp/bench-gate/` and builds release binaries from those refs. Benchmark scripts, catalogs, fixtures, FSDB preparation helpers, and compare logic come from the current working tree. The helper refuses to run from a dirty current worktree because current benchmark tooling is part of the measurement apparatus and must be reproducible.

Same-format FST and FSDB comparisons use functional checks plus median timing. Timing first fails when revised median time exceeds golden median time by more than `max(5%, 5ms)` by default. When a same-format suite fails only because of median timing, with no functional mismatches, missing/invalid artifacts, or timeout warnings, the gate runs a separate best-sample confirmation over those failed tests using the minimum hyperfine sample from each binary. Timing is accepted when `revised_best - golden_best <= max(golden_best * 5%, 5ms)` for every confirmed test. Mean timing is still recorded by hyperfine but is not a gate metric. Cross-format FST-vs-FSDB checks are functional-only within each capture because FST and FSDB use different readers and timing them against each other is not meaningful. Cross-format checks use an explicit ignored-test list for metadata-only hierarchy and signal cases where FST and FSDB expose arrays, memories, or scalar ranges with different path strings; each ignored test and reason is recorded in the compare manifest.

Default gate output has this shape:

    tmp/bench-gate/gates/<timestamp>-<baseline>..<revised>/
      e2e-fst/
        baseline/     # FST artifacts for the baseline binary
        revised/      # FST artifacts for the revised binary
      e2e-fsdb/       # present when FSDB is captured
        baseline/
        revised/
      baseline/       # baseline capture manifest and logs
      revised/        # revised capture manifest and logs
      compare/
      checkouts/
      manifest.json
      summary.md

`just bench-capture <ref>` builds the selected ref, captures it with current benchmark tooling, and prints the `.../run` directory. Pass that printed `run` directory to `just bench-compare`, not the parent directory that also contains the checkout. Standalone compare output is separate from gate output and defaults to `tmp/bench-gate/compares/<timestamp>-compare/` with its own `manifest.json` and `README.md`.

    just bench-capture vX.Y.Z
    just bench-capture HEAD
    just bench-compare tmp/bench-gate/captures/<baseline>/run tmp/bench-gate/captures/<revised>/run

FSDB mode defaults to `auto`. Auto mode skips FSDB when Verdi is unavailable or when both refs lack FSDB support, captures FSDB when Verdi is available and both refs support it, and fails on asymmetric FSDB support because that comparison is not equivalent. FSDB capture uses a generated runnable catalog under each capture directory and records any omitted tests in the manifest; currently this excludes VCD-style scalar element paths such as `foo.[0]` because converted RTL FSDB fixtures expose those signals with FSDB-specific names that old and current release binaries cannot resolve from the FST-derived catalog. Use `just bench-gate <baseline-ref> <revised-ref> never` only when intentionally skipping FSDB performance review, and record that rationale in the release notes or checklist. Use `python3 -B tools/bench/gate.py --baseline-ref <ref> --revised-ref <ref> --fsdb always` when FSDB capture is required and missing support or Verdi should fail immediately.

The gate screens selected benchmarks for regressions on the machine where it runs. It is not a general performance guarantee. Use the previous release tag as the baseline for major and minor releases. For patch releases, either run the gate when the change may affect performance, or record why the gate was skipped for clearly non-performance changes such as documentation-only or release-metadata-only updates.

## CLI End-to-End Benchmarks

The end-to-end CLI harness is `bench/e2e/perf.py`. It is Python-stdlib only and uses `hyperfine` for timing. Default FST test definitions live in `bench/e2e/tests.json`. FSDB benchmark definitions live in generated `bench/e2e/tests_fsdb.json`; `fsdb.md` owns the FSDB catalog, Verdi, and fixture details. Release-gate catalogs should use at least 10 measured hyperfine runs and 5 warmup runs. The pre-commit smoke catalog `bench/e2e/tests_commit.json` intentionally keeps 1 measured run and 0 warmups.

Common focused commands:

    python3 bench/e2e/perf.py list
    python3 bench/e2e/perf.py run --binary current=target/release/wavepeek --filter '^info_'
    python3 bench/e2e/perf.py run --binary current=target/release/wavepeek --run-dir bench/e2e/runs/<run-id> --missing-only
    python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/<run-id>/current
    python3 bench/e2e/perf.py compare --revised <dir> --golden <dir> --max-negative-delta-pct 5 --max-negative-delta-seconds 0.005
    python3 bench/e2e/perf.py confirm --revised <dir> --golden <dir> --test <name> --max-negative-delta-pct 5 --max-negative-delta-seconds 0.005
    just update-bench-e2e-fsdb-catalog
    just check-bench-e2e-fsdb-catalog

Pass one or more `--binary label=path` arguments to choose the binaries used by generated commands. The runner always writes one labeled artifact directory per binary and defaults to a round-robin schedule that runs each selected test on all binaries before moving to the next test. The runner is greedy by default: each test is preflighted with the JSON functional command before timing, and a per-test command failure writes `TEST.failure.json` then continues with the remaining matrix. Pass `--fail-fast` when debugging to stop on the first preflight or timing harness failure. Each successful test has both `TEST.hyperfine.json` and `TEST.wavepeek.json`; failed tests have `TEST.failure.json` instead, so argument-error paths are not timed as fast benchmark results.

Timing compare mode compares only tests with complete success artifacts on both sides. Golden failure plus revised success, and failures on both sides, are reported as skipped uncomparable tests. Revised failure where golden succeeded, missing outcomes without explicit failure artifacts, functional mismatches, and timing threshold violations fail the comparison. Compare and gate manifests include comparable counts, skipped/failed uncomparable counts, and grouped failure records. If median timing is the only failure, `perf.py confirm` can check selected tests with best samples; the gate runs this confirmation automatically for failed same-format timing tests. Cross-format gate checks use `--functional-only --allow-golden-extra` because the FSDB runnable catalog can be a subset of the FST catalog, plus repeated `--ignore-functional-test NAME=REASON` entries for known metadata-only path-shape differences.

Some E2E catalogs include paired sampling-mode tests with matching names ending in `sample_native` and `sample_pre_edge`. Use the normal run reports to inspect native and pre-edge timings side by side. All `change` and `property` catalog commands must pass `--on` explicitly; wildcard, plain-signal, and mixed-trigger workloads must also pass `--sample-mode native` because the CLI default is pre-edge sampling for edge-only triggers.

Low-level `bench-e2e-run` and `bench-e2e-fsdb-run` just recipes are private development helpers. They capture ad hoc ignored runs and do not update committed baselines.

Use fresh run directories for local experiments. Benchmark run artifacts are evidence, but they are not repository source artifacts unless a maintainer explicitly asks to preserve a specific result outside the ignored run locations.
