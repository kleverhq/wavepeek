---
name: review-plan
description: Review PLAN.md for completeness, contradictions, feasibility, and execution readiness.
---

## Inputs
- PLAN.md path and any focus areas or constraints.

## Workflow
1. Extract goals, constraints, assumptions, and non-goals.
2. Run the checklist and note gaps, contradictions, and unclear steps.
3. Classify findings by severity: Blocker, Major, Minor, Nit.
4. Provide concrete fixes or targeted questions for each finding.
5. Conclude with a go/no-go recommendation and required changes.

## Output rules
- Keep summaries short; lead with findings.
- Reference sections by name (e.g., "Requirements", "Implementation Plan").
- Avoid rewriting the entire plan; focus on actionable corrections.

## Checklist
- Goals are measurable and aligned to the problem statement.
- Non-goals prevent scope creep and are explicit.
- Requirements and constraints are complete and testable.
- Proposed solution addresses each requirement; key data flows are clear.
- Alternatives and trade-offs are considered.
- Risks include mitigations or detection signals.
- Rollout and rollback plans are defined where needed.
- Observability and success metrics are specified.
- Tasks are atomic (~2-4h), ordered, and each has Goal/Inputs/Known-unknowns/Steps/Outputs.
- Definition of Done is concrete and maps to tasks and goals.
- Terminology and assumptions are consistent across the plan.

## References
- Template: `.opencode/skills/write-plan/references/plan-template.md`.
