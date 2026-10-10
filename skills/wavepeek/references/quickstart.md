# Quickstart

This page takes you from installation to your first `wavepeek` query.

## Install

Install a prebuilt binary. Prebuilt binaries support only VCD/FST.

To install a specific version, see the instructions for the required release on [Releases](https://github.com/kleverhq/wavepeek/releases).

Otherwise, follow the instructions below to install the latest version.

macOS and Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://kleverhq.github.io/wavepeek/install.sh | sh
```

Windows PowerShell:

```bash
powershell -ExecutionPolicy Bypass -c "irm https://kleverhq.github.io/wavepeek/install.ps1 | iex"
```

Cargo is also available as a fallback:

```bash
cargo install wavepeek
```

Or install from source (clone the repository first):

```bash
cargo install --path .
```

## Install with FSDB

FSDB support is source-only and requires Linux x86_64 and the Synopsys Verdi FSDB Reader SDK. Before running the command below, set `VERDI_HOME` to a valid Verdi installation.

```bash
cargo install wavepeek --features fsdb
```

To check whether FSDB is enabled in the current binary, use `--help`:

```text
$ wavepeek --help | grep FSDB
- FSDB - enabled
```

## Setup skill

The `wavepeek` skill is bundled into the binary. Besides `SKILL.md`, it includes reference documentation, examples and helper scripts.
To extract the skill bundle, use:

```bash
wavepeek skill <OUTPUT-DIRECTORY>
```

Alternatively, copy and paste this into your agent:

```text
Check whether `wavepeek` is installed:

wavepeek --version

If that succeeds, run:

wavepeek skill ./wavepeek-skill

`wavepeek skill <DIRECTORY>` extracts the complete, version-matched skill package into a new or empty directory. Install the entire extracted package according to your agent harness's skill rules. Replace the older skill entirely if the extracted version is newer.
```

## First run

Start with a waveform dump. Example dumps are available from [Ondas fixtures](https://github.com/kleverhq/ondas-fixtures#installation). Follow its installation instructions to download and verify selected waveforms.

Check the dump bounds and time unit:

```bash
wavepeek info --waves your_waves.fst
```

Discover the hierarchy:

```bash
wavepeek scope --waves your_waves.fst --tree
```

## Next steps

Learn concepts:

- [Commands](commands.md) - command model and common conventions.
- [Waveform formats](waveforms.md) - formats, performance, and FSDB support.
- [Paths, signals and scopes](paths.md) - canonical paths, relative paths, and bit projections.
- [Time units and windows](timeunits.md) - time tokens and query boundaries.
- [Clocks and sampling](sampling.md) - event times, sample times, and pre-edge sampling.
- [Boolean conditions](predicates.md) - Boolean expressions over waveform values.

Get practical usage examples:

- [Explore dump](explore-dump.md) - get dump bounds, navigate the hierarchy, and search signals.
- [Inspect values](inspect-values.md) - sample values at explicit points and get a table of changes.
- [Evaluate properties](evaluate-properties.md) - evaluate Boolean expressions and check whether a property holds.
- [Extract transfers](extract-transfers.md) - find transfers and get payload data under handshakes and valid/ready strobes.
- [Extract AMBA AXI](extract-axi.md) - map signals, get a table of transfers, and use AXI profiles.
- [Extract AMBA AXI-Stream](extract-axis.md) - map signals, get a table of transfers, and use AXI-Stream profiles.
- [Extract AMBA AHB](extract-ahb.md) - map signals, get a table of phase events, and use AHB profiles.
- [Extract AMBA APB](extract-apb.md) - map signals, get a table of phase events, and use APB profiles.
- [Extract AMBA ATB](extract-atb.md) - map signals, get a table of events, and use ATB profiles.
