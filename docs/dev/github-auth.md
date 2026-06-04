# Optional GitHub Authentication

The local devcontainer supports optional, repo-scoped GitHub authentication for maintainers and trusted local agents. The default is deliberately boring: a fresh clone builds and runs without a token, and no host GitHub CLI credential store is mounted into the container. Boring is good here. Exciting credential plumbing is how secrets escape and start a career in incident response.

## Security model

The devcontainer reads an optional host-side env file at container creation time:

```text
~/.config/wavepeek/github.env
```

`.devcontainer/initialize.sh` creates that path outside the repository and points it at an empty file by default. `.devcontainer/devcontainer.json` passes it to Docker with `--env-file`, and `.devcontainer/setup-github-auth.sh` configures repo-local Git credential behavior only when `GH_TOKEN` or `GITHUB_TOKEN` is present.

The devcontainer does **not** mount host `~/.config/gh`. GitHub CLI can authenticate from `GH_TOKEN` or `GITHUB_TOKEN`, and mounting the host GitHub CLI config would expose whatever broader host login happens to exist. It could also persist a `gh auth login` fallback token back onto the host. Do not add that mount back unless you want the security model to become interpretive dance.

The container sets `WAVEPEEK_UPSTREAM_REPO=kleverhq/wavepeek` and `GH_PROMPT_DISABLED=1`. It intentionally does **not** set `GH_REPO`; that variable changes the default repository used by many `gh` commands and can surprise fork contributors whose `origin` points at their fork.

When scripts need the upstream repository, pass it explicitly:

```bash
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO"
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO"
gh run list -R "$WAVEPEEK_UPSTREAM_REPO"
```

## Host env files

Use this host-side layout:

```text
~/.config/wavepeek/github.empty.env
~/.config/wavepeek/github.env -> github.empty.env or a maintainer env file
```

`github.empty.env` is created automatically and should stay empty. `github.env` is the active file Docker reads when the container is created. Maintainers may point it at a token-bearing env file for trusted work.

A token-bearing file such as `~/.config/wavepeek/github.maintainer.env` is convenient for trusted branches, but it is not safe to leave at a predictable readable host path when opening an untrusted PR devcontainer. Repository-controlled `initializeCommand` runs on the host before Docker reads `--env-file`; malicious PR code could modify that host-side script and read any maintainer token file your host user can read. Charming, in the same way a bear trap is charming.

Docker reads `--env-file` only when creating the container. After switching `github.env`, recreate or rebuild the devcontainer so the environment actually changes. Merely attaching to an existing container keeps the old environment. Existing containers do not live-reload this file. Naturally, because that would be convenient.

## Maintainer setup

For trusted local work in `kleverhq/wavepeek`, create a fine-grained Personal Access Token with repository access limited to `kleverhq/wavepeek`.

Recommended settings:

```text
Token type: Fine-grained personal access token
Resource owner: kleverhq
Repository access: Only selected repositories
Selected repository: kleverhq/wavepeek
Expiration: 30-90 days preferred
```

Recommended baseline repository permissions:

```text
Metadata: Read
Contents: Read and write
Pull requests: Read and write
Issues: Read and write
Actions: Read
```

Add only when the task actually needs it:

```text
Actions: Read and write
Workflows: Read and write
```

Avoid permissions such as `Administration`, `Secrets`, `Webhooks`, `Security advisories`, `Dependabot secrets`, and `Repository rules` unless there is a specific reviewed reason.

Create the maintainer env file on the host without putting the token in shell history:

```bash
mkdir -p ~/.config/wavepeek
chmod 700 ~/.config/wavepeek
umask 077
read -r -s -p "GitHub token: " wavepeek_github_token
printf '\n'
{
  printf 'GH_TOKEN=%s\n' "$wavepeek_github_token"
  printf 'GITHUB_TOKEN=%s\n' "$wavepeek_github_token"
  printf 'WAVEPEEK_GITHUB_ROLE=maintainer\n'
} > ~/.config/wavepeek/github.maintainer.env
unset wavepeek_github_token
chmod 600 ~/.config/wavepeek/github.maintainer.env
```

Do not run that snippet with shell tracing enabled. If you prefer a password manager or another secret store, generate `github.env` from it only for trusted work and remove the generated file afterward.

Activate it for trusted local work:

```bash
ln -sf github.maintainer.env ~/.config/wavepeek/github.env
```

Then recreate or rebuild the devcontainer. If using an editor reopen flow, make sure it creates a new container instead of attaching to the existing one.

Inside the recreated container, validation should work for a token with sufficient access:

```bash
gh repo view "$WAVEPEEK_UPSTREAM_REPO"
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
```

Git HTTPS operations against `https://github.com/kleverhq/wavepeek.git` can use the token through a repo-local credential helper. The helper stores shell logic in `.git/config`, not the token value, and only returns credentials for the configured upstream repository path.

## Disabling credentials for external PR review

Treat code from fork PRs as untrusted. Switching `github.env` to the empty file only removes the token from newly created containers; it does not protect a readable maintainer token file from a malicious repo-controlled `initializeCommand` running on the host.

Before reviewing or running an external PR in the devcontainer, make the maintainer token unavailable to the host user that will run the PR's devcontainer configuration. Use one of these approaches:

- use a separate host account, VM, or codespace that has no maintainer token file;
- remove the token-bearing env file from disk and recreate it after the review;
- keep the token in a password manager or other store unavailable to the devcontainer host command, and generate `github.env` only for trusted work;
- create the devcontainer from a trusted base-branch configuration with `github.env` empty, then fetch or check out the PR inside that already-created container.

Also switch the active env file to the empty default:

```bash
ln -sf github.empty.env ~/.config/wavepeek/github.env
```

Then recreate or rebuild the devcontainer. Do not merely attach to an existing container; it may still have the old token environment.

Verify inside the new container:

```bash
test -z "${GH_TOKEN-}"
test -z "${GITHUB_TOKEN-}"
test ! -e /home/ubuntu/.config/gh/hosts.yml
```

Those checks confirm that no maintainer token is present in the container environment and that host GitHub CLI config is not mounted. They do not prove a readable token file is absent from the host; handle that before running untrusted PR devcontainer configuration.

## Fork contributor flow

External contributors do not need a `kleverhq/wavepeek` token for normal development.

Recommended remotes for fork contributors:

```text
origin   = git@github.com:<contributor-login>/wavepeek.git
upstream = https://github.com/kleverhq/wavepeek.git
```

`.devcontainer/setup-github-auth.sh` adds or updates `upstream` when `origin` is not the upstream repository. It must not rewrite `origin`; contributors should keep pushing their branches to their own forks.

Typical flow:

```bash
git checkout -b my-change
# edit, build, test
git push -u origin my-change
```

The GitHub web UI is sufficient for opening a pull request. Contributors who already have their own GitHub CLI auth may use:

```bash
gh pr create \
  -R kleverhq/wavepeek \
  --head "<contributor-login>:my-change" \
  --base main \
  --fill
```

Do not require this CLI path.

## Verification

Syntax checks after changing the devcontainer auth setup:

```bash
bash -n .devcontainer/initialize.sh
bash -n .devcontainer/setup-github-auth.sh
python3 -m json.tool .devcontainer/devcontainer.json >/dev/null
```

No-token container checks:

```bash
test -z "${GH_TOKEN-}"
test -z "${GITHUB_TOKEN-}"
test ! -e /home/ubuntu/.config/gh/hosts.yml
```

For fork checkouts where `origin` is not `kleverhq/wavepeek`, also verify the upstream remote:

```bash
git remote get-url upstream
```

Maintainer-token checks:

```bash
gh repo view "$WAVEPEEK_UPSTREAM_REPO"
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
```

Secret leakage checks:

```bash
# Remotes must not contain tokens.
git remote -v

# Git config must not contain token literals.
git config --local --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
git config --global --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
```

Documentation sanity check when changing this workflow:

```bash
rg -n '\.config/gh|GitHub CLI state|github-auth|github\.env|WAVEPEEK_UPSTREAM_REPO|GH_REPO|GH_TOKEN|GITHUB_TOKEN' \
  .devcontainer/AGENTS.md AGENTS.md docs/dev CHANGELOG.md
```

Use the search output to catch stale claims that host `~/.config/gh` is mounted or that `GH_REPO` is globally set.

## Troubleshooting

If `gh repo view "$WAVEPEEK_UPSTREAM_REPO"` fails with a token present, check that the fine-grained PAT is approved for the organization, has access to `kleverhq/wavepeek`, and has the permissions needed for the command. Some failures are expected for fork contributors using their own token without upstream write access.

If switching between maintainer and empty credentials appears to have no effect, recreate or rebuild the devcontainer. Docker does not reread `--env-file` for an existing container.

If `upstream` points somewhere unexpected, inspect `origin` first:

```bash
git remote -v
```

The setup script only adds or updates `upstream` when `origin` is not `kleverhq/wavepeek`; it never rewrites `origin`.
