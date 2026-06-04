#!/usr/bin/env bash
set -eu

if [ "$#" -ne 1 ]; then
    printf '%s\n' "usage: $0 <github-token>" >&2
    exit 2
fi

token="$1"
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
