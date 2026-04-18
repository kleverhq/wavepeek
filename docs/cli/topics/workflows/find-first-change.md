---
id: workflows/find-first-change
title: Find first change
summary: Start wide, then narrow the earliest transition that matters.
section: workflows
see_also:
  - commands/change
  - concepts/time
  - troubleshooting/empty-results
---
# Find first change

This workflow helps you narrow an early transition without flooding the result
set.

## Recipe

1. Use `wavepeek info` to confirm dump bounds.
2. Start with `wavepeek change` over a short time range and a focused signal
   list.
3. Tighten `--from`, `--to`, or `--on` until the earliest relevant transition is
   isolated.

## When to stop widening

If the output starts to grow quickly, stop and reduce the query shape before you
turn off bounds.
