# fsdb_demo

Standalone feasibility probe for the lazy-loading requirement in
`docs/fsdb_proposal.md`.

## What this demo is trying to prove

The RFC's main gating hypothesis is:

1. a Rust binary can be built on a licensed Verdi host,
2. the Verdi/FsdbReader-dependent code can live behind a narrow native bridge,
3. the main binary can start and run a non-FSDB command without loading Verdi,
4. the FSDB path can still succeed once the bridge is loaded lazily.

This directory is intentionally self-contained so you can copy it to a Verdi host
and test only that hypothesis, without touching the main `wavepeek` crate.

Run the commands below from inside the copied `fsdb_demo/` directory.

For a copy-paste validation checklist and result interpretation, see
`VALIDATION.md`.

For a structured report you can hand to another agent or send back here, see
`REPORT_TEMPLATE.md`.

## What you need on the Verdi machine

- Linux host
- Rust toolchain with Cargo
- a C++ compiler (`c++` or `CXX=/path/to/g++`)
- `VERDI_HOME` pointing to the Verdi install root
- readable FSDB file for probing

Expected Verdi layout under `VERDI_HOME`:

- `share/FsdbReader/ffrAPI.h`
- `share/FsdbReader/linux64/libnffr.so`
- `share/FsdbReader/linux64/libnsys.so`

If your environment needs extra ABI/toolchain flags, you can pass them through:

- `FSDB_DEMO_EXTRA_CXXFLAGS`
- `FSDB_DEMO_EXTRA_LDFLAGS`

That is based on the same kind of build friction shown by public examples such as
`fsdb-parse`, GTKWave's FSDB notes, and VaporView's FSDB discussion.

## Commands

Build the demo:

```bash
export VERDI_HOME=/path/to/verdi
cargo build
```

Inspect what got built:

```bash
cargo run -- build-info
```

Non-FSDB smoke path that should not load Verdi at all:

```bash
cargo run -- noop
```

Real FSDB probe path:

```bash
cargo run -- probe --waves /path/to/file.fsdb
```

Default bridge lookup is:

1. `--bridge /explicit/path/to/libfsdb_verdi_bridge.so`
2. `libfsdb_verdi_bridge.so` next to the executable
3. the compiled Cargo build-tree path reported by `build-info`

So this demo is best treated as a copied source tree that you rebuild on the
licensed host. A prebuilt binary-plus-bridge kept side by side is only a partial
experiment: the bridge still carries the build host's Verdi loader path unless
you arrange compatible runtime loader configuration yourself. This is **not** by
itself a proof of the final `cargo install` story from the RFC.

## Suggested verification flow on the licensed host

1. Build with `VERDI_HOME` set.
2. Run `build-info` and note the `verdi-bridge-path`.
3. Inspect the main binary and the bridge separately:

   ```bash
   ldd target/debug/fsdb_demo
   ldd <verdi-bridge-path-from-build-info>
   readelf -d target/debug/fsdb_demo | grep NEEDED
   ```

   Desired signal:

   - `target/debug/fsdb_demo` should **not** list `libnffr.so` / `libnsys.so`
   - the bridge `.so` **should** list them

4. Run `noop` with the same built binary.

   That is the local stand-in for the RFC's requirement that a non-FSDB command
   from the same binary must still start successfully even when the Verdi runtime
   is not touched.

5. Run `probe --waves ...` and confirm that the lazy-loaded bridge can actually
   open the FSDB through FsdbReader.

6. For the stronger negative-control check from the RFC, re-run the already-built
   binary in an environment where Verdi runtime libraries are intentionally not
   resolvable (for example, a clean shell/container without the relevant loader
   setup) and confirm that `./target/debug/fsdb_demo noop` still works.

This demo gives useful evidence, but it does **not** fully satisfy the RFC gate
unless you perform that negative-control step.

Even after that, this remains a **demo-level feasibility check**. Full RFC sign-off
still requires repeating the same kind of runtime proof with a real
FSDB-enabled `wavepeek` build and a real VCD/FST command.

## Why this is plausible from public sources

Public sources already show that the hard part is not "can FsdbReader be called
from custom code?" — that part is clearly yes:

- `fsdb-parse` builds direct C++ tools on top of `ffrAPI.h` and `libnffr.so`
- GTKWave documents FSDB builds against Verdi/VCS-provided headers and libs
- VaporView and waveform_mcp both use native wrapper approaches around the same
  reader stack

What public sources do **not** really prove by themselves is the exact lazy-load
shape needed by this RFC. This demo isolates that question while staying honest
about the remaining packaging/runtime proof still needed.

## Notes / limitations

- This is a feasibility probe, not product code.
- It only targets Linux.
- The native Verdi bridge is compiled only when `VERDI_HOME` is set.
- The repo also builds a tiny mock bridge so automated tests can validate the
  lazy-loading Rust/FFI path without requiring Verdi.
- `end-time-raw` is reported in the dump's native ticks; interpret it together
  with `scale-unit`.
