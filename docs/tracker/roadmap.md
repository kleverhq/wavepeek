# Roadmap

This document tracks planned milestone scope and expected delivery order.
It is intentionally forward-looking and may change as priorities evolve.

For factual release outcomes (what actually shipped), use `CHANGELOG.md`.

## Planning Notes

- Version targets here are provisional planning markers, not guarantees.
- `CHANGELOG.md` is the source of truth for shipped scope.
- `backlog.md` may contain ideas or open questions that are not yet scheduled
  into a milestone.

## Near-Term Roadmap

### v1.0 — next major: complete FSDB support

The next major milestone is planned as `v1.0` and is scoped around completing
FSDB support.

Planned scope:
- make FSDB a first-class waveform input alongside the existing VCD/FST flows;
- close known FSDB parity gaps, including value sampling for currently
  unsupported recovered types where the FSDB Reader SDK exposes stable data;
- harden FSDB hierarchy, value, change, property, docs, schema, and CI coverage
  enough to treat FSDB behavior as part of the stable command contract.

Non-goals for `v1.0`:
- do not schedule non-FSDB proposals from `backlog.md` into this milestone;
- do not expand the command surface solely to absorb unrelated backlog ideas.

