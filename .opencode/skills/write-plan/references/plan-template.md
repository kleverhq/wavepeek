# Title - One line, precise and specific

## Summary
- 2-4 sentences: what changes, why now, and expected impact.

## Goals
- Bullet list of measurable outcomes.

## Non-Goals
- Explicitly list what is out of scope.

## Background / Context
- Key facts, constraints, and prior decisions.

## Problem Statement
- What is broken or missing; who is affected; why it matters.

## Requirements
- Functional and non-functional requirements.
- Include constraints (performance, security, compliance, timelines).

## Proposed Solution
- High-level approach and components.
- Include data flows, APIs, and storage changes if relevant.

## Alternatives Considered
- 1-3 options with trade-offs and rejection rationale.

## Risks and Mitigations
- Risks, likelihood/impact, and how to mitigate or detect.

## Rollout / Migration Plan
- Phases, compatibility, data migration, and rollback.

## Observability
- Metrics, logs, alerts, and dashboards required.

## Open Questions
- List remaining unknowns or decisions needed.

## Assumptions
- Explicit assumptions used to finalize the plan.

## Definition of Done
- A checklist of concrete verifications that prove the RFC is fully implemented.
- Prefer objective checks (tests to run, flags to enable, dashboards/alerts to exist, docs to be updated).
- Avoid vague items like "works" or "is done".

## Implementation Plan (Task Breakdown)
- Break the work into sequential, atomic tasks (~2-4h SWE granularity) that can be executed in order.
- Each task must include: Goal, Inputs, Known-unknowns, Steps (executable TODO list), Outputs.
- Steps should be imperative, ordered by dependency, and reference concrete targets (files, modules, commands, flags).
- Example:
  ### Task 1: Add config flag (~2-4h)
  - Goal: Make feature controllable via configuration.
  - Inputs: Access to `config/settings.yaml`; agreed flag name and default.
  - Known-unknowns: Whether existing config loader supports the new type/shape.
  - Steps:
    1. Add config flag `enable_x` to `config/settings.yaml` with default `false`.
    2. Update config parsing in `src/...` and add unit tests in `tests/...`.
  - Outputs: Flag is present, parsed, documented, and covered by tests.
