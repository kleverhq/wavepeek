# `$VERDI_HOME` map for FSDB work in wavepeek

Review date: 2026-05-20  
`$VERDI_HOME`: `/opt/verdi`

> Important: Verdi/Synopsys content is proprietary. Do not copy headers, libraries, docs, or generated excerpts into this repository. Treat `/opt/verdi` as a local SDK/documentation installation. Treat `.fsdb` files as binary; do not read them with text tools.

## Summary

- Total size: about `5.8G`.
- Main branches:
  - `$VERDI_HOME/doc` — about `1.1G`, HTML/PDF documentation plus `*.txt.gz` text extractions useful for searching.
  - `$VERDI_HOME/share` — about `4.7G`, SDK/API files, libraries, examples, PLI/NPI/VIA tools.
- For **FSDB reading** in wavepeek, the primary candidate is `$VERDI_HOME/share/FsdbReader` (`ffrAPI.h`, `libnffr.so`, examples).
- For **FSDB writing**, use `$VERDI_HOME/share/FsdbWriter` (`ffwAPI.h`, `libnffw.so`).
- For higher-level access or Verdi integration, inspect `$VERDI_HOME/share/NPI`.

## Documentation: where to look

### Entry points

| Path | Purpose |
|---|---|
| `$VERDI_HOME/doc/HTML/index.html` | Main HTML documentation entry point. |
| `$VERDI_HOME/doc/HTML/search.html` | Local HTML docs search page. |
| `$VERDI_HOME/doc/HTML/sitemap.xml` | Machine-readable list of HTML topics. |
| `$VERDI_HOME/doc/bookshelf.pdf` | Overall documentation bookshelf. |
| `$VERDI_HOME/doc/HTML/pdf/` | PDF versions of most HTML docs. |
| `$VERDI_HOME/doc/.*.txt.gz` | Text extractions of PDFs; useful with `zgrep` and less noisy than HTML. |

### Most useful FSDB-related docs

| Topic | HTML | PDF |
|---|---|---|
| FSDB Reader API | `$VERDI_HOME/doc/HTML/verdi_fsdb_reader/` | `$VERDI_HOME/doc/HTML/pdf/verdi_fsdb_reader.pdf` |
| FSDB Reader summary | `$VERDI_HOME/doc/HTML/verdi_fsdb_reader_summary/` | `$VERDI_HOME/doc/HTML/pdf/verdi_fsdb_reader_summary.pdf` |
| FSDB Writer API | `$VERDI_HOME/doc/HTML/verdi_fsdb_writer/` | `$VERDI_HOME/doc/HTML/pdf/verdi_fsdb_writer.pdf` |
| FSDB Writer summary | `$VERDI_HOME/doc/HTML/verdi_fsdb_writer_summary/` | `$VERDI_HOME/doc/HTML/pdf/verdi_fsdb_writer_summary.pdf` |
| Linking/dumping with Synopsys simulators | `$VERDI_HOME/doc/HTML/verdi_linking_dumping/` | `$VERDI_HOME/doc/HTML/pdf/verdi_linking_dumping.pdf` |
| Linking/dumping with other simulators | `$VERDI_HOME/doc/HTML/verdi_linking_dumping_othersim/` | `$VERDI_HOME/doc/HTML/pdf/verdi_linking_dumping_othersim.pdf` |
| Verdi command reference | `$VERDI_HOME/doc/HTML/verdi_command_ref/` | `$VERDI_HOME/doc/HTML/pdf/verdi_command_ref.pdf` |
| Verdi Tcl reference | `$VERDI_HOME/doc/HTML/verdi_tcl/` | `$VERDI_HOME/doc/HTML/pdf/verdi_tcl.pdf` |
| VC Apps / NPI docs | `$VERDI_HOME/doc/HTML/verdi_vc_apps_npi/` | `$VERDI_HOME/doc/HTML/pdf/verdi_vc_apps_npi.pdf` |
| Python NPI waveform | `$VERDI_HOME/doc/HTML/verdi_python_npi_waveform/` | `$VERDI_HOME/doc/HTML/pdf/verdi_python_npi_waveform.pdf` |
| Python NPI waveform writer | `$VERDI_HOME/doc/HTML/verdi_python_npi_waveform_writer/` | `$VERDI_HOME/doc/HTML/pdf/verdi_python_npi_waveform_writer.pdf` |

FSDB-heavy text docs by `zgrep -i -c fsdb`:

- `doc/.verdi_vc_apps_npi.txt.gz` — ~12884 hits.
- `doc/.verdi_linking_dumping_othersim.txt.gz` — ~1947 hits.
- `doc/.verdi_linking_dumping.txt.gz` — ~1802 hits.
- `doc/.verdi_command_ref.txt.gz` — ~1669 hits.
- `doc/.verdi_fsdb_writer.txt.gz` — ~1549 hits.
- `doc/.verdi_fsdb_reader.txt.gz` — ~1106 hits.

Useful commands:

```sh
zgrep -i "ffrOpen" "$VERDI_HOME/doc/.verdi_fsdb_reader.txt.gz"
zgrep -i "fsdbDumpvars" "$VERDI_HOME/doc/.verdi_linking_dumping.txt.gz"
find "$VERDI_HOME/doc/HTML/verdi_fsdb_reader" -type f -name '*.html' | sort
```

## SDK/API and libraries

### 1. FSDB Reader API (`ffr*`) — primary candidate for wavepeek

Paths:

- `$VERDI_HOME/share/FsdbReader/`
- `$VERDI_HOME/share/FsdbReader_pure/`

Key files:

| Path | Purpose |
|---|---|
| `$VERDI_HOME/share/FsdbReader/ffrAPI.h` | Main C++ header for FSDB Reader API. |
| `$VERDI_HOME/share/FsdbReader/ffrKit.h` | Reader API types/utilities. |
| `$VERDI_HOME/share/FsdbReader/fsdbShr.h` | Shared FSDB types for reader/writer. |
| `$VERDI_HOME/share/FsdbReader/linux64/libnffr.so` | Linux x86_64 reader library. |
| `$VERDI_HOME/share/FsdbReader/linux64/libnsys.so` | Companion runtime library. |
| `$VERDI_HOME/share/FsdbReader/linux64_gcc950/libnffr.so` | Alternate build for GCC 9.5 ABI. |
| `$VERDI_HOME/share/FsdbReader/doc/verdi_fsdb_reader.pdf` | SDK-local copy of the reader PDF. |
| `$VERDI_HOME/share/FsdbReader/example/` | C++ API examples. |

Sizes: `FsdbReader` ~`131M`, `FsdbReader_pure` ~`38M`.

Examples in `$VERDI_HOME/share/FsdbReader/example/`:

- Basic: `read_verilog.cpp`, `read_vhdl.cpp`, `read_mix.cpp`, `read_analog.cpp`.
- Navigation/value changes: `time_based.cpp`, `jump_test.cpp`, `read_offset.cpp`.
- Metadata: `test_file_info.cpp`, `test_sig_arr.cpp`, `trvs_mda_cell.cpp`.
- Other: `transaction.cpp`, `free_api.cpp`, `non_shared_ffrobj.cpp`, `all_vcs_have_been_read.cpp`.

API concepts visible from headers/docs:

- File checks: `ffrObject::ffrIsFSDB`, `ffrGetFSDBInfo`.
- Opening: `ffrOpen`, `ffrOpen2`, `ffrOpen3`.
- Hierarchy: tree callback, `ffrReadScopeVarTree`, scope/var callback data.
- Signals: idcode-based access, variable info/type/bit size.
- Values: traverse handles, time-based traversal, view windows, signal value load/unload.
- Supports Verilog/VHDL/mixed/analog/transaction/MDA/string/enum/real/property/glitch/dumpoff-dumpon data.

For wavepeek: start with `verdi_fsdb_reader.pdf` plus `read_verilog.cpp`, `time_based.cpp`, and `test_file_info.cpp`. That is likely the shortest official path to opening an FSDB, building a hierarchy/signal index, and reading value changes.

### 2. FSDB Writer API (`ffw*`)

Path:

- `$VERDI_HOME/share/FsdbWriter/`

Key files:

| Path | Purpose |
|---|---|
| `$VERDI_HOME/share/FsdbWriter/ffwAPI.h` | Main C/C++ header for FSDB Writer API. |
| `$VERDI_HOME/share/FsdbWriter/fsdbShr.h` | Shared FSDB types. |
| `$VERDI_HOME/share/FsdbWriter/linux64/libnffw.so` | Linux writer library. |
| `$VERDI_HOME/share/FsdbWriter/example/` | C++ examples for writing different data types. |

Size: ~`9.8M`.

Examples include `scope.cpp`, `vc.cpp`, `memory.cpp`, `transaction_recording.cpp`, `real.cpp`, `string.cpp`, `vhdl.cpp`, `sv_dt.cpp`, `dumpoff.cpp`, `autoswf.cpp`, `bus.cpp`, `clk.cpp`, and `data_type.cpp`.

For wavepeek this is secondary if the goal is read-only FSDB support. It may still be useful for generating tiny test FSDB files via the official library, if licensing and environment allow it.

### 3. Transaction FSDB API

Path:

- `$VERDI_HOME/share/fsdbTrans_API/`

Key files:

| Path | Purpose |
|---|---|
| `$VERDI_HOME/share/fsdbTrans_API/inc/fsdbTrans_API.h` | C API for writing transaction debug FSDB. |
| `$VERDI_HOME/share/fsdbTrans_API/inc/sc_fsdbTrans_API.h` | SystemC wrapper. |
| `$VERDI_HOME/share/fsdbTrans_API/inc/scv_fsdbTrans_API.h` | SCV wrapper. |
| `$VERDI_HOME/share/fsdbTrans_API/lib/linux64/libFTAPI.so` | Linux transaction API library. |
| `$VERDI_HOME/share/fsdbTrans_API/lib/linux64/libvfs.so` | Runtime dependency. |
| `$VERDI_HOME/share/fsdbTrans_API/examples/basic/` | `capi_example.cpp`, `Makefile`, `README`. |

Size: ~`102M`.

This is not a general waveform reader. It is useful for understanding or creating transaction streams.

### 4. NPI / VC Apps APIs

Path:

- `$VERDI_HOME/share/NPI/`

Key files and directories:

| Path | Purpose |
|---|---|
| `$VERDI_HOME/share/NPI/inc/npi_fsdb.h` | NPI FSDB reader model header. |
| `$VERDI_HOME/share/NPI/inc/npi_fsdbw.h` | NPI FSDB writer model header. |
| `$VERDI_HOME/share/NPI/inc/npi_fsdb_trans.h` | Transaction FSDB model. |
| `$VERDI_HOME/share/NPI/inc/npi_fsdbw_trans.h` | Transaction writer model. |
| `$VERDI_HOME/share/NPI/lib/linux64/libNPI.so` | Main NPI library. |
| `$VERDI_HOME/share/NPI/lib/linux64/libNPI_base.so` | Base NPI library. |
| `$VERDI_HOME/share/NPI/lib/linux64/libnpiL1.so` | NPI L1 helper library. |
| `$VERDI_HOME/share/NPI/lib/linux64/libvfs.so` | Runtime dependency. |
| `$VERDI_HOME/share/NPI/python/pynpi/` | Python bindings/modules for NPI. |
| `$VERDI_HOME/share/NPI/L1/C/src/FsdbLib/` | C L1 helper source for FSDB library. |
| `$VERDI_HOME/share/NPI/L1/C/src/FsdbWLib/` | C L1 helper source for FSDB writer library. |
| `$VERDI_HOME/share/NPI/L1/SV/src/FsdbWLib_SV/` | SV-side writer helpers. |
| `$VERDI_HOME/share/NPI/L1/TCL/src/FSDB_L1/` | Tcl helpers. |

Size: ~`632M`.

NPI FSDB example directories:

- `$VERDI_HOME/share/NPI/example/via_examples/NPI_Libraries/FSDB_Library/`
  - 23 focused examples: `npi_fsdb_sig_value_at`, `npi_fsdb_sig_value_between`, `npi_fsdb_sig_vc_count`, `npi_fsdb_hier_tree_dump_scope`, `npi_fsdb_hier_tree_dump_sig`, `npi_fsdb_convert_time_in/out`, etc.
- `$VERDI_HOME/share/NPI/example/via_examples/NPI_Models/FSDB_Model/`
  - 37 model-level examples: `npi_fsdb_open`, `npi_fsdb_close`, `npi_fsdb_iter_top_scope`, `npi_fsdb_iter_sig`, `npi_fsdb_create_vct`, `npi_fsdb_goto_time`, `npi_fsdb_vct_value`, etc.
- `$VERDI_HOME/share/NPI/example/via_examples/NPI_Models/FSDB_Writer_Model/reader_example/`

For wavepeek: NPI may be conceptually higher-level, but it is a heavier dependency. Compare it against the direct `FsdbReader` path; direct FFR looks more compact for a CLI/library integration.

### 5. PLI / simulator dumping support

Path:

- `$VERDI_HOME/share/PLI/`

Size: ~`1.8G`.

Key locations:

| Path | Purpose |
|---|---|
| `$VERDI_HOME/share/PLI/README.PLI` | Simulator/platform/language matrix and list of `$fsdbDump*` tasks/procedures. |
| `$VERDI_HOME/share/PLI/VCS/linux64/` | VCS PLI libs: `libnovas.so`, `libNovasAPI.so`, `pli.a`, `novas.tab`, `verdi.tab`, `novas.vhd`. |
| `$VERDI_HOME/share/PLI/IUS/linux64/` | Cadence/IUS/Xcelium-style PLI/FMI/CFC libs: `libpli.so`, `libfmi.so`, `libcfc.so`, `fsdb_nc_mix.h`, `novas.vhd`. |
| `$VERDI_HOME/share/PLI/MODELSIM/linux64/` | ModelSim libs: `novas_fli.so`, `pli.a`, `libMTISC.so`, `novas.vhd`. |
| `$VERDI_HOME/share/PLI/lib/linux64/` | Many `libsscore_*` builds for VCS/Xcelium/ModelSim/SystemC/Z01X/FS versions. |
| `$VERDI_HOME/share/PLI/systemc/` | SystemC/SCV dumper headers/libs by version. |

From `README.PLI`: supported dumping commands include `$fsdbDumpvars`, `$fsdbDumpMDA`, `$fsdbDumpSVA`, `$fsdbDumpon`, `$fsdbDumpoff`, `$fsdbDumpfile`, `$fsdbSwitchDumpfile`, `$fsdbAutoSwitchDumpfile`, `$fsdbDumpflush`, and `$fsdbDumpFinish` for Verilog/VHDL/Tcl flows.

For wavepeek: PLI matters only if we need to create FSDB through a simulator or understand feature provenance. It is not the first layer for reading existing FSDB files.

### 6. VIA / ready-made FSDB utilities

Path:

- `$VERDI_HOME/share/VIA/`

Size: ~`542M`.

Useful FSDB-related tools:

- `$VERDI_HOME/share/VIA/Apps/Bin/fsdbSigQ.pl`
- `$VERDI_HOME/share/VIA/Apps/Bin/getFsdbScopeHier.pl`
- `$VERDI_HOME/share/VIA/Apps/Bin/signalExistCheckFSDB.pl`
- `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdbManip`
- `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdbReportMDA`
- `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdbReportVCX`
- `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdbReportForce`
- `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdbReportClkFreq`
- `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdbFreqDuty`
- `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdbMinPulse`
- `$VERDI_HOME/share/VIA/Apps/Bin/npi_digital_fsdb_cmp`
- `$VERDI_HOME/share/VIA/Apps/Bin/npi_analog_fsdb_cmp`

App sources/docs/examples live under:

- `$VERDI_HOME/share/VIA/Apps/FsdbInvestigation/`
- `$VERDI_HOME/share/VIA/demo/waveform/`

These are useful for black-box sanity checks: compare future wavepeek output against official utilities. Healthy paranoia is cheaper than debugging a proprietary waveform format by vibes.

### 7. Other potentially relevant `share` directories

| Path | Comment |
|---|---|
| `$VERDI_HOME/share/log2transdb/` | `libTransFsdbWriterAPI.*`, rules for AXI/CHI/CXL/etc.; log to transaction DB/FSDB conversion. |
| `$VERDI_HOME/share/pa_writer/` | Protocol analyzer writer templates/SV/C++ pieces. |
| `$VERDI_HOME/share/dbWriter/` | DB writer libs/includes; not a first candidate. |
| `$VERDI_HOME/share/libcmd/` | `libnclt`, `libnsys`; common command/runtime libs. |
| `$VERDI_HOME/share/novas_dnd/` | Drag-and-drop/common Novas libs; likely GUI support. |
| `$VERDI_HOME/share/verdi_perf/` | Performance examples; contains some FSDB samples. |
| `$VERDI_HOME/share/verdi_gcc/` | Bundled legacy C++/STL headers. |
| `$VERDI_HOME/share/python/` | Bundled Python environment/libs. |
| `$VERDI_HOME/share/jre`, `$VERDI_HOME/share/eclipse` | Docs/GUI runtime, not FSDB API. |

## Sample FSDB files

Found 73 `*.fsdb` files under `$VERDI_HOME/share`. Do not read them as text.

Main groups:

- 37 under `$VERDI_HOME/share/NPI/example/via_examples/NPI_Models/FSDB_Model/`
- 23 under `$VERDI_HOME/share/NPI/example/via_examples/NPI_Libraries/FSDB_Library/`
- 7 under `$VERDI_HOME/share/verdi_perf/perfExamples/`
- 2 under `$VERDI_HOME/share/VIA/demo/waveform/`
- 1 under `$VERDI_HOME/share/NPI/example/via_examples/NPI_Models/FSDB_Writer_Model/`
- 1 each under a few NPI miscellaneous demo design directories.

Use them only through official tools/API or binary-safe metadata commands (`file`, `ls`, `du`, checksums). If a future test needs a fixture, prefer generating a tiny fixture in a controlled script and keep licensing constraints in mind.

## Build/link notes observed

- Reader libs:
  - `$VERDI_HOME/share/FsdbReader/linux64/libnffr.so`
  - `$VERDI_HOME/share/FsdbReader/linux64/libnsys.so`
- Writer libs:
  - `$VERDI_HOME/share/FsdbWriter/linux64/libnffw.so`
- NPI libs:
  - `$VERDI_HOME/share/NPI/lib/linux64/libNPI.so`
  - `$VERDI_HOME/share/NPI/lib/linux64/libvfs.so`
- Transaction API libs:
  - `$VERDI_HOME/share/fsdbTrans_API/lib/linux64/libFTAPI.so`
  - `$VERDI_HOME/share/fsdbTrans_API/lib/linux64/libvfs.so`

`ldd` observations:

- `libnffr.so` and `libnffw.so` depend mainly on system `libpthread`, `libm`, and `libc`.
- `libNPI.so` additionally depends on local `$VERDI_HOME/share/NPI/lib/linux64/libvfs.so`.
- `libFTAPI.so` additionally depends on local `$VERDI_HOME/share/fsdbTrans_API/lib/linux64/libvfs.so`.

Makefiles/examples are partly old and contain internal Synopsys build paths. Treat them as recipes, not drop-in build files. Expect to adjust include paths, library paths, `LD_LIBRARY_PATH`, and ABI selection (`linux64` vs `linux64_gcc950`).

## Recommended reading order for the next agent

1. Read this map. Congratulations, you avoided the first swamp.
2. For reading FSDB:
   - `$VERDI_HOME/doc/HTML/pdf/verdi_fsdb_reader.pdf`
   - `$VERDI_HOME/share/FsdbReader/ffrAPI.h`
   - `$VERDI_HOME/share/FsdbReader/fsdbShr.h`
   - Examples: `read_verilog.cpp`, `time_based.cpp`, `test_file_info.cpp`, `read_analog.cpp`.
3. For integration tradeoffs:
   - Compare direct `FsdbReader` with `$VERDI_HOME/share/NPI/inc/npi_fsdb.h` plus NPI examples.
4. For simulator-produced FSDB and dump options:
   - `$VERDI_HOME/doc/HTML/pdf/verdi_linking_dumping.pdf`
   - `$VERDI_HOME/doc/HTML/pdf/verdi_linking_dumping_othersim.pdf`
   - `$VERDI_HOME/share/PLI/README.PLI`
5. For command-line validation tools:
   - Inspect `$VERDI_HOME/share/VIA/Apps/FsdbInvestigation/` and `$VERDI_HOME/share/VIA/Apps/Bin/npiFsdb*`.

## Search recipes

```sh
# Fast text search in extracted PDF text
zgrep -i "view window" "$VERDI_HOME/doc/.verdi_fsdb_reader.txt.gz"
zgrep -i "npi_fsdb_open" "$VERDI_HOME/doc/.verdi_vc_apps_npi.txt.gz"
zgrep -i "fsdbDumpMDA" "$VERDI_HOME/doc/.verdi_linking_dumping.txt.gz"

# List reader/writer example files
find "$VERDI_HOME/share/FsdbReader/example" -maxdepth 1 -type f -printf '%f\n' | sort
find "$VERDI_HOME/share/FsdbWriter/example" -maxdepth 1 -type f -printf '%f\n' | sort

# List NPI FSDB examples
find "$VERDI_HOME/share/NPI/example/via_examples" -path '*FSDB*' -maxdepth 8 -type d | sort

# Binary-safe sample inventory
find "$VERDI_HOME" -type f -iname '*.fsdb' -printf '%p %s\n' | sort
```

## Caveats / open questions

- The FSDB on-disk format is not documented here as an open specification. Official access is through proprietary APIs. A native parser would be a reverse-engineering project: expensive, legally prickly, and technically irritating. A smoking transformer with NDAs.
- Headers explicitly mention VC Apps Access Program licensing for Reader/Writer/NPI. Verify legal and redistribution constraints before wiring this into CI or shipping binaries.
- Decide early what “FSDB support in wavepeek” means:
  - local optional plugin using installed Verdi libraries, or
  - external helper binary, or
  - no direct FSDB support unless `$VERDI_HOME` is present.
- Do not commit `/opt/verdi` headers, libraries, docs, or generated excerpts into the repository.
