# Git and Contribution Hygiene

Use conventional commits. Pre-merge CI checks the complete message of each commit in a pull request. Run the local [native SDK checks](quality.md#native-sdk-checks-before-opening-a-pr) before opening a PR.

Commit small logical milestones. For broad refactors, commit after each independently validated slice so stale-reference fallout can be found and reverted without excavating a single heroic rubble pile.

Keep generated tool outputs and logs in ignored repository-root `tmp/`. Never clean it globally or delete files owned by others.

## Remove Legacy WavePeek Hooks

If this clone previously used WavePeek's hook installer, inspect the configured paths in the main checkout and in each linked worktree.

```sh
git config --show-origin --show-scope --get-all core.hooksPath
```

WavePeek used `wavepeek-hooks` inside the worktree Git directory or the shared Git directory. One older version used a shared path ending in `wavepeek/git-hooks/<repository-id>`. Confirm the directory belongs to WavePeek's installer by checking for its dispatcher and `dev` copy.

If the directory contains other hooks, keep the configured path. Rename each confirmed WavePeek dispatcher, for example from `pre-commit` to `pre-commit.wavepeek-disabled`, so Git still runs the other hooks.

If the directory contains only WavePeek's installation, remove its setting. Substitute the exact path shown for the matching scope.

```sh
git config --worktree --fixed-value --unset-all core.hooksPath '/exact/worktree/path'
git config --local --fixed-value --unset-all core.hooksPath '/exact/shared/path'
```

Run the `--worktree` command in each affected checkout. Run the `--local` command once for the shared setting. The removed settings deactivate the installed copies. Keep any paths configured for other hooks.

Older versions also installed Pre-commit directly in the shared Git directory's `hooks/`. Find that directory with `git rev-parse --path-format=absolute --git-common-dir`. Inspect each of `pre-commit` and `commit-msg` and confirm it was installed for WavePeek with `--config=.pre-commit-config.yaml`. From the host, run `pre-commit uninstall --hook-type pre-commit` for the confirmed `pre-commit` hook. Use `--hook-type commit-msg` for a confirmed `commit-msg` hook. You can run Pre-commit through `uvx pre-commit` on the host. The uninstall restores any previous `.legacy` hook.

## GitHub and Fork Remotes

Fork contributors should keep `origin` pointed at their fork and use `upstream` for `https://github.com/kleverhq/wavepeek.git`. Agents, Git identity and signing, credentials, pushes, issues, and pull requests remain on the host; the credentialless container runs repository tools and quality gates only.

Commands that intentionally target the upstream repository should pass it explicitly, for example `gh pr list -R kleverhq/wavepeek`. Browser-based PR creation remains supported and must not require GitHub CLI authentication.

Before proposing substantial work, check GitHub Milestones and open GitHub Issues. If the change needs product or maintainer discussion, open or reference an issue before starting a PR.
