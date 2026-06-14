# Git and Contribution Hygiene

Use conventional commits. The commit-msg hook runs Commitizen through `just check-commit`, and pre-commit hooks are installed by `just dev-setup`.

Commit small logical milestones. For broad refactors, commit after each independently validated slice so stale-reference fallout can be found and reverted without excavating a single heroic rubble pile.

Do not bypass hooks with `--no-verify` unless the user or maintainer explicitly asks. If a hook fails, fix the cause, rerun the relevant command, and retry the commit.

Use repository-root `tmp/` for ignored scratch files, logs, and ad hoc outputs. Do not globally clean it or delete arbitrary existing files because another agent or the user may own them.

Use `docs/tracker/wip/` for branch-local tracked artifacts that need review or must survive across agent sessions. Those artifacts should be removed before merging to the default branch unless a maintainer intentionally keeps them for handoff.

## GitHub and Fork Remotes

Fork contributors should keep `origin` pointed at their fork and use `upstream` for `https://github.com/kleverhq/wavepeek.git`. `.devcontainer/setup-github-auth.sh` may add or update `upstream` when `origin` is not the upstream repository, but it must not rewrite `origin`.

Commands that intentionally target the upstream repository should pass it explicitly, for example `gh pr list -R "$WAVEPEEK_UPSTREAM_REPO"` or `gh pr list -R kleverhq/wavepeek`. Browser-based PR creation remains supported and must not require GitHub CLI authentication. Token handling and external-PR safety rules live in `github-auth.md`.

Before proposing substantial work, check GitHub Milestones and open GitHub Issues. If the change needs product or maintainer discussion, open or reference an issue before starting a PR.
