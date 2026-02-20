# wavepeek

[![CI](https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/wavepeek.svg)](https://crates.io/crates/wavepeek)

`wavepeek` is a deterministic CLI for inspecting RTL waveforms (`.vcd`/`.fst`) in scripts, CI, and LLM-driven workflows.

> Early stage project: command surface is stabilizing, but not complete yet.

## Why

- In RTL debugging, waveforms are the primary artifact, but most existing tooling is GUI-first.
- LLM agents and CI jobs need short, composable commands instead of interactive navigation.
- Raw dumps (especially large VCD/FST files) are too heavy for direct, repeated analysis in context-limited systems.
- `wavepeek` closes this gap with deterministic, bounded, machine-friendly waveform queries.

## Quick Start

Install:

```bash
cargo install wavepeek
# or from source
cargo install --path .
```

Run:

```bash
wavepeek info --waves ./dump.fst
wavepeek scope --waves ./dump.fst --tree
wavepeek signal --waves ./dump.fst --scope top.cpu --filter '.*clk.*'
```

By default, commands print human-readable output. Add `--json` for strict machine output:

```bash
wavepeek info --waves ./dump.fst --json
```

Note: example `.fst` dumps can be downloaded from `rtl-artifacts` releases: https://github.com/kleverhq/rtl-artifacts

## Agentic Flows (Soon)

- A dedicated `SKILL.md` for agent workflows is planned.
- Planned compatibility targets: OpenCode, Codex CLI, and Claude Code.
- An MCP server for tool-native agent integration is also planned.

## Commands

| Command | Status | Purpose |
| --- | --- | --- |
| `info` | available | Print dump metadata (`time_unit`, `time_start`, `time_end`) |
| `scope` | available | List hierarchy scopes (deterministic DFS, optional `--tree`) |
| `signal` | available | List signals in a scope with metadata |
| `at` | planned | Signal values at a specific time |
| `change` | planned | Value snapshots over a time range |
| `when` | planned | Cycles where expression is true |
| `schema` | available | Print canonical JSON schema used by `--json` output |

Use `wavepeek --help` and `wavepeek <command> --help` for complete flag details.

## Development

- Preferred workflow uses `Makefile` targets aligned with CI.
- In devcontainer/CI image, run:

```bash
make bootstrap
make check
make test
```

## License

Apache-2.0
