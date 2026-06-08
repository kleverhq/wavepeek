---
id: troubleshooting/unsupported-signal-encodings
title: Unsupported signal encodings
description: Understand why `value` and `change` reject some signals and when to switch to expression-based queries instead.
section: troubleshooting
see_also:
  - commands/signal
  - commands/value
  - commands/change
  - commands/property
  - reference/expression-language
---
# Unsupported signal encodings

`value` and `change` are built for sampled bit-vector signals.

If you point them at signals recorded with other encodings, the command can fail with an error like:

```text
error: signal: signal 'top.temp' has unsupported non-bit-vector encoding
```

Common examples are real-valued, string-like, or other non-bit-vector dump encodings.

## Why this happens

`value` prints concrete sampled values, and `change` compares sampled values across timestamps.

Those command surfaces currently expect signals that can be represented as ordinary Verilog-style bit-vector literals.

## Use `signal` first to confirm what you selected

If you are not sure whether the path is correct, start with:

```text
wavepeek signal --waves dump.vcd --scope top --recursive --abs --max 50
```

That helps confirm the exact path and whether you accidentally selected a different signal than intended.

## When `property` is a better fit

If your real question is logical rather than display-oriented, `property` is often the right fallback.

Examples:

- whether a condition ever became true,
- whether a string or enum comparison matched,
- whether an event-like source triggered at selected timestamps.

`property` uses the expression surface documented in `reference/expression-language`, which is broader than the raw sampled-value surfaces used by `value` and `change`.

## What to do when you still need a raw value timeline

If the signal is not a bit-vector, `value` and `change` are not the right tools today.

In practice, the safest recovery is to reformulate the question:

- use `property` to test a condition over time,
- sample a related bit-vector signal instead,
- or inspect the waveform in a GUI when you need the original non-bit-vector rendering.

## Quick recovery checklist

1. Confirm the exact signal path with `scope` and `signal`.
2. Check whether the selected signal is really the one you meant to inspect.
3. If the signal is non-bit-vector, switch from `value` or `change` to `property` when a Boolean question is enough.
4. If you need literal non-bit-vector rendering, use a waveform viewer instead of forcing the `value` or `change` surfaces.
