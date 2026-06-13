---
id: commands/signal
title: Signal command
description: Explore signals within a scope.
section: commands
see_also:
  - commands/overview
  - commands/scope
  - commands/value
  - reference/command-model
  - reference/machine-output
---
# Signal command

Use `signal` when you already know the scope, but do not yet know the exact signal names worth sampling.

This is usually the step between `scope` and `value`/`change`/`property`:

- confirm the signal spelling the dump actually uses,
- confirm whether it is local to the scope or hidden in a child scope,
- see basic metadata such as kind and width,
- copy the canonical path you will use in later commands.

For exact syntax and flags, run `wavepeek help signal`.

## Start with direct signals in one scope

By default, `signal` lists only signals declared directly in `--scope`:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --max 10
cfg kind=parameter width=8
clk kind=wire width=1
data kind=reg width=8
```

Use this early when you want a quick answer without walking the whole subtree.

## Narrow the list by signal name

`--filter` is a regular expression over the signal name:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --filter '^c.*' --max 10
cfg kind=parameter width=8
clk kind=wire width=1
```

This is the fastest way to answer questions like “is it `clk`, `clock`, or `core_clk`?”.

## Search into child scopes

Add `--recursive` when the signal may live below the selected scope:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --recursive --max 10
cfg kind=parameter width=8
clk kind=wire width=1
data kind=reg width=8
cpu.valid kind=wire width=1
mem.ready kind=wire width=1
```

In recursive human output, nested rows are shown as scope-relative names.

## Keep recursive search shallow

If full descent is too noisy, cap it with `--max-depth`:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --recursive --max-depth 1 --max 10
clk kind=wire width=1
reset_n kind=wire width=1
cpu.valid kind=wire width=1
mem.ready kind=wire width=1
```

This is useful when you only want one level below the selected scope. `--max-depth 0` is equivalent to staying local to `--scope`.

## Show canonical paths when you want to reuse them

Use `--abs` when you want copy-pasteable canonical paths for later commands:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --recursive --abs --max 10
top.cfg kind=parameter width=8
top.clk kind=wire width=1
top.data kind=reg width=8
top.cpu.valid kind=wire width=1
top.mem.ready kind=wire width=1
```

Without `--abs`, recursive output is easier to read. With `--abs`, it is easier to reuse.

## Use JSON when another tool will consume the result

`--json` keeps the stable machine contract and always includes canonical paths:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --recursive --json --max 10
{"$schema":"https://kleverhq.github.io/wavepeek/wavepeek_v1.json","command":"signal","data":[{"name":"cfg","path":"top.cfg","kind":"parameter","width":8},{"name":"clk","path":"top.clk","kind":"wire","width":1},{"name":"data","path":"top.data","kind":"reg","width":8},{"name":"valid","path":"top.cpu.valid","kind":"wire","width":1},{"name":"ready","path":"top.mem.ready","kind":"wire","width":1}],"diagnostics":[]}
```

Use this in scripts and agents when human formatting is not reliable enough.

## Watch for truncation and disabled-limit diagnostics

`signal` is bounded by default. If `--max` cuts the result, the command still succeeds and emits a diagnostic:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --max 1
cfg kind=parameter width=8
warning[WPK-W0002]: truncated output to 1 entries (use --max to increase limit)
```

If you disable a limit explicitly, that is also reported as a diagnostic:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --recursive --max unlimited --max-depth unlimited
cfg kind=parameter width=8
clk kind=wire width=1
data kind=reg width=8
cpu.valid kind=wire width=1
mem.ready kind=wire width=1
warning[WPK-W0001]: limit disabled: --max=unlimited
warning[WPK-W0001]: limit disabled: --max-depth=unlimited
```

In human mode diagnostics go to stderr. In JSON mode they appear in the `diagnostics` array.

## Non-obvious behavior

- `signal` does not discover scopes. Use `wavepeek scope` beforehand if you are not sure about the exact `--scope` path.
- `--filter` matches the leaf signal name, not the displayed recursive prefix and not the full canonical path. For example:

```text
$ wavepeek signal --waves path/to/dump.vcd --scope top --recursive --filter '^cpu\.' --max 10
```

  This succeeds with no rows, because the names are `valid` and `ready`; `cpu.` is only part of the human display string.
- Recursive human output is scope-relative by default, but JSON `path` fields stay canonical.
- Signal kinds are not limited to wires. You may see `reg`, `parameter`, and other stable signal kind aliases; backend-specific VHDL spellings are normalized to the stable contract surface.
- `--max-depth` requires `--recursive`.
- An empty match is a valid result, not an error; it emits `WPK-W0003`.
