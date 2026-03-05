# wavepeek CLI Recipes

## Quick Start

Use these commands as a deterministic baseline:

```bash
wavepeek info --waves ./dump.fst
wavepeek scope --waves ./dump.fst --tree
wavepeek signal --waves ./dump.fst --scope top.cpu --filter '.*clk.*'
wavepeek value --waves ./dump.fst --at 100ns --scope top.cpu --signals clk,reset_n,state
wavepeek change --waves ./dump.fst --from 0ns --to 500ns --scope top.cpu --signals clk,state --on 'posedge clk'
```

## Progressive Disclosure via Help

Use built-in help as the default discovery path:

```bash
wavepeek -h
wavepeek info --help
wavepeek scope --help
wavepeek signal --help
wavepeek value --help
wavepeek change --help
wavepeek schema --help
```

Recommendation:
- Prefer `--help` output first when composing commands, then use docs for cross-command recipes.

## Task-to-Command Mapping

1. Check dump bounds and unit:

```bash
wavepeek info --waves <FILE>
```

2. Explore hierarchy:

```bash
wavepeek scope --waves <FILE> --max 200 --max-depth 8 --filter '.*'
wavepeek scope --waves <FILE> --tree
```

3. Discover signals in a scope:

```bash
wavepeek signal --waves <FILE> --scope <SCOPE>
wavepeek signal --waves <FILE> --scope <SCOPE> --recursive --max-depth 2
wavepeek signal --waves <FILE> --scope <SCOPE> --filter '.*(clk|rst).*' --abs
```

4. Sample point-in-time values:

```bash
wavepeek value --waves <FILE> --at <TIME> --scope <SCOPE> --signals a,b,c
wavepeek value --waves <FILE> --at <TIME> --signals top.cpu.a,top.cpu.b
```

5. Inspect range changes:

```bash
wavepeek change --waves <FILE> --from <START> --to <END> --scope <SCOPE> --signals a,b --on '*'
wavepeek change --waves <FILE> --from <START> --to <END> --scope <SCOPE> --signals a,b --on 'posedge clk'
wavepeek change --waves <FILE> --from <START> --to <END> --signals top.cpu.a,top.cpu.b --on 'posedge top.cpu.clk or negedge top.cpu.rst_n'
```

6. Switch to machine output:

```bash
wavepeek info --waves <FILE> --json
wavepeek signal --waves <FILE> --scope <SCOPE> --json
wavepeek change --waves <FILE> --from <START> --to <END> --scope <SCOPE> --signals a,b --json
wavepeek schema
```

## JSON Pipeline Recipes (`--json` + `jq`)

1. Take the first discovered scope and list its signals:

```bash
scope="$(wavepeek scope --waves <FILE> --json | jq -r '.data[0].path')"
wavepeek signal --waves <FILE> --scope "$scope" --json
```

2. Take first N signal paths from a scope and sample values:

```bash
signals="$(wavepeek signal --waves <FILE> --scope <SCOPE> --json | jq -r '.data[:5] | map(.path) | join(\",\")')"
wavepeek value --waves <FILE> --at 100ns --signals "$signals" --json
```

3. Build a `change` query from discovered signals:

```bash
signals="$(wavepeek signal --waves <FILE> --scope <SCOPE> --filter '.*(clk|state).*' --json | jq -r '.data | map(.path) | join(\",\")')"
wavepeek change --waves <FILE> --from 0ns --to 500ns --signals "$signals" --on '*' --json
```

4. Print compact signal/value pairs from `value` JSON:

```bash
wavepeek value --waves <FILE> --at 100ns --signals top.cpu.clk,top.cpu.state --json \
  | jq -r '.data.signals[] | "\(.path)=\(.value)"'
```

5. Pipe warnings separately for CI checks:

```bash
wavepeek change --waves <FILE> --from 0ns --to 1us --signals top.cpu.clk --json \
  | jq '{command, warnings_count: (.warnings | length), warnings}'
```

## Event Expression Notes (`--on`)

`--on` expression design follows SystemVerilog-style event control (clocking events) patterns.

Supported patterns:

```text
*                           # any tracked signal change
clk                         # any change of signal clk
posedge clk                 # rising edge
negedge clk                 # falling edge
edge clk                    # either edge
posedge clk or negedge rst  # union with "or"
posedge clk, negedge rst    # union with ","
```

Current limitation:
- `iff` clauses are parsed but not executed in `change`; avoid `iff` in production guidance.

## Time and Limit Rules

- Always pass explicit time units (`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`).
- Keep `--from <= --to`.
- Keep times inside dump bounds from `info`.
- Keep times aligned to dump tick (`time_unit` from `info`).
- Default `--max` is bounded; use `unlimited` only on explicit user request.

## Troubleshooting Playbook

1. "invalid --on expression":
- Reduce expression to `*` and re-add terms one by one.

2. "time out of bounds" or "not aligned":
- Run `wavepeek info --waves <FILE>`.
- Snap time tokens to valid aligned values.

3. "signal not found":
- Re-run `scope` and `signal` to confirm exact canonical path.
- Use `--scope` with short names or pass canonical names without `--scope`.

4. Too much output:
- Add/narrow `--filter`.
- Lower `--max`.
- Reduce `--max-depth` or disable `--recursive`.
- Narrow `--from/--to`.
