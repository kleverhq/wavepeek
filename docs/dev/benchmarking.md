# Benchmarking Guide

Benchmark work must run in the devcontainer or CI image so fixture availability, Rust toolchain versions, and helper behavior match project gates. Preserve the public behavior contracts in `docs/public/reference/command-model.md` and `docs/public/reference/machine-output.md` while optimizing.

## Manual performance gate

`wavepeek` does not commit benchmark baseline runs. Release performance review compares explicit source refs in one controlled environment and stores generated artifacts under ignored paths such as `tmp/bench-gate/`.

Public benchmark entrypoints are:

    just bench-gate <baseline-ref> [revised-ref] [fsdb-mode]
    just bench-capture [ref] [fsdb-mode]
    just bench-compare <golden-capture-dir> <revised-capture-dir>

`just bench-gate vX.Y.Z HEAD` clones both refs under `tmp/bench-gate/`, captures FST end-to-end benchmarks and expression microbenchmarks for each ref, captures FSDB end-to-end benchmarks when Verdi is available and both refs support FSDB, then compares revised results against the baseline. The default thresholds are 5% maximum negative delta for FST and FSDB end-to-end benchmarks and 15% maximum negative delta for expression microbenchmarks. The helper records manifests, logs, and summaries next to the captures.

Default gate output has this shape:

    tmp/bench-gate/gates/<timestamp>-<baseline>..<revised>/
      baseline/       # compare input for the baseline capture
      revised/        # compare input for the revised capture
      compare/
      checkouts/
      manifest.json
      summary.md

`just bench-capture <ref>` writes a standalone capture and prints the `.../run` directory. Pass that printed `run` directory to `just bench-compare`, not the parent directory that also contains the checkout.

    just bench-capture vX.Y.Z
    just bench-capture HEAD
    just bench-compare tmp/bench-gate/captures/<baseline>/run tmp/bench-gate/captures/<revised>/run

FSDB mode defaults to `auto`. Auto mode skips FSDB when Verdi is unavailable or when both refs lack FSDB support, captures FSDB when Verdi is available and both refs support it, and fails on asymmetric FSDB support because that comparison is not equivalent. Use `just bench-gate <baseline-ref> <revised-ref> never` only when intentionally skipping FSDB performance review, and record that rationale in the release notes or checklist. Use `python3 -B tools/bench/gate.py gate --baseline-ref <ref> --revised-ref <ref> --fsdb always` when FSDB capture is required and missing support or Verdi should fail immediately.

The gate screens selected benchmarks for regressions on the machine where it runs. It is not a general performance guarantee. Use the previous release tag as the baseline for major and minor releases. For patch releases, either run the gate when the change may affect performance, or record why the gate was skipped for clearly non-performance changes such as documentation-only or release-metadata-only updates.

The helper refuses to benchmark `HEAD` from a dirty source worktree by default because cloned refs contain only committed state. Commit or stash changes before release use, or run `python3 -B tools/bench/gate.py ... --allow-dirty-source` only when intentionally benchmarking committed refs while unrelated local edits exist.

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

Set `WAVEPEEK_BIN` to choose the binary used by generated commands. Each run writes per-test timing JSON, captured wavepeek JSON, and a run-level `README.md` report. Timing compare mode fails on matched-test threshold violations, functional `data` mismatches, or missing/invalid artifacts. The manual gate additionally fails when golden and revised end-to-end artifact sets differ, so release comparisons do not silently pass on partial intersections.

Low-level `bench-e2e-run` and `bench-e2e-fsdb-run` just recipes are private development helpers. They capture ad hoc ignored runs and do not update committed baselines.

## Expression Microbenchmarks

Expression-engine microbenchmarks live under `bench/expr/`. The functional Criterion bench targets are `expr_syntax`, `expr_logical`, `expr_event`, and `expr_waveform_host`; the suite catalog is `bench/expr/suites.json`.

Common focused commands:

    python3 bench/expr/perf.py list
    cargo test --bench expr_syntax --bench expr_logical --bench expr_event --bench expr_waveform_host
    python3 bench/expr/perf.py run --run-dir bench/expr/runs/<run-id>
    python3 bench/expr/perf.py run --run-dir bench/expr/runs/<run-id> --missing-only
    python3 bench/expr/perf.py report --run-dir bench/expr/runs/<run-id>
    python3 bench/expr/perf.py compare --revised <dir> --golden <dir> --max-negative-delta-pct 15

The private `bench-expr-run` just recipe captures an ad hoc ignored run and does not update committed baselines. Expression compare mode requires matching catalog fingerprints and selected suites. The manual gate does not require matching Criterion crate versions because release comparisons may cross dependency updates.

Use fresh run directories for local experiments. Benchmark run artifacts are evidence, but they are not repository source artifacts unless a maintainer explicitly asks to preserve a specific result outside the ignored run locations.
