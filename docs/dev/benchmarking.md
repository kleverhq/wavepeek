# Benchmarking Guide

Benchmark work must run in the devcontainer or CI image so fixture availability, Rust toolchain versions, and helper behavior match project gates. Preserve the public behavior contracts in `docs/public/reference/command-model.md` and `docs/public/reference/machine-output.md` while optimizing.

## Manual performance gate

`wavepeek` does not commit benchmark baseline runs. Release performance review compares explicit source refs in one controlled environment and stores generated artifacts under ignored paths such as `tmp/bench-gate/`.

Public benchmark entrypoints are:

    just bench-gate <baseline-ref> [revised-ref] [fsdb-mode]
    just bench-capture [ref] [fsdb-mode]
    just bench-compare <golden-capture-dir> <revised-capture-dir>

`just bench-gate vX.Y.Z HEAD` clones both refs under `tmp/bench-gate/`, builds both refs before measurement, prepares both FSDB fixture sets before FSDB measurement, then runs FST and FSDB suites in baseline/revised pairs. Same-format FST and FSDB comparisons use timing plus functional checks with a default 5% maximum negative delta. Cross-format FST-vs-FSDB checks are functional-only within each capture because FST and FSDB use different readers and timing them against each other is not meaningful.

Default gate output has this shape:

    tmp/bench-gate/gates/<timestamp>-<baseline>..<revised>/
      baseline/       # compare input for the baseline capture
      revised/        # compare input for the revised capture
      compare/
      checkouts/
      manifest.json
      summary.md

`just bench-capture <ref>` writes a standalone capture and prints the `.../run` directory. Pass that printed `run` directory to `just bench-compare`, not the parent directory that also contains the checkout. Standalone compare output is separate from gate output and defaults to `tmp/bench-gate/compares/<timestamp>-compare/` with its own `manifest.json` and `README.md`.

    just bench-capture vX.Y.Z
    just bench-capture HEAD
    just bench-compare tmp/bench-gate/captures/<baseline>/run tmp/bench-gate/captures/<revised>/run

FSDB mode defaults to `auto`. Auto mode skips FSDB when Verdi is unavailable or when both refs lack FSDB support, captures FSDB when Verdi is available and both refs support it, and fails on asymmetric FSDB support because that comparison is not equivalent. FSDB capture uses a generated runnable catalog under each capture directory and records any omitted tests in the manifest; currently this excludes VCD-style scalar element paths such as `foo.[0]` because converted RTL FSDB fixtures expose those signals with FSDB-specific names that old and current release binaries cannot resolve from the FST-derived catalog. Use `just bench-gate <baseline-ref> <revised-ref> never` only when intentionally skipping FSDB performance review, and record that rationale in the release notes or checklist. Use `python3 -B tools/bench/gate.py --baseline-ref <ref> --revised-ref <ref> --fsdb always` when FSDB capture is required and missing support or Verdi should fail immediately.

The gate screens selected benchmarks for regressions on the machine where it runs. It is not a general performance guarantee. Use the previous release tag as the baseline for major and minor releases. For patch releases, either run the gate when the change may affect performance, or record why the gate was skipped for clearly non-performance changes such as documentation-only or release-metadata-only updates.

The helper refuses to benchmark `HEAD` from a dirty source worktree because cloned refs contain only committed state. Commit or stash changes before release use.

## CLI End-to-End Benchmarks

The end-to-end CLI harness is `bench/e2e/perf.py`. It is Python-stdlib only and uses `hyperfine` for timing. Default FST test definitions live in `bench/e2e/tests.json`. FSDB benchmark definitions live in generated `bench/e2e/tests_fsdb.json`; `fsdb.md` owns the FSDB catalog, Verdi, and fixture details.

Common focused commands:

    python3 bench/e2e/perf.py list
    python3 bench/e2e/perf.py run --filter '^info_'
    python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/<run-id> --missing-only
    python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/<run-id>
    python3 bench/e2e/perf.py compare --revised <dir> --golden <dir> --max-negative-delta-pct 5
    just update-bench-e2e-fsdb-catalog
    just check-bench-e2e-fsdb-catalog

Set `WAVEPEEK_BIN` to choose the binary used by generated commands. Each run writes per-test timing JSON, captured wavepeek JSON, and a run-level `README.md` report. Timing compare mode fails on matched-test threshold violations, functional `data` mismatches, or missing/invalid artifacts. The manual gate additionally fails when golden and revised end-to-end artifact sets differ, so release comparisons do not silently pass on partial intersections. Cross-format gate checks use `--functional-only --allow-golden-extra` because the FSDB runnable catalog can be a subset of the FST catalog.

Low-level `bench-e2e-run` and `bench-e2e-fsdb-run` just recipes are private development helpers. They capture ad hoc ignored runs and do not update committed baselines.

Use fresh run directories for local experiments. Benchmark run artifacts are evidence, but they are not repository source artifacts unless a maintainer explicitly asks to preserve a specific result outside the ignored run locations.
