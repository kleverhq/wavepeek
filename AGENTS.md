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

## Child Maps

- Devcontainer setup notes: `.devcontainer/AGENTS.md`
- FSDB lazy-load demo: `fsdb_demo/AGENTS.md`
- GitHub automation assets: `.github/AGENTS.md`
- OpenCode assets: `.opencode/AGENTS.md`
- Performance and benchmark work: `bench/AGENTS.md`
- Documentation map: `docs/AGENTS.md`
- Schema contract assets: `schema/AGENTS.md`
- Release/support scripts: `scripts/AGENTS.md`
- Source code: `src/AGENTS.md`
- Test suites: `tests/AGENTS.md`

## Breadcrumb Policy (`AGENTS.md` Network)

- Treat each `AGENTS.md` as a local map, not a full manual.
- Keep this root file short; canonical policy and product details live in `docs/`.
- When creating a new durable directory with tracked files or directory-specific workflow rules, add a local `AGENTS.md` in the same change.
- Each local `AGENTS.md` should link:
  - back to the nearest parent map (or root `AGENTS.md`),
  - sideways to canonical source-of-truth docs/files for that area,
  - forward to child directories that also define `AGENTS.md`.
- Keep breadcrumbs concise and non-duplicative; prefer cross-links over copied guidance.
- Update breadcrumb cross-links whenever directories are added, moved, or repurposed.
- Skip one-off/generated/vendor-style leaf directories unless a local map materially improves navigation.

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
