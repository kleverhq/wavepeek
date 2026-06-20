# Benchmarking Guide

Benchmark work must run in the devcontainer or CI image so fixture availability, Rust toolchain versions, and helper behavior match project gates. Preserve the public behavior contracts in `docs/public/reference/command-model.md` and `docs/public/reference/machine-output.md` while optimizing.

## CLI End-to-End Benchmarks

The end-to-end CLI harness is `bench/e2e/perf.py`. It is Python-stdlib only and uses `hyperfine` for timing. Default FST test definitions live in `bench/e2e/tests.json`; committed FST baseline artifacts live under `bench/e2e/runs/baseline_fst/`. FSDB benchmark definitions live in generated `bench/e2e/tests_fsdb.json`, with committed baseline artifacts under `bench/e2e/runs/baseline_fsdb/`; `fsdb.md` owns the FSDB catalog and fixture details.

Common commands:

    python3 bench/e2e/perf.py list
    python3 bench/e2e/perf.py run --filter '^info_'
    python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/<run-id> --missing-only
    python3 bench/e2e/perf.py report --run-dir bench/e2e/runs/<run-id>
    python3 bench/e2e/perf.py compare --revised <dir> --golden <dir> --max-negative-delta-pct 5
    python3 bench/e2e/perf.py compare --functional-only --revised <fsdb-run> --golden bench/e2e/runs/baseline_fst
    just update-bench-e2e-fsdb-catalog
    just check-bench-e2e-fsdb-catalog

Set `WAVEPEEK_BIN` to choose the binary used by generated commands. Each run writes per-test timing JSON, captured wavepeek JSON, and a run-level `README.md` report. Timing compare mode fails on matched-test threshold violations, functional `data` mismatches, or missing/invalid artifacts. Functional-only compare skips timing thresholds but still fails data mismatches, invalid or missing JSON artifacts, timeout payloads, and unmatched tests unless `--allow-golden-extra` is intentionally used for a filtered smoke.

Some E2E catalogs include paired sampling-mode tests with matching names ending in `sample_native` and `sample_pre_edge`. Use the normal run reports to inspect native and pre-edge timings side by side; do not refresh committed baseline artifacts just to add those pairs.

Use `just bench-e2e-update-baseline` and `just bench-e2e-run` for the default FST baseline flow. Use `just bench-e2e-fsdb-update-baseline`, `just bench-e2e-fsdb-run`, and `just bench-e2e-fsdb-smoke-commit` only in a Verdi-equipped environment; see `fsdb.md` for generated FSDB artifacts and repository-safety rules.

## Expression Microbenchmarks

Expression-engine microbenchmarks live under `bench/expr/`. The functional Criterion bench targets are `expr_syntax`, `expr_logical`, `expr_event`, and `expr_waveform_host`; the suite catalog is `bench/expr/suites.json`.

Common commands:

    python3 bench/expr/perf.py list
    cargo test --bench expr_syntax --bench expr_logical --bench expr_event --bench expr_waveform_host
    python3 bench/expr/perf.py run --run-dir bench/expr/runs/<run-id>
    python3 bench/expr/perf.py run --run-dir bench/expr/runs/<run-id> --missing-only
    python3 bench/expr/perf.py report --run-dir bench/expr/runs/baseline
    python3 bench/expr/perf.py compare --revised <dir> --golden bench/expr/runs/baseline --max-negative-delta-pct 15 --require-matching-metadata cargo_version rustc_version criterion_version environment_note

`just bench-expr-update-baseline` refreshes the committed expression baseline through a guarded replace-in-place flow. `just bench-expr-run` captures and compares an ad hoc revised run against the baseline.

Use fresh run directories for local experiments unless the plan explicitly promotes a run artifact. Benchmark run artifacts are evidence; do not tidy them into nonsense just because the filenames look busy.
