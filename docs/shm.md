# SHM development

The optional Cargo feature `shm = ["ondas/shm-lib"]` reads complete SHM database
directories through Ondas and a local Xcelium reader. Default builds remain
VCD/FST-only. Native commands retain their existing arguments and output formats.
SHM is unavailable in the browser.

## Installation and runtime

Building requires Linux x86_64 GNU, a C++17 compiler, binutils, zlib development
files, and `XCELIUM_HOME` pointing to a compatible Xcelium installation (2021+).
Ondas selects `tools/lib/64bit/libsst2.so` and validates its reader interface.
Keep that installation available when running the binary. A library with SONAME
`libsst2.so` also needs explicit runtime lookup. `tools/shm/with_shm_env.sh` exposes
only the selected reader through `tmp/shm-runtime/`, avoiding Xcelium's bundled
C++ runtime. See the packaged Quickstart for source installation and loader setup.

Configure installations and any converter license in an ignored private
devcontainer profile. The public image contains no Cadence payloads. Conversion
requires the 64-bit `simvisdbutil` launcher and a configured Cadence license;
reading uses the Ondas reader directly.

## Inputs and limits

Pass the complete database directory to `--waves`, including its `.dsn` and `.trn`
components. Individual components are unsupported. Recognition accepts `.shm`
case-insensitively and complete databases without that suffix.

Bit values, real/string expressions, and event occurrences use the shared engine.
Ondas does not provide SHM signedness or enumeration labels. Converted VCD inputs
can expose generic `bit_vector` declaration kinds; value and query semantics are
tested separately from that metadata difference. Converted recordings can also
report an earlier start bound, affecting initial `change` rows. Recording gaps and malformed
components produce file errors.

## Recipes

Run from the host:

```sh
./dev just check-shm-env
./dev just check-shm-build
./dev just test-shm
./dev just bench-e2e-shm-smoke
./dev just bench-e2e-shm-run
```

Unset `XCELIUM_HOME` skips conditional SHM gates. A configured missing library is
an error. Ondas owns build-time ABI checks; the build gate also launches the binary
outside Cargo. `lint`, `check`, `test`, and `ci` run their SHM gates when
the installation is configured. The smoke recipe requires the SDK. See
[quality.md](quality.md) for the pre-PR checks. Combined builds require the
SDK for each enabled native format.

`prepare-shm-test-fixtures` converts a small set of existing VCD fixtures into
ignored `tests/fixtures/shm/`. Short native recordings from `shm_rich.sv` and
`escaped_instances.sv` cover typed values and escaped instance names.
`prepare-shm-fixtures [test-name-regex]` prepares
only the selected benchmark inputs. FST benchmarks use an existing current VCD
or a temporary FST-to-VCD conversion, then convert to SHM outside timed queries.
Preparation validates DSN/TRN output and source/converter stamps before reuse.
It refuses to replace unowned databases and preserves existing output on failure.

`update-bench-e2e-shm-catalog` and `check-bench-e2e-shm-catalog` maintain
`bench/e2e/tests_shm.json` from the FST workload catalog. The focused SHM benchmark
recipe uses the existing E2E harness; multi-ref capture remains unchanged.

Do not commit generated SHM databases, vendor libraries, licenses, headers, or
installed documentation. Keep disposable conversion logs and results under `tmp/`.
