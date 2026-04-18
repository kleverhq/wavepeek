---
name: wavepeek
description: Use this skill when you need to inspect or analyze `.vcd` and `.fst` waveforms. Load it for dump metadata, scope/signal discovery, point samples, range deltas, event-based queries, and JSON-backed automation.
---

When there is a waveform question or waveform analysis request use `wavepeek` to do the work.

Use `wavepeek` as a deterministic workflow: establish dump bounds, discover hierarchy, query values or changes, then switch to JSON for automation. Tailor every command to the actual waveform path, scope, signals, and time window from the task.

## Agent posture

- Treat waveform files as CLI inputs, not text to inspect directly.
- Waveform dumps are often large; do not open them with generic file-reading tools or try to inspect them line by line.
- `.fst` files are binary; never read them directly as text or with generic read tools.
- Even `.vcd` files should be inspected through `wavepeek`, not by reading the raw dump.
- Confirm the waveform path, analysis goal, known scope or signals, and any target time window.
- Use help for command/flag discovery only when needed: `wavepeek -h`, then `wavepeek <command> --help`.
- Prefer deterministic execution: `info` -> `scope`/`signal` -> `value`/`change` -> `--json` post-processing.
- If the user asks for analysis, return the findings first; include exact commands only when they help.

## Core flow

1. Confirm `--waves`, goal, known scope or signals, and any target time window.
2. Establish valid bounds before any time query:
   - `wavepeek info --waves <FILE>`
3. Discover hierarchy in two passes:
   - `wavepeek scope --waves <FILE>`
   - `wavepeek scope --waves <FILE> --tree`
   - `wavepeek signal --waves <FILE> --scope <SCOPE>`
   - add `--recursive --max-depth <N>` only when needed
4. Query data with bounded ranges:
   - `wavepeek value --waves <FILE> --at <TIME> --scope <SCOPE> --signals a,b,c`
   - `wavepeek change --waves <FILE> --from <START> --to <END> --scope <SCOPE> --signals a,b --on 'posedge clk'`
5. For scripts, CI, or agent-side analysis:
   - add `--json`
   - use `wavepeek schema` for the JSON contract
   - prefer `jq` pipelines over brittle text parsing
6. Convert command output into the user answer:
   - summarize conclusions plainly
   - mention exact times, signals, scopes, and transitions
   - include raw commands or JSON details only when useful

## Ready recipes

```bash
wavepeek info --waves ./dump.fst
wavepeek scope --waves ./dump.fst --tree
wavepeek signal --waves ./dump.fst --scope top.cpu --filter '.*(clk|rst).*' --abs
wavepeek value --waves ./dump.fst --at 100ns --scope top.cpu --signals clk,reset_n,state
wavepeek value --waves ./dump.fst --at 100ns --signals top.cpu.clk,top.cpu.reset_n
wavepeek change --waves ./dump.fst --from 0ns --to 500ns --scope top.cpu --signals clk,state --on 'posedge clk'
```

```bash
scope="$(wavepeek scope --waves <FILE> --json | jq -r '.data[0].path')"
signals="$(wavepeek signal --waves <FILE> --scope "$scope" --json | jq -r '.data[:5] | map(.path) | join(",")')"
wavepeek value --waves <FILE> --at 100ns --signals "$signals" --json
```

## Rules

- Always include explicit time units: `zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`.
- Treat bare numeric times as invalid in commands and follow-up suggestions.
- Keep `--from <= --to`, stay inside dump bounds from `info`, and align times to the dump tick.
- Respect bounded output: keep `--max` default unless the user explicitly wants more; use `unlimited` only by request.
- Preserve user signal order; `value` follows input `--signals` order.
- Prefer canonical signal paths when names are ambiguous; use `--scope` only for short names.
- Explain that `--on` uses SystemVerilog-style event control syntax such as `*`, `posedge clk`, or `posedge clk or negedge rst_n`.
- `iff` clauses are parsed but not executed in `change`; avoid suggesting `iff` in production guidance.
- Warn that `property` is parse-level only; runtime execution is not implemented.
- When analysis requires chaining or filtering, default to `--json` plus `jq` rather than ad-hoc text parsing.
- Do not rely on raw dump reads for analysis; use `wavepeek` commands for both `.vcd` and `.fst` inputs.

## Failure recovery

- Missing or invalid args: return the corrected full command, or ask one targeted question if the waveform path or target signal set is unknown.
- Time out of bounds or misaligned: run `info`, then snap `--at`, `--from`, or `--to` to valid aligned values.
- Scope or signal lookup failure: rerun `scope`, then `signal`, and switch to canonical paths.
- Too much output: narrow the time window, add `--filter`, lower `--max`, or remove recursion.
- Invalid `--on`: reduce to `*`, then rebuild the expression term by term.
