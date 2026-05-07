---
id: workflows/find-first-change
title: Find first change
summary: Start wide, then narrow the earliest transition that matters.
section: workflows
see_also:
  - commands/change
  - reference/command-model
  - troubleshooting/empty-results
---
# Find first change

This workflow helps you narrow an early transition without flooding the result set.

## Recipe

1. Use `wavepeek info` to confirm dump bounds and time unit.
2. Use `wavepeek scope` and `wavepeek signal` if you need to confirm hierarchy and signal names.
3. Start with `wavepeek change` over a short time range and a focused signal list.
4. Tighten `--from`, `--to`, or `--on` until the earliest relevant transition is isolated.

## When to stop widening

If the output starts to grow quickly, stop and reduce the query shape before you turn off bounds. Keep the query bounded so the next result set remains explainable.
