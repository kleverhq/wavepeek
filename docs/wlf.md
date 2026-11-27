# WLF Development

Use the private profile in `environment.md` and the installation instructions in the packaged Quickstart. `./dev just check-wlf-build` checks the link and launches the binary outside Cargo.

Test and benchmark preparation requires `vcd2wlf`. Source-backed cases use the same generated VCD for tests and benchmarks. FST corpus inputs stream through VCD with standard declaration kinds.

Conversion can change declaration kinds, represent escaped or indexed names as literal components, and combine distinct declarations into ambiguous names. It can also add a first tick at zero. Use pinned native fixtures for WLF-specific hierarchy and value behavior. See `benchmarking.md` for cross-format comparisons.
