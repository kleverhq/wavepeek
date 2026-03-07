# wavepeek

[![CI](https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/wavepeek.svg)](https://crates.io/crates/wavepeek)

`wavepeek` is a deterministic CLI for inspecting RTL waveforms (`.vcd`/`.fst`) in scripts, CI, and LLM-driven workflows.

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

Run a complete inspection flow:

```bash
# 0) Get a dump
# Note: example `.fst` dumps can be downloaded from `rtl-artifacts` releases: https://github.com/kleverhq/rtl-artifacts
WAVES=./dump.fst

# 1) Check dump bounds and time unit
wavepeek info --waves "$WAVES"

# 2) Discover hierarchy
wavepeek scope --waves "$WAVES" --tree

# 3) Find relevant signals in a scope (--filter is a regex)
wavepeek signal --waves "$WAVES" --scope top.cpu --filter '.*(clk|rst|state).*'

# 4) Sample values at one timestamp
wavepeek value --waves "$WAVES" --at 100ns --scope top.cpu --signals reset_n,state

# 5) Inspect transitions over a time window (--on is a SystemVerilog-like clocking event expression)
wavepeek change --waves "$WAVES" --from 0ns --to 500ns --scope top.cpu --signals state --on 'posedge clk'
```

By default, commands print human-readable output. Add `--json` for strict machine output:

```bash
wavepeek info --waves "$WAVES" --json
```

Chain commands in scripts with `jq`:

```bash
scope="$(wavepeek scope --waves "$WAVES" --json | jq -r '.data[0].path')"
wavepeek signal --waves "$WAVES" --scope "$scope" --json | jq '.data[:5]'
```


## Agentic Flows

`wavepeek` ships with a ready-to-install skill:

- Skill folder (repo): `https://github.com/kleverhq/wavepeek/tree/main/.opencode/skills/wavepeek`

Install via your agent:

- Ask your coding agent to install the skill from the repository path above (for example, with a skill-installer workflow if your harness supports one).

Manual install (copy the `.opencode/skills/wavepeek` folder):

- Codex CLI: `~/.codex/skills/wavepeek`
- Claude Code: `~/.claude/skills/wavepeek`
- OpenCode (project-local): `<your-project>/.opencode/skills/wavepeek`

Note: an MCP server for tool-native agent integration is not available yet, but is planned.

## Commands

| Command | Status | Purpose |
| --- | --- | --- |
| `info` | available | Print dump metadata (`time_unit`, `time_start`, `time_end`) |
| `scope` | available | List hierarchy scopes (deterministic DFS, optional `--tree`) |
| `signal` | available | List signals in a scope with metadata |
| `value` | available | Signal values at a specific time |
| `change` | available | Delta snapshots over a time range with `--on` event triggers |
| `property` | available (runtime unimplemented) | Property checks over event triggers |
| `schema` | available | Print canonical JSON schema used by `--json` output |

Use progressive disclosure via built-in help: `wavepeek -h`, then `wavepeek <command> --help`.

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
