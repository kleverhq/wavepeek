---
id: reference/machine-output
title: Machine Output Contract
description: Stable stdout, stderr, JSON envelope, JSONL stream, diagnostic, schema, fatal-error, and exit-code behavior.
section: reference
see_also:
  - reference/command-model
  - commands/schema
  - commands/skill
---
# Machine Output Contract

This document is normative for stdout, stderr, JSON-mode behavior, JSONL stream behavior, schema linkage, diagnostics, and exit codes.

## 1. Stdout and Stderr Responsibilities

On success, a command writes its main payload to stdout.

- In human-readable mode, non-fatal diagnostics are written to stderr as plain text.
- In `--json` mode, non-fatal diagnostics are carried inside the JSON payload.
- In `--jsonl` mode, waveform commands write one JSON object per stdout line; non-fatal diagnostics are diagnostic records in that stream.
- In `schema` mode, stdout contains exactly one JSON Schema document.

For non-streaming modes, stdout is empty on failure and process-level failures are reported on stderr only. In `--jsonl` mode, a fatal error after `begin` can leave partial stdout without a final `end`; consumers must treat that stream as incomplete.

## 2. JSON Envelope for Stable `--json` Commands

When a stable JSON-producing command succeeds under `--json`, it emits one JSON object with this shape:

```json
{
  "$schema": "https://kleverhq.github.io/wavepeek/schema-output-v2.2.json",
  "command": "<command>",
  "data": {},
  "diagnostics": []
}
```

The semantics of the envelope fields are:

- `$schema` is serialized literally as `$schema` and points at the exact canonical schema artifact for the current output schema family version.
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

`kind` is one of `info`, `warning`, or `error`. `warning` and `error` diagnostics always include a stable `code` matching `WPK-W####` or `WPK-E####`. `info` diagnostics omit `code`.

The exact JSON shapes for every command are defined by the current schema artifact such as `schema/output.json` and by `wavepeek schema`. Current v2 schemas are extension-friendly: consumers should ignore unknown object fields unless they intentionally pin to a stricter historical contract.

The stable JSON-producing commands currently include the waveform-inspection commands plus `docs topics --json` and `docs search --json`. Human-only helper surfaces such as `skill` and human-only docs subcommands such as `docs show` and `docs export` do not silently change output modes; unsupported `--json` combinations fail as argument errors and leave stdout empty.

`extract generic` data is an array of rows. Each row has `time`, `sample_time`, `source`, and ordered `payload` entries:

```json
{
  "time": "25ns",
  "sample_time": "24999ps",
  "source": "transfer",
  "payload": [
    {"path": "top.dut.data", "value": "32'hdeadbeef"}
  ]
}
```

Payload paths are canonical in JSON and JSONL output.

`extract axi` data is an object with AXI context and transfer rows. It has `name`, `profile`, `issue`, `mappings`, and `transfers`. Each transfer has `time`, `sample_time`, `profile`, `channel`, and a `payload` object keyed by lowercase AXI standard signal name. Supported profiles are AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP. AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 use Issue H.c metadata; AXI5, AXI5-Lite, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP use Issue L metadata. AXI5 and ACE5-LiteDVM can include `ac` and `cr` DVM channels but not `cd`. ACE and ACE5 include `ac`, `cr`, and `cd` coherency channels in addition to the five base AXI channels. The schema enumerates supported profiles, channels, and payload keys per profile/channel; payload keys are optional because rows include only mapped payload signals. Mapping paths are canonical.

`extract axistream` data has `name`, `profile`, `issue`, `tready_mode`, `mappings`, and `transfers`. Profiles are AXI4-Stream (`axi4-stream`) and AXI5-Stream (`axi5-stream`); both use Issue B. Each transfer has `time`, `sample_time`, `profile`, and a payload object keyed by mapped AXI-Stream payload standard names. There is no channel field because one invocation maps one stream interface. `tready_mode` is `mapped` or `implicit-high`, and an implicit-high context cannot contain a `tready` mapping. Mapping and payload key sets exclude AXI5-Stream wake-up and check/parity signals.

## 3. JSONL Stream for Waveform Commands

Waveform commands also support `--jsonl` for newline-delimited JSON output. JSONL means each stdout line is an independent JSON object, and the full stdout stream is not wrapped in an array.

A successful stream has these record types:

```jsonl
{"type":"begin","seq":0,"command":"change","$schema":"https://kleverhq.github.io/wavepeek/schema-stream-v2.2.json"}
{"type":"item","seq":1,"command":"change","item":{"time":"5ns","sample_time":"5ns","signals":[{"path":"top.clk","value":"1'h1"}]}}
{"type":"diagnostic","seq":2,"command":"change","diagnostic":{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}}
{"type":"end","seq":3,"command":"change","summary":{"status":"ok","items":1,"diagnostics":1,"truncated":true}}
```

Rules for successful JSONL streams:

- `begin` is first and has `seq: 0`.
- `seq` increases by one for every record.
- `command` is stable across the stream.
- `item` records carry the same row payload shape used inside `--json` data arrays for array-producing commands, the transfer row shape for `extract axi` or `extract axistream`, or the `info` data object for `info`.
- `change`, `property`, and `extract` rows include both `time` and `sample_time`. `time` is the selected event timestamp; `sample_time` is where values were printed, evaluated, or extracted.
- `extract axi` and `extract axistream` streams include protocol context on the `begin` record and repeat `profile` on each transfer item so each JSONL row can be validated independently. AXI-Stream begin context also includes `tready_mode`.
- `diagnostic` records carry the same diagnostic object shape used by `--json`.
- `end` is last on successful completion and reports `summary.status: "ok"`, item count, diagnostic count, and whether output was truncated.

The checked-in stream schema, such as `schema/stream.json`, validates one JSONL record at a time. Consumers must validate stream-level invariants themselves: first record is `begin`, last successful record is `end`, sequence numbers are contiguous, commands match, and summary counts match the records seen.

If the process exits non-zero or a stream lacks a final `end` record, treat the stream as incomplete. A consumer that intentionally closes stdout early, for example by piping to `head`, may stop the producer without a fatal error.

`--json` and `--jsonl` are mutually exclusive. `--jsonl` is available only on waveform-inspection commands: `info`, `scope`, `signal`, `value`, `change`, `property`, `extract axi`, `extract axistream`, and `extract generic`.

## 4. `schema` Command Behavior

`wavepeek schema` is the authority for the machine-readable output contract.

Its behavior is special and fixed:

- it accepts no waveform input,
- it accepts no command-specific flags or positional arguments,
- it writes exactly one JSON Schema document to stdout,
- it does not wrap that document in the normal command envelope, and
- without selectors, its output bytes match the canonical JSON envelope snapshot, `schema/output.json`.

`wavepeek schema --stream` prints the canonical JSONL record schema snapshot, `schema/stream.json`. That schema describes one stream record, not a whole JSONL stream.

`wavepeek schema --input` prints the canonical JSON input document schema snapshot, `schema/input.json`. Current input document kinds are `extract.generic.sources`, used by `wavepeek extract generic --source`; `extract.axi.source`, used by `wavepeek extract axi --source`; and `extract.axistream.source`, used by `wavepeek extract axistream --source`.

## 5. Diagnostic Behavior

Diagnostics do not change the exit code. A command can therefore succeed with diagnostics.

Common diagnostic cases include truncated output, explicitly disabled limits, and semantically empty-but-valid queries such as a selected range with no matching signal changes.

Diagnostic transport depends on the output mode:

- human-readable mode sends diagnostics to stderr,
- `--json` mode stores diagnostics in the envelope's `diagnostics` array,
- `--jsonl` mode stores diagnostics as `diagnostic` records before the final `end` record.

Human-readable diagnostics use these formats:

```text
info: <message>
warning[WPK-W0002]: <message>
error[WPK-E0001]: <message>
```

When `DEBUG=1` is set, commands may also write debug event lines to stderr. These lines are independent of command diagnostics and fatal errors. Each debug line is a JSON object with `kind: "debug"`, `message`, `timestamp_ns`, and an object-valued `details` field. The `details` content is intentionally free-form.

## 6. Fatal Error Format and Exit Codes

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

## 7. Human Output Flexibility

Human-readable output is part of the user experience, but it is intentionally less rigid than the machine contract. Commands may improve human formatting over time as long as they preserve the semantic guarantees documented in `command-model` and the stricter cases called out by command-specific help or tests.
