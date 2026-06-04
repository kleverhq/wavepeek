# FSDB Development

FSDB support is an optional native backend for `wavepeek`. Default builds remain VCD/FST-only and do not require Synopsys Verdi. FSDB-enabled builds are local Linux x86_64 builds that link against a user-provided Verdi FSDB Reader SDK.

## User and command contract

Build FSDB support explicitly:

```sh
VERDI_HOME=/path/to/verdi cargo install wavepeek --features fsdb
```

An FSDB-enabled binary keeps the same user-facing command surface as VCD/FST for supported data:

```sh
wavepeek info     --waves dump.fsdb --json
wavepeek scope    --waves dump.fsdb --max-depth 3
wavepeek signal   --waves dump.fsdb --scope top.u0 --recursive
wavepeek value    --waves dump.fsdb --scope top.u0 --signals clk,data --at 10ns
wavepeek change   --waves dump.fsdb --scope top.u0 --signals state --on 'posedge clk'
wavepeek property --waves dump.fsdb --scope top.u0 --on 'posedge clk' --eval 'valid && ready'
```

Supported FSDB command coverage is digital bit-vector/integral data for `info`, `scope`, `signal`, `value`, `change`, and `property`. Real and string FSDB value decoding are not supported; commands that need those values fail with the existing `signal` error category.

A default binary that sees FSDB-looking input reports that FSDB support requires rebuilding with the `fsdb` Cargo feature and a local Verdi SDK.

## Verdi SDK contract

`VERDI_HOME` must point at a local licensed Verdi installation containing the FSDB Reader SDK. The build and helper checks expect at least:

```text
$VERDI_HOME/share/FsdbReader/ffrAPI.h
$VERDI_HOME/share/FsdbReader/ffrKit.h
$VERDI_HOME/share/FsdbReader/fsdbShr.h
$VERDI_HOME/share/FsdbReader/<abi>/libnffr.so
$VERDI_HOME/share/FsdbReader/<abi>/libnsys.so
```

The default Reader ABI directory is `linux64`. Override it only for local SDK/toolchain compatibility:

- `WAVEPEEK_FSDB_ABI=<name>` selects `$VERDI_HOME/share/FsdbReader/<name>`; for example `linux64_gcc950`.
- `WAVEPEEK_FSDB_READER_LIBDIR=<path>` selects an explicit Reader library directory.

`WAVEPEEK_FSDB_READER_LIBDIR` changes the library directory only; `VERDI_HOME` is still required for headers.

By default, `build.rs` embeds the selected Reader library directory as an ELF rpath/RUNPATH. This makes local FSDB binaries runnable without extra `LD_LIBRARY_PATH` setup on the build machine. Set `WAVEPEEK_FSDB_EMBED_RPATH=0` only for packaging or loader environments that provide the Verdi libraries another way.

## Devcontainer behavior

The devcontainer sets `VERDI_HOME=/opt/verdi`. Before container startup, `.devcontainer/initialize.sh` prepares `~/.cache/wavepeek/verdi` from the host `VERDI_HOME`; the container bind-mounts that path at `/opt/verdi`. An empty `/opt/verdi` means Verdi is unavailable, not broken.

Use the helper probe to distinguish available, skipped, and broken states:

```sh
just check-fsdb-env
python3 -B tools/fsdb/check_fsdb_env.py --require
```

The devcontainer also exposes selected Verdi FSDB utilities on `PATH` through `.devcontainer/verdi-tool-wrapper.sh`, including tools such as `vcd2fsdb`, `fst2vcd`, `fsdb2vcd`, `fsdbdebug`, and `fsdbextract`. Use those wrapper commands for local debugging and fixture conversion instead of hard-coding `$VERDI_HOME/bin/...` paths.

## Quality gates

The main gates are FSDB-aware but not Verdi-mandatory:

- `just lint` runs `just lint-fsdb` when Verdi is available.
- `just check` runs `just check-fsdb-build` when Verdi is available.
- `just test` and `just ci` run `just test-fsdb` when Verdi is available.
- `just bench-e2e-smoke-commit` runs `just bench-e2e-fsdb-smoke-commit` when Verdi is available.

When Verdi is absent, the wrapper prints the `skip: fsdb: ...` message from `tools/fsdb/check_fsdb_env.py` and continues. If Verdi-related environment variables are set but inconsistent, the gates fail instead of silently skipping.

Focused FSDB recipes:

```sh
just check-fsdb-env
just lint-fsdb
just check-fsdb-build
just prepare-fsdb-fixtures
just test-fsdb
just bench-e2e-fsdb-smoke-commit
```

## Fixtures and benchmarks

Do not commit generated `.fsdb` fixtures. `just prepare-fsdb-fixtures` creates ignored FSDB files from two sources:

- committed hand-written VCD fixtures under `tests/fixtures/hand/`, written to `tests/fixtures/fsdb/`;
- RTL `.fst` artifacts under `RTL_ARTIFACTS_DIR`, written as neighboring ignored `.fsdb` files for benchmark parity.

`bench/e2e/tests_fsdb.json` is generated from `bench/e2e/tests.json` by replacing RTL artifact `.fst` paths with `.fsdb` paths. Update the FST catalog first, then run:

```sh
just update-bench-e2e-fsdb-catalog
just check-bench-e2e-fsdb-catalog
```

FSDB benchmark baselines live under `bench/e2e/runs/baseline_fsdb/`. Refresh them only in a Verdi-equipped environment with the dedicated FSDB benchmark recipes.

## Repository safety policy

The public repository may contain project-owned source that optionally integrates with local Verdi headers and libraries. It must not contain proprietary Verdi payloads or derived copies.

Do not commit:

- Verdi headers, libraries, installed documentation, or excerpts from installed documentation;
- generated bindings or copied declarations derived from proprietary headers;
- generated `.fsdb` fixtures unless redistribution rights are explicit and documented;
- converter logs or debug dumps that expose proprietary design contents.

Treat `.fst` and `.fsdb` waveform dumps as binary data. Inspect them through `wavepeek`, Verdi tools, fixture helpers, or binary-safe metadata commands such as `find`, `stat`, `ls`, `du`, and checksums.

## Local Verdi research surface

If deeper local SDK research is needed, keep it local and cite only paths or public behavior in repository docs. Useful starting points inside a licensed installation are:

```text
$VERDI_HOME/share/FsdbReader/
$VERDI_HOME/doc/HTML/index.html
$VERDI_HOME/doc/HTML/pdf/verdi_fsdb_reader.pdf
```

Do not copy local manual text, headers, examples, or generated API extracts into the repository. Summarize project decisions in this file, source comments, or public docs without reproducing proprietary material.
