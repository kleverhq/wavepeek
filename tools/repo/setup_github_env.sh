#!/usr/bin/env bash
set -eu

if [ "$#" -ne 0 ]; then
    printf '%s\n' "usage: $0" >&2
    printf '%s\n' "reads the GitHub token from the terminal or stdin" >&2
    exit 2
fi

config_dir="$HOME/.config/wavepeek"

if [ -e "$config_dir" ] && [ ! -d "$config_dir" ]; then
    printf '%s\n' "error: $config_dir exists and is not a directory" >&2
    printf '%s\n' "edit the GitHub auth env files manually." >&2
    exit 1
fi

if [ -d "$config_dir" ] && [ -n "$(ls -A "$config_dir")" ]; then
    printf '%s\n' "error: $config_dir exists and is not empty" >&2
    printf '%s\n' "edit the GitHub auth env files manually." >&2
    printf '%s\n' "expected active file: $config_dir/github.env" >&2
    exit 1
fi

if [ -t 0 ]; then
    printf '%s' "GitHub token: " >&2
    if ! IFS= read -r -s token; then
        printf '\n%s\n' "error: failed to read GitHub token" >&2
        exit 1
    fi
    printf '\n' >&2
else
    if ! IFS= read -r token; then
        printf '%s\n' "error: failed to read GitHub token" >&2
        exit 1
    fi
fi

if [ -z "$token" ]; then
    printf '%s\n' "error: GitHub token must not be empty" >&2
    exit 1
fi

umask 077
mkdir -p "$config_dir"
chmod 700 "$config_dir"

: > "$config_dir/github.empty.env"
{
    printf 'GH_TOKEN=%s\n' "$token"
    printf 'GITHUB_TOKEN=%s\n' "$token"
    printf 'WAVEPEEK_GITHUB_ROLE=maintainer\n'
} > "$config_dir/github.maintainer.env"
ln -s "github.maintainer.env" "$config_dir/github.env"

chmod 600 \
    "$config_dir/github.empty.env" \
    "$config_dir/github.maintainer.env"

printf '%s\n' "ok: wrote $config_dir/github.maintainer.env"
printf '%s\n' "ok: activated $config_dir/github.env"
