# Workflow Guidance

## Scope

This directory owns GitHub Actions workflow YAML.

## Source of Truth

- Workflow automation: `../../docs/dev/automation.md`
- Quality gates: `../../docs/dev/quality.md`
- Stable local task interface: `../../justfile`
- Workflow helper scripts: `../../tools/`

## Local Guidance

- Validate workflow changes with `just check-actions`.
- Keep inline shell or Python in workflow `run` and `runCmd` blocks to short glue up to 5 logical lines.
- Move longer logic into a separate testable helper under `../../tools/`, then invoke that helper from the workflow.
