---
id: reference/machine-output
title: Machine Output Contract
description: Stable stdout, stderr, JSON envelope, warning, schema, error, and exit-code behavior.
section: reference
see_also:
  - reference/command-model
  - commands/schema
  - commands/skill
---
# Machine Output Contract

This document is normative for stdout, stderr, JSON-mode behavior, schema linkage, warnings, and exit codes.

## 1. Stdout and Stderr Responsibilities

On success, a command writes its main payload to stdout.

- In human-readable mode, warnings are written to stderr as plain text.
- In `--json` mode, warnings are carried inside the JSON payload.
- In `schema` mode, stdout contains exactly one JSON Schema document.

On error, stdout is empty. Errors are reported on stderr only.

## 2. JSON Envelope for Stable `--json` Commands

When a stable JSON-producing command succeeds under `--json`, it emits one JSON object with this shape:

```json
{
  "$schema": "https://kleverhq.github.io/wavepeek/wavepeek_v<major>.json",
  "command": "<command>",
  "data": {},
  "warnings": []
}
```

The semantics of the envelope fields are:

- `$schema` is serialized literally as `$schema` and points at the canonical schema artifact for the running wavepeek major version.
- `command` identifies the executed subcommand and disambiguates the shape of `data`.
- `data` carries the command payload. Depending on the command contract, it may be an object or an array.
- `warnings` is an array of free-form warning strings in deterministic order.

The exact JSON shapes for every command are defined by the current major artifact such as `schema/wavepeek_v0.json` and by `wavepeek schema`.

The stable JSON-producing commands currently include the waveform-inspection commands plus `docs topics --json` and `docs search --json`. Human-only helper surfaces such as `skill` and human-only docs subcommands such as `docs show` and `docs export` do not silently change output modes; unsupported `--json` combinations fail as argument errors and leave stdout empty.

## 3. `schema` Command Behavior

`wavepeek schema` is the authority for the machine-readable output contract.

Its behavior is special and fixed:

- it accepts no waveform input,
- it accepts no command-specific flags or positional arguments,
- it writes exactly one JSON Schema document to stdout,
- it does not wrap that document in the normal command envelope, and
- its output bytes match the canonical artifact for the running major version, such as `schema/wavepeek_v0.json`.

## 4. Warning Behavior

Warnings do not change the exit code. A command can therefore succeed with warnings.

Common warning cases include truncated output, explicitly disabled limits, and semantically empty-but-valid queries such as a selected range with no matching signal changes.

Warning transport depends on the output mode:

- human-readable mode sends warnings to stderr,
- `--json` mode stores warnings in the envelope's `warnings` array.

## 5. Error Format and Exit Codes

Errors are fail-fast and machine-parseable. The stderr format is:

```text
error: <category>: <message>
```

Representative categories include `args`, `file`, `scope`, `signal`, and `expr`.

Exit codes are stable at the process level:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User-facing error such as bad arguments, missing scopes/signals, or invalid expressions |
| `2` | File-level error such as open, parse, or unsupported-format failures |

Errors are never wrapped in the JSON success envelope. Even in `--json` mode, stdout stays empty on failure and stderr carries the formatted error line.

## 6. Human Output Flexibility

Human-readable output is part of the user experience, but it is intentionally less rigid than the machine contract. Commands may improve human formatting over time as long as they preserve the semantic guarantees documented in `command-model` and the stricter cases called out by command-specific help or tests.
