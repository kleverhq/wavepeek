#!/bin/sh
set -eu

mkdir -p \
    "$HOME/.config/opencode" \
    "$HOME/.local/share/opencode" \
    "$HOME/.cache/opencode" \
    "$HOME/.config/gh"

verdi_mount_parent="$HOME/.cache/wavepeek-devcontainer"
verdi_mount="$verdi_mount_parent/verdi"

mkdir -p "$verdi_mount_parent"

use_empty_verdi_mount() {
    if [ -L "$verdi_mount" ] || [ -f "$verdi_mount" ]; then
        rm -f "$verdi_mount"
    fi
    mkdir -p "$verdi_mount"
}

if [ -n "${VERDI_HOME:-}" ] && [ -d "$VERDI_HOME" ]; then
    if [ -L "$verdi_mount" ] || [ -f "$verdi_mount" ]; then
        rm -f "$verdi_mount"
    elif [ -d "$verdi_mount" ]; then
        rmdir "$verdi_mount" 2>/dev/null || {
            printf '%s\n' "error: devcontainer: $verdi_mount exists and is not empty; remove it so VERDI_HOME can be mounted" >&2
            exit 1
        }
    fi

    ln -s "$VERDI_HOME" "$verdi_mount"
elif [ -n "${VERDI_HOME:-}" ]; then
    printf '%s\n' "warning: devcontainer: VERDI_HOME is set but is not a directory: $VERDI_HOME; mounting empty /opt/verdi" >&2
    use_empty_verdi_mount
else
    use_empty_verdi_mount
fi
