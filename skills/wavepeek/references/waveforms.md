# Waveform formats

This page describes the supported waveform dump formats. Native CLI builds support
VCD, FST, and GHW by default and optional FSDB/SHM/WLF/WDB/VWDB. The browser accepts VCD/FST only.
Additional Ondas library readers do not imply WavePeek product support.
Signal kind aliases describe declarations; expression semantics use value
encoding and metadata, not the input format or its reader name.

All dump-orientation and waveform-inspection commands require `--waves <FILE>` and operate on a single dump per invocation. The path can be a file or a complete SHM or VWDB database directory. See [Quickstart](quickstart.md) for optional reader builds.

Output limits bound returned rows. Checking exact completeness may require the
next matching row. Reader setup and initial state can read a prefix of the dump;
incremental output does not guarantee bounded reader or native SDK memory.
Selected values can fail decoding before a matching row, including a query with
no matches. See [Machine output](machine-output.md#5-fatal-errors-and-exit-codes)
for failure records and exit codes.

## VCD

VCD is a textual waveform format. It is portable and widely available, but large VCD dumps can be much larger than equivalent binary formats. Queries against large VCD files may require expensive text parsing and high memory use, even when the final result is small.

For very large dumps, a narrow-looking command such as `scope`, `signal`, or `value` can still be dominated by waveform parsing rather than by printing the selected rows. Use VCD when it is the only available dump or when portability matters more than repeated-query performance.

Converting a VCD dump to FST can help when the converted file will be queried repeatedly. The conversion itself costs time and disk space, so it is not automatically worthwhile for one or two queries. Conversion can also change or drop format-specific metadata. For example, hierarchy kind labels or type details may not survive every conversion path. If a workflow depends on that metadata, compare a small representative query before relying on converted output.

## FST

FST is a compact, indexed waveform format. It is usually the best default for repeated inspection with `wavepeek` because metadata and selected signal data can often be loaded without reconstructing the entire dump body.

FST tends to use much less memory than large VCD input and is well suited for scripted debug loops where the same dump is queried many times.

## GHW

Use uncompressed GHDL Waveform (GHW) files recorded at 1fs. WavePeek reads each stored tick as one femtosecond, so `info` reports `time_unit: 1fs`. Check the simulator resolution before opening a dump. For example, a dump recorded at 1ps uses ticks 1000 times larger than WavePeek assumes, so its timestamps and time queries are incorrect.

Query readable record fields by the paths in `signal` output. The reader validates the full GHW body to establish the time bounds, including for `info`.

## FSDB

FSDB is a compact proprietary waveform format read through the Synopsys Verdi FSDB Reader SDK in FSDB-enabled `wavepeek` builds. FSDB value sampling can be fast after setup, but short one-shot CLI calls may spend most of their time in native reader setup, hierarchy loading, and hierarchy teardown.

FSDB modport and interface-reference scopes use the normalized scope kind `interface`.

This means an FSDB file can be far smaller than a VCD file and still take seconds per independent command. The requested value lookup itself may not be the expensive step; repeated setup can dominate.

FSDB real and ASCII string values support the existing expression operations in `property`, `iff` and `when`. Bit-oriented value rows and extraction payloads require integral data.

## SHM

SHM is a Cadence database directory containing design (`.dsn`) and transition
(`.trn`) components. Build with `--features shm` and select a compatible Xcelium
reader through `XCELIUM_HOME`; see [Quickstart](quickstart.md#install-with-shm).
Keep the complete directory available throughout the query. Individual components
and browser uploads are unsupported.

SHM uses the same native commands and output formats as VCD/FST. Integral values,
real/string expressions, and recorded event occurrences use the shared query
engine. Available declaration metadata depends on the recording and reader:
signedness and enumeration labels are unavailable, and VCD conversion can expose
`wire`, `reg`, and parameter declarations as generic `bit_vector` kinds.

Recorded start bounds can differ between converted formats. `info` reports the
SHM reader's bounds; an earlier start can add an initial row to `change` output.
Compare bounds and initial rows when validating a conversion.

Each invocation opens the native reader. Query cost also depends on the selected
histories and window; use measurements from your design to compare SHM with FSDB.

## WLF

WavePeek reads Questa waveform log files with the `wlf` feature. Ondas recognizes the `.wlf` extension without case sensitivity. See [Quickstart](quickstart.md#install-with-wlf) for setup.

Read values with the same commands as for VCD and FST. Expressions can use supported reals, ASCII strings, and named events. Bit-oriented output requires integral values. If recording started late, a signal has no value until its first recorded sample.

## VWDB

Pass the directory with `vwdb_info.json` and its design components. Earlier recordings can have a `.shm` suffix. Ondas identifies the recording from its contents.

A combined recording can contain a separate SHM directory. Some SystemC recordings store their digital histories there. To query those histories, enable `shm` and pass that directory to `--waves`. It has its own time grid.

WavePeek uses the recorded widths, values, and enum labels supplied by Ondas. Dynamic strings can be absent from the recorded hierarchy. A recording gap makes persistent values unavailable until they are recorded again.

## WDB

WDB is the Vivado waveform database file format. Use a native build with `--features wdb` and the compatible runtime from [Quickstart](quickstart.md#install-with-wdb).

The hierarchy can include compiler-generated scopes and declarations without recorded values. Use `scope` and `signal` to get paths from the recording. WDB enum labels can be used in expressions when the recording supplies the enum table.

Selecting a dynamic SystemVerilog string or a named event returns an unsupported-signal error. Changing unsigned 129-bit enums can return a stale value. See the [Ondas WDB reader limits](https://github.com/kleverhq/ondas/blob/v3.0.2/docs/wdb-reader.md#declaration-support-does-not-establish-recording-support).

## Performance expectations

`wavepeek` performance depends on the waveform format, input size, command, requested signals, storage speed, and memory. Each process pays the reader setup cost. When command semantics allow it, request the needed scopes, signals, timestamps, or time window in one command.

The table below shows one representative design dumped to three formats and queried on one machine. Treat it as scale guidance only.

| Format | Example file size | Representative one-shot command time | Peak memory |
| --- | ---: | ---: | ---: |
| FSDB | 15 MiB | 4.7-5.3 s | about 2.0 GiB |
| VCD | 3.4 GiB | 14.8-15.0 s | about 5.3 GiB |
| FST | 35 MiB | 0.2-0.7 s | about 100 MiB |

The main lesson is the ordering, not the exact numbers: large textual VCD input can be expensive, FST is usually much cheaper for repeated inspection, and FSDB one-shot commands can be dominated by setup cost.

Use the dump format you already have when you need one quick answer and conversion would take longer than the query. Prefer FST when you control dump generation or expect repeated scripted inspection. Use FSDB directly when preserving FSDB-specific metadata matters or when conversion is unavailable. Avoid large VCD files for repeated automation when an FST equivalent is available.
