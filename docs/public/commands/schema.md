---
id: commands/schema
title: Schema command
description: Fetch exact JSON contracts for output, stream records, and structured input documents.
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
  "additionalProperties": true,
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
- which schema artifact should be saved or pinned in CI,
- which structured JSON input documents are accepted by source-file modes such as `extract generic --source`.

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
  "extract axi",
  "extract axistream",
  "extract generic",
  "docs topics",
  "docs search"
]
```

This is the quickest way to confirm whether a command family has a stable machine contract.

## See which `data` shapes exist

The `data` field is a tagged union keyed by `command`. You can inspect the referenced definitions directly:

```text
$ wavepeek schema | jq '.properties.data.anyOf | map(.["$ref"])'
[
  "#/$defs/infoData",
  "#/$defs/scopeData",
  "#/$defs/signalData",
  "#/$defs/valueData",
  "#/$defs/changeData",
  "#/$defs/propertyData",
  "#/$defs/extractAxiData",
  "#/$defs/extractAxiStreamData",
  "#/$defs/extractGenericData",
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

This keeps the schema matched to the binary's embedded contract snapshots. This build prints the same bytes as `schema/output.json`. The v2 schema allows additive object fields and requires the exact `$schema` URL published for the current schema family version.

## Fetch the JSONL stream record schema

Waveform commands support `--jsonl` for newline-delimited JSON streams. A JSONL stream is a sequence of independent JSON objects, so the stream schema validates one record at a time rather than the whole stdout stream.

Use `--stream` to print that record schema:

```text
$ wavepeek schema --stream | jq -r '.title'
wavepeek JSONL stream record
```

This build prints the same bytes as `schema/stream.json`. Stream consumers should validate each line against this schema and separately check ordering rules such as first `begin`, final `end`, contiguous `seq`, and matching summary counts.

## Fetch the JSON input document schema

Some commands accept structured JSON input. Source-file extraction uses `extract.generic.sources` for `extract generic`, `extract.axi.source` for `extract axi`, and `extract.axistream.source` for `extract axistream`.

Use `--input` to print the input schema:

```text
$ wavepeek schema --input | jq -r '.title'
wavepeek JSON input documents
```

This build prints the same bytes as `schema/input.json`. Runtime validation still enforces semantic rules that JSON Schema cannot express directly, such as unique source names and unique payload entries within each source.
