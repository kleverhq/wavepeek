---
id: concepts/time
title: Time semantics
summary: Understand how wavepeek normalizes explicit time values and bounded ranges.
section: concepts
see_also:
  - commands/change
  - commands/property
---
# Time semantics

All explicit time values in `wavepeek` require an integer magnitude and a unit
suffix such as `ns` or `us`.

## Normalized timestamps

The CLI converts requested times into the dump's native time unit before it
runs the command. Results then print normalized timestamps in that same unit.

## Inclusive ranges

When a command accepts `--from` and `--to`, both boundaries are inclusive. A
bounded query therefore includes rows at the exact start and end timestamps when
those timestamps satisfy the command's other filters.
