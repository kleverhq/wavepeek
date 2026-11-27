# Development Workflow

Use Conventional Commits. For pull requests targeting `main` or `dev*`, GitHub Actions runs `just ci` on the merge checkout.

CI runs Rust unit and integration tests through the coverage gate with a default minimum of 90% for lines, functions, and regions. Add tests for uncovered behavior when the gate fails.

Run workflow changes through `./dev just check-actions`. Keep inline `run` and `runCmd` scripts to five logical lines of glue. Put longer logic in a testable helper under `tools/`.

Treat the `just` recipe or workflow step as the automation interface. Keep helpers deterministic and non-interactive. Describe their use in a script header or group README, keep tests with the helper, and register new test groups in `test-aux`.

For fork work, keep `origin` on the fork and name the upstream repository explicitly in commands that target it. `CONTRIBUTING.md` defines where proposals and bug reports belong.

## Native checks before opening a PR

Before opening a PR, run `./dev just ci-native` in the private profile for changes that affect all readers. For a change confined to one native reader, use its focused suite below. These recipes require the applicable reader, run lint and CLI regressions, and launch the binary outside Cargo.

```sh
./dev just lint-fsdb test-fsdb bench-e2e-fsdb-smoke
./dev just lint-shm check-shm-build test-shm bench-e2e-shm-smoke
```

## Upgrading a checkout with legacy WavePeek hooks

Run `git config --show-origin --show-scope --get-all core.hooksPath` in the main checkout and each linked worktree. Identify WavePeek's old dispatcher and `dev` copy in `wavepeek-hooks` or a path ending in `wavepeek/git-hooks/<repository-id>`. If other hooks share the directory, disable only the WavePeek dispatchers by renaming them.

For a directory dedicated to WavePeek, run `git config --worktree --fixed-value --unset-all core.hooksPath '/exact/worktree/path'` in each affected worktree. Use `--local` and the exact shared path for a repository-wide setting.

If a hook in the shared Git directory's `hooks/` uses WavePeek's old `--config=.pre-commit-config.yaml`, run `uvx pre-commit uninstall --hook-type pre-commit` for the confirmed hook. Use `--hook-type commit-msg` for a confirmed commit message hook. The uninstall restores any previous `.legacy` hook.
