---
name: wavepeek
description: Use this skill when you need to inspect or analyze `.vcd`, `.fst`, or `.fsdb` waveforms with the wavepeek CLI. Load it for dump metadata, hierarchy/signal discovery, point samples, range changes, property checks, and JSON-backed automation.
---

When there is a waveform question or waveform analysis request, use `wavepeek` to do the work.

Treat this skill as a safe entrypoint and router. It should get you to the right installed help or docs topic instead of copying the full command reference.

## Safety posture

- Treat waveform files as CLI inputs, not text to inspect directly.
- `.fst` and `.fsdb` files are binary; never read them directly with generic read tools.
- Even `.vcd` files can be large; prefer `wavepeek` commands over raw dump reads.
- Confirm the waveform path, analysis goal, known scope or signals, and any target time window.
- Keep queries bounded unless the user explicitly asks for more output.

## Progressive disclosure

Use the installed binary as the source of truth for exact syntax:

- `wavepeek -h` for compact lookup.
- `wavepeek --help` for detailed top-level help.
- `wavepeek help <command-path...>` for detailed command or nested-command help.
- `wavepeek docs topics` to list packaged narrative and reference topics.
- `wavepeek docs search <query>` when you do not know the topic ID.

Useful reference topics:

- `wavepeek docs show reference/command-model` for time, scope/name, bounded-output, and ordering rules.
- `wavepeek docs show reference/machine-output` for JSON envelope, diagnostics, fatal errors, schema, and exit behavior.
- `wavepeek docs show reference/expression-language` for `change --on` and `property --on` / `property --eval` expressions.

## Core workflow

1. Establish dump bounds and time unit:

       wavepeek info --waves <FILE>

2. Discover hierarchy and signals when names are unknown:

       wavepeek scope --waves <FILE>
       wavepeek signal --waves <FILE> --scope <SCOPE>

3. Query data with the command that matches the task:

       wavepeek value --waves <FILE> --at <TIME[,TIME...]> --signals <SIGNALS>
       wavepeek change --waves <FILE> --from <START> --to <END> --signals <SIGNALS> --on <EVENT>
       wavepeek property --waves <FILE> --from <START> --to <END> --on <EVENT> --eval <EXPR>

4. For scripts, CI, or agent-side post-processing, prefer supported `--json` output and consult:

       wavepeek schema
       wavepeek docs show reference/machine-output

5. Convert command output into the user answer. Report the findings first; include exact commands only when they help reproduce or audit the result.
