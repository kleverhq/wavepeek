# FSDB Development

Use the private profile described in `environment.md` and the source installation instructions in `skills/wavepeek/references/quickstart.md`. Ondas embeds the canonical paths of the selected FSDB Reader libraries. Keep the installation at those paths when running the binary. A binary built in a container uses container paths. Reader libraries with an ELF SONAME are unsupported.

Ondas suppresses reader stdout during the whole reader lifecycle to protect the CLI output stream. Reader stderr can still be visible in machine modes. A value-support error can occur before a predicate matches because the query can decode selected values first.

Use `./dev just test-fsdb` for test fixtures derived from VCD and for pinned native fixtures that cover string and logic cases conversion cannot reproduce. FSDB smoke uses the test fixtures and opens `m2_core.fsdb`. Benchmark preparation reuses a current VCD next to the FST when available and otherwise uses a temporary conversion. See `benchmarking.md` for parity exceptions.

Keep generated FSDB inputs and converter logs in ignored paths. Keep installed manuals, copied declarations, and generated bindings from proprietary headers out of Git.
