# Agent Guide (wavepeek)

This file is a repository map for coding agents. Use it as the first entry point,
then follow referenced paths to deeper docs.

## Start Here

- Product design entrypoint and contracts: `docs/design/index.md`
- Delivery milestones and versions: `docs/ROADMAP.md`
- Development workflow and coding conventions: `docs/DEVELOPMENT.md`
- Release checklist and rollback notes: `docs/RELEASE.md`
- Active backlog, open design questions, and tech debt: `docs/BACKLOG.md`
- Execution plans (active/completed): `docs/exec-plans/`
- Actual shipped changes by release: `CHANGELOG.md`

For docs-local navigation, read `docs/AGENTS.md`.

## Child Maps

- Devcontainer setup notes: `.devcontainer/AGENTS.md`
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
- Add a local `AGENTS.md` only for key navigation nodes or when a directory needs ad hoc agent guidance that is not obvious from its parent map.
- Do not create `AGENTS.md` files just because a directory has tracked files; prefer the nearest parent map unless a local file materially improves navigation or workflow safety.
- Each local `AGENTS.md` that exists should include paths:
  - back to the nearest parent map (or root `AGENTS.md`),
  - sideways to canonical source-of-truth docs/files for that area,
  - forward to child directories that also define `AGENTS.md`.
- Write file and directory paths as plain code spans, for example `docs/AGENTS.md`; do not use full Markdown links.
- Resolve every path relative to the `AGENTS.md` file that contains it, not relative to the repository root.
- Keep breadcrumbs concise and non-duplicative; prefer path references over copied guidance.
- Update breadcrumb paths whenever directories with local maps are added, moved, repurposed, or removed.
- Skip leaf, one-off, generated, vendor-style, or self-evident directories unless a local map materially improves navigation or workflow safety.

## Core Workflow

- This repository is container-first. Run `make` targets inside devcontainer/CI image.
- Standard quality gate: `make ci`.
- Local pre-handoff gate: `make check` (format/lint/schema/build + commit-msg check).
- Test-inclusive gate: `make ci` (same checks + `cargo test`).
- Do not bypass hooks unless the user explicitly requests it.
- GitHub Actions workflows live under `.github/workflows/`; use `docs/DEVELOPMENT.md` for workflow gates and `docs/RELEASE.md` for release process context.

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
