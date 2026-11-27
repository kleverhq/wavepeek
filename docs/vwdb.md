# VWDB Development

Use the private profile in `environment.md` and the build requirements in `skills/wavepeek/references/quickstart.md`. Ondas loads the selected reader when it opens the recording. To verify loading outside Cargo, run `./dev just check-vwdb-build`.

`./dev just ci-vwdb` requires the VWDB reader. It checks lint, a native open, CLI regressions, and the five smoke queries. Source-backed recordings require a configured simulation license. The recipe writes them under `tmp/vwdb-fixtures/`. See `benchmarking.md` for published workloads, catalog generation, checksums, and the release gate.
