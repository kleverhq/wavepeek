---
id: commands/scope
title: Scope command
description: Explore hierarchy scopes.
section: commands
see_also:
  - commands/overview
  - commands/signal
  - reference/command-model
  - reference/machine-output
---
# Scope command

Use `scope` when you know the dump but do not know its hierarchy yet.

This command answers questions like:

- what is the top-level instance name,
- which sub-blocks exist under it,
- how deep the hierarchy goes,
- what canonical scope paths should be used in later commands.

In practice, `scope` is usually an early step before `signal`, `value`, `change`, or `property`.

For exact syntax and flags, run `wavepeek help scope`.

## Start by listing the hierarchy roots and immediate children

If you just need to orient yourself, run `scope` with a small dump and the default flat view:

```text
$ wavepeek scope --waves path/to/dump.vcd --max 50
0 top kind=module
1 top.cpu kind=module
1 top.mem kind=module
```

The left number is the hierarchy depth. The path is the canonical scope path you can reuse in later commands.

## Narrow the result to the part of the design you care about

`--filter` matches the full scope path, not only the last segment:

```text
$ wavepeek scope --waves path/to/dump.vcd --max 50 --filter '.*cpu.*'
1 top.cpu kind=module
```

Use this when a large dump has many repeated block names and you want to find candidate paths quickly.

## Stop at a specific depth

If full traversal is too noisy, cap it with `--max-depth`:

```text
$ wavepeek scope --waves path/to/dump.fst --max 50 --max-depth 0
0 top kind=module
```

`--max-depth 0` is a useful shortcut when you only want the dump roots.

## Switch to tree view for visual navigation

Flat output is better for grepping and copy-pasting paths. Tree mode is better for reading structure:

```text
$ wavepeek scope --waves path/to/dump.vcd --tree --max 50
top kind=module
├── cpu kind=module
└── mem kind=module
```

Use `--tree` when you want to understand parent/child relationships at a glance.

## Use JSON when another tool will consume the result

`--json` keeps the stable machine contract and returns canonical paths explicitly:

```text
$ wavepeek scope --waves path/to/dump.fst --max 50 --json
{"$schema":"https://kleverhq.github.io/wavepeek/wavepeek_v1.json","command":"scope","data":[{"path":"top","depth":0,"kind":"module"},{"path":"top.cpu","depth":1,"kind":"module"},{"path":"top.mem","depth":1,"kind":"module"}],"diagnostics":[]}
```

Use this in scripts, agents, or when you want deterministic parsing instead of human formatting.

## Watch for truncation and disabled-limit diagnostics

`scope` is bounded by default. If `--max` cuts the result, the command still succeeds but emits a diagnostic:

```text
$ wavepeek scope --waves path/to/dump.vcd --max 2
0 top kind=module
1 top.cpu kind=module
warning[WPK-W0002]: truncated output to 2 entries (use --max to increase limit)
```

If you disable a bound explicitly, that also produces a diagnostic so automation can tell the query was intentionally unbounded:

```text
$ wavepeek scope --waves path/to/dump.vcd --max unlimited
0 top kind=module
1 top.cpu kind=module
1 top.mem kind=module
warning[WPK-W0001]: limit disabled: --max=unlimited
```

## Non-obvious behavior

- `scope` always starts from the dump roots. There is no `--scope` flag on this command.
- `--filter` is a regular expression over the full canonical path.
- `--tree` affects only human output. With `--json`, the result stays a flat array and `--tree` is ignored.
- Scope `kind` is not limited to `module`. For example:

```text
$ wavepeek scope --waves path/to/dump.fst --tree --max 50
top kind=module
├── cpu kind=module
├── helper kind=function
└── worker kind=task
```

- An empty match is still success: the command prints no rows and emits `WPK-W0003`.
- Once you have the right path, the next step is usually `wavepeek signal --scope <that-path> ...`.
