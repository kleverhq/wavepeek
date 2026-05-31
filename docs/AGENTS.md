# Documentation Guidance

## Source of Truth

- Public user docs: `public/intro.md`, `public/reference/`
- Maintainer docs: `dev/`
- Tracking docs: `tracker/backlog.md`, `tracker/roadmap.md`, `tracker/wip/`
- Packaged skill source: `skills/wavepeek.md`
- Shipped release history: `../CHANGELOG.md`

## Local Guidance

- Keep public embedded docs under `public/` focused on user-visible behavior and offline command guidance.
- Keep maintainer workflow, style, release, and architecture guidance under `dev/`.
- Keep branch-local tracked artifacts under `tracker/wip/` and clear them before merge unless a maintainer wants handoff context.
- Do not use breadcrumbs as directory indexes; point to the authoritative doc and add only local gotchas.
