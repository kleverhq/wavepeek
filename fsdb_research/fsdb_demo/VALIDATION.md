# FSDB Demo Validation

Run this from inside `fsdb_demo/` on a licensed Linux host with Verdi.

After running it, fill `REPORT_TEMPLATE.md` so the results can be reviewed by a
human, another agent, or sent back here for analysis.

## 1. Prerequisites

You need:

- `VERDI_HOME=/path/to/verdi`
- Rust/Cargo
- `c++` or `g++`
- one readable `.fsdb`
- `ldd` and `readelf`

Expected Verdi files:

- `share/FsdbReader/ffrAPI.h`
- `share/FsdbReader/linux64/libnffr.so`
- `share/FsdbReader/linux64/libnsys.so`

## 2. Build

```bash
export VERDI_HOME=/path/to/verdi
cargo build
```

Interpretation:

- pass: the public-source bridge can be compiled against local FsdbReader
- fail: this is usually toolchain / ABI / include-path friction, not yet proof that lazy loading is impossible

If needed, retry with:

```bash
export FSDB_DEMO_EXTRA_CXXFLAGS='...'
export FSDB_DEMO_EXTRA_LDFLAGS='...'
```

## 3. Inspect what was built

```bash
cargo run -- build-info
```

Interpretation:

- `verdi-bridge-status: built` means the real bridge was compiled
- `verdi-bridge-path: ...` gives the `.so` path to inspect below
- `skipped-no-verdi-home` means you forgot `VERDI_HOME`

## 4. Check linkage isolation

```bash
ldd target/debug/fsdb_demo
readelf -d target/debug/fsdb_demo | grep NEEDED
ldd <verdi-bridge-path-from-build-info>
```

Desired result:

- `target/debug/fsdb_demo` should **not** depend on `libnffr.so` or `libnsys.so`
- the Verdi bridge `.so` **should** depend on them

Interpretation:

- if only the bridge depends on Verdi libs: good evidence for lazy-load isolation
- if the main binary depends on Verdi libs: this is a failure for the RFC hypothesis in its current form

## 5. Prove non-FSDB startup path works

```bash
cargo run -- noop
```

Expected output shape:

```text
command: noop
status: ok
...
```

Interpretation:

- pass: the binary starts and executes a non-FSDB path without touching the bridge
- fail: the lazy-load boundary is suspect even before checking real FSDB access

## 6. Prove FSDB path works when invoked

```bash
cargo run -- probe --waves /path/to/file.fsdb
```

Expected result:

- success output containing at least:
  - `bridge-kind: verdi`
  - `signal-count: ...`
  - `scale-unit: ...`
  - `end-time-raw: ...`

Interpretation:

- pass: lazy runtime loading worked and FsdbReader could open the file
- fail with bridge/runtime error: loader or Verdi runtime problem
- fail with file-recognition/open error: likely file issue or FsdbReader compatibility issue

## 7. Run the RFC negative control

This is the important one.

Take the already-built binary and run it in an environment where Verdi runtime
libraries are **not** resolvable at startup.

Examples:

- a clean shell without the relevant loader setup
- a container/host where the Verdi runtime path is absent from loader resolution

Then run:

```bash
./target/debug/fsdb_demo noop
```

Interpretation:

- pass: strongest evidence that the main binary itself does not require Verdi at startup
- fail before command execution because `libnffr.so`/`libnsys.so` cannot be resolved: the RFC lazy-loading requirement is not met

## 8. How to interpret the overall outcome

### Strong positive evidence

All of these are true:

1. build succeeds
2. main binary has no direct Verdi dependency in `ldd`/`readelf`
3. `noop` works normally
4. `probe --waves ...` works on a real FSDB
5. negative-control `noop` still works when Verdi runtime is unresolved

Conclusion: the lazy-loading hypothesis looks viable on this validated host/toolchain/Verdi combination.

Important: this is still **feasibility evidence for the demo**, not full RFC sign-off. The RFC's final acceptance bar is stricter: after `wavepeek` grows a real `fsdb` feature, the equivalent runtime proof must be repeated with an FSDB-enabled `wavepeek` binary running a real VCD/FST command while Verdi runtime libraries are unavailable.

### Partial evidence only

If 1-4 pass but step 5 was not run, then you have useful progress, but **not yet** the full RFC proof.

### Negative evidence

If the main binary itself links against Verdi runtime libs, or if the negative-control `noop` fails before execution, then the current approach does not satisfy the RFC gating requirement.

## 9. Notes

- `end-time-raw` is in the dump's native ticks, not normalized picoseconds.
- This demo is best used as a copied source tree rebuilt on the licensed host.
- A copied prebuilt binary+bridge is only a partial experiment because the bridge still carries build-host loader assumptions unless you intentionally manage runtime loader configuration.
