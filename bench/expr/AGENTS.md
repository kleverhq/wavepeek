# Expression Microbenchmark Guidance

## Source of Truth

- Benchmark workflow: `../../docs/dev/benchmarking.md`
- Expression semantics: `../../docs/public/reference/expression-language.md`
- Planning context: `../../docs/tracker/roadmap.md`
- Criterion bench registration: `../../Cargo.toml`

## Local Guidance

- Rust Criterion targets in this area are `expr_syntax.rs`, `expr_logical.rs`, `expr_event.rs`, and `expr_waveform_host.rs`.
- `suites.json` plus `perf.py` own the suite catalog and grouped run-directory format.
- Committed exported runs live under `runs/`; `runs/baseline/` is the durable baseline.
- Treat other run directories as local or review artifacts unless a plan explicitly promotes them.
- Keep exported `*.raw.csv`, `summary.json`, and `README.md` schema stable; provenance fields may vary between captures.
