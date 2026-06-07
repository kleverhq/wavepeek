#!/usr/bin/env bash
set -euo pipefail

upstream_repo="${WAVEPEEK_UPSTREAM_REPO:-kleverhq/wavepeek}"
token="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

credential_helper='!wavepeek_github_credential_helper() {
          test "$1" = get || exit 0

          protocol=""
          host=""
          path=""

          while IFS= read -r line; do
            case "$line" in
              protocol=*) protocol="${line#protocol=}" ;;
              host=*) host="${line#host=}" ;;
              path=*) path="${line#path=}" ;;
            esac
          done

          test "$protocol" = https || exit 0
          test "$host" = github.com || exit 0

          repo="${WAVEPEEK_UPSTREAM_REPO:-kleverhq/wavepeek}"
          case "$path" in
            "$repo"|"$repo/"|"$repo.git"|"$repo.git/")
              echo username=x-access-token
              echo password="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
              ;;
          esac
        }; wavepeek_github_credential_helper'

# Remove previous repo-local helpers installed by this script, if any. The
# value patterns avoid touching unrelated local credential helpers. Older
# versions installed a broad github.com helper; current setup installs
# path-specific helpers for the upstream repo with an empty first helper entry
# that resets broader inherited helpers for that path.
git config --local --unset-all \
    credential.https://github.com.helper \
    '^!wavepeek_github_credential_helper' 2>/dev/null || true
git config --local --unset-all \
    credential.https://github.com.helper \
    '^$' 2>/dev/null || true

for credential_url in \
    "https://github.com/${upstream_repo}" \
    "https://github.com/${upstream_repo}.git"
do
    git config --local --unset-all \
        "credential.${credential_url}.helper" \
        '^!wavepeek_github_credential_helper' 2>/dev/null || true
    git config --local --unset-all \
        "credential.${credential_url}.helper" \
        '^$' 2>/dev/null || true
done

if [ -n "$token" ]; then
    # Scope Git credential lookup to repository paths and install the helper on
    # both common upstream URL spellings. The empty helper entry resets broader
    # inherited helpers for the upstream path, so stale editor/global helpers do
    # not preempt the repo token during a plain `git push origin ...`.
    git config --local credential.useHttpPath true

    # The helper stores no secret in git config. It reads GH_TOKEN/GITHUB_TOKEN
    # from the process environment at credential lookup time and only returns
    # it for the configured upstream repository.
    for credential_url in \
        "https://github.com/${upstream_repo}" \
        "https://github.com/${upstream_repo}.git"
    do
        git config --local --add "credential.${credential_url}.helper" ""
        git config --local --add "credential.${credential_url}.helper" "$credential_helper"
    done

    if command -v gh >/dev/null 2>&1; then
        if gh repo view "$upstream_repo" >/dev/null 2>&1; then
            echo "GitHub auth configured; upstream repo is reachable: $upstream_repo"
        else
            echo "GitHub token is present, but $upstream_repo is not reachable with it."
            echo "This may be expected for fork contributors, unapproved organization fine-grained PATs, or insufficient token permissions."
        fi
    else
        echo "GitHub token is present, but GitHub CLI is not installed; skipped gh validation."
    fi
else
    echo "GH_TOKEN/GITHUB_TOKEN is not set; GitHub auth setup skipped."
fi

# For fork-based checkouts, keep origin unchanged and add/update an upstream
# remote pointing to kleverhq/wavepeek. Do not rewrite contributor forks.
origin_url="$(git remote get-url origin 2>/dev/null || true)"

if ! printf '%s\n' "$origin_url" | grep -Eq 'github.com[:/]kleverhq/wavepeek(\.git)?$'; then
    if git remote get-url upstream >/dev/null 2>&1; then
        git remote set-url upstream "https://github.com/${upstream_repo}.git"
    else
        git remote add upstream "https://github.com/${upstream_repo}.git"
    fi
fi
