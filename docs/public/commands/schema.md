---
id: commands/schema
title: Schema command
summary: Fetch the exact JSON contract this build guarantees for `--json` output.
section: commands
see_also:
  - commands/overview
  - reference/machine-output
---
# Schema command

Use `schema` when you are integrating `wavepeek` into scripts, agents, validators, or test harnesses.

Instead of inferring JSON shape from a few examples, ask the binary for the contract it ships:

```text
$ wavepeek schema | head -n 10
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "wavepeek JSON output envelope",
  "description": "Canonical schema for wavepeek --json command outputs.",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "$schema",
    "command",
```

For exact syntax, run `wavepeek help schema`.

## When to use it

Use `schema` when you need to answer practical questions such as:

- which commands have stable `--json` output,
- which `command` values are valid,
- which `$defs` describe each command's `data`,
- which schema artifact should be saved or pinned in CI.

`schema` does not need a waveform file and does not inspect one.

## Check which commands are covered

If you only need to know which JSON payloads exist, inspect the `command` enum:

```text
$ wavepeek schema | jq '.properties.command.enum'
[
  "info",
  "scope",
  "signal",
  "value",
  "change",
  "property",
  "docs topics",
  "docs search"
]
```

This is the quickest way to confirm whether a command family has a stable machine contract.

## See which `data` shapes exist

The `data` field is a tagged union keyed by `command`. You can inspect the referenced definitions directly:

```text
$ wavepeek schema | jq '.properties.data.oneOf | map(.["$ref"])'
[
  "#/$defs/infoData",
  "#/$defs/scopeData",
  "#/$defs/signalData",
  "#/$defs/valueData",
  "#/$defs/changeData",
  "#/$defs/propertyData",
  "#/$defs/docsTopicsData",
  "#/$defs/docsSearchData"
]
```

Use this when you are generating typed bindings or wiring a validator.

## Save the exact schema from the installed build

Redirect stdout if another tool needs the artifact as a file:

```text
$ wavepeek schema > wavepeek-schema.json
$ jq -r '.title' wavepeek-schema.json
wavepeek JSON output envelope
```

This keeps the schema version-matched to the binary you are actually running.
