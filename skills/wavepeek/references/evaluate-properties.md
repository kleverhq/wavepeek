# Evaluate properties

Use `property` when the question is about a Boolean condition rather than a raw signal table. `--on` selects when to check, `--eval` defines the condition, and `--capture` selects which results to keep.

## When did a condition first match?

Use `--capture match --max 1`:

```text
$ wavepeek property --waves dump.fst \
    --scope tb.dut.cpu \
    --on 'posedge clk iff reset_n' \
    --eval "state == 4'h7" \
    --capture match --max 1
@840ns sample@839ns match
```

A truncation warning after the row means more matches exist. It does not invalidate the first match.

## Did a condition ever become true or false?

The default `switch` capture reports both transitions of the Boolean result:

```text
$ wavepeek property --waves dump.fst \
    --scope tb.dut \
    --from 100ns --to 300ns \
    --on 'posedge clk' \
    --eval 'req && !ack'
@140ns sample@139ns assert
@180ns sample@179ns deassert
```

Use `--capture assert` or `--capture deassert` when only one transition matters. Use `--capture match` when every selected event where the expression is true should produce a row.

Transition captures use the condition at `--from` as their baseline, even when
the window has no events. The baseline creates no result row. In pre-edge mode,
a transition capture skips an event exactly at `--from`; match capture can use
its preceding tick when that tick is inside the dump.

## How many handshakes occurred?

Count all matching clock events without printing them:

```text
$ wavepeek property --waves dump.fst \
    --scope tb.dut.bus \
    --on 'posedge clk iff reset_n' \
    --eval 'valid && ready' \
    --capture match --summary --max unlimited
complete: true
returned: 37
limit: null
total: 37
```

Use `--max unlimited` for an exact total. With a numeric limit, `total` can remain unknown if the scan stops early.

A numeric limit counts result rows. After reaching it, `property` checks for
another matching row or the end of the window. Exactly that many matches can
therefore have `complete: true`. Selected operands can fail validation or
decoding before an event matches; conditional evaluation does not guarantee
deferred reader errors.

## Which requests targeted this address?

Predicates can combine a handshake with payload conditions:

```text
wavepeek property --waves dump.fst \
  --scope tb.dut.requester \
  --on 'posedge clk iff reset_n' \
  --eval "req_valid && req_ready && (req_addr == 32'h00001000)" \
  --capture match
```

This reports the matching timestamps but not the request payload. Use `extract generic` when payload values are needed in the same row.

## What if the condition is asynchronous?

Use native sampling for change-driven checks:

```text
wavepeek property --waves dump.fst \
  --scope tb.dut \
  --on '*' --sample-mode native \
  --eval "error_code != 8'h00" \
  --capture assert
```

See [Boolean conditions](predicates.md) for common expression forms and [Clocks and sampling](sampling.md) before comparing a clocked property with RTL or SVA behavior.
