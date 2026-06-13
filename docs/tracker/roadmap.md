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

No large feature milestone is currently scheduled. This is intentional: the
near-term plan is to keep the shipped command surface stable and spend the next
cycle on bug fixes, documentation clarifications, and user-experience
improvements discovered through real usage.

Planned near-term scope:
- fix correctness, parity, and regression bugs as they are found;
- improve CLI wording, help text, diagnostics, and documentation where users or
  agents hit ambiguity;
- keep quality gates, fixtures, schema artifacts, and benchmark baselines in
  sync with the stable command contract.

Non-goals for the near term:
- do not schedule major command-surface expansions;
- do not promote exploratory backlog proposals into committed milestone scope
  without a separate design decision;
- do not treat roadmap silence as an implicit feature promise.
