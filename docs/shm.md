# SHM Development

Use the private profile described in `environment.md` and the loader setup in `skills/wavepeek/references/quickstart.md`. Expose only the selected reader to the runtime loader. Adding the full Xcelium library directory can also load its bundled C++ runtime. `./dev just check-shm-build` launches the binary outside Cargo to check loading.

Conversion requires a configured license and the 64-bit converter launcher. Use `./dev just test-shm` to exercise the native reader. The rich-value test fixture is recorded directly because conversion loses string types. Converted inputs can have generic declaration kinds or an earlier start bound, which can change initial `change` rows. Ondas does not expose SHM signedness or enumeration labels.

Conversion reuses output only when the source and converter stamps match. Keep generated databases and conversion logs in ignored paths. See `benchmarking.md` for focused SHM workloads.
