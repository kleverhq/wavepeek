---
name: review-task
description: Review task implementation for completeness, alignment, and code quality.
---

## Inputs
- PLAN (`PLAN.md` + task number or task brief) and agreed Definition of Done.
- Latest diff/commits, key files touched, and test results.

## Workflow
1. Collect context: scope, non-goals, constraints, risks, required artifacts.
2. Map scope to changes: ensure all planned outputs are represented in the diff.
3. Review checklist:
   - Completeness: goals and DoD items met; docs/tests/flags present.
   - Consistency: plan, code, and docs align with no contradictions.
   - Best practices: repo conventions, error handling, security, performance.
   - Alignment: no scope creep; constraints honored.
   - Quality: correctness, readability, maintainability, test coverage.
4. Findings output:
   - List issues by severity (Blocker, Major, Minor, Nit).
   - Include file:line when possible and a concrete fix suggestion.
   - Note missing evidence (e.g., tests not run).
5. Re-review after fixes to confirm resolution and DoD alignment.

## Output rules
- Keep feedback concise and actionable.
- Ask one targeted question only if missing info blocks a verdict.

## References
- Task format aligns with `.opencode/skills/write-plan/references/plan-template.md`.
