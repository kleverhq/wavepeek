---
id: commands/property
title: Property command
summary: Evaluate an expression on event-selected timestamps and capture assertion-style results.
section: commands
see_also:
  - concepts/time
  - concepts/selectors
---
# Property command

Use `property` when you care about whether a condition holds at selected event
times, not about printing raw sampled values.

## Start with a simple trigger

Prefer a small event selector first, then widen it only when you have confirmed
the scope and expression names.

## Choose capture mode intentionally

`match` keeps every matching timestamp, while `switch`, `assert`, and
`deassert` narrow the report to assertion-style state observations.
