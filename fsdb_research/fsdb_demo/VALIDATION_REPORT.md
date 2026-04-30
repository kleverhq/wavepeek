# FSDB Demo Validation Report

## 1. Outcome Summary

- Report date: 2026-04-30T12:39:33+00:00
- Runner: OpenCode / gpt-5.5
- Verdict: strong positive evidence for the standalone demo on this host
- Short summary:
- The demo rebuilt successfully against both bundled Verdi installs: `T-2022.06` and `X-2025.06`.
- The Rust `fsdb_demo` binary has no direct ELF dependency on Verdi reader or writer libraries.
- The lazy-loaded reader bridge depends on `libnffr.so` and `libnsys.so`, as intended.
- A separate FsdbWriter-based fixture generator produced real FSDB files, and the lazy reader bridge opened them successfully.
- Negative-control `noop` worked with `VERDI_HOME` and `LD_LIBRARY_PATH` unset.

## 2. Repository / Artifact Info

- Source folder used: `/workspaces/feat-fsdb/fsdb_research/fsdb_demo`
- Git snapshot: `5a57591` plus local uncommitted demo changes for Verdi API compatibility and the FsdbWriter fixture generator
- Was the demo rebuilt on the validation host: yes
- Build layout note: validation used separate target directories so both Verdi versions could be tested from one source tree.
- `T-2022.06` target dir: `target/verdi-T-2022.06`
- `X-2025.06` target dir: `target/verdi-X-2025.06`

## 3. Host / Toolchain / Verdi Matrix

- Host label: `kamino`
- Linux distro: Ubuntu 24.04.3 LTS
- Kernel / architecture: `Linux kamino 6.11.0-26-generic #26~24.04.1-Ubuntu SMP PREEMPT_DYNAMIC Thu Apr 17 19:20:47 UTC 2 x86_64 x86_64 x86_64 GNU/Linux`
- `rustc --version`: `rustc 1.93.0 (254b59607 2026-01-19)`
- `cargo --version`: `cargo 1.93.0 (083ac5135 2025-12-15)`
- `c++ --version`: `c++ (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0`
- glibc / loader: `ldd (Ubuntu GLIBC 2.39-0ubuntu8.7) 2.39`
- `LD_LIBRARY_PATH`: unset / empty during negative-control and probe commands
- Extra flags used: none
- `FSDB_DEMO_EXTRA_CXXFLAGS`: unset
- `FSDB_DEMO_EXTRA_LDFLAGS`: unset

### Verdi Versions

- Version: `T-2022.06`
- `VERDI_HOME`: `/workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06`
- Reader files found: `share/FsdbReader/ffrAPI.h`, `share/FsdbReader/linux64/libnffr.so`, `share/FsdbReader/linux64/libnsys.so`
- Writer files found: `share/FsdbWriter/ffwAPI.h`, `share/FsdbWriter/linux64/libnffw.so`

- Version: `X-2025.06`
- `VERDI_HOME`: `/workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06`
- Reader files found: `share/FsdbReader/ffrAPI.h`, `share/FsdbReader/linux64/libnffr.so`, `share/FsdbReader/linux64/libnsys.so`
- Writer files found: `share/FsdbWriter/ffwAPI.h`, `share/FsdbWriter/linux64/libnffw.so`

## 4. Input Files

- Original external readable `.fsdb`: not present under `fsdb_research`.
- Validation workaround: generated tiny real FSDB files using the bundled Verdi `FsdbWriter` API.
- Generator source: `native/fsdb_fixture_writer.cpp`
- `T-2022.06` FSDB path: `target/verdi-T-2022.06/validation.fsdb`
- `T-2022.06` FSDB size: `7945 bytes`
- `X-2025.06` FSDB path: `target/verdi-X-2025.06/validation.fsdb`
- `X-2025.06` FSDB size: `7960 bytes`
- Fixture content: one Verilog scope `top`, one 1-bit `clk` signal, value changes at raw times `0`, `10`, `20`, `30`, scale unit `1ps`.

## 5. Build Step

### T-2022.06

Command:

```bash
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06" CARGO_TARGET_DIR="target/verdi-T-2022.06" cargo build
```

Status: pass

Important output:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
```

### X-2025.06

Command:

```bash
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06" CARGO_TARGET_DIR="target/verdi-X-2025.06" cargo build
```

Status: pass

Important output:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
```

## 6. build-info Output

### T-2022.06

Command:

```bash
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06" CARGO_TARGET_DIR="target/verdi-T-2022.06" cargo run -- build-info
```

Output:

```text
mock-bridge-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-T-2022.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_mock_bridge.so
verdi-bridge-status: built
verdi-bridge-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-T-2022.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_verdi_bridge.so
fsdb-writer-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-T-2022.06/debug/build/fsdb_demo-136084d13bcdfadf/out/fsdb_fixture_writer
verdi-home: /workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06
```

### X-2025.06

Command:

```bash
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06" CARGO_TARGET_DIR="target/verdi-X-2025.06" cargo run -- build-info
```

Output:

```text
mock-bridge-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-X-2025.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_mock_bridge.so
verdi-bridge-status: built
verdi-bridge-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-X-2025.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_verdi_bridge.so
fsdb-writer-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-X-2025.06/debug/build/fsdb_demo-136084d13bcdfadf/out/fsdb_fixture_writer
verdi-home: /workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06
```

## 7. Linkage Isolation Checks

### Main Binary Linkage

Command:

```bash
ldd target/verdi-T-2022.06/debug/fsdb_demo
ldd target/verdi-X-2025.06/debug/fsdb_demo
```

T-2022.06 output:

```text
linux-vdso.so.1
libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1
libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
/lib64/ld-linux-x86-64.so.2
```

X-2025.06 output:

```text
linux-vdso.so.1
libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1
libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
/lib64/ld-linux-x86-64.so.2
```

Does main binary mention `libnffr.so`, `libnsys.so`, or `libnffw.so`: no

### Main Binary Dynamic Section

Command:

```bash
readelf -d target/verdi-T-2022.06/debug/fsdb_demo
readelf -d target/verdi-X-2025.06/debug/fsdb_demo
```

Observed `NEEDED` entries for both builds:

```text
Shared library: [libgcc_s.so.1]
Shared library: [libc.so.6]
Shared library: [ld-linux-x86-64.so.2]
```

Does main binary record Verdi libraries as `NEEDED`: no

### Reader Bridge Linkage

Command:

```bash
ldd target/verdi-T-2022.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_verdi_bridge.so
ldd target/verdi-X-2025.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_verdi_bridge.so
```

T-2022.06 relevant output:

```text
libnffr.so => /workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06/share/FsdbReader/linux64/libnffr.so
libnsys.so => /workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06/share/FsdbReader/linux64/libnsys.so
```

X-2025.06 relevant output:

```text
libnffr.so => /workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06/share/FsdbReader/linux64/libnffr.so
libnsys.so => /workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06/share/FsdbReader/linux64/libnsys.so
```

Does the bridge mention `libnffr.so` / `libnsys.so`: yes

### Writer Tool Linkage

Command:

```bash
ldd target/verdi-T-2022.06/debug/build/fsdb_demo-136084d13bcdfadf/out/fsdb_fixture_writer
ldd target/verdi-X-2025.06/debug/build/fsdb_demo-136084d13bcdfadf/out/fsdb_fixture_writer
```

T-2022.06 relevant output:

```text
libnffw.so => /workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06/share/FsdbWriter/linux64/libnffw.so
```

X-2025.06 relevant output:

```text
libnffw.so => /workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06/share/FsdbWriter/linux64/libnffw.so
```

Linkage interpretation: pass. Verdi reader and writer dependencies are outside the Rust main binary.

## 8. Non-FSDB Smoke Path

### T-2022.06

Command:

```bash
env -u VERDI_HOME -u LD_LIBRARY_PATH target/verdi-T-2022.06/debug/fsdb_demo noop
```

Status: pass

Output:

```text
command: noop
status: ok
verdi-bridge-status: built
```

### X-2025.06

Command:

```bash
env -u VERDI_HOME -u LD_LIBRARY_PATH target/verdi-X-2025.06/debug/fsdb_demo noop
```

Status: pass

Output:

```text
command: noop
status: ok
verdi-bridge-status: built
```

Interpretation: pass. The binary starts and executes a non-FSDB command without loading the Verdi bridge.

## 9. Real FSDB Probe Path

### Generate FSDB Inputs

Commands:

```bash
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06" CARGO_TARGET_DIR="target/verdi-T-2022.06" cargo run -- generate --out target/verdi-T-2022.06/validation.fsdb
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06" CARGO_TARGET_DIR="target/verdi-X-2025.06" cargo run -- generate --out target/verdi-X-2025.06/validation.fsdb
```

T-2022.06 output:

```text
writer-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-T-2022.06/debug/build/fsdb_demo-136084d13bcdfadf/out/fsdb_fixture_writer
output-path: target/verdi-T-2022.06/validation.fsdb
status: ok
```

X-2025.06 output:

```text
writer-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-X-2025.06/debug/build/fsdb_demo-136084d13bcdfadf/out/fsdb_fixture_writer
output-path: target/verdi-X-2025.06/validation.fsdb
status: ok
```

### Probe Matching-Version FSDBs

Commands:

```bash
env -u VERDI_HOME -u LD_LIBRARY_PATH target/verdi-T-2022.06/debug/fsdb_demo probe --waves target/verdi-T-2022.06/validation.fsdb
env -u VERDI_HOME -u LD_LIBRARY_PATH target/verdi-X-2025.06/debug/fsdb_demo probe --waves target/verdi-X-2025.06/validation.fsdb
```

T-2022.06 output:

```text
FSDB Reader, Release Verdi_T-2022.06, RH Linux x86_64/64bit, 04/30/2026
(C) 1996 - 2026 by Synopsys, Inc.
logDir = /tmp/
bridge-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-T-2022.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_verdi_bridge.so
bridge-kind: verdi
waveform-path: target/verdi-T-2022.06/validation.fsdb
signal-count: 1
end-time-raw: 30
scale-unit: 1ps
message: opened FSDB and traversed hierarchy
```

X-2025.06 output:

```text
FSDB Reader, Release Verdi_X-2025.06, RH Linux x86_64/64bit, 04/30/2026
(C) 1996 - 2026 by Synopsys, Inc.
logDir = /tmp/
bridge-path: /workspaces/feat-fsdb/fsdb_research/fsdb_demo/target/verdi-X-2025.06/debug/build/fsdb_demo-136084d13bcdfadf/out/libfsdb_verdi_bridge.so
bridge-kind: verdi
waveform-path: target/verdi-X-2025.06/validation.fsdb
signal-count: 1
end-time-raw: 30
scale-unit: 1ps
message: opened FSDB and traversed hierarchy
```

Status: pass for both matching-version probes.

## 10. RFC Negative Control

Negative-control environment description:

```text
The already-built Rust binary was run directly with VERDI_HOME and LD_LIBRARY_PATH removed from the environment. No Verdi path was provided through the loader environment. The Verdi install directories still existed on disk because this validation used the local bundled vendor trees; the check therefore proves startup isolation for this binary, not behavior after physically removing the Verdi tree.
```

Commands:

```bash
env -u VERDI_HOME -u LD_LIBRARY_PATH target/verdi-T-2022.06/debug/fsdb_demo noop
env -u VERDI_HOME -u LD_LIBRARY_PATH target/verdi-X-2025.06/debug/fsdb_demo noop
```

Status: pass for both builds

Output:

```text
command: noop
status: ok
verdi-bridge-status: built
```

Interpretation: pass. The main binary does not require Verdi runtime libraries at process startup.

## 11. Deviations / Workarounds

- The original validation prerequisite expected one readable external `.fsdb`; none was present under `fsdb_research`.
- A minimal FsdbWriter generator was added to create a real FSDB fixture locally.
- Real Verdi reader headers required a small bridge source compatibility fix: no `TRUE` / `FALSE` macros and `ffrIsFSDB` / `ffrOpen3` accept mutable `str_T` (`char *`).
- Separate `CARGO_TARGET_DIR` values were used for the two Verdi versions.
- No custom compiler path, ABI flags, symlinks, copied libraries, or manual loader environment were required.

## 12. Cross-Version Compatibility Observation

- `X-2025.06` reader can open the FSDB generated by `T-2022.06`, with a warning that the file was generated using a previous version.
- `T-2022.06` reader cannot open the FSDB generated by `X-2025.06`; it reports that FSDB file version `6.3` is higher than reader version `6.1`, then `ffrOpen3` returns null.

## 13. Test Commands

Commands:

```bash
cargo fmt --check
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/T-2022.06" CARGO_TARGET_DIR="target/verdi-T-2022.06" cargo test
VERDI_HOME="/workspaces/feat-fsdb/fsdb_research/verdi/X-2025.06" CARGO_TARGET_DIR="target/verdi-X-2025.06" cargo test
```

Status: pass

Observed result:

```text
4 unit tests passed, 4 CLI tests passed, and doc tests passed for both Verdi target directories.
```

## 14. Final Classification

- Classification: strong positive evidence
- Why: build, linkage isolation, non-FSDB startup, real FSDB generation, real FSDB lazy probe, and negative-control `noop` all passed for both tested Verdi versions.
- Remaining RFC caveat: this validates the standalone demo only. The final product acceptance still requires repeating equivalent runtime proof with a real FSDB-enabled `wavepeek` build and a non-FSDB `wavepeek` command while Verdi runtime libraries are unavailable at startup.
