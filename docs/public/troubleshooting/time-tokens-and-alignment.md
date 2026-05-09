---
id: troubleshooting/time-tokens-and-alignment
title: Time tokens and alignment
summary: Fix time-argument errors caused by missing units, out-of-range timestamps, or dump-resolution mismatch.
section: troubleshooting
see_also:
  - commands/info
  - commands/value
  - commands/change
  - commands/property
  - reference/command-model
---
# Time tokens and alignment

Most time-related failures come from one of four causes:

- the token has no unit,
- the token is not an integer,
- the token is outside the dump bounds,
- the token cannot be represented exactly at the dump resolution.

## Start with the dump's own time contract

Use `wavepeek info` first. It tells you the dump bounds and the dump time unit that later commands validate against.

That matters because `wavepeek` does not guess, round, or silently clamp time values.

## Use integer-plus-unit tokens only

Accepted time tokens are an integer magnitude plus a unit suffix such as `ps`, `ns`, or `us`.

Valid examples:

```text
10ps
25ns
1us
```

Invalid examples:

```text
10
1.5ns
```

If you pass a bare number or a fractional token, the command fails with an argument error such as `invalid time token ... expected <integer><unit>`.

## Stay inside the dump bounds

Time queries are checked against the recorded dump range.

For example, if the dump covers `0ns` through `10ns`, then `11ns` is rejected:

```text
error: args: time '11ns' is outside dump bounds [0ns, 10ns]
```

This applies to `value --at` and to `change` or `property` window boundaries.

## Match the dump resolution exactly

A dump with `1ns` precision cannot represent `15ps` exactly.

That produces an error like:

```text
error: args: time '15ps' is not aligned to dump resolution '1ns'
```

The fix is to choose a timestamp that is an exact multiple of the dump resolution.

## Quick recovery checklist

1. Run `wavepeek info --waves <dump>`.
2. Copy the reported bounds and resolution.
3. Rewrite the query with explicit units.
4. Keep the timestamp or window inside the reported bounds.
5. If needed, round your reasoning manually to an exact multiple of the dump resolution before rerunning the command.
