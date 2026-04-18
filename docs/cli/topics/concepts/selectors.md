---
id: concepts/selectors
title: Scope and selector rules
summary: Choose between canonical paths, scoped names, and ordered signal lists without ambiguity.
section: concepts
see_also:
  - commands/change
  - commands/property
---
# Scope and selector rules

`wavepeek` uses canonical dump-derived paths as the stable naming model.

## Prefer canonical paths

Without `--scope`, names should be canonical full paths such as `top.cpu.clk`.

## Scoped short names

When a command accepts `--scope`, short names are resolved inside that scope.
Use this mode when you want compact signal lists but still want deterministic
resolution rules.

## Ordered signal lists

Commands that accept `--signals` preserve the input order you provide. This is
useful when you want stable output columns or stable JSON post-processing.
