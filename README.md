<p align="center">
  <img src="docs/wavepeek.svg" alt="wavepeek" width="900">
</p>

<p align="center">
  A CLI for querying RTL waveform dumps.<br>
  Supports hierarchy browsing, signal inspection, Boolean condition checking, and event extraction.
</p>

<p align="center">
  <a href="https://kleverhq.github.io/wavepeek/">Playground</a>
  · <a href="https://kleverhq.github.io/wavepeek/latest/">Documentation</a>
  · <a href="https://kleverhq.github.io/wavepeek/latest/quickstart/">Quickstart</a>
  · <a href="https://github.com/kleverhq/wavepeek/releases">Releases</a>
</p>

<p align="center">
  <a href="https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml?query=event%3Apull_request"><img src="https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml/badge.svg?event=pull_request" alt="PR CI"></a>
  <a href="https://github.com/kleverhq/wavepeek/releases"><img src="https://img.shields.io/github/v/release/kleverhq/wavepeek" alt="GitHub release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/kleverhq/wavepeek" alt="Apache-2.0 license"></a>
</p>

`wavepeek` is a stateless CLI that runs non-interactive queries over waveform files. Its main use cases are LLM-driven debugging and other automated workflows.

Each invocation opens one VCD/FST/FSDB file or SHM database directory, runs one query, writes text, JSON, or JSONL, and exits. It starts on demand and does not require a background service. For example, this command returns the value of `top.data` at `10ns`:

```text
$ wavepeek value --waves dump.vcd --at 10ns --signals top.data
@10ns top.data=8'h0f
```

`wavepeek` is not a GUI or TUI waveform viewer. It does not provide real-time waveform streaming, live simulator connections, or waveform comparison.

## Commands

| Task                                                         | Commands                                                                        |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------- |
| Read dump metadata and browse the recorded hierarchy         | `info`, `scope`, `signal`                                                       |
| Read values at a time or on selected events                  | `value`, `change`                                                               |
| Find when a Boolean condition matches, asserts, or deasserts | `property`                                                                      |
| Extract custom handshakes or synchronous events              | `extract generic`                                                               |
| Extract AMBA transfers or phase events                       | `extract axi`, `extract axistream`, `extract ahb`, `extract apb`, `extract atb` |

The protocol extractors do not check protocol compliance or reconstruct high-level transactions. Their output contains observed channel transfers or protocol phase events with sampled payload and context.

## Getting started

Open the [Playground](https://kleverhq.github.io/wavepeek/) to run `wavepeek` in a browser.

To install `wavepeek` on macOS or Linux, run:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://kleverhq.github.io/wavepeek/install.sh | sh
```

The prebuilt binary supports VCD and FST. Optional FSDB and SHM support require source builds on Linux x86_64 with local Verdi or Xcelium reader libraries.

Alternatively, ask your agent to do the setup:

```text
Install the latest release from https://github.com/kleverhq/wavepeek/releases. Run 'wavepeek skill' to get the skill.
```

Check the version after installation:

```bash
wavepeek --version
```

See the [Quickstart](https://kleverhq.github.io/wavepeek/latest/quickstart/) for Windows, Cargo, source installation, FSDB/SHM setup, and further examples.

## Agent skill

Extract the skill package included in the binary:

```bash
wavepeek skill ./wavepeek-skill
```

The package matches the installed `wavepeek` version. Its directory contains `SKILL.md`, offline references, examples, helper scripts, and provenance. Follow your agent harness's instructions to install the directory.

## Documentation

| Topic                        | Link                                                                                                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Installation and first query | [Quickstart](https://kleverhq.github.io/wavepeek/latest/quickstart/)                                                                                    |
| Command groups               | [Commands](https://kleverhq.github.io/wavepeek/latest/commands/)                                                                                        |
| Hierarchy and signal values  | [Explore dump](https://kleverhq.github.io/wavepeek/latest/explore-dump/) · [Inspect values](https://kleverhq.github.io/wavepeek/latest/inspect-values/) |
| Boolean conditions           | [Evaluate properties](https://kleverhq.github.io/wavepeek/latest/evaluate-properties/)                                                                  |
| Generic and AMBA extraction  | [Extract transfers](https://kleverhq.github.io/wavepeek/latest/extract-transfers/)                                                                      |
| JSON and JSONL output        | [Machine output](https://kleverhq.github.io/wavepeek/latest/machine-output/)                                                                            |
| Syntax and flags             | [CLI reference](https://kleverhq.github.io/wavepeek/latest/cli-reference/)                                                                              |

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0