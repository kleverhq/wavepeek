---
id: troubleshooting/scoped-vs-canonical-names
title: Scoped vs canonical names
summary: Resolve name lookup failures by matching your signal spelling to whether `--scope` is set.
section: troubleshooting
see_also:
  - commands/scope
  - commands/signal
  - commands/value
  - commands/change
  - commands/property
  - reference/command-model
---
# Scoped vs canonical names

Most name-resolution mistakes come from mixing two naming modes:

- **canonical mode**: no `--scope`; pass full paths such as `top.cpu.clk`,
- **scoped mode**: `--scope top.cpu`; pass names relative to that scope such as `clk`.

Do not mix the two modes in one query.

## Canonical mode: no `--scope`

Without `--scope`, `wavepeek` resolves signal names from the dump root.

Use full canonical paths:

```text
wavepeek value --waves dump.vcd --at 10ns --signals top.cpu.clk,top.cpu.state
wavepeek change --waves dump.vcd --signals top.cpu.clk,top.cpu.state --from 0ns --to 20ns
wavepeek property --waves dump.vcd --on 'posedge top.cpu.clk' --eval 'top.cpu.state == 8'h03'
```

Short names such as `clk` do not resolve in this mode unless the dump really contains a top-level canonical path with that exact name.

## Scoped mode: `--scope <path>`

With `--scope`, keep names relative to that scope:

```text
wavepeek value --waves dump.vcd --at 10ns --scope top.cpu --signals clk,state
wavepeek change --waves dump.vcd --scope top.cpu --signals clk,state --on 'posedge clk' --from 0ns --to 20ns
wavepeek property --waves dump.vcd --scope top.cpu --on 'posedge clk' --eval "state == 8'h03"
```

In this mode, do not repeat the scope prefix inside `--signals`, `--on`, or `--eval`.

## Command-specific reminders

### `value`

- without `--scope`: `--signals` expects canonical paths,
- with `--scope`: `--signals` must be scope-relative.

## `change`

- `--signals` follows the same rule as `value`,
- in scoped mode, names inside `--on` must also stay relative to the selected scope.

## `property`

- names inside both `--on` and `--eval` follow the same scoped-versus-canonical rule,
- if you set `--scope top.cpu`, write `clk` or `state`, not `top.cpu.clk` or `top.cpu.state`.

## How to recover quickly

1. Use `wavepeek scope` to confirm the exact scope path.
2. Use `wavepeek signal --scope <path>` to confirm the exact signal names.
3. Decide whether you want canonical mode or scoped mode.
4. Rewrite all signal references in that query to match the chosen mode.

If a query still fails, it usually means the scope path is wrong, the signal name spelling is wrong, or the signal lives in a different scope than expected.
