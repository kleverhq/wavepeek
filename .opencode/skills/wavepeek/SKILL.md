---
name: wavepeek
description: Guide users through effective wavepeek CLI usage for `.vcd` and `.fst` waveform files with deterministic, machine-friendly command flows. Use this skill when the user asks how to inspect dump metadata, discover scopes/signals, sample values at specific times, analyze changes over time windows, or integrate wavepeek JSON output into scripts/CI/agent workflows.
---

# Use wavepeek CLI

## Overview

Drive wavepeek usage as an explicit command workflow: start from dump metadata, narrow hierarchy, inspect signals, then run value/change queries with bounded ranges and deterministic output.
Prefer concrete command suggestions over generic advice, and tailor every command to the user-provided waveform path, scope, signals, and time window.

## Workflow

1. Confirm context before proposing commands:
   - Waveform file path (`--waves` target).
   - Goal (`metadata`, `scope discovery`, `point sample`, `range delta`, `JSON automation`).
   - Known scope/signal names and time window, if any.
2. Start with metadata to establish valid time bounds and unit:
   - Use `wavepeek info --waves <FILE>`.
3. Discover hierarchy and signals in two passes:
   - Use `wavepeek scope --waves <FILE>` (add `--tree` when visual hierarchy helps).
   - Use `wavepeek signal --waves <FILE> --scope <SCOPE>` (use `--recursive` and optional `--max-depth` only when needed).
4. Run value-level queries:
   - Point-in-time: `wavepeek value --waves <FILE> --at <TIME> --signals <SIGS> [--scope <SCOPE>]`.
   - Range deltas: `wavepeek change --waves <FILE> --from <TIME> --to <TIME> --signals <SIGS> [--scope <SCOPE>] [--on <EVENT_EXPR>]`.
5. Switch to machine mode when user needs automation:
   - Add `--json` to waveform commands.
   - Use `wavepeek schema` when a consumer needs the JSON contract.
   - Prefer `jq` pipelines to chain commands (extract scope/signal paths from one command and feed the next).
6. Keep queries bounded and deterministic:
   - Respect `--max` defaults unless user explicitly needs larger output.
   - Use `unlimited` only when user accepts larger outputs.
   - Keep time windows narrow first, then expand.

## Operational Rules

- Treat CLI help as the primary progressive-disclosure interface:
  - Start with `wavepeek -h`.
  - Then use `wavepeek <command> --help` for command-specific flags, behavior, and examples.
  - Prefer help-driven guidance so users can work self-serve without external docs.
- Always include explicit units in time values (`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`).
- Treat bare numeric times as invalid for user-facing guidance.
- Keep signal order from user intent; `value` output follows input `--signals` order.
- Prefer canonical paths when ambiguity exists; use `--scope` for short signal names.
- Explain that `--on` / `EVENT_EXPR` follow SystemVerilog-style event control semantics (clocking events).
- Warn that `property` is parse-level only and runtime is currently unimplemented.
- When user asks for command chaining, default to `--json | jq` instead of brittle text parsing.

## Error-Handling Pattern

When a command fails, convert the error into the next deterministic action:
- Missing/invalid args: provide the corrected full command.
- Invalid/misaligned/out-of-bounds time: run `info`, then adjust `--at/--from/--to` to valid aligned bounds.
- Scope/signal resolution failure: run `scope` then `signal` to re-discover exact names.
- Oversized result: tighten `--max`, add `--filter`, narrow time window, or remove recursion.

## References

- Use [references/cli-recipes.md](references/cli-recipes.md) for command templates and ready-to-send user scenarios.
- Load the reference only when building concrete command suggestions or troubleshooting specific failures.
