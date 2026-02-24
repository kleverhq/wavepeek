# Agent Guide (wavepeek)

This file is a repository map for coding agents. Use it as the first entry point,
then follow links to deeper docs.

## Start Here

- Product design and contracts: `docs/DESIGN.md`
- Delivery milestones and versions: `docs/ROADMAP.md`
- Development workflow and coding conventions: `docs/DEVELOPMENT.md`
- Release checklist and rollback notes: `docs/RELEASE.md`
- Active backlog and tech debt: `docs/BACKLOG.md`
- Execution plans (active/completed): `docs/exec-plans/`
- Actual shipped changes by release: `CHANGELOG.md`

For docs-local navigation, read `docs/AGENTS.md`.

## Core Workflow

- This repository is container-first. Run `make` targets inside devcontainer/CI image.
- Standard quality gate: `make ci`.
- Local pre-handoff gate: `make check` (format/lint/schema/build + commit-msg check).
- Test-inclusive gate: `make ci` (same checks + `cargo test`).
- Do not bypass hooks unless the user explicitly requests it.

## Agent-Assisted Coding

- OpenCode is the primary agent toolchain for this repository.
- OpenCode runtime settings and command permissions are in `opencode.json`.
- Custom OpenCode agents live in `.opencode/agent/`.
- Custom OpenCode skills live in `.opencode/skills/`.
- Complex features/refactors should use `exec-plan` skill.
- Implementation review should use `ask-review` skill with the `review` agent.
- Periodic repository cleanup and simplification should use `repo-gc` skill.

## Critical Tool Safety Rule

- Treat all `.fst` files as binary files, not text.
- NEVER call the `Read` tool on `.fst` files under any circumstances.
- If information from a `.fst` file is needed, use a non-`Read` workflow and ask the user for guidance when required.

## Devcontainer Notes

For non-obvious container decisions, fixture provisioning, and environment
rationale, see `.devcontainer/AGENTS.md`.
