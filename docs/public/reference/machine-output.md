---
id: reference/machine-output
title: Machine Output Contract
description: Stable stdout, stderr, JSON envelope, diagnostic, schema, fatal-error, and exit-code behavior.
section: reference
see_also:
  - reference/command-model
  - commands/schema
  - commands/skill
---
# Machine Output Contract

This document is normative for stdout, stderr, JSON-mode behavior, schema linkage, diagnostics, and exit codes.

## 1. Stdout and Stderr Responsibilities

On success, a command writes its main payload to stdout.

- In human-readable mode, non-fatal diagnostics are written to stderr as plain text.
- In `--json` mode, non-fatal diagnostics are carried inside the JSON payload.
- In `schema` mode, stdout contains exactly one JSON Schema document.

On failure, stdout is empty. Process-level failures are reported on stderr only.

## 2. JSON Envelope for Stable `--json` Commands

When a stable JSON-producing command succeeds under `--json`, it emits one JSON object with this shape:

```json
{
  "$schema": "https://kleverhq.github.io/wavepeek/wavepeek_v<major>.json",
  "command": "<command>",
  "data": {},
  "diagnostics": []
}
```

The semantics of the envelope fields are:

- `$schema` is serialized literally as `$schema` and points at the canonical schema artifact for the running wavepeek major version.
- `command` identifies the executed subcommand and disambiguates the shape of `data`.
- `data` carries the command payload. Depending on the command contract, it may be an object or an array.
- `diagnostics` is an array of typed diagnostic objects in deterministic order.

A diagnostic object has `kind`, `message`, and sometimes `code`:

```json
{
  "kind": "warning",
  "code": "WPK-W0002",
  "message": "truncated output to 1 entries (use --max to increase limit)"
}
```

`kind` is one of `info`, `warning`, `error`, or `debug`. `warning`, `error`, and `debug` diagnostics always include a stable `code` matching `WPK-W####`, `WPK-E####`, or `WPK-D####`. `info` diagnostics omit `code`. Diagnostics may include object-valued `details`; debug codes define the shape of their own `details` object.

The exact JSON shapes for every command are defined by the current major artifact such as `schema/wavepeek_v1.json` and by `wavepeek schema`.

The stable JSON-producing commands currently include the waveform-inspection commands plus `docs topics --json` and `docs search --json`. Human-only helper surfaces such as `skill` and human-only docs subcommands such as `docs show` and `docs export` do not silently change output modes; unsupported `--json` combinations fail as argument errors and leave stdout empty.

## 3. `schema` Command Behavior

`wavepeek schema` is the authority for the machine-readable output contract.

Its behavior is special and fixed:

- it accepts no waveform input,
- it accepts no command-specific flags or positional arguments,
- it writes exactly one JSON Schema document to stdout,
- it does not wrap that document in the normal command envelope, and
- its output bytes match the canonical artifact for the running major version, such as `schema/wavepeek_v1.json`.

## 4. Diagnostic Behavior

Diagnostics do not change the exit code. A command can therefore succeed with diagnostics.

Common diagnostic cases include truncated output, explicitly disabled limits, semantically empty-but-valid queries such as a selected range with no matching signal changes, and opt-in maintainer debug output.

Diagnostic transport depends on the output mode:

- human-readable mode sends diagnostics to stderr,
- `--json` mode stores diagnostics in the envelope's `diagnostics` array.

Human-readable diagnostics use these formats:

```text
info: <message>
warning[WPK-W0002]: <message>
error[WPK-E0001]: <message>
debug[WPK-D1002]: <message>
```

`DEBUG=1` enables performance debug diagnostics for successful waveform commands only: `info`, `scope`, `signal`, `value`, `change`, and `property`. Helper commands such as `schema`, `docs`, and `help` do not emit performance debug diagnostics. Fatal failures keep the fatal stderr contract and do not produce a JSON diagnostics envelope.

The current debug codes are:

- `WPK-D0001`: generic debug message. `details` is optional and has no command-specific schema.
- `WPK-D1001`: performance context. `details` identifies the command, waveform backend, and waveform format.
- `WPK-D1002`: performance phase timing. `details` contains `phase`, `duration_ns`, `status`, and optional extensible `metrics`.
- `WPK-D1003`: performance summary. `details` contains the command, total duration, and phase count.

Performance `details` objects are structured by debug code. `WPK-D1002` is reused for all timed phases; the concrete phase name is in `details.phase`, for example `backend.open`, `metadata.load`, `signal.resolve`, or `value.sample`. Details are extensible so future releases may add counters without changing the diagnostic code. Performance diagnostics must not expose signal values.

## 5. Fatal Error Format and Exit Codes

Process-level failures are fail-fast and machine-parseable. The stderr format is:

```text
fatal: <category>: <message>
```

Representative categories include `args`, `file`, `scope`, `signal`, and `expr`.

Exit codes are stable at the process level:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User-facing failure such as bad arguments, missing scopes/signals, or invalid expressions |
| `2` | File-level failure such as open, parse, or unsupported-format failures |

Fatal errors are never wrapped in the JSON success envelope. Even in `--json` mode, stdout stays empty on failure and stderr carries the formatted fatal line.

## 6. Human Output Flexibility

Human-readable output is part of the user experience, but it is intentionally less rigid than the machine contract. Commands may improve human formatting over time as long as they preserve the semantic guarantees documented in `command-model` and the stricter cases called out by command-specific help or tests.
