# Explore a dump

Start here when you have a waveform file but do not yet know its time range, hierarchy, or signal names. The usual flow is `info`, then `scope`, then `signal`.

## How long is this waveform?

Check the dump bounds before choosing timestamps or time windows:

```text
$ wavepeek info --waves dump.fst
time_unit: 1ns
time_start: 0ns
time_end: 250000ns
```

The dump may use a different unit or start after zero. Reuse the reported units in later `--at`, `--from`, and `--to` arguments.

## Which blocks were dumped?

A shallow tree is usually enough to find the design root:

```text
$ wavepeek scope --waves dump.fst --tree --max-depth 2 --max 30
tb kind=module
└── dut kind=module
    ├── cpu kind=module
    ├── dma kind=module
    └── peripherals kind=module
```

If the hierarchy is large, filter by a likely block name:

```text
wavepeek scope --waves dump.fst --filter '.*(cpu|axi|uart).*' --max-depth 4
```

The filter matches full canonical scope paths.

## What is the exact signal name?

Once you know the scope, search its direct signals:

```text
$ wavepeek signal --waves dump.fst --scope tb.dut.cpu --filter '.*(clk|reset|state).*'
clk kind=wire width=1
reset_n kind=wire width=1
state kind=reg width=4
```

For direct signals, `--filter` matches the signal name. With `--recursive`, it matches the full logical path from the dump root, even when the search starts in a nested scope. Use `--scope` and `--max-depth` to limit the search area. For example, search from the root for `dust_bunnies` under a scope whose name contains `u_couch`.

```text
wavepeek signal --waves dump.fst \
  --scope '' --recursive --max-depth 5 \
  --filter 'u_couch.*dust_bunnies$' --abs --max 30
```

In recursive searches, replace a name-only pattern such as `^clk$` with `(^|\.)clk$` to find ordinary `clk` names at any depth.

Use logical spelling in the regex. For the path `top."block.dot"."data.bus"`, include the double quotes and escape each literal dot in the pattern. Shell single quotes preserve the regex as written.

```sh
wavepeek signal --waves dump.vcd --scope top --recursive \
  --filter '^top\."block\.dot"\."data\.bus"$' --abs
```

The default SV output prints `top.\block.dot .\data.bus `, with a space at the end of each escaped component. `--path-style logical` prints `top."block.dot"."data.bus"` for the same filter.

Text output uses paths relative to the selected scope by default. Use `--abs` to print full paths that can be copied into later commands.

## What if the result is truncated?

`scope` and `signal` limit their output by default. `--max` applies after filtering. A truncation warning means the query found more matches than it printed. Narrow the filter or hierarchy depth before raising `--max`.

See [Paths, signals and scopes](paths.md) for name resolution and [Time units and windows](timeunits.md) for time-token rules.
