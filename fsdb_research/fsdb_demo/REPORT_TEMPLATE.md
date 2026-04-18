# FSDB Demo Validation Report Template

Use this template after running `VALIDATION.md`.

Goal: give enough information for both:

- a human reviewer
- another agent
- ChatGPT / OpenCode follow-up analysis

If a step fails, keep the partial report and include the exact error/output.

---

## 1. Outcome Summary

- Report date:
- Runner:
- Verdict:
  - [ ] strong positive evidence
  - [ ] partial evidence only
  - [ ] negative evidence
  - [ ] inconclusive
- Short summary in 3-5 bullets:
  -
  -
  -

## 2. Repository / Artifact Info

- Source folder used:
- Git commit / archive version / snapshot identifier:
- Was the demo rebuilt on the licensed host?
  - [ ] yes
  - [ ] no
- If not rebuilt, explain how the binary/bridge were copied:

## 3. Host / Toolchain / Verdi Matrix

- Hostname or host label:
- Linux distro / version:
- Kernel:
- CPU architecture:
- `rustc --version`:
- `cargo --version`:
- `c++ --version` or `g++ --version`:
- Verdi version:
- `VERDI_HOME`:
- License environment / assumptions:
- Native runtime / stdlib notes (glibc, libstdc++, toolset provenance, etc.):
- Extra flags used:
  - `FSDB_DEMO_EXTRA_CXXFLAGS=`
  - `FSDB_DEMO_EXTRA_LDFLAGS=`
- Loader-related environment relevant to the run:
  - `LD_LIBRARY_PATH=`
  - other notes:

## 4. Input File

- FSDB file path:
- Approximate file size:
- Any known notes about the file:
  - simulator/source:
  - expected timescale if known:
  - anything unusual:

## 5. Build Step

Command run:

```bash
export VERDI_HOME=...
cargo build
```

Status:

- [ ] pass
- [ ] fail

Important output:

```text
PASTE BUILD OUTPUT HERE
```

Interpretation / notes:

-

## 6. build-info Output

Command:

```bash
cargo run -- build-info
```

Output:

```text
PASTE build-info OUTPUT HERE
```

Extracted key fields:

- `verdi-bridge-status:`
- `verdi-bridge-path:`
- `mock-bridge-path:`

## 7. Linkage Isolation Checks

Commands:

```bash
ldd target/debug/fsdb_demo
readelf -d target/debug/fsdb_demo | grep NEEDED
ldd <verdi-bridge-path>
```

### 7.1 `ldd target/debug/fsdb_demo`

```text
PASTE OUTPUT HERE
```

Does main binary mention `libnffr.so` or `libnsys.so`?

- [ ] no
- [ ] yes

### 7.2 `readelf -d target/debug/fsdb_demo | grep NEEDED`

```text
PASTE OUTPUT HERE
```

Does main binary record Verdi libraries as `NEEDED`?

- [ ] no
- [ ] yes

### 7.3 `ldd <verdi-bridge-path>`

```text
PASTE OUTPUT HERE
```

Does the bridge mention `libnffr.so` / `libnsys.so`?

- [ ] yes
- [ ] no

Linkage interpretation:

-

## 8. Non-FSDB Smoke Path

Command:

```bash
cargo run -- noop
```

Status:

- [ ] pass
- [ ] fail

Output:

```text
PASTE noop OUTPUT HERE
```

Interpretation:

-

## 9. Real FSDB Probe Path

Command:

```bash
cargo run -- probe --waves /path/to/file.fsdb
```

Status:

- [ ] pass
- [ ] fail

Output:

```text
PASTE probe OUTPUT HERE
```

Observed key fields if successful:

- `bridge-kind:`
- `signal-count:`
- `scale-unit:`
- `end-time-raw:`

Interpretation:

-

## 10. RFC Negative Control

Describe how you created an environment where Verdi runtime libraries were not
resolvable at startup.

Example options:

- clean shell
- separate container
- removed loader path
- unset `LD_LIBRARY_PATH`
- copied binary to another host without Verdi runtime resolution

Negative-control environment description:

```text
DESCRIBE EXACTLY WHAT YOU CHANGED HERE
```

Command:

```bash
./target/debug/fsdb_demo noop
```

Status:

- [ ] pass
- [ ] fail
- [ ] not run

Output:

```text
PASTE NEGATIVE-CONTROL OUTPUT HERE
```

If it failed before command execution, paste the exact loader error.

Interpretation:

-

## 11. Deviations / Workarounds

List anything non-default that was required:

- custom compiler path
- ABI flags
- symlinks
- copied libraries
- modified loader environment
- older GCC workaround
- manual edits

Details:

-

## 12. Final Classification

Choose one:

- [ ] strong positive evidence
- [ ] partial evidence only
- [ ] negative evidence
- [ ] inconclusive

Why:

-

## 13. Reviewer Questions / Open Problems

Questions for the reviewing agent or for follow-up analysis:

1.
2.
3.

## 14. Minimum Data Required for Remote Analysis

If you do not want to send the full report, at minimum send these items:

1. host/toolchain/Verdi versions
2. full `cargo build` result if build failed
3. full `cargo run -- build-info` output
4. full `ldd target/debug/fsdb_demo` output
5. full `readelf -d target/debug/fsdb_demo | grep NEEDED` output
6. full `ldd <verdi-bridge-path>` output
7. full `cargo run -- noop` output
8. full `cargo run -- probe --waves ...` output
9. exact negative-control setup description
10. full negative-control `./target/debug/fsdb_demo noop` output

Without those items, external analysis will likely be incomplete or ambiguous.
